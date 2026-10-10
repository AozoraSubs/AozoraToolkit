use aozora_core::settings::{Settings, SETTINGS_SCHEMA_VERSION};
use std::fs;
use std::path::PathBuf;
use tauri::AppHandle;

pub use aozora_core::settings::validate_tool_dir;

/// 应用配置根目录（PLAN §8：settings.json / presets / history 都在这里）。
///
/// - dev 构建：可执行文件旁的 `config/`（本机从工作区启动的进程无法写
///   `%APPDATA%`，见 PLAN 完成记录）；
/// - 发布构建：系统应用配置目录 `%APPDATA%\com.aozora.encode`。
#[cfg(dev)]
pub fn config_dir(_app: &AppHandle) -> Result<PathBuf, String> {
    let exe_dir = std::env::current_exe()
        .map_err(|e| format!("无法确定程序目录：{e}"))?
        .parent()
        .map(|p| p.to_path_buf())
        .ok_or_else(|| "无法确定程序目录".to_string())?;
    Ok(exe_dir.join("config"))
}

/// 发布构建的配置根目录（见 [`config_dir`]）。
#[cfg(not(dev))]
pub fn config_dir(app: &AppHandle) -> Result<PathBuf, String> {
    use tauri::Manager;
    app.path()
        .app_config_dir()
        .map_err(|e| format!("无法获取配置目录：{e}"))
}

fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(config_dir(app)?.join("settings.json"))
}

/// 用户预设目录（`<配置根目录>/presets`）。
pub fn presets_dir(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(config_dir(app)?.join("presets"))
}

/// 任务历史文件（`<配置根目录>/history/jobs.json`）。
pub fn history_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(config_dir(app)?.join("history").join("jobs.json"))
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
