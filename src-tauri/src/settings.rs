use aozora_core::settings::{Settings, SETTINGS_SCHEMA_VERSION};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

pub use aozora_core::settings::validate_tool_dir;

fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_config_dir()
        .map(|dir| dir.join("settings.json"))
        .map_err(|e| format!("无法获取配置目录：{e}"))
}

/// 读取设置；文件不存在时返回默认值。未知 schema 版本返回可读错误。
pub fn load(app: &AppHandle) -> Result<Settings, String> {
    let path = settings_path(app)?;
    if !path.exists() {
        return Ok(Settings::default());
    }
    let text = fs::read_to_string(&path)
        .map_err(|e| format!("读取设置文件失败（{}）：{e}", path.display()))?;
    let settings: Settings = serde_json::from_str(&text)
        .map_err(|e| format!("设置文件格式无效（{}）：{e}", path.display()))?;
    if settings.schema_version != SETTINGS_SCHEMA_VERSION {
        return Err(format!(
            "设置文件版本不受支持：{}（当前支持版本 {}）",
            settings.schema_version, SETTINGS_SCHEMA_VERSION
        ));
    }
    Ok(settings)
}

/// 写入设置，必要时创建目录。
pub fn save(app: &AppHandle, settings: &Settings) -> Result<(), String> {
    let path = settings_path(app)?;
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)
            .map_err(|e| format!("创建配置目录失败（{}）：{e}", dir.display()))?;
    }
    let text =
        serde_json::to_string_pretty(settings).map_err(|e| format!("序列化设置失败：{e}"))?;
    fs::write(&path, text).map_err(|e| format!("写入设置文件失败（{}）：{e}", path.display()))
}
