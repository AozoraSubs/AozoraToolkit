//! 进程可写性自检：在当前进程内对候选目录做"创建探针文件并删除"的实测，
//! 用于诊断"应用进程被拒绝写入"类问题（区分进程级限制与目录权限）。

use serde::Serialize;
use std::path::Path;

/// 单个目录的自检结果。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WriteAccessEntry {
    pub path: String,
    pub writable: bool,
    /// 失败时的系统错误原文（如 "拒绝访问。(os error 5)"）。
    pub error: Option<String>,
}

/// 对每个候选目录创建并删除探针文件，报告当前进程的实际可写性。
/// 目录不存在等情况同样如实记录（writable=false + 错误原因）。
pub fn write_access_report(dirs: &[impl AsRef<Path>]) -> Vec<WriteAccessEntry> {
    dirs.iter()
        .map(|dir| {
            let dir = dir.as_ref();
            let probe = dir.join(format!(".aozora-writecheck-{}.tmp", std::process::id()));
            match std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&probe)
            {
                Ok(file) => {
                    drop(file);
                    let _ = std::fs::remove_file(&probe);
                    WriteAccessEntry {
                        path: dir.display().to_string(),
                        writable: true,
                        error: None,
                    }
                }
                Err(e) => WriteAccessEntry {
                    path: dir.display().to_string(),
                    writable: false,
                    error: Some(e.to_string()),
                },
            }
        })
        .collect()
}

/// 把自检结果格式化为多行文本（用于错误提示）。
pub fn format_write_access_report(entries: &[WriteAccessEntry]) -> String {
    let mut lines = String::from("进程可写性自检（当前应用进程实测）：");
    for entry in entries {
        let status = if entry.writable {
            "可写"
        } else {
            "不可写"
        };
        match &entry.error {
            Some(error) => lines.push_str(&format!("\n  {status}  {}（{error}）", entry.path)),
            None => lines.push_str(&format!("\n  {status}  {}", entry.path)),
        }
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(tag: &str) -> std::path::PathBuf {
        let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("test-tmp")
            .join(format!("aozora-diag-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn report_marks_writable_and_missing_dirs() {
        let writable = temp_dir("writable");
        let missing = writable.join("no-such-dir");
        let report = write_access_report(&[&writable, &missing]);
        assert_eq!(report.len(), 2);
        assert!(report[0].writable, "writable dir 应可写");
        assert!(report[0].error.is_none());
        assert!(!report[1].writable, "缺失目录应不可写");
        assert!(report[1].error.is_some());
        std::fs::remove_dir_all(&writable).ok();
    }

    #[test]
    fn format_lists_all_entries() {
        let entries = vec![
            WriteAccessEntry {
                path: "C:/x".to_string(),
                writable: true,
                error: None,
            },
            WriteAccessEntry {
                path: "D:/y".to_string(),
                writable: false,
                error: Some("拒绝访问。(os error 5)".to_string()),
            },
        ];
        let text = format_write_access_report(&entries);
        assert!(text.contains("可写  C:/x"), "text: {text}");
        assert!(text.contains("不可写  D:/y"), "text: {text}");
        assert!(text.contains("拒绝访问"), "text: {text}");
    }
}
