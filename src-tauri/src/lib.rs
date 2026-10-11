mod commands;
mod jobs;
mod settings;

use serde::Serialize;
use std::path::PathBuf;
use tauri::AppHandle;

#[derive(Serialize)]
struct AppInfo {
    /// 面向用户的显示名（取 `tauri.conf.json` 的 `productName`，
    /// 与安装包、任务栏、设置页展示保持一致；不用 crate 名）。
    name: String,
    version: String,
    /// 周快照版本号（编译期注入；正式构建为 None）
    snapshot: Option<&'static str>,
}

/// Returns basic application identity information for the UI.
#[tauri::command]
fn app_info(app: AppHandle) -> AppInfo {
    let package = app.package_info();
    AppInfo {
        name: package.name.clone(),
        version: package.version.to_string(),
        snapshot: option_env!("AOZORA_SNAPSHOT_VERSION"),
    }
}

/// 可执行文件所在目录（安装目录；开发时为 target 目录）。
/// dev 构建使用（WebView2 数据目录、压制临时目录）；release 构建无调用方。
#[cfg_attr(not(dev), allow(dead_code))]
pub(crate) fn exe_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_default()
}

/// 能力检测结果的 JSON 结构（集成测试用，不启动 FFmpeg）。
///
/// 前端按 camelCase 字段名读取该结构，这里构造一份固定的报告序列化一次，
/// 保证字段名与 `CapabilityReport` 的改动同步。
pub fn capability_dto_for_test() -> String {
    use aozora_core::capabilities::{CapabilityReport, EncoderAvailability, GpuAdapter};
    use aozora_core::encode::EncoderVendor;

    let report = CapabilityReport {
        encoders: vec![EncoderAvailability {
            id: "libx264".to_string(),
            label: "H.264（libx264，软件）".to_string(),
            vendor: EncoderVendor::Software,
            codec: "h264".to_string(),
            hardware: false,
            listed: true,
            usable: true,
            reason: None,
        }],
        encoder_count: 1,
        filter_count: 1,
        has_subtitles_filter: true,
        hw_accels: vec!["cuda".to_string()],
        gpus: vec![GpuAdapter {
            name: "测试适配器".to_string(),
            vendor: "nvidia".to_string(),
            driver: Some("1.2.3".to_string()),
        }],
        warnings: Vec::new(),
    };
    serde_json::to_string(&report).unwrap_or_else(|_| "{}".to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
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
            commands::settings::set_custom_tool_dir,
            commands::jobs::preview_job_args,
            commands::jobs::enqueue_job,
            commands::jobs::cancel_job,
            commands::jobs::remove_queued_job,
            commands::jobs::queue_snapshot,
            commands::presets::list_presets,
            commands::presets::create_preset,
            commands::presets::rename_preset,
            commands::presets::delete_preset,
            commands::presets::import_preset_file,
            commands::presets::export_preset_file,
            commands::history::list_job_history,
            commands::history::delete_job_history,
            commands::history::clear_job_history
        ])
        .manage(jobs::JobState::default())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
