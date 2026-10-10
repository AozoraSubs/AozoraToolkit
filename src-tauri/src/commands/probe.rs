use crate::settings;
use aozora_core::capabilities::{self, CapabilityReport};
use aozora_core::media_probe::{self, ToolDetectionResult};
use aozora_core::probe::{self, MediaProbeResult};
use std::path::{Path, PathBuf};
use tauri::AppHandle;

/// 在阻塞线程池中执行检测，避免占用 Tauri 主线程导致界面卡顿。
async fn run_blocking<T, F>(task: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, String> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(task)
        .await
        .map_err(|e| format!("后台任务执行失败：{e}"))?
}

/// 检测 FFmpeg / FFprobe：PATH → 安装目录 bin → 自定义路径。
#[tauri::command]
pub async fn detect_tools(app: AppHandle) -> Result<ToolDetectionResult, String> {
    run_blocking(move || {
        let settings = settings::load(&app)?;
        Ok(media_probe::detect_tools(&settings))
    })
    .await
}

/// 检测当前 FFmpeg 实际可用的编码器、滤镜与硬件加速能力。
#[tauri::command]
pub async fn detect_capabilities(app: AppHandle) -> Result<CapabilityReport, String> {
    run_blocking(move || {
        let settings = settings::load(&app)?;
        let detection = media_probe::detect_tools(&settings);
        let ffmpeg = detection.ffmpeg.ok_or_else(|| {
            "未检测到可用的 FFmpeg，无法进行能力检测。请安装 FFmpeg 或在设置中指定路径。"
                .to_string()
        })?;
        Ok(capabilities::detect_capabilities(Path::new(&ffmpeg.path)))
    })
    .await
}

/// 使用 FFprobe 探测媒体文件并返回结构化信息。
#[tauri::command]
pub async fn probe_media_file(app: AppHandle, path: String) -> Result<MediaProbeResult, String> {
    run_blocking(move || {
        let file = PathBuf::from(&path);
        if !file.is_file() {
            return Err(format!("文件不存在或不是文件：{path}"));
        }
        let settings = settings::load(&app)?;
        let detection = media_probe::detect_tools(&settings);
        let ffprobe = detection.ffprobe.ok_or_else(|| {
            "未检测到可用的 FFprobe，无法探测媒体文件。请安装 FFmpeg 或在设置中指定路径。"
                .to_string()
        })?;
        probe::probe_media(Path::new(&ffprobe.path), &file)
    })
    .await
}
