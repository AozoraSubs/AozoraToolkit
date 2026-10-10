//! 单任务队列状态机（PLAN §4.3：单任务运行 + 排队任务）。
//!
//! 同一时刻只运行一个编码任务，其余任务排队等待；本模块只维护队列状态
//! 与冲突检查，不涉及进程与 Tauri（由应用层驱动执行）。

use serde::Serialize;
use std::collections::VecDeque;

/// 队列中任务的元数据（用于界面展示与冲突检查）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobMeta {
    pub id: String,
    /// 任务显示名（输入文件名）。
    pub name: String,
    pub input_path: String,
    /// 计划写入的最终输出路径。
    pub output_path: String,
    pub queued_at_ms: u64,
}

/// 队列快照（推送给界面，避免界面自行推断状态）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueSnapshot {
    pub active: Option<JobMeta>,
    pub queued: Vec<JobMeta>,
}

/// 任务队列：`active` 为正在运行的任务，`queued` 为等待运行的任务。
#[derive(Debug, Default, Clone)]
pub struct JobQueue {
    active: Option<JobMeta>,
    queued: VecDeque<JobMeta>,
}

impl JobQueue {
    pub fn new() -> Self {
        Self::default()
    }

    /// 是否有任务正在运行。
    pub fn is_idle(&self) -> bool {
        self.active.is_none()
    }

    pub fn active(&self) -> Option<&JobMeta> {
        self.active.as_ref()
    }

    pub fn queued(&self) -> impl Iterator<Item = &JobMeta> {
        self.queued.iter()
    }

    /// 入队。拒绝重复 ID 以及与运行中 / 排队任务相同的输出路径
    /// （否则前一个任务的成功成品会被后一个任务覆盖）。
    pub fn enqueue(&mut self, meta: JobMeta) -> Result<(), String> {
        if self.active.as_ref().is_some_and(|job| job.id == meta.id)
            || self.queued.iter().any(|job| job.id == meta.id)
        {
            return Err(format!("任务已存在：{}", meta.id));
        }
        let target = normalize_path(&meta.output_path);
        let conflict = self
            .active
            .as_ref()
            .map(|job| (job, "运行中"))
            .into_iter()
            .chain(self.queued.iter().map(|job| (job, "排队中")))
            .find(|(job, _)| normalize_path(&job.output_path) == target);
        if let Some((job, state)) = conflict {
            return Err(format!(
                "已有{state}的任务输出到同一路径（{}）：{}",
                meta.output_path, job.name
            ));
        }
        self.queued.push_back(meta);
        Ok(())
    }

    /// 取出下一个待运行任务并置为运行中；已有运行中任务时返回 `None`。
    pub fn take_next(&mut self) -> Option<JobMeta> {
        if self.active.is_some() {
            return None;
        }
        let next = self.queued.pop_front()?;
        self.active = Some(next.clone());
        Some(next)
    }

    /// 运行中任务结束（无论成功、失败或取消）。
    pub fn finish_active(&mut self) {
        self.active = None;
    }

    /// 移除排队中的任务（运行中的任务只能取消，不能移除）。
    pub fn remove(&mut self, id: &str) -> Result<JobMeta, String> {
        if self.active.as_ref().is_some_and(|job| job.id == id) {
            return Err("任务正在运行，请使用取消而不是移除".to_string());
        }
        let index = self
            .queued
            .iter()
            .position(|job| job.id == id)
            .ok_or_else(|| format!("排队任务不存在：{id}"))?;
        self.queued
            .remove(index)
            .ok_or_else(|| format!("排队任务不存在：{id}"))
    }

    /// 任务在队列中的位置（1 起算，仅排队中的任务）。
    pub fn position(&self, id: &str) -> Option<usize> {
        self.queued
            .iter()
            .position(|job| job.id == id)
            .map(|index| index + 1)
    }

    pub fn snapshot(&self) -> QueueSnapshot {
        QueueSnapshot {
            active: self.active.clone(),
            queued: self.queued.iter().cloned().collect(),
        }
    }
}

