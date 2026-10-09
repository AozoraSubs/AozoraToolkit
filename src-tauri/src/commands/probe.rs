use crate::settings;
use aozora_core::media_probe::{self, ToolDetectionResult};
use tauri::AppHandle;

/// 检测 FFmpeg / FFprobe：PATH → 安装目录 bin → 自定义路径。
#[tauri::command]
pub fn detect_tools(app: AppHandle) -> Result<ToolDetectionResult, String> {
    let settings = settings::load(&app)?;
    Ok(media_probe::detect_tools(&settings))
}
