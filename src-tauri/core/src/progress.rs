//! FFmpeg `-progress pipe:1` 机器可读进度解析。
//!
//! 输出为按块组织的 `key=value` 行，每块以 `progress=continue|end` 结束。
//! 不同 FFmpeg 版本字段有差异（`out_time_ms` / `out_time_us` / `out_time`），
//! 解析时按版本兼容顺序处理。

use std::collections::HashMap;

/// 一块进度记录（`-progress` 输出的一个键值块）。
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ProgressRecord {
    pub frame: Option<u64>,
    pub fps: Option<f64>,
    /// 原始码率字符串（可能为 "N/A"）。
    pub bitrate: Option<String>,
    pub total_size: Option<u64>,
    /// 输出时间（微秒；FFmpeg ≥ 5 的 `out_time_ms`，或由旧字段换算）。
    pub out_time_us: Option<u64>,
    /// 编码速度（如 3.2 表示 3.2x；无法解析时为 None）。
    pub speed: Option<f64>,
    /// "continue" | "end"。
    pub progress: Option<String>,
    pub drop_frames: Option<u64>,
    pub dup_frames: Option<u64>,
}

impl ProgressRecord {
    /// 输出时间（秒）。
    pub fn out_time_secs(&self) -> Option<f64> {
        self.out_time_us.map(|us| us as f64 / 1_000_000.0)
    }
}

/// 解析 `key=value` 行；空键或没有 `=` 时返回 None。
pub fn parse_key_value(line: &str) -> Option<(&str, &str)> {
    let (key, value) = line.split_once('=')?;
    let key = key.trim();
    if key.is_empty() {
        return None;
    }
    Some((key, value.trim()))
}

/// 解析速度字段（如 "3.2x"；"N/A" 等无法解析时返回 None）。
pub fn parse_speed(value: &str) -> Option<f64> {
    let numeric = value.strip_suffix('x').unwrap_or(value);
    numeric.parse::<f64>().ok().filter(|v| v.is_finite())
}

/// 解析输出时间（微秒）。优先 `out_time_ms`（FFmpeg ≥ 5），
/// 其次 `out_time_us`，最后解析旧版 `out_time=HH:MM:SS.ffffff`。
pub fn parse_out_time_us(fields: &HashMap<String, String>) -> Option<u64> {
    if let Some(value) = fields
        .get("out_time_ms")
        .or_else(|| fields.get("out_time_us"))
    {
        return value.parse::<u64>().ok();
    }
    let time = fields.get("out_time")?;
    let mut parts = time.split(':');
    let hours: f64 = parts.next()?.trim().parse().ok()?;
    let minutes: f64 = parts.next()?.trim().parse().ok()?;
    let seconds: f64 = parts.next()?.trim().parse().ok()?;
    if hours < 0.0 || minutes < 0.0 || seconds < 0.0 {
        return None;
    }
    Some(((hours * 3600.0 + minutes * 60.0 + seconds) * 1_000_000.0) as u64)
}

fn parse_u64_flexible(value: &str) -> Option<u64> {
    value
        .parse::<u64>()
        .ok()
        .or_else(|| value.parse::<f64>().ok().map(|v| v as u64))
}

fn build_record(fields: &HashMap<String, String>) -> ProgressRecord {
    ProgressRecord {
        frame: fields.get("frame").and_then(|v| parse_u64_flexible(v)),
        fps: fields.get("fps").and_then(|v| v.parse().ok()),
        bitrate: fields.get("bitrate").cloned(),
        total_size: fields.get("total_size").and_then(|v| parse_u64_flexible(v)),
        out_time_us: parse_out_time_us(fields),
        speed: fields.get("speed").and_then(|v| parse_speed(v)),
        progress: fields.get("progress").cloned(),
        drop_frames: fields
            .get("drop_frames")
            .and_then(|v| parse_u64_flexible(v)),
        dup_frames: fields.get("dup_frames").and_then(|v| parse_u64_flexible(v)),
    }
}

