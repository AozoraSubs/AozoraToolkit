use crate::settings;
use tauri::AppHandle;

/// 设置或清除自定义 FFmpeg / FFprobe 目录（校验存在且为文件夹）。
#[tauri::command]
pub fn set_custom_tool_dir(app: AppHandle, dir: Option<String>) -> Result<(), String> {
    if let Some(dir) = &dir {
        settings::validate_tool_dir(std::path::Path::new(dir))?;
    }
    let mut settings = settings::load(&app)?;
    settings.custom_tool_dir = dir.map(std::path::PathBuf::from);
    settings::save(&app, &settings)
}
