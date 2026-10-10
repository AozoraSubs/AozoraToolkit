//! 压制命令层：任务入队、队列调度、取消与命令预览。
//!
//! 调度模型（PLAN §4.3）：同一时刻只运行一个编码任务，其余任务排队；
//! 每次入队与每个任务结束后调用 [`pump`] 尝试启动下一个任务。
//! 所有校验与路径规划都在**入队时**完成，用户能立刻看到必然失败的原因。

use crate::jobs::{
    EnqueueResultDto, JobFinishedDto, JobLogDto, JobProgressDto, JobStartedDto, JobState,
    PreparedJob,
};
use crate::settings;
use aozora_core::capabilities;
use aozora_core::diagnostics;
use aozora_core::encode::{self, EncodeConfig};
use aozora_core::history::{truncate_log_tail, HistoryStore, JobRecord};
use aozora_core::job::{run_encode, EncodeEvent, EncodeRequest};
use aozora_core::media_probe;
use aozora_core::planner;
use aozora_core::queue::{JobMeta, QueueSnapshot};
use aozora_core::unix_now_ms;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use tauri::{AppHandle, Emitter, State};
use tokio::sync::{mpsc, watch};

static JOB_COUNTER: AtomicU64 = AtomicU64::new(0);

/// 转发给前端的日志尾部保留行数（用于写入任务历史）。
const LOG_TAIL_KEEP: usize = 400;

/// 压制临时目录：FFmpeg 先写入这里，成功后移动/复制到最终输出。
///
/// - dev 构建：可执行文件旁的 `encode-tmp`（软件目录下，保证可写）；
/// - 发布构建：应用配置目录下的 `encode-tmp`（安装目录 Program Files
///   不可写，配置目录是应用自有的可写位置）。
#[cfg_attr(dev, allow(unused_variables))]
fn encode_temp_dir(app: &AppHandle) -> Result<PathBuf, String> {
    #[cfg(dev)]
    {
        Ok(crate::exe_dir().join("encode-tmp"))
    }
    #[cfg(not(dev))]
    {
        use tauri::Manager;
        app.path()
            .app_config_dir()
            .map(|dir| dir.join("encode-tmp"))
            .map_err(|e| format!("无法获取应用配置目录：{e}"))
    }
}

/// 目标目录不可写时，附带一份"进程可写性自检"报告到提示文本：
/// 列出当前应用进程在常见目录（临时目录、用户目录、桌面、Downloads、
/// D 盘、软件目录）的实际可写性，帮助一次性定位是进程级限制还是目录权限。
fn destination_unwritable_notice(
    destination_error: &str,
    destination_dir: &Path,
    temp_dir: &Path,
) -> String {
    let mut candidates: Vec<PathBuf> = Vec::new();
    candidates.push(destination_dir.to_path_buf());
    candidates.push(temp_dir.to_path_buf());
    candidates.push(std::env::temp_dir());
    if let Ok(profile) = std::env::var("USERPROFILE") {
        candidates.push(PathBuf::from(&profile));
        candidates.push(PathBuf::from(&profile).join("Desktop"));
        candidates.push(PathBuf::from(&profile).join("Downloads"));
    }
    if Path::new("D:\\").exists() {
        candidates.push(PathBuf::from("D:\\"));
    }
    let report =
        diagnostics::format_write_access_report(&diagnostics::write_access_report(&candidates));
    format!("目标目录不可写，成品将保存到应用临时目录：{destination_error}\n{report}")
}

/// 生成任务 ID：时间戳 + 递增计数器，仅含文件名安全字符。
fn next_job_id() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let counter = JOB_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("e-{nanos:x}-{counter}")
}

/// 生成参数预览（无副作用，不检查输出路径，仅校验配置）。
#[tauri::command]
pub fn preview_job_args(config: EncodeConfig, input_path: String) -> Result<Vec<String>, String> {
    encode::validate_config(&config)?;
    let output = if config.output.path.as_os_str().is_empty() {
        PathBuf::from("输出.mkv")
    } else {
        config.output.path.clone()
    };
    let args = planner::build_args(&config, Path::new(&input_path), &output)?;
    Ok(args
        .into_iter()
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect())
}

