//! 预设命令层：列出、保存、重命名、删除、导入与导出。
//!
//! 文件读写全部在 Rust 侧完成（前端不获得任意文件系统权限）；
//! 导入时严格校验 JSON 格式、字段与取值范围。

use crate::settings;
use aozora_core::preset::{Preset, PresetInput, PresetList, PresetStore, MAX_PRESET_FILE_BYTES};
use std::path::{Path, PathBuf};
use tauri::AppHandle;

/// 在阻塞线程池中执行文件读写，避免占用 Tauri 主线程。
async fn run_blocking<T, F>(task: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, String> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(task)
        .await
        .map_err(|e| format!("后台任务执行失败：{e}"))?
}

fn store(app: &AppHandle) -> Result<PresetStore, String> {
    Ok(PresetStore::new(settings::presets_dir(app)?))
}

/// 列出内置预设与用户预设；损坏的预设文件以错误列表返回。
#[tauri::command]
pub async fn list_presets(app: AppHandle) -> Result<PresetList, String> {
    run_blocking(move || Ok(store(&app)?.list())).await
}

/// 保存当前编码配置为新的用户预设。
#[tauri::command]
pub async fn create_preset(app: AppHandle, preset: PresetInput) -> Result<Preset, String> {
    run_blocking(move || store(&app)?.create(preset, aozora_core::unix_now_ms())).await
}

/// 重命名用户预设（内置预设不可修改）。
#[tauri::command]
pub async fn rename_preset(app: AppHandle, id: String, name: String) -> Result<Preset, String> {
    run_blocking(move || store(&app)?.rename(&id, &name, aozora_core::unix_now_ms())).await
}

/// 删除用户预设（内置预设不可删除）。
#[tauri::command]
pub async fn delete_preset(app: AppHandle, id: String) -> Result<(), String> {
    run_blocking(move || store(&app)?.delete(&id)).await
}

/// 校验预设文件扩展名（命令层不信任前端传来的任意路径）。
fn require_json_extension(path: &Path, action: &str) -> Result<(), String> {
    let is_json = path
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("json"));
    if !is_json {
        return Err(format!(
            "{action}的文件扩展名必须是 .json：{}",
            path.display()
        ));
    }
    Ok(())
}

/// 从 JSON 文件导入预设；文件内容与编码参数都会校验，ID 重新分配。
#[tauri::command]
pub async fn import_preset_file(app: AppHandle, path: String) -> Result<Preset, String> {
    run_blocking(move || {
        let file = PathBuf::from(&path);
        require_json_extension(&file, "导入")?;
        if !file.is_file() {
            return Err(format!("预设文件不存在或不是文件：{path}"));
        }
        let metadata = std::fs::metadata(&file)
            .map_err(|e| format!("读取预设文件信息失败（{}）：{e}", file.display()))?;
        if metadata.len() > MAX_PRESET_FILE_BYTES {
            return Err(format!(
                "预设文件过大（{} 字节，上限 {} 字节）",
                metadata.len(),
                MAX_PRESET_FILE_BYTES
            ));
        }
        let text = std::fs::read_to_string(&file)
            .map_err(|e| format!("读取预设文件失败（{}）：{e}", file.display()))?;
        store(&app)?.import_text(&text, aozora_core::unix_now_ms())
    })
    .await
}

/// 导出预设到 JSON 文件（内置预设导出为普通用户预设）。
#[tauri::command]
pub async fn export_preset_file(app: AppHandle, id: String, path: String) -> Result<(), String> {
    run_blocking(move || {
        let target = PathBuf::from(&path);
        require_json_extension(&target, "导出")?;
        if target.is_dir() {
            return Err(format!("导出目标是文件夹：{}", target.display()));
        }
        let text = store(&app)?.export_text(&id)?;
        if let Some(dir) = target.parent().filter(|p| !p.as_os_str().is_empty()) {
            if !dir.is_dir() {
                return Err(format!("导出目录不存在：{}", dir.display()));
            }
        }
        std::fs::write(&target, text)
            .map_err(|e| format!("写入预设文件失败（{}）：{e}", target.display()))
    })
    .await
}
