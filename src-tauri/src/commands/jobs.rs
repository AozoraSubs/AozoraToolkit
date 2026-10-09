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

    // 4. 生成参数数组与任务专属临时输出路径
    let job_id = next_job_id();
    let temp_path = planner::temp_output_path(&output, &job_id);
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
                final_path: output.clone(),
                overwrite: config.output.overwrite,
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