/// 增量解析 `-progress` 键值流：每遇到 `progress=` 行产出该块记录。
#[derive(Debug, Default)]
pub struct ProgressParser {
    fields: HashMap<String, String>,
}

impl ProgressParser {
    pub fn new() -> Self {
        Self::default()
    }

    /// 喂入一行；当一块记录结束时返回完整记录。
    pub fn push_line(&mut self, line: &str) -> Option<ProgressRecord> {
        let (key, value) = parse_key_value(line)?;
        self.fields.insert(key.to_string(), value.to_string());
        if key == "progress" {
            return Some(self.take_record());
        }
        None
    }

    /// 流结束时冲刷未以 `progress=` 结束的残块。
    pub fn flush(&mut self) -> Option<ProgressRecord> {
        if self.fields.is_empty() {
            return None;
        }
        Some(self.take_record())
    }

    fn take_record(&mut self) -> ProgressRecord {
        build_record(&std::mem::take(&mut self.fields))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_key_value_extracts_trimmed_pairs() {
        assert_eq!(parse_key_value("frame=123"), Some(("frame", "123")));
        assert_eq!(parse_key_value("  speed= 3.2x "), Some(("speed", "3.2x")));
        assert_eq!(parse_key_value("no equals sign"), None);
        assert_eq!(parse_key_value("=value"), None);
    }

    #[test]
    fn parse_speed_handles_common_forms() {
        assert_eq!(parse_speed("3.2x"), Some(3.2));
        assert_eq!(parse_speed("1.05x"), Some(1.05));
        assert_eq!(parse_speed("N/A"), None);
        assert_eq!(parse_speed("0x"), Some(0.0));
    }

    #[test]
    fn parse_out_time_prefers_ms_then_us_then_legacy() {
        let mut fields = HashMap::new();
        fields.insert("out_time_ms".to_string(), "1234567".to_string());
        assert_eq!(parse_out_time_us(&fields), Some(1_234_567));

        let mut fields = HashMap::new();
        fields.insert("out_time_us".to_string(), "7654321".to_string());
        assert_eq!(parse_out_time_us(&fields), Some(7_654_321));

        let mut fields = HashMap::new();
        fields.insert("out_time".to_string(), "00:01:02.345678".to_string());
        assert_eq!(parse_out_time_us(&fields), Some(62_345_678));
    }

    #[test]
    fn parser_emits_records_per_block() {
        let mut parser = ProgressParser::new();
        let mut records = Vec::new();
        for line in [
            "frame=1",
            "fps=25.0",
            "speed=3.2x",
            "out_time_ms=1000000",
            "progress=continue",
            "frame=2",
            "out_time_ms=2000000",
            "speed=N/A",
            "progress=end",
        ] {
            if let Some(record) = parser.push_line(line) {
                records.push(record);
            }
        }
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].progress.as_deref(), Some("continue"));
        assert_eq!(records[0].frame, Some(1));
        assert_eq!(records[0].speed, Some(3.2));
        assert_eq!(records[0].out_time_secs(), Some(1.0));
        assert_eq!(records[1].progress.as_deref(), Some("end"));
        assert_eq!(records[1].speed, None);
    }

    #[test]
    fn flush_returns_partial_block_once() {
        let mut parser = ProgressParser::new();
        assert!(parser.push_line("frame=10").is_none());
        assert!(parser.push_line("out_time_ms=500000").is_none());
        let record = parser.flush().expect("残块应可冲刷");
        assert_eq!(record.frame, Some(10));
        assert!(parser.flush().is_none());
    }

    #[test]
    fn out_time_secs_conversion() {
        let record = ProgressRecord {
            out_time_us: Some(2_500_000),
            ..Default::default()
        };
        assert_eq!(record.out_time_secs(), Some(2.5));
    }
}
