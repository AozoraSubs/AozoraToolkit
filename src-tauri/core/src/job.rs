//! FFmpeg 编码运行器：子进程启动、stdout/stderr 持续读取、进度事件、
//! 取消与临时输出收尾。只依赖 tokio，不依赖 Tauri。

use crate::progress::{ProgressParser, ProgressRecord};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::{Duration, Instant};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::{mpsc, watch};

/// 进度事件节流间隔（PLAN §6.2：UI 更新节流）。
pub const PROGRESS_THROTTLE: Duration = Duration::from_millis(250);
/// 失败摘要取用的 stderr 尾部行数。
pub const ERROR_TAIL_LINES: usize = 15;
/// stderr 环形缓冲容量（用于失败摘要）。
pub const LOG_BUFFER_CAPACITY: usize = 400;

/// 面向 UI 的进度快照。
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressSnapshot {
    /// 0–100；时长未知时保持 0 且 `duration_known` 为 false。
    pub percent: f64,
    pub out_time_secs: f64,
    pub duration_known: bool,
    pub duration_secs: Option<f64>,
    pub speed: Option<f64>,
    pub fps: Option<f64>,
    pub eta_secs: Option<f64>,
}

/// 运行器向外部（UI 转发层）发出的事件。
#[derive(Debug, Clone)]
pub enum EncodeEvent {
    /// stderr 日志行。
    Log(String),
    /// 进度快照（已节流）。
    Progress(ProgressSnapshot),
}

/// 任务最终状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EncodeStatus {
    Completed,
    Failed,
    Cancelled,
}

/// 一次压制运行的结果。
#[derive(Debug, Clone)]
pub struct EncodeResult {
    pub status: EncodeStatus,
    /// 失败原因（含 stderr 尾部摘要）。
    pub error: Option<String>,
    /// 成功时的最终输出路径。
    pub output_path: Option<PathBuf>,
    pub last_progress: Option<ProgressSnapshot>,
    pub elapsed: Duration,
}

/// 运行器的输入参数。
#[derive(Debug, Clone)]
pub struct EncodeRequest {
    pub ffmpeg_path: PathBuf,
    /// 已生成的参数数组（输出为临时路径，见 [`crate::planner::build_args`]）。
    pub args: Vec<OsString>,
    pub input_path: PathBuf,
    /// FFmpeg 写入的临时输出路径。
    pub temp_path: PathBuf,
    /// 成功后重命名到的最终输出路径。
    pub final_path: PathBuf,
    /// 最终输出已存在时是否覆盖。
    pub overwrite: bool,
    /// 输入媒体时长（秒），用于进度百分比与 ETA；未知时为 None。
    pub duration_secs: Option<f64>,
    /// 取消信号（true = 请求取消）。
    pub cancel: watch::Receiver<bool>,
}

/// 由一条进度记录生成快照。
pub fn snapshot_from_record(
    record: &ProgressRecord,
    duration_secs: Option<f64>,
) -> ProgressSnapshot {
    let out = record.out_time_secs().unwrap_or(0.0);
    let is_end = record.progress.as_deref() == Some("end");
    let (percent, duration_known, duration) = match duration_secs {
        Some(d) if d > 0.0 => {
            let percent = if is_end {
                100.0
            } else {
                ((out / d) * 100.0).clamp(0.0, 100.0)
            };
            (percent, true, Some(d))
        }
        _ => (0.0, false, None),
    };
    let eta_secs = match (duration, record.speed) {
        (Some(d), Some(speed)) if speed > 0.0 && d > out => Some((d - out) / speed),
        _ => None,
    };
    ProgressSnapshot {
        percent,
        out_time_secs: out,
        duration_known,
        duration_secs: duration,
        speed: record.speed,
        fps: record.fps,
        eta_secs,
    }
}

fn tail_summary(log_tail: &VecDeque<String>) -> String {
    if log_tail.is_empty() {
        return "无错误输出".to_string();
    }
    let mut lines: Vec<&str> = log_tail
        .iter()
        .rev()
        .take(ERROR_TAIL_LINES)
        .map(String::as_str)
        .collect();
    lines.reverse();
    lines.join(" | ")
}

/// 可读的退出结果描述。Windows 下 FFmpeg 会以负 errno 退出
/// （如权限不足为 -13，显示为 exit code: 0xfffffff3），这里解码常见值。
fn describe_exit(exit: &Result<std::process::ExitStatus, std::io::Error>) -> String {
    match exit {
        Ok(status) => {
            let base = format!("退出码：{status}");
            match status.code() {
                Some(code) if code < 0 => {
                    let hint = match code {
                        -13 => "（权限不足：输出目录不可写，或目标文件被占用/只读）",
                        -2 => "（路径不存在）",
                        _ => "（负 errno，见上方错误详情）",
                    };
                    format!("{base}（{code}）{hint}")
                }
                _ => base,
            }
        }
        Err(e) => format!("等待进程退出失败：{e}"),
    }
}

