use aozora_core::settings::{Settings, SETTINGS_SCHEMA_VERSION};
use std::fs;
use std::path::PathBuf;
use tauri::AppHandle;

pub use aozora_core::settings::validate_tool_dir;

/// dev 构建：配置放在可执行文件旁的 `config/`（本机从工作区启动的进程
/// 无法写 %APPDATA%，见 PLAN 完成记录）。
#[cfg(dev)]
fn settings_path(_app: &AppHandle) -> Result<PathBuf, String> {
    let exe_dir = std::env::current_exe()
        .map_err(|e| format!("无法确定程序目录：{e}"))?
        .parent()
        .map(|p| p.to_path_buf())
        .ok_or_else(|| "无法确定程序目录".to_string())?;
    Ok(exe_dir.join("config").join("settings.json"))
}

/// 发布构建：配置放在系统应用配置目录（%APPDATA%\com.aozora.encode）。
#[cfg(not(dev))]
fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    use tauri::Manager;
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
