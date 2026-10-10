//! 压制任务状态与事件 DTO：单任务队列 + 取消信号（由 Tauri 状态管理）。
//!
//! 队列元数据在 [`aozora_core::queue::JobQueue`] 中维护（可独立测试），
//! 本层只保存执行所需的数据（已备好的参数数组与路径）与取消通道。

use aozora_core::encode::EncodeConfig;
use aozora_core::job::{EncodeStatus, ProgressSnapshot};
use aozora_core::queue::{JobMeta, JobQueue, QueueSnapshot};
use serde::Serialize;
use std::collections::HashMap;
use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard};
use tokio::sync::watch;

/// 已备好的执行数据：校验、能力检查与路径规划在入队时完成，
/// 队列中的任务启动时不再做必然性检查。
#[derive(Debug, Clone)]
pub struct PreparedJob {
    pub meta: JobMeta,
    /// 执行时使用的编码配置快照（用于"载入配置后重试"与历史记录）。
    pub config: EncodeConfig,
    pub duration_secs: Option<f64>,
    pub ffmpeg_path: PathBuf,
    pub args: Vec<OsString>,
    pub input_path: PathBuf,
    /// FFmpeg 写入的临时输出路径。
    pub temp_path: PathBuf,
    /// 成功后移动到的最终输出路径。
    pub final_path: PathBuf,
    pub overwrite: bool,
    /// 启动前需要提醒用户的事项（如目标目录不可写、已回退保存位置）。
    pub notice: Option<String>,
}

struct ActiveCancel {
    job_id: String,
    tx: watch::Sender<bool>,
}

/// Tauri 托管的任务状态（`Arc` 使后台运行任务可持有句柄）。
#[derive(Clone, Default)]
pub struct JobState {
    queue: Arc<Mutex<JobQueue>>,
    prepared: Arc<Mutex<HashMap<String, PreparedJob>>>,
    cancel: Arc<Mutex<Option<ActiveCancel>>>,
}

impl JobState {
    /// 队列状态是简单数据，锁中毒（持锁线程 panic）时直接取回内部值继续使用，
    /// 避免整个任务系统因一次 panic 永久失效。
    fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
        mutex
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// 入队（拒绝重复 ID 与输出路径冲突）。
    pub fn enqueue(&self, meta: JobMeta) -> Result<(), String> {
        Self::lock(&self.queue).enqueue(meta)
    }

    /// 保存任务的执行数据。
    pub fn store_prepared(&self, job: PreparedJob) {
        Self::lock(&self.prepared).insert(job.meta.id.clone(), job);
    }

    /// 丢弃任务的执行数据（入队失败时回滚）。
    pub fn drop_prepared(&self, job_id: &str) {
        Self::lock(&self.prepared).remove(job_id);
    }

    /// 取出下一个待运行任务（空闲时才会返回）及其执行数据。
    ///
    /// 入队顺序保证"先存执行数据、再进队列"，因此数据缺失只可能是内部错误；
    /// 调用方需要结束该任务，避免队列卡死。
    pub fn take_next(&self) -> Option<(JobMeta, Option<PreparedJob>)> {
        let meta = Self::lock(&self.queue).take_next()?;
        let payload = Self::lock(&self.prepared).remove(&meta.id);
        Some((meta, payload))
    }

    /// 运行中任务结束。
    pub fn finish_active(&self) {
        Self::lock(&self.queue).finish_active();
    }

    /// 移除排队中的任务（同时丢弃其执行数据）。
    pub fn remove_queued(&self, job_id: &str) -> Result<(), String> {
        Self::lock(&self.queue).remove(job_id)?;
        Self::lock(&self.prepared).remove(job_id);
        Ok(())
    }

    /// 任务在队列中的位置（1 起算）。
    pub fn position(&self, job_id: &str) -> Option<usize> {
        Self::lock(&self.queue).position(job_id)
    }

    pub fn snapshot(&self) -> QueueSnapshot {
        Self::lock(&self.queue).snapshot()
    }

    /// 注册运行中任务的取消通道。
    pub fn set_cancel(&self, job_id: &str, tx: watch::Sender<bool>) {
        *Self::lock(&self.cancel) = Some(ActiveCancel {
            job_id: job_id.to_string(),
            tx,
        });
    }

    /// 清除取消通道（任务结束后，避免误取消后续任务）。
    pub fn clear_cancel(&self, job_id: &str) {
        let mut cancel = Self::lock(&self.cancel);
        if cancel.as_ref().is_some_and(|c| c.job_id == job_id) {
            *cancel = None;
        }
    }

    /// 请求取消正在运行的任务。
    pub fn cancel(&self, job_id: &str) -> Result<(), String> {
        let cancel = Self::lock(&self.cancel);
        let active = cancel
            .as_ref()
            .ok_or_else(|| "当前没有正在运行的任务".to_string())?;
        if active.job_id != job_id {
            return Err(format!("任务 {job_id} 不存在或已结束"));
        }
        active
            .tx
            .send(true)
            .map_err(|_| "任务已结束，无法取消".to_string())
    }
}

// ---------- 事件载荷 ----------

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobStartedDto {
    pub job_id: String,
    /// 任务显示名（输入文件名）。
    pub name: String,
    pub input_path: String,
    /// 计划写入的最终输出路径（可能因目标目录不可写而回退）。
    pub output_path: String,
    /// 输入媒体时长（秒），用于进度百分比与 ETA。
    pub duration_secs: Option<f64>,
    /// 需要提醒用户的事项（例如目标目录不可写、已回退到应用临时目录）。
    pub notice: Option<String>,
    pub started_at_ms: u64,
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
    /// 写入任务历史时的错误（例如历史文件版本不受支持）。
    pub history_error: Option<String>,
}

/// 入队结果：任务已开始运行或已在队列中等待。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnqueueResultDto {
    pub job_id: String,
    pub started: bool,
    /// 排队位置（1 起算）；已开始运行时为 null。
    pub position: Option<usize>,
}