/// 成功时把临时输出移动到最终输出（覆盖策略在此执行）。
///
/// 临时输出位于应用自己的临时目录：与目标同卷时用重命名（原子且零拷贝）；
/// 跨卷重命名失败时回退为"复制到目的地 + 删除临时文件"。
fn finalize_output(request: &EncodeRequest) -> Result<(), String> {
    let target = &request.final_path;
    if target.exists() {
        if !request.overwrite {
            return Err(format!(
                "输出文件已存在（未启用覆盖）：{}",
                target.display()
            ));
        }
        if target.is_dir() {
            return Err(format!("输出路径是文件夹：{}", target.display()));
        }
        // Windows 下只读属性的文件无法被删除：先清属性再移除。
        crate::planner::clear_readonly(target);
        std::fs::remove_file(target)
            .map_err(|e| format!("无法移除已存在的输出文件（{}）：{e}", target.display()))?;
    }
    std::fs::rename(&request.temp_path, target).or_else(|rename_err| {
        // 跨卷时重命名不可用：复制到目的地，成功后删除临时文件。
        std::fs::copy(&request.temp_path, target)
            .map(|_| {
                let _ = std::fs::remove_file(&request.temp_path);
            })
            .map_err(|copy_err| {
                format!(
                    "移动临时输出失败（{} → {}）：{rename_err}；复制回退也失败：{copy_err}",
                    request.temp_path.display(),
                    target.display()
                )
            })
    })
}