/// 路径比较用的规范化形式：Windows / macOS 下大小写不敏感。
fn normalize_path(path: &str) -> String {
    let unified = path.replace('\\', "/");
    if cfg!(any(windows, target_os = "macos")) {
        unified.to_lowercase()
    } else {
        unified
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn meta(id: &str, output: &str) -> JobMeta {
        JobMeta {
            id: id.to_string(),
            name: format!("{id}.mkv"),
            input_path: format!("D:/影片/{id}.mkv"),
            output_path: output.to_string(),
            queued_at_ms: 0,
        }
    }

    #[test]
    fn jobs_run_one_at_a_time_in_fifo_order() {
        let mut queue = JobQueue::new();
        assert!(queue.is_idle());
        queue.enqueue(meta("a", "D:/out/a.mkv")).unwrap();
        queue.enqueue(meta("b", "D:/out/b.mkv")).unwrap();
        assert_eq!(queue.snapshot().queued.len(), 2);
        assert_eq!(queue.position("b"), Some(2));

        let first = queue.take_next().expect("应取出队首任务");
        assert_eq!(first.id, "a");
        assert!(!queue.is_idle());
        // 运行中不再取下一个
        assert!(queue.take_next().is_none());
        assert_eq!(queue.active().map(|job| job.id.as_str()), Some("a"));

        queue.finish_active();
        let second = queue.take_next().expect("应取出下一个任务");
        assert_eq!(second.id, "b");
        queue.finish_active();
        assert!(queue.take_next().is_none());
        assert!(queue.snapshot().active.is_none());
    }

    #[test]
    fn enqueue_rejects_duplicate_ids() {
        let mut queue = JobQueue::new();
        queue.enqueue(meta("a", "D:/out/a.mkv")).unwrap();
        let err = queue.enqueue(meta("a", "D:/out/other.mkv")).unwrap_err();
        assert!(err.contains("任务已存在"), "error: {err}");
    }

    #[test]
    fn enqueue_rejects_output_path_conflicts() {
        let mut queue = JobQueue::new();
        queue.enqueue(meta("a", "D:/out/same.mkv")).unwrap();
        let err = queue.enqueue(meta("b", "d:\\OUT\\SAME.mkv")).unwrap_err();
        assert!(err.contains("同一路径"), "error: {err}");

        // 运行中的任务同样参与冲突检查
        let mut queue = JobQueue::new();
        queue.enqueue(meta("a", "D:/out/same.mkv")).unwrap();
        queue.take_next();
        let err = queue.enqueue(meta("b", "D:/out/same.mkv")).unwrap_err();
        assert!(err.contains("运行中"), "error: {err}");
    }

    #[test]
    fn remove_only_applies_to_queued_jobs() {
        let mut queue = JobQueue::new();
        queue.enqueue(meta("a", "D:/out/a.mkv")).unwrap();
        queue.enqueue(meta("b", "D:/out/b.mkv")).unwrap();

        let removed = queue.remove("b").unwrap();
        assert_eq!(removed.id, "b");
        assert_eq!(queue.snapshot().queued.len(), 1);
        let err = queue.remove("b").unwrap_err();
        assert!(err.contains("不存在"), "error: {err}");

        queue.take_next();
        let err = queue.remove("a").unwrap_err();
        assert!(err.contains("正在运行"), "error: {err}");
    }

    #[test]
    fn snapshot_reflects_state() {
        let mut queue = JobQueue::new();
        queue.enqueue(meta("a", "D:/out/a.mkv")).unwrap();
        queue.enqueue(meta("b", "D:/out/b.mkv")).unwrap();
        queue.take_next();
        let snapshot = queue.snapshot();
        assert_eq!(snapshot.active.map(|job| job.id), Some("a".to_string()));
        assert_eq!(snapshot.queued.len(), 1);
        assert_eq!(snapshot.queued[0].id, "b");
    }
}
