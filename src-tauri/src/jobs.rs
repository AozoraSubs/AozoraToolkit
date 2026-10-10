//! 压制任务状态与事件 DTO：单任务运行（M2），由 Tauri 状态管理。

use aozora_core::job::{EncodeStatus, ProgressSnapshot};
use serde::Serialize;
use std::sync::{Arc, Mutex};
use tokio::sync::watch;

/// 当前活动任务。M2 只允许同时运行一个任务。
pub struct ActiveJob {
    pub id: String,
    pub cancel_tx: watch::Sender<bool>,
}

/// Tauri 托管的任务状态（`Arc` 使运行器任务可持有句柄）。
#[derive(Clone, Default)]
pub struct JobState {
    pub active: Arc<Mutex<Option<ActiveJob>>>,
}

// ---------- 事件载荷 ----------

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobStartedDto {
    pub job_id: String,
    /// 需要提醒用户的事项（例如目标目录不可写、已回退到应用临时目录）。
    pub notice: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobProgressDto {
    pub job_id: String,
    pub percent: f64,
    pub out_time_secs: f64,
    pub duration_known: bool,
    pub duration_secs: Option<f64>,
    pub speed: Option<f64>,
    pub fps: Option<f64>,
    pub eta_secs: Option<f64>,
}

impl JobProgressDto {
    pub fn new(job_id: &str, snapshot: &ProgressSnapshot) -> Self {
        Self {
            job_id: job_id.to_string(),
            percent: snapshot.percent,
            out_time_secs: snapshot.out_time_secs,
            duration_known: snapshot.duration_known,
            duration_secs: snapshot.duration_secs,
            speed: snapshot.speed,
            fps: snapshot.fps,
            eta_secs: snapshot.eta_secs,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobLogDto {
    pub job_id: String,
    pub line: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobFinishedDto {
    pub job_id: String,
    pub status: EncodeStatus,
    pub error: Option<String>,
    pub output_path: Option<String>,
    pub elapsed_secs: f64,
}
