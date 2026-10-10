//! 任务历史持久化（PLAN §4.3 / §8）。
//!
//! 历史保存在配置目录的 `history/jobs.json`：只记录结构化字段与日志尾部，
//! **不保存完整命令行**；日志尾部用于失败诊断，写入时按行数截断。

use crate::encode::EncodeConfig;
use crate::job::EncodeStatus;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const JOB_HISTORY_SCHEMA_VERSION: u32 = 1;
/// 历史记录条数上限（超出时丢弃最旧记录）。
pub const MAX_HISTORY_ENTRIES: usize = 50;
/// 每条记录保存的日志尾部行数上限。
pub const MAX_HISTORY_LOG_LINES: usize = 80;
/// 单行日志保存长度上限（字符）。
pub const MAX_HISTORY_LOG_LINE_CHARS: usize = 300;

/// 一条已完成（成功 / 失败 / 取消）的任务记录。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobRecord {
    pub id: String,
    /// 任务显示名（输入文件名）。
    pub name: String,
    pub input_path: String,
    /// 实际成品路径（成功且未回退时与配置一致）。
    #[serde(default)]
    pub output_path: Option<String>,
    pub status: EncodeStatus,
    #[serde(default)]
    pub error: Option<String>,
    /// 启动时展示给用户的提示（如目标目录不可写、已回退保存位置）。
    #[serde(default)]
    pub notice: Option<String>,
    /// 执行时使用的编码配置，用于"载入配置后重试"。
    pub config: EncodeConfig,
    #[serde(default)]
    pub duration_secs: Option<f64>,
    pub started_at_ms: u64,
    pub finished_at_ms: u64,
    pub elapsed_secs: f64,
    /// 日志尾部（失败诊断用，已截断）。
    #[serde(default)]
    pub log_tail: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct HistoryFile {
    schema_version: u32,
    jobs: Vec<JobRecord>,
}

/// 截断日志尾部：只保留最后 [`MAX_HISTORY_LOG_LINES`] 行，
/// 单行超过 [`MAX_HISTORY_LOG_LINE_CHARS`] 字符时截断。
pub fn truncate_log_tail(lines: &[String]) -> Vec<String> {
    let skip = lines.len().saturating_sub(MAX_HISTORY_LOG_LINES);
    lines[skip..]
        .iter()
        .map(|line| {
            if line.chars().count() > MAX_HISTORY_LOG_LINE_CHARS {
                let truncated: String = line.chars().take(MAX_HISTORY_LOG_LINE_CHARS).collect();
                format!("{truncated}…")
            } else {
                line.clone()
            }
        })
        .collect()
}

/// 任务历史存储（`jobs.json`），记录按最新在前保存。
#[derive(Debug, Clone)]
pub struct HistoryStore {
    path: PathBuf,
}