/// 入队前的完整准备：配置与路径校验、FFmpeg 解析、烧录字幕能力检查、
/// 参数生成与临时/最终输出路径规划。
fn prepare_job(
    app: &AppHandle,
    config: EncodeConfig,
    input_path: String,
    duration_secs: Option<f64>,
) -> Result<PreparedJob, String> {
    // 1. 配置与路径校验（在启动子进程前完成，避免必然失败的任务）
    encode::validate_config(&config)?;
    let input = PathBuf::from(&input_path);
    let output = config.output.path.clone();
    planner::check_paths(&input, &output, config.output.overwrite)?;

    // 2. 解析当前生效的 FFmpeg
    let settings = settings::load(app)?;
    let detection = media_probe::detect_tools(&settings);
    let ffmpeg = detection.ffmpeg.ok_or_else(|| {
        "未检测到可用的 FFmpeg，无法开始压制。请安装 FFmpeg 或在设置中指定路径。".to_string()
    })?;

    // 2.1 烧录字幕：检查外部字幕文件存在性 + FFmpeg 的 libass 滤镜能力
    if config.subtitles.mode == encode::SubtitleMode::Burn {
        let subtitle_path = config
            .subtitles
            .external_path
            .as_ref()
            .expect("validate_config 已保证存在");
        if !subtitle_path.is_file() {
            return Err(format!(
                "字幕文件不存在或不是文件：{}",
                subtitle_path.display()
            ));
        }
        if !capabilities::has_filter(Path::new(&ffmpeg.path), "subtitles") {
            return Err(
                "当前 FFmpeg 构建不支持 subtitles 滤镜（libass），无法烧录字幕".to_string(),
            );
        }
    }

    // 3. 生成任务专属临时输出路径（应用自己的临时目录），并在启动前完成
    //    可写性探测：应用临时目录 + 最终输出目录。目标目录不可写时
    //    （例如受限/沙箱环境）回退到应用临时目录，保证压制始终可以完成，
    //    并通过 notice 明确告知用户成品位置。
    let job_id = next_job_id();
    let temp_dir = encode_temp_dir(app)?;
    std::fs::create_dir_all(&temp_dir)
        .map_err(|e| format!("创建压制临时目录失败（{}）：{e}", temp_dir.display()))?;
    let temp_path = planner::temp_output_path(&temp_dir, &output, &job_id);
    planner::verify_writable(&temp_path)?;
    let (final_path, notice, overwrite) =
        match planner::verify_destination_writable(&output, &job_id) {
            Ok(()) => (output.clone(), None, config.output.overwrite),
            Err(destination_error) => {
                // 回退到应用临时目录：覆盖同名回退成品（应用自有目录），
                // 并告知用户目标目录不可写的原因 + 进程可写性自检报告。
                let destination_dir = output
                    .parent()
                    .filter(|p| !p.as_os_str().is_empty())
                    .unwrap_or_else(|| Path::new("."))
                    .to_path_buf();
                let notice = Some(destination_unwritable_notice(
                    &destination_error,
                    &destination_dir,
                    &temp_dir,
                ));
                (
                    planner::fallback_output_path(&temp_dir, &output, &job_id),
                    notice,
                    true,
                )
            }
        };
    let args = planner::build_args(&config, &input, &temp_path)?;

    let name = input
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| job_id.clone());
    Ok(PreparedJob {
        meta: JobMeta {
            id: job_id,
            name,
            input_path,
            output_path: output.display().to_string(),
            queued_at_ms: unix_now_ms(),
        },
        config,
        duration_secs,
        ffmpeg_path: PathBuf::from(&ffmpeg.path),
        args,
        input_path: input,
        temp_path,
        final_path,
        overwrite,
        notice,
    })
}

/// 入队一个压制任务：校验与准备立即执行，随后在队列空闲时启动。
#[tauri::command]
pub async fn enqueue_job(
    app: AppHandle,
    state: State<'_, JobState>,
    config: EncodeConfig,
    input_path: String,
    duration_secs: Option<f64>,
) -> Result<EnqueueResultDto, String> {
    let prepared = {
        let app = app.clone();
        tauri::async_runtime::spawn_blocking(move || {
            prepare_job(&app, config, input_path, duration_secs)
        })
        .await
        .map_err(|e| format!("后台任务执行失败：{e}"))??
    };

    // 先保存执行数据再入队：队列一旦能看到任务，其数据必然可取，
    // 避免调度时出现"任务已在运行但没有数据"的中间状态。
    let meta = prepared.meta.clone();
    state.store_prepared(prepared);
    if let Err(error) = state.enqueue(meta.clone()) {
        state.drop_prepared(&meta.id);
        return Err(error);
    }

    let job_id = meta.id;
    let started = pump(&app, &state);
    emit_queue(&app, &state);
    let position = if started {
        None
    } else {
        state.position(&job_id)
    };
    Ok(EnqueueResultDto {
        job_id,
        started,
        position,
    })
}

/// 请求取消正在运行的任务；最终状态由运行器置为 Cancelled。
#[tauri::command]
pub fn cancel_job(state: State<'_, JobState>, job_id: String) -> Result<(), String> {
    state.cancel(&job_id)
}

/// 移除排队中的任务（运行中的任务需要取消）。
#[tauri::command]
pub fn remove_queued_job(
    app: AppHandle,
    state: State<'_, JobState>,
    job_id: String,
) -> Result<(), String> {
    state.remove_queued(&job_id)?;
    emit_queue(&app, &state);
    Ok(())
}

/// 当前队列快照（界面初始化时同步一次，之后由 `job:queue` 事件推送）。
#[tauri::command]
pub fn queue_snapshot(state: State<'_, JobState>) -> QueueSnapshot {
    state.snapshot()
}

