mod commands;
mod settings;

use serde::Serialize;
use std::path::PathBuf;

#[derive(Serialize)]
struct AppInfo {
    name: &'static str,
    version: &'static str,
}

/// Returns basic application identity information for the UI.
#[tauri::command]
fn app_info() -> AppInfo {
    AppInfo {
        name: env!("CARGO_PKG_NAME"),
        version: env!("CARGO_PKG_VERSION"),
    }
}

/// 可执行文件所在目录（安装目录；开发时为 target 目录）。
fn exe_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_default()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // 主窗口在配置中标记为 `create: false`，在此手动创建，
            // 以便 dev 构建指定可执行文件旁的 WebView2 数据目录。
            if let Some(config) = app.config().app.windows.first() {
                let builder = tauri::WebviewWindowBuilder::from_config(app.handle(), config)?;
                // dev 构建：本机从工作区启动的进程无法写 %LOCALAPPDATA%（见 PLAN
                // 完成记录），因此数据目录放在可执行文件旁（工作区内可写）。
                // 发布构建保持默认位置（%LOCALAPPDATA%\com.aozora.encode）。
                #[cfg(dev)]
                let builder = builder.data_directory(exe_dir().join("webview-data"));
                builder.build()?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            app_info,
            commands::probe::detect_tools,
            commands::probe::detect_capabilities,
            commands::probe::probe_media_file,
            commands::settings::set_custom_tool_dir
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