impl HistoryStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// 读取历史；文件不存在时返回空列表，版本不受支持或文件损坏时报错
    /// （调用方应把错误展示给用户，而不是静默重建历史）。
    pub fn load(&self) -> Result<Vec<JobRecord>, String> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }
        let text = std::fs::read_to_string(&self.path)
            .map_err(|e| format!("读取任务历史失败（{}）：{e}", self.path.display()))?;
        if text.trim().is_empty() {
            return Ok(Vec::new());
        }
        let file: HistoryFile = serde_json::from_str(&text)
            .map_err(|e| format!("任务历史文件格式无效（{}）：{e}", self.path.display()))?;
        if file.schema_version != JOB_HISTORY_SCHEMA_VERSION {
            return Err(format!(
                "任务历史版本不受支持：{}（当前支持版本 {}，文件：{}）",
                file.schema_version,
                JOB_HISTORY_SCHEMA_VERSION,
                self.path.display()
            ));
        }
        Ok(file.jobs)
    }

    /// 新增一条记录（最新在前），超出上限时丢弃最旧记录；返回写入后的列表。
    pub fn prepend(&self, record: JobRecord) -> Result<Vec<JobRecord>, String> {
        let mut jobs = self.load()?;
        jobs.retain(|existing| existing.id != record.id);
        jobs.insert(0, record);
        jobs.truncate(MAX_HISTORY_ENTRIES);
        self.write(&jobs)?;
        Ok(jobs)
    }

    /// 删除单条记录；返回写入后的列表。
    pub fn remove(&self, id: &str) -> Result<Vec<JobRecord>, String> {
        let mut jobs = self.load()?;
        let before = jobs.len();
        jobs.retain(|record| record.id != id);
        if jobs.len() == before {
            return Err(format!("任务记录不存在：{id}"));
        }
        self.write(&jobs)?;
        Ok(jobs)
    }

    /// 清空历史（删除文件，保留目录）。
    pub fn clear(&self) -> Result<(), String> {
        if !self.path.exists() {
            return Ok(());
        }
        std::fs::remove_file(&self.path)
            .map_err(|e| format!("清除任务历史失败（{}）：{e}", self.path.display()))
    }

    /// 原子写入：先写临时文件，再重命名覆盖。
    fn write(&self, jobs: &[JobRecord]) -> Result<(), String> {
        if let Some(dir) = self.path.parent() {
            std::fs::create_dir_all(dir)
                .map_err(|e| format!("创建历史目录失败（{}）：{e}", dir.display()))?;
        }
        let file = HistoryFile {
            schema_version: JOB_HISTORY_SCHEMA_VERSION,
            jobs: jobs.to_vec(),
        };
        let mut text =
            serde_json::to_string_pretty(&file).map_err(|e| format!("序列化任务历史失败：{e}"))?;
        text.push('\n');
        let temp = self.path.with_extension("json.tmp");
        std::fs::write(&temp, text)
            .map_err(|e| format!("写入任务历史失败（{}）：{e}", temp.display()))?;
        std::fs::rename(&temp, &self.path).map_err(|e| {
            let _ = std::fs::remove_file(&temp);
            format!("保存任务历史失败（{}）：{e}", self.path.display())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encode::EncodeConfig;
    use std::path::PathBuf;

    fn temp_store(tag: &str) -> (HistoryStore, PathBuf) {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("test-tmp")
            .join(format!("aozora-history-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        (
            HistoryStore::new(dir.join("history").join("jobs.json")),
            dir,
        )
    }

    fn record(id: &str, status: EncodeStatus, finished_at_ms: u64) -> JobRecord {
        JobRecord {
            id: id.to_string(),
            name: format!("样片-{id}.mkv"),
            input_path: r"D:\影片\样片.mkv".to_string(),
            output_path: Option::None,
            status,
            error: Option::None,
            notice: Option::None,
            config: EncodeConfig::default(),
            duration_secs: Some(120.0),
            started_at_ms: finished_at_ms.saturating_sub(1000),
            finished_at_ms,
            elapsed_secs: 1.0,
            log_tail: vec!["frame=1".to_string()],
        }
    }

    #[test]
    fn missing_file_loads_as_empty_and_clear_is_idempotent() {
        let (store, _dir) = temp_store("empty");
        assert_eq!(store.load().unwrap(), Vec::new());
        store.clear().unwrap();
        store.clear().unwrap();
    }

    #[test]
    fn prepend_orders_newest_first_and_survives_restart() {
        let (store, dir) = temp_store("persist");
        store.prepend(record("a", EncodeStatus::Failed, 1)).unwrap();
        store
            .prepend(record("b", EncodeStatus::Completed, 2))
            .unwrap();

        // 模拟应用重启：同一路径新建存储实例
        let reopened = HistoryStore::new(dir.join("history").join("jobs.json"));
        let jobs = reopened.load().unwrap();
        assert_eq!(jobs.len(), 2);
        assert_eq!(jobs[0].id, "b");
        assert_eq!(jobs[1].id, "a");
        assert_eq!(jobs[1].status, EncodeStatus::Failed);
        assert_eq!(jobs[1].config, EncodeConfig::default());
    }

    #[test]
    fn prepend_same_id_replaces_previous_entry() {
        let (store, _dir) = temp_store("replace");
        store.prepend(record("a", EncodeStatus::Failed, 1)).unwrap();
        store
            .prepend(record("a", EncodeStatus::Completed, 2))
            .unwrap();
        let jobs = store.load().unwrap();
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].status, EncodeStatus::Completed);
    }

    #[test]
    fn history_is_truncated_to_max_entries() {
        let (store, _dir) = temp_store("truncate");
        for index in 0..(MAX_HISTORY_ENTRIES + 5) {
            store
                .prepend(record(
                    &format!("j{index}"),
                    EncodeStatus::Completed,
                    index as u64,
                ))
                .unwrap();
        }
        let jobs = store.load().unwrap();
        assert_eq!(jobs.len(), MAX_HISTORY_ENTRIES);
        assert_eq!(jobs[0].id, format!("j{}", MAX_HISTORY_ENTRIES + 4));
        assert!(!jobs.iter().any(|j| j.id == "j0"), "最旧记录应被丢弃");
    }

    #[test]
    fn remove_and_clear_work() {
        let (store, _dir) = temp_store("remove");
        store
            .prepend(record("a", EncodeStatus::Completed, 1))
            .unwrap();
        store
            .prepend(record("b", EncodeStatus::Cancelled, 2))
            .unwrap();

        let jobs = store.remove("a").unwrap();
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].id, "b");
        let err = store.remove("a").unwrap_err();
        assert!(err.contains("不存在"), "error: {err}");

        store.clear().unwrap();
        assert_eq!(store.load().unwrap(), Vec::new());
    }

    #[test]
    fn unsupported_version_and_corrupt_file_report_errors() {
        let (store, _dir) = temp_store("version");
        std::fs::create_dir_all(store.path().parent().unwrap()).unwrap();
        std::fs::write(store.path(), r#"{"schemaVersion":9,"jobs":[]}"#).unwrap();
        let err = store.load().unwrap_err();
        assert!(err.contains("版本不受支持"), "error: {err}");
        // 版本不受支持时不覆盖用户文件
        let err = store
            .prepend(record("a", EncodeStatus::Completed, 1))
            .unwrap_err();
        assert!(err.contains("版本不受支持"), "error: {err}");
        assert!(std::fs::read_to_string(store.path())
            .unwrap()
            .contains("\"schemaVersion\":9"));

        std::fs::write(store.path(), "not json at all").unwrap();
        let err = store.load().unwrap_err();
        assert!(err.contains("格式无效"), "error: {err}");
    }

    #[test]
    fn log_tail_is_truncated_by_lines_and_line_length() {
        let lines: Vec<String> = (0..(MAX_HISTORY_LOG_LINES + 10))
            .map(|i| format!("line {i}"))
            .collect();
        let tail = truncate_log_tail(&lines);
        assert_eq!(tail.len(), MAX_HISTORY_LOG_LINES);
        assert_eq!(tail[0], format!("line {}", 10));

        let long = vec!["x".repeat(MAX_HISTORY_LOG_LINE_CHARS + 50)];
        let tail = truncate_log_tail(&long);
        assert_eq!(tail[0].chars().count(), MAX_HISTORY_LOG_LINE_CHARS + 1);
        assert!(tail[0].ends_with('…'));
    }
}
