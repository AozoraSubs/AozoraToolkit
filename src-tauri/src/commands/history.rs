//! 任务历史命令层：列表、删除单条与清空。

use crate::settings;
use aozora_core::history::{HistoryStore, JobRecord};
use tauri::AppHandle;

async fn run_blocking<T, F>(task: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, String> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(task)
        .await
        .map_err(|e| format!("后台任务执行失败：{e}"))?
}

fn store(app: &AppHandle) -> Result<HistoryStore, String> {
    Ok(HistoryStore::new(settings::history_path(app)?))
}

/// 读取任务历史（最新在前）。
#[tauri::command]
pub async fn list_job_history(app: AppHandle) -> Result<Vec<JobRecord>, String> {
    run_blocking(move || store(&app)?.load()).await
}

/// 删除单条任务历史。
#[tauri::command]
pub async fn delete_job_history(app: AppHandle, job_id: String) -> Result<Vec<JobRecord>, String> {
    run_blocking(move || store(&app)?.remove(&job_id)).await
}

/// 清空任务历史。
#[tauri::command]
pub async fn clear_job_history(app: AppHandle) -> Result<(), String> {
    run_blocking(move || store(&app)?.clear()).await
}