/// 尝试启动下一个任务；返回是否启动了任务。
///
/// 只有队列由空闲转为运行时才会取出任务，因此这里清理遗留临时文件是安全的
/// （此时没有其他任务的 FFmpeg 在写临时文件）。注意该清理跑在应用自己的
/// 临时目录上，前提是同一时间只有一个应用实例在压制。
fn pump(app: &AppHandle, state: &JobState) -> bool {
    // 上限仅用于防止内部状态异常时无限循环（正常情况一次即返回）。
    for _ in 0..64 {
        let Some((meta, payload)) = state.take_next() else {
            return false;
        };
        let Some(job) = payload else {
            // 执行数据缺失属内部错误：结束该任务并继续下一个，避免队列卡死。
            let _ = app.emit(
                "job:log",
                JobLogDto {
                    job_id: meta.id.clone(),
                    line: "任务数据缺失，已跳过该任务".to_string(),
                },
            );
            state.finish_active();
            continue;
        };
        let job_id = job.meta.id.clone();
        if let Some(dir) = job.temp_path.parent() {
            planner::clean_stale_temp_files(dir);
        }
        let (cancel_tx, cancel_rx) = watch::channel(false);
        state.set_cancel(&job_id, cancel_tx);

        let _ = app.emit(
            "job:started",
            JobStartedDto {
                job_id,
                name: job.meta.name.clone(),
                input_path: job.meta.input_path.clone(),
                output_path: job.final_path.display().to_string(),
                duration_secs: job.duration_secs,
                notice: job.notice.clone(),
                started_at_ms: unix_now_ms(),
            },
        );

        let task_app = app.clone();
        let task_state = state.clone();
        tauri::async_runtime::spawn(async move {
            run_job(task_app, task_state, job, cancel_rx).await;
        });
        return true;
    }
    false
}

/// 运行一个任务：转发日志与进度、写入任务历史、收尾并调度下一个任务。
async fn run_job(
    app: AppHandle,
    state: JobState,
    job: PreparedJob,
    cancel_rx: watch::Receiver<bool>,
) {
    let PreparedJob {
        meta,
        config,
        duration_secs,
        ffmpeg_path,
        args,
        input_path,
        temp_path,
        final_path,
        overwrite,
        notice,
    } = job;
    let job_id = meta.id.clone();
    let started_at_ms = unix_now_ms();

    // 事件转发：日志同时保留尾部用于写入任务历史
    let (event_tx, mut event_rx) = mpsc::unbounded_channel::<EncodeEvent>();
    let forward_app = app.clone();
    let forward_id = job_id.clone();
    let forwarder = tauri::async_runtime::spawn(async move {
        let mut tail: Vec<String> = Vec::with_capacity(LOG_TAIL_KEEP);
        while let Some(event) = event_rx.recv().await {
            match event {
                EncodeEvent::Log(line) => {
                    if tail.len() >= LOG_TAIL_KEEP {
                        tail.remove(0);
                    }
                    tail.push(line.clone());
                    let _ = forward_app.emit(
                        "job:log",
                        JobLogDto {
                            job_id: forward_id.clone(),
                            line,
                        },
                    );
                }
                EncodeEvent::Progress(snapshot) => {
                    let _ = forward_app
                        .emit("job:progress", JobProgressDto::new(&forward_id, &snapshot));
                }
            }
        }
        tail
    });

    let result = run_encode(
        EncodeRequest {
            ffmpeg_path,
            args,
            input_path,
            temp_path,
            final_path,
            overwrite,
            duration_secs,
            cancel: cancel_rx,
        },
        event_tx,
    )
    .await;
    // 等转发结束，保证日志与进度事件先于 finished 送达界面
    let log_tail = forwarder.await.unwrap_or_default();
    let finished_at_ms = unix_now_ms();

    let record = JobRecord {
        id: job_id.clone(),
        name: meta.name.clone(),
        input_path: meta.input_path.clone(),
        output_path: result.output_path.as_ref().map(|p| p.display().to_string()),
        status: result.status,
        error: result.error.clone(),
        notice: notice.clone(),
        config,
        duration_secs,
        started_at_ms,
        finished_at_ms,
        elapsed_secs: result.elapsed.as_secs_f64(),
        log_tail: truncate_log_tail(&log_tail),
    };
    let history_error = match settings::history_path(&app) {
        Ok(path) => HistoryStore::new(path).prepend(record).err(),
        Err(e) => Some(e),
    };
    if let Some(error) = &history_error {
        let _ = app.emit(
            "job:log",
            JobLogDto {
                job_id: job_id.clone(),
                line: format!("任务历史写入失败：{error}"),
            },
        );
    }

    state.clear_cancel(&job_id);
    state.finish_active();
    let _ = app.emit(
        "job:finished",
        JobFinishedDto {
            job_id,
            status: result.status,
            error: result.error,
            output_path: result.output_path.map(|p| p.display().to_string()),
            elapsed_secs: result.elapsed.as_secs_f64(),
            history_error,
        },
    );
    emit_queue(&app, &state);
    // 队列中还有任务时继续运行
    pump(&app, &state);
}

fn emit_queue(app: &AppHandle, state: &JobState) {
    let _ = app.emit("job:queue", state.snapshot());
}
