use crate::jobs::{ActiveJob, JobFinishedDto, JobLogDto, JobProgressDto, JobStartedDto, JobState};
use crate::settings;
use aozora_core::encode::{self, EncodeConfig};
use aozora_core::job::{run_encode, EncodeEvent, EncodeRequest};
use aozora_core::media_probe;
use aozora_core::planner;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use tauri::{AppHandle, Emitter, State};
use tokio::sync::{mpsc, watch};

static JOB_COUNTER: AtomicU64 = AtomicU64::new(0);

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

/// 启动一个压制任务（异步命令：仅完成校验与调度，编码在后台任务中运行）。
#[tauri::command]
pub async fn start_job(
    app: AppHandle,
    state: State<'_, JobState>,
    config: EncodeConfig,
    input_path: String,
    duration_secs: Option<f64>,
) -> Result<String, String> {
    // 1. M2 单任务限制
    {
        let active = state
            .active
            .lock()
            .map_err(|_| "任务状态锁失效".to_string())?;
        if let Some(job) = active.as_ref() {
            return Err(format!(
                "已有任务正在运行（{}）。请先取消或等待其完成。",
                job.id
            ));
        }
    }

    // 2. 配置与路径校验（在启动子进程前完成，避免必然失败的任务）
    encode::validate_config(&config)?;
    let input = PathBuf::from(&input_path);
    let output = config.output.path.clone();
    planner::check_paths(&input, &output, config.output.overwrite)?;

    // 3. 解析当前生效的 FFmpeg
    let settings = settings::load(&app)?;
    let detection = media_probe::detect_tools(&settings);
    let ffmpeg = detection.ffmpeg.ok_or_else(|| {
        "未检测到可用的 FFmpeg，无法开始压制。请安装 FFmpeg 或在设置中指定路径。".to_string()
    })?;

    // 4. 生成参数数组与任务专属临时输出路径（应用自己的临时目录），
    //    并在启动前完成可写性探测：应用临时目录 + 最终输出目录。
    //    目标目录不可写时（例如受限/沙箱环境）回退到应用临时目录，
    //    保证压制始终可以完成，并通过 notice 明确告知用户成品位置。
    let job_id = next_job_id();
    let temp_dir = encode_temp_dir(&app)?;
    std::fs::create_dir_all(&temp_dir)
        .map_err(|e| format!("创建压制临时目录失败（{}）：{e}", temp_dir.display()))?;
    planner::clean_stale_temp_files(&temp_dir);
    let temp_path = planner::temp_output_path(&temp_dir, &output, &job_id);
    planner::verify_writable(&temp_path)?;
    let (final_path, fallback_notice, overwrite) =
        match planner::verify_destination_writable(&output, &job_id) {
            Ok(()) => (output.clone(), None, config.output.overwrite),
            Err(destination_error) => {
                // 回退到应用临时目录：覆盖同名回退成品（应用自有目录），
                // 并告知用户目标目录不可写的原因。
                let notice = Some(format!(
                    "目标目录不可写，成品将保存到应用临时目录：{destination_error}"
                ));
                (
                    planner::fallback_output_path(&temp_dir, &output),
                    notice,
                    true,
                )
            }
        };
    let args = planner::build_args(&config, &input, &temp_path)?;

    // 5. 注册活动任务
    let (cancel_tx, cancel_rx) = watch::channel(false);
    {
        let mut active = state
            .active
            .lock()
            .map_err(|_| "任务状态锁失效".to_string())?;
        *active = Some(ActiveJob {
            id: job_id.clone(),
            cancel_tx,
        });
    }

    // 6. 事件转发：运行器事件 → Tauri 事件
    let (event_tx, mut event_rx) = mpsc::unbounded_channel::<EncodeEvent>();
    let forward_app = app.clone();
    let forward_job_id = job_id.clone();
    let forwarder = tauri::async_runtime::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            match event {
                EncodeEvent::Log(line) => {
                    let _ = forward_app.emit(
                        "job:log",
                        JobLogDto {
                            job_id: forward_job_id.clone(),
                            line,
                        },
                    );
                }
                EncodeEvent::Progress(snapshot) => {
                    let _ = forward_app.emit(
                        "job:progress",
                        JobProgressDto::new(&forward_job_id, &snapshot),
                    );
                }
            }
        }
    });

    // 7. 后台运行；等待转发完成后再发 finished，保证事件顺序
    let run_app = app.clone();
    let run_active = state.active.clone();
    let run_id = job_id.clone();
    tauri::async_runtime::spawn(async move {
        let result = run_encode(
            EncodeRequest {
                ffmpeg_path: PathBuf::from(&ffmpeg.path),
                args,
                input_path: input,
                temp_path,
                final_path,
                overwrite,
                duration_secs,
                cancel: cancel_rx,
            },
            event_tx,
        )
        .await;
        let _ = forwarder.await;

        if let Ok(mut active) = run_active.lock() {
            if active.as_ref().map(|job| job.id == run_id).unwrap_or(false) {
                *active = None;
            }
        }
        let _ = run_app.emit(
            "job:finished",
            JobFinishedDto {
                job_id: run_id,
                status: result.status,
                error: result.error,
                output_path: result.output_path.map(|p| p.display().to_string()),
                elapsed_secs: result.elapsed.as_secs_f64(),
            },
        );
    });

    let _ = app.emit(
        "job:started",
        JobStartedDto {
            job_id: job_id.clone(),
            notice: fallback_notice,
        },
    );
    Ok(job_id)
}

/// 请求取消正在运行的任务；最终状态由运行器置为 Cancelled。
#[tauri::command]
pub fn cancel_job(state: State<'_, JobState>, job_id: String) -> Result<(), String> {
    let active = state
        .active
        .lock()
        .map_err(|_| "任务状态锁失效".to_string())?;
    let job = active
        .as_ref()
        .ok_or_else(|| "当前没有正在运行的任务".to_string())?;
    if job.id != job_id {
        return Err(format!("任务 {job_id} 不存在或已结束"));
    }
    job.cancel_tx
        .send(true)
        .map_err(|_| "任务已结束，无法取消".to_string())
}