/// 运行一次压制：持续读取 stdout（进度）与 stderr（日志），
/// 响应取消信号，成功后收尾临时输出。
pub async fn run_encode(
    mut request: EncodeRequest,
    events: mpsc::UnboundedSender<EncodeEvent>,
) -> EncodeResult {
    let started = Instant::now();

    // 清理同名残留临时文件（任务 ID 唯一，仅可能是上次异常退出的遗留）。
    if request.temp_path.exists() {
        let _ = std::fs::remove_file(&request.temp_path);
    }

    // 启动前已收到取消请求：直接按取消处理，不再启动 FFmpeg。
    if *request.cancel.borrow() {
        return EncodeResult {
            status: EncodeStatus::Cancelled,
            error: None,
            output_path: None,
            last_progress: None,
            elapsed: started.elapsed(),
        };
    }

    let mut ffmpeg_command = Command::new(&request.ffmpeg_path);
    crate::process_flags::hide_console_tokio(&mut ffmpeg_command);
    let mut child = match ffmpeg_command
        .args(&request.args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
    {
        Ok(child) => child,
        Err(e) => {
            let message = format!("无法启动 {}：{e}", request.ffmpeg_path.display());
            let _ = events.send(EncodeEvent::Log(message.clone()));
            return EncodeResult {
                status: EncodeStatus::Failed,
                error: Some(message),
                output_path: None,
                last_progress: None,
                elapsed: started.elapsed(),
            };
        }
    };

    let stdout = child.stdout.take().expect("stdout 已配置为管道");
    let stderr = child.stderr.take().expect("stderr 已配置为管道");
    let mut stdout_lines = BufReader::new(stdout).lines();
    let mut stderr_lines = BufReader::new(stderr).lines();

    let mut parser = ProgressParser::new();
    let mut last_progress: Option<ProgressSnapshot> = None;
    // 初始化为"已过期"，使第一条进度事件立即发出。
    let mut last_emit = started.checked_sub(PROGRESS_THROTTLE).unwrap_or(started);
    let mut log_tail: VecDeque<String> = VecDeque::with_capacity(LOG_BUFFER_CAPACITY);
    let mut cancel_requested = false;
    let mut exit_result: Option<Result<std::process::ExitStatus, std::io::Error>> = None;
    let mut stdout_done = false;
    let mut stderr_done = false;

    loop {
        if exit_result.is_some() && stdout_done && stderr_done {
            break;
        }
        tokio::select! {
            // 进度（stdout，-progress pipe:1）
            line = stdout_lines.next_line(), if !stdout_done => {
                match line {
                    Ok(Some(line)) => {
                        if let Some(record) = parser.push_line(&line) {
                            let snapshot = snapshot_from_record(&record, request.duration_secs);
                            let is_end = record.progress.as_deref() == Some("end");
                            last_progress = Some(snapshot.clone());
                            if is_end || last_emit.elapsed() >= PROGRESS_THROTTLE {
                                last_emit = Instant::now();
                                let _ = events.send(EncodeEvent::Progress(snapshot));
                            }
                        }
                    }
                    Ok(None) => stdout_done = true,
                    Err(_) => stdout_done = true,
                }
            }
            // 日志（stderr）
            line = stderr_lines.next_line(), if !stderr_done => {
                match line {
                    Ok(Some(line)) => {
                        if log_tail.len() == LOG_BUFFER_CAPACITY {
                            log_tail.pop_front();
                        }
                        log_tail.push_back(line.clone());
                        let _ = events.send(EncodeEvent::Log(line));
                    }
                    Ok(None) => stderr_done = true,
                    Err(_) => stderr_done = true,
                }
            }
            // 子进程退出
            status = child.wait(), if exit_result.is_none() => {
                exit_result = Some(status);
            }
            // 取消信号
            changed = request.cancel.changed(), if !cancel_requested => {
                if changed.is_ok() && *request.cancel.borrow() {
                    cancel_requested = true;
                    let _ = events.send(EncodeEvent::Log(
                        "收到取消请求，正在终止 FFmpeg 进程…".to_string(),
                    ));
                    // Windows 上为 TerminateProcess，进程被强制结束。
                    let _ = child.start_kill();
                }
            }
        }
    }

    // 冲刷未以 progress= 结束的残块（部分构建可能在结束时缺少结束行）。
    if let Some(record) = parser.flush() {
        let snapshot = snapshot_from_record(&record, request.duration_secs);
        if last_progress.as_ref() != Some(&snapshot) {
            last_progress = Some(snapshot.clone());
            let _ = events.send(EncodeEvent::Progress(snapshot));
        }
    }

    let exit = exit_result.expect("循环退出时必有退出结果");
    let exit_success = exit.as_ref().map(|s| s.success()).unwrap_or(false);
    let status = if cancel_requested {
        EncodeStatus::Cancelled
    } else if exit_success {
        EncodeStatus::Completed
    } else {
        EncodeStatus::Failed
    };

    match status {
        EncodeStatus::Completed => {
            // 成功收尾：确保最终进度显示 100%（时长已知时）。
            if let Some(progress) = last_progress.as_mut() {
                if progress.duration_known {
                    progress.percent = 100.0;
                }
            }
            match finalize_output(&request) {
                Ok(()) => EncodeResult {
                    status: EncodeStatus::Completed,
                    error: None,
                    output_path: Some(request.final_path.clone()),
                    last_progress,
                    elapsed: started.elapsed(),
                },
                Err(e) => {
                    let _ = std::fs::remove_file(&request.temp_path);
                    EncodeResult {
                        status: EncodeStatus::Failed,
                        error: Some(e),
                        output_path: None,
                        last_progress,
                        elapsed: started.elapsed(),
                    }
                }
            }
        }
        EncodeStatus::Failed => {
            let _ = std::fs::remove_file(&request.temp_path);
            let summary = tail_summary(&log_tail);
            let mut error = format!("FFmpeg {}。{}", describe_exit(&exit), summary);
            // FFmpeg 对权限问题只报 "Permission denied"，补充可操作的排查提示。
            if summary.contains("Permission denied") || summary.contains("Access is denied") {
                error.push_str(
                    "。提示：输出目录可能无写入权限、目标文件被其他程序占用或为只读，也可能被杀毒软件或“受控文件夹访问”拦截",
                );
            }
            EncodeResult {
                status: EncodeStatus::Failed,
                error: Some(error),
                output_path: None,
                last_progress,
                elapsed: started.elapsed(),
            }
        }
        EncodeStatus::Cancelled => {
            let _ = std::fs::remove_file(&request.temp_path);
            EncodeResult {
                status: EncodeStatus::Cancelled,
                error: None,
                output_path: None,
                last_progress,
                elapsed: started.elapsed(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encode;
    use crate::encode::EncodeConfig;
    use crate::media_probe::find_in_path;
    use crate::planner;
    use std::path::Path;

    fn temp_dir(tag: &str) -> PathBuf {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("test-tmp")
            .join(format!("aozora-job-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn ffmpeg_on_path() -> PathBuf {
        let path_entries: Vec<PathBuf> = std::env::var_os("PATH")
            .map(|p| std::env::split_paths(&p).collect())
            .unwrap_or_default();
        find_in_path("ffmpeg", &path_entries).expect("ffmpeg 未在 PATH 中检测到")
    }

    fn config_for(encoder: &str, preset: &str, quality: u32) -> EncodeConfig {
        let spec = encode::encoder_spec(encoder).expect("未知编码器");
        let quality_mode = match spec.family {
            encode::EncoderFamily::Software => encode::QualityMode::Crf,
            encode::EncoderFamily::Nvenc => encode::QualityMode::Cq,
        };
        let mut config = EncodeConfig::default();
        config.video.encoder_id = encoder.to_string();
        config.video.quality_mode = quality_mode;
        config.video.preset = preset.to_string();
        config.video.quality_value = quality;
        config.video.pixel_format = None;
        config
    }

    fn collect_events() -> (
        mpsc::UnboundedSender<EncodeEvent>,
        mpsc::UnboundedReceiver<EncodeEvent>,
    ) {
        mpsc::unbounded_channel()
    }

    /// 用真实 ffmpeg 生成测试输入（testsrc + 正弦音频）。
    fn generate_input(ffmpeg: &Path, output: &Path, seconds: u32) {
        let mut command = std::process::Command::new(ffmpeg);
        command
            .args(["-y", "-f", "lavfi", "-i"])
            .arg("testsrc=size=320x240:rate=25")
            .args([
                "-f",
                "lavfi",
                "-i",
                "sine=frequency=440:sample_rate=48000",
                "-t",
            ])
            .arg(seconds.to_string())
            .args([
                "-c:v", "libx264", "-pix_fmt", "yuv420p", "-ac", "2", "-c:a", "aac",
            ])
            .arg(output);
        let status = command.output().expect("生成测试输入失败");
        assert!(
            status.status.success(),
            "ffmpeg 生成测试输入失败：{}",
            String::from_utf8_lossy(&status.stderr)
                .lines()
                .last()
                .unwrap_or("无错误输出")
        );
    }

    fn all_events(receiver: &mut mpsc::UnboundedReceiver<EncodeEvent>) -> Vec<EncodeEvent> {
        let mut events = Vec::new();
        while let Ok(event) = receiver.try_recv() {
            events.push(event);
        }
        events
    }

    #[test]
    fn snapshot_computes_percent_and_eta() {
        let record = ProgressRecord {
            out_time_us: Some(5_000_000),
            speed: Some(2.0),
            progress: Some("continue".to_string()),
            ..ProgressRecord::default()
        };
        let snapshot = snapshot_from_record(&record, Some(20.0));
        assert!(snapshot.duration_known);
        assert_eq!(snapshot.percent, 25.0);
        assert_eq!(snapshot.eta_secs, Some(7.5));

        let snapshot = snapshot_from_record(&record, None);
        assert!(!snapshot.duration_known);
        assert_eq!(snapshot.percent, 0.0);
        assert_eq!(snapshot.eta_secs, None);
    }

    #[test]
    fn snapshot_clamps_percent_and_forces_end() {
        let mut record = ProgressRecord {
            out_time_us: Some(21_000_000),
            progress: Some("continue".to_string()),
            ..ProgressRecord::default()
        };
        assert_eq!(snapshot_from_record(&record, Some(20.0)).percent, 100.0);

        record.progress = Some("end".to_string());
        record.out_time_us = Some(1_000_000);
        assert_eq!(snapshot_from_record(&record, Some(20.0)).percent, 100.0);
    }

    #[test]
    fn snapshot_eta_requires_positive_speed_and_remaining() {
        let mut record = ProgressRecord {
            out_time_us: Some(2_000_000),
            speed: Some(0.0),
            ..ProgressRecord::default()
        };
        assert_eq!(snapshot_from_record(&record, Some(20.0)).eta_secs, None);
        record.speed = Some(10.0);
        record.out_time_us = Some(20_000_000);
        assert_eq!(snapshot_from_record(&record, Some(20.0)).eta_secs, None);
    }

    #[cfg(windows)]
    #[test]
    fn finalize_output_overwrites_readonly_target() {
        let dir = temp_dir("finalize-readonly");
        let temp = dir.join("tmp.aozora-part-e-1.mkv");
        let target = dir.join("out.mkv");
        std::fs::write(&temp, "新内容").unwrap();
        std::fs::write(&target, "旧内容").unwrap();
        let mut permissions = std::fs::metadata(&target).unwrap().permissions();
        permissions.set_readonly(true);
        std::fs::set_permissions(&target, permissions).unwrap();

        let request = EncodeRequest {
            ffmpeg_path: PathBuf::new(),
            args: Vec::new(),
            input_path: dir.join("in.mkv"),
            temp_path: temp.clone(),
            final_path: target.clone(),
            overwrite: true,
            duration_secs: None,
            cancel: watch::channel(false).1,
        };
        assert!(finalize_output(&request).is_ok());
        assert_eq!(std::fs::read(&target).unwrap(), "新内容".as_bytes());
        assert!(!temp.exists(), "临时文件应已重命名");
        std::fs::remove_dir_all(&dir).ok();
    }

    /// 实机验证：真实 ffmpeg 完成一次压制（非 ASCII / 空格 / 括号路径），
    /// 验证进度事件、成功收尾（临时文件重命名）与输出可被 FFprobe 读取。
    #[tokio::test]
    #[ignore = "requires ffmpeg and ffprobe on PATH"]
    async fn encode_success_with_non_ascii_paths() {
        use crate::probe::probe_media;

        let ffmpeg = ffmpeg_on_path();
        let path_entries: Vec<PathBuf> = std::env::var_os("PATH")
            .map(|p| std::env::split_paths(&p).collect())
            .unwrap_or_default();
        let ffprobe = find_in_path("ffprobe", &path_entries).expect("ffprobe 未在 PATH 中检测到");

        // 目录与文件名含空格、中文与括号；临时文件写入独立的应用临时目录
        let dir = temp_dir("success").join("测试 目录 (1)");
        let app_tmp = temp_dir("success").join("encode-tmp");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(&app_tmp).unwrap();
        let input = dir.join("样片 [demo].mkv");
        let output = dir.join("样片 [demo].encoded.mkv");
        generate_input(&ffmpeg, &input, 2);

        let duration_secs = probe_media(&ffprobe, &input)
            .expect("输入探测失败")
            .duration_secs;

        let config = config_for("libx264", "veryfast", 23);
        let job_id = "e-test-success";
        let temp = planner::temp_output_path(&app_tmp, &output, job_id);
        let args = planner::build_args(&config, &input, &temp).unwrap();
        let (_cancel_tx, cancel_rx) = watch::channel(false);
        let (event_tx, mut event_rx) = collect_events();

        let result = run_encode(
            EncodeRequest {
                ffmpeg_path: ffmpeg,
                args,
                input_path: input.clone(),
                temp_path: temp.clone(),
                final_path: output.clone(),
                overwrite: false,
                duration_secs,
                cancel: cancel_rx,
            },
            event_tx,
        )
        .await;

        assert_eq!(
            result.status,
            EncodeStatus::Completed,
            "error: {:?}",
            result.error
        );
        assert!(output.is_file(), "最终输出不存在");
        assert!(!temp.exists(), "临时文件未清理");
        let progress = result.last_progress.expect("缺少最终进度");
        assert_eq!(progress.percent, 100.0);

        let events = all_events(&mut event_rx);
        assert!(
            events.iter().any(|e| matches!(e, EncodeEvent::Progress(_))),
            "未收到进度事件"
        );
        assert!(
            events.iter().any(|e| matches!(e, EncodeEvent::Log(_))),
            "未收到日志事件"
        );

        // 输出可被 FFprobe 读取且保留视频/音频流
        let info = probe_media(&ffprobe, &output).expect("输出探测失败");
        assert_eq!(info.video_streams.len(), 1);
        assert_eq!(info.audio_streams.len(), 1);
        assert!(info.duration_secs.is_some_and(|d| d > 0.0));

        std::fs::remove_dir_all(dir).ok();
        std::fs::remove_dir_all(app_tmp).ok();
    }

    /// 实机验证：输出封装格式由配置里的 container 决定（MP4 输出可读、扩展名一致）。
    #[tokio::test]
    #[ignore = "requires ffmpeg and ffprobe on PATH"]
    async fn encode_honours_configured_container() {
        use crate::encode::Container;
        use crate::probe::probe_media;

        let path_entries: Vec<PathBuf> = std::env::var_os("PATH")
            .map(|value| std::env::split_paths(&value).collect())
            .unwrap_or_default();
        let ffmpeg = find_in_path("ffmpeg", &path_entries).expect("ffmpeg 未在 PATH 中检测到");
        let ffprobe = find_in_path("ffprobe", &path_entries).expect("ffprobe 未在 PATH 中检测到");

        let dir = temp_dir("container");
        std::fs::create_dir_all(&dir).unwrap();
        let input = dir.join("input.mkv");
        generate_input(&ffmpeg, &input, 1);

        for (container, ext) in [(Container::Mkv, "mkv"), (Container::Mp4, "mp4")] {
            let output = dir.join(format!("out-{ext}.{ext}"));
            let mut config = config_for("libx264", "veryfast", 23);
            config.output.container = container;
            config.output.path = output.clone();
            let temp = planner::temp_output_path(&dir, &output, "e-test-container");
            // 临时输出沿用最终扩展名，FFmpeg 依扩展名选择封装器
            assert_eq!(temp.extension().unwrap(), ext);
            let args = planner::build_args(&config, &input, &temp).unwrap();
            let (_cancel_tx, cancel_rx) = watch::channel(false);
            let (event_tx, _event_rx) = collect_events();

            let result = run_encode(
                EncodeRequest {
                    ffmpeg_path: ffmpeg.clone(),
                    args,
                    input_path: input.clone(),
                    temp_path: temp.clone(),
                    final_path: output.clone(),
                    overwrite: false,
                    duration_secs: None,
                    cancel: cancel_rx,
                },
                event_tx,
            )
            .await;

            assert_eq!(
                result.status,
                EncodeStatus::Completed,
                "{ext} 失败：{:?}",
                result.error
            );
            let info = probe_media(&ffprobe, &output).expect("输出探测失败");
            // MP4 与 MKV 都应保留视频与音频流
            assert_eq!(info.video_streams.len(), 1, "{ext} 视频流数量不符");
            assert_eq!(info.audio_streams.len(), 1, "{ext} 音频流数量不符");
            assert!(!temp.exists(), "{ext} 临时文件未清理");
        }
        std::fs::remove_dir_all(&dir).ok();
    }

    /// 实机验证：输入不存在 → 失败，临时文件被清理，不产生成品。
    #[tokio::test]
    #[ignore = "requires ffmpeg on PATH"]
    async fn encode_failure_cleans_temp() {
        let ffmpeg = ffmpeg_on_path();
        let dir = temp_dir("failure");
        let missing_input = dir.join("不存在的输入.mkv");
        let output = dir.join("out.mkv");

        let config = config_for("libx264", "veryfast", 23);
        let temp = planner::temp_output_path(&dir, &output, "e-test-failure");
        let args = planner::build_args(&config, &missing_input, &temp).unwrap();
        let (_cancel_tx, cancel_rx) = watch::channel(false);
        let (event_tx, mut event_rx) = collect_events();

        let result = run_encode(
            EncodeRequest {
                ffmpeg_path: ffmpeg,
                args,
                input_path: missing_input,
                temp_path: temp.clone(),
                final_path: output.clone(),
                overwrite: false,
                duration_secs: None,
                cancel: cancel_rx,
            },
            event_tx,
        )
        .await;

        assert_eq!(result.status, EncodeStatus::Failed);
        let error = result.error.expect("失败应带错误信息");
        assert!(!error.is_empty());
        assert!(!temp.exists(), "失败后临时文件应被清理");
        assert!(!output.exists(), "失败后不应出现成品");
        let _ = all_events(&mut event_rx);
        std::fs::remove_dir_all(&dir).ok();
    }

    /// 实机验证：编码中途取消 → 状态 Cancelled，临时文件被清理，无成品。
    /// 使用 veryslow 预设拉长编码时长（320×240 testsrc 在 veryfast 下
    /// 仅需约 0.5 秒，取消信号会落在任务结束之后）。
    #[tokio::test]
    #[ignore = "requires ffmpeg on PATH"]
    async fn encode_cancel_kills_process_and_cleans_temp() {
        let ffmpeg = ffmpeg_on_path();
        let dir = temp_dir("cancel");
        let input = dir.join("长样片.mkv");
        let output = dir.join("out.mkv");
        generate_input(&ffmpeg, &input, 120);

        let config = config_for("libx264", "veryslow", 23);
        let temp = planner::temp_output_path(&dir, &output, "e-test-cancel");
        let args = planner::build_args(&config, &input, &temp).unwrap();
        let (cancel_tx, cancel_rx) = watch::channel(false);
        let (event_tx, mut event_rx) = collect_events();

        let handle = tokio::spawn(run_encode(
            EncodeRequest {
                ffmpeg_path: ffmpeg,
                args,
                input_path: input,
                temp_path: temp.clone(),
                final_path: output.clone(),
                overwrite: false,
                duration_secs: Some(120.0),
                cancel: cancel_rx,
            },
            event_tx,
        ));

        // 等待首个进度事件后再取消，确保编码已真正开始。
        let mut saw_progress = false;
        for _ in 0..100 {
            if event_rx
                .try_recv()
                .is_ok_and(|e| matches!(e, EncodeEvent::Progress(_)))
            {
                saw_progress = true;
                break;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        assert!(saw_progress, "取消前未观察到任何进度事件");
        cancel_tx.send(true).expect("取消信号发送失败");

        let result = handle.await.expect("运行器任务异常结束");
        assert_eq!(
            result.status,
            EncodeStatus::Cancelled,
            "error: {:?}",
            result.error
        );
        assert!(!temp.exists(), "取消后临时文件应被清理");
        assert!(!output.exists(), "取消后不应出现成品");
        std::fs::remove_dir_all(&dir).ok();
    }

    /// 实机验证：成功收尾遵循覆盖策略（覆盖开启时替换旧文件）。
    #[tokio::test]
    #[ignore = "requires ffmpeg on PATH"]
    async fn encode_overwrite_replaces_existing_output() {
        let ffmpeg = ffmpeg_on_path();
        let dir = temp_dir("overwrite");
        let input = dir.join("in.mkv");
        let output = dir.join("out.mkv");
        generate_input(&ffmpeg, &input, 1);
        std::fs::write(&output, "旧内容").unwrap();

        let config = config_for("libx264", "veryfast", 23);
        let temp = planner::temp_output_path(&dir, &output, "e-test-overwrite");
        let args = planner::build_args(&config, &input, &temp).unwrap();
        let (_cancel_tx, cancel_rx) = watch::channel(false);
        let (event_tx, _event_rx) = collect_events();

        let result = run_encode(
            EncodeRequest {
                ffmpeg_path: ffmpeg,
                args,
                input_path: input,
                temp_path: temp.clone(),
                final_path: output.clone(),
                overwrite: true,
                duration_secs: None,
                cancel: cancel_rx,
            },
            event_tx,
        )
        .await;

        assert_eq!(
            result.status,
            EncodeStatus::Completed,
            "error: {:?}",
            result.error
        );
        assert!(!temp.exists(), "临时文件未清理");
        assert_ne!(
            std::fs::read(&output).unwrap(),
            "旧内容".as_bytes(),
            "旧输出未被替换"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    /// 实机验证：NVENC 编码器参数（p1/p7 预设 + CQ）可被真实 FFmpeg 接受。
    /// 需要 NVIDIA GPU；无 GPU 的机器上该测试会失败（仅显式 --ignored 运行）。
    #[tokio::test]
    #[ignore = "requires ffmpeg on PATH and an NVIDIA GPU"]
    async fn encode_nvenc_presets_and_cq_are_accepted() {
        let ffmpeg = ffmpeg_on_path();
        let dir = temp_dir("nvenc");
        let input = dir.join("in.mkv");
        // NVENC 有最小分辨率限制，使用 1280x720 输入。
        generate_input_720p(&ffmpeg, &input, 1);

        for (encoder, preset) in [
            ("h264_nvenc", "p1"),
            ("h264_nvenc", "p7"),
            ("hevc_nvenc", "p5"),
            ("av1_nvenc", "p5"),
        ] {
            let output = dir.join(format!("{encoder}-{preset}.mkv"));
            let config = config_for(encoder, preset, 21);
            let temp = planner::temp_output_path(&dir, &output, "e-test-nvenc");
            let args = planner::build_args(&config, &input, &temp).unwrap();
            let (_cancel_tx, cancel_rx) = watch::channel(false);
            let (event_tx, _event_rx) = collect_events();

            let result = run_encode(
                EncodeRequest {
                    ffmpeg_path: ffmpeg.clone(),
                    args,
                    input_path: input.clone(),
                    temp_path: temp.clone(),
                    final_path: output.clone(),
                    overwrite: false,
                    duration_secs: None,
                    cancel: cancel_rx,
                },
                event_tx,
            )
            .await;

            assert_eq!(
                result.status,
                EncodeStatus::Completed,
                "{encoder} {preset} 失败：{:?}",
                result.error
            );
            assert!(output.is_file(), "{encoder} 输出不存在");
            assert!(!temp.exists(), "{encoder} 临时文件未清理");
        }
        std::fs::remove_dir_all(&dir).ok();
    }

    /// 生成 720p 测试输入（NVENC 最小分辨率限制）。
    fn generate_input_720p(ffmpeg: &Path, output: &Path, seconds: u32) {
        let status = std::process::Command::new(ffmpeg)
            .args(["-y", "-f", "lavfi", "-i"])
            .arg("testsrc=size=1280x720:rate=25")
            .args(["-t"])
            .arg(seconds.to_string())
            .args(["-c:v", "libx264", "-pix_fmt", "yuv420p"])
            .arg(output)
            .output()
            .expect("生成 720p 测试输入失败");
        assert!(
            status.status.success(),
            "ffmpeg 生成 720p 测试输入失败：{}",
            String::from_utf8_lossy(&status.stderr)
                .lines()
                .last()
                .unwrap_or("无错误输出")
        );
    }

    /// 实机验证：字幕保留（复制）与丢弃两种模式。
    /// 生成带字幕轨（SRT，含中文）的 MKV 输入，分别以 copy / drop
    /// 压制，验证输出字幕轨的有无。
    #[tokio::test]
    #[ignore = "requires ffmpeg and ffprobe on PATH"]
    async fn encode_preserves_and_drops_subtitles() {
        use crate::encode::SubtitleMode;
        use crate::probe::probe_media;

        let ffmpeg = ffmpeg_on_path();
        let path_entries: Vec<PathBuf> = std::env::var_os("PATH")
            .map(|p| std::env::split_paths(&p).collect())
            .unwrap_or_default();
        let ffprobe = find_in_path("ffprobe", &path_entries).expect("ffprobe 未在 PATH 中检测到");

        let dir = temp_dir("subtitles");
        let base = dir.join("in.mp4");
        generate_input(&ffmpeg, &base, 2);
        let srt = dir.join("字幕.srt");
        std::fs::write(
            &srt,
            "1\r\n00:00:00,000 --> 00:00:01,000\r\n中文字幕 测试\r\n\r\n\
             2\r\n00:00:01,000 --> 00:00:02,000\r\nSecond line\r\n",
        )
        .unwrap();
        let input = dir.join("带字幕.mkv");
        let mux = std::process::Command::new(&ffmpeg)
            .args(["-y", "-i"])
            .arg(&base)
            .arg("-i")
            .arg(&srt)
            .args(["-map", "0", "-map", "1", "-c", "copy", "-c:s", "srt"])
            .arg(&input)
            .output()
            .expect("封装字幕 MKV 失败");
        assert!(
            mux.status.success(),
            "封装字幕 MKV 失败：{}",
            String::from_utf8_lossy(&mux.stderr)
                .lines()
                .last()
                .unwrap_or("无错误输出")
        );
        let input_info = probe_media(&ffprobe, &input).expect("输入探测失败");
        assert_eq!(input_info.subtitle_streams.len(), 1, "输入应含 1 条字幕轨");

        for (mode, expect_subtitles) in [(SubtitleMode::Copy, true), (SubtitleMode::Drop, false)] {
            let output = dir.join(format!("out-{mode:?}.mkv"));
            let mut config = config_for("libx264", "veryfast", 23);
            config.subtitles.mode = mode;
            let temp = planner::temp_output_path(&dir, &output, "e-test-sub");
            let args = planner::build_args(&config, &input, &temp).unwrap();
            let (_cancel_tx, cancel_rx) = watch::channel(false);
            let (event_tx, _event_rx) = collect_events();

            let result = run_encode(
                EncodeRequest {
                    ffmpeg_path: ffmpeg.clone(),
                    args,
                    input_path: input.clone(),
                    temp_path: temp.clone(),
                    final_path: output.clone(),
                    overwrite: false,
                    duration_secs: None,
                    cancel: cancel_rx,
                },
                event_tx,
            )
            .await;

            assert_eq!(
                result.status,
                EncodeStatus::Completed,
                "{mode:?} 失败：{:?}",
                result.error
            );
            let out_info = probe_media(&ffprobe, &output).expect("输出探测失败");
            if expect_subtitles {
                assert_eq!(
                    out_info.subtitle_streams.len(),
                    1,
                    "{mode:?} 应保留 1 条字幕轨"
                );
                assert_eq!(
                    out_info.subtitle_streams[0].codec_name, "subrip",
                    "{mode:?} 字幕编码应为 subrip"
                );
            } else {
                assert!(
                    out_info.subtitle_streams.is_empty(),
                    "{mode:?} 不应保留字幕轨"
                );
            }
            assert!(!temp.exists(), "{mode:?} 临时文件未清理");
        }
        std::fs::remove_dir_all(&dir).ok();
    }

    /// 实机验证：外部字幕文件（SRT）烧录进画面（libass `subtitles` 滤镜）。
    #[tokio::test]
    #[ignore = "requires ffmpeg and ffprobe on PATH with libass"]
    async fn encode_burns_external_subtitles() {
        use crate::encode::SubtitleMode;
        use crate::probe::probe_media;

        let ffmpeg = ffmpeg_on_path();
        let path_entries: Vec<PathBuf> = std::env::var_os("PATH")
            .map(|p| std::env::split_paths(&p).collect())
            .unwrap_or_default();
        let ffprobe = find_in_path("ffprobe", &path_entries).expect("ffprobe 未在 PATH 中检测到");

        let dir = temp_dir("burn");
        let input = dir.join("in.mkv");
        generate_input(&ffmpeg, &input, 2);
        // 含中文与括号的文件名，覆盖转义路径
        let srt = dir.join("字幕 [外挂].srt");
        std::fs::write(
            &srt,
            "1\r\n00:00:00,000 --> 00:00:02,000\r\n中文字幕 烧录测试\r\n",
        )
        .unwrap();

        let mut config = config_for("libx264", "veryfast", 23);
        config.subtitles.mode = SubtitleMode::Burn;
        config.subtitles.external_path = Some(srt.clone());

        let output = dir.join("out.mkv");
        let temp = planner::temp_output_path(&dir, &output, "e-test-burn");
        let args = planner::build_args(&config, &input, &temp).unwrap();
        let (_cancel_tx, cancel_rx) = watch::channel(false);
        let (event_tx, mut event_rx) = collect_events();

        let result = run_encode(
            EncodeRequest {
                ffmpeg_path: ffmpeg.clone(),
                args,
                input_path: input.clone(),
                temp_path: temp.clone(),
                final_path: output.clone(),
                overwrite: false,
                duration_secs: None,
                cancel: cancel_rx,
            },
            event_tx,
        )
        .await;

        assert_eq!(
            result.status,
            EncodeStatus::Completed,
            "烧录失败：{:?}",
            result.error
        );
        assert!(output.is_file(), "输出不存在");
        assert!(!temp.exists(), "临时文件未清理");

        // libass 滤镜初始化日志证明字幕真实渲染，而非仅仅拼接参数
        let events = all_events(&mut event_rx);
        assert!(
            events.iter().any(|e| matches!(
                e,
                EncodeEvent::Log(line) if line.contains("libass")
            )),
            "日志应包含 libass 滤镜初始化信息"
        );

        // 输出可探测且时长一致；烧录模式下输出不应含字幕轨
        let out_info = probe_media(&ffprobe, &output).expect("输出探测失败");
        assert!(
            out_info.subtitle_streams.is_empty(),
            "烧录模式不应保留字幕轨"
        );
        assert!(out_info.duration_secs.is_some_and(|d| d > 0.0));

        std::fs::remove_dir_all(&dir).ok();
    }
}
