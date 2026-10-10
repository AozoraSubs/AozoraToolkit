use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

// ---------- ffprobe JSON 原始结构 ----------

#[derive(Debug, Default, Deserialize)]
struct FfprobeOutput {
    #[serde(default)]
    format: Option<RawFormat>,
    #[serde(default)]
    streams: Vec<RawStream>,
    #[serde(default)]
    chapters: Vec<RawChapter>,
}

#[derive(Debug, Deserialize)]
struct RawFormat {
    format_name: Option<String>,
    format_long_name: Option<String>,
    duration: Option<String>,
    size: Option<String>,
    bit_rate: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawStream {
    index: Option<u32>,
    codec_type: Option<String>,
    codec_name: Option<String>,
    profile: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    pix_fmt: Option<String>,
    bits_per_raw_sample: Option<String>,
    avg_frame_rate: Option<String>,
    r_frame_rate: Option<String>,
    sample_rate: Option<String>,
    channels: Option<u32>,
    channel_layout: Option<String>,
    #[serde(default)]
    tags: HashMap<String, String>,
    #[serde(default)]
    disposition: Disposition,
}

#[derive(Debug, Default, Deserialize)]
struct Disposition {
    #[serde(default)]
    default: u32,
    #[serde(default)]
    forced: u32,
}

#[derive(Debug, Deserialize)]
struct RawChapter {}

// ---------- 面向 UI 的结果结构 ----------

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoStreamDto {
    pub index: u32,
    pub codec_name: String,
    pub profile: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub pixel_format: Option<String>,
    pub bit_depth: Option<u32>,
    pub frame_rate: Option<String>,
    pub language: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioStreamDto {
    pub index: u32,
    pub codec_name: String,
    pub sample_rate: Option<String>,
    pub channels: Option<u32>,
    pub channel_layout: Option<String>,
    pub language: Option<String>,
    pub title: Option<String>,
    pub default: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubtitleStreamDto {
    pub index: u32,
    pub codec_name: String,
    pub language: Option<String>,
    pub title: Option<String>,
    pub default: bool,
    pub forced: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaProbeResult {
    pub file_path: String,
    pub format_name: Option<String>,
    pub format_long_name: Option<String>,
    pub duration_secs: Option<f64>,
    pub size_bytes: Option<u64>,
    pub bit_rate: Option<u64>,
    pub video_streams: Vec<VideoStreamDto>,
    pub audio_streams: Vec<AudioStreamDto>,
    pub subtitle_streams: Vec<SubtitleStreamDto>,
    pub chapter_count: usize,
    pub attachment_count: usize,
}

fn parse_u64(s: &Option<String>) -> Option<u64> {
    s.as_deref().and_then(|v| v.parse().ok())
}

fn parse_f64(s: &Option<String>) -> Option<f64> {
    s.as_deref().and_then(|v| v.parse().ok())
}

fn bit_depth(raw: &RawStream) -> Option<u32> {
    raw.bits_per_raw_sample
        .as_deref()
        .and_then(|v| v.parse().ok())
        .or_else(|| raw.pix_fmt.as_deref().and_then(bit_depth_from_pix_fmt))
}

/// 从像素格式名推断位深（如 yuv420p10le → 10）。仅接受 8–16 的合理值。
fn bit_depth_from_pix_fmt(pix_fmt: &str) -> Option<u32> {
    let bytes = pix_fmt.as_bytes();
    for (i, &c) in bytes.iter().enumerate() {
        if c == b'p' {
            let mut digits = String::new();
            let mut j = i + 1;
            while j < bytes.len() && bytes[j].is_ascii_digit() {
                digits.push(bytes[j] as char);
                j += 1;
            }
            if let Ok(v) = digits.parse::<u32>() {
                if (8..=16).contains(&v) {
                    return Some(v);
                }
            }
        }
    }
    None
}

fn frame_rate(raw: &RawStream) -> Option<String> {
    raw.avg_frame_rate
        .clone()
        .filter(|v| v != "0/0")
        .or_else(|| raw.r_frame_rate.clone().filter(|v| v != "0/0"))
}

fn build_result(file_path: &Path, output: &FfprobeOutput) -> MediaProbeResult {
    let format = output.format.as_ref();
    let mut video = Vec::new();
    let mut audio = Vec::new();
    let mut subtitles = Vec::new();
    let mut attachments = 0usize;

    for raw in &output.streams {
        match raw.codec_type.as_deref() {
            Some("video") => video.push(VideoStreamDto {
                index: raw.index.unwrap_or_default(),
                codec_name: raw.codec_name.clone().unwrap_or_else(|| "未知".to_string()),
                profile: raw.profile.clone(),
                width: raw.width,
                height: raw.height,
                pixel_format: raw.pix_fmt.clone(),
                bit_depth: bit_depth(raw),
                frame_rate: frame_rate(raw),
                language: raw.tags.get("language").cloned(),
                title: raw.tags.get("title").cloned(),
            }),
            Some("audio") => audio.push(AudioStreamDto {
                index: raw.index.unwrap_or_default(),
                codec_name: raw.codec_name.clone().unwrap_or_else(|| "未知".to_string()),
                sample_rate: raw.sample_rate.clone(),
                channels: raw.channels,
                channel_layout: raw.channel_layout.clone(),
                language: raw.tags.get("language").cloned(),
                title: raw.tags.get("title").cloned(),
                default: raw.disposition.default != 0,
            }),
            Some("subtitle") => subtitles.push(SubtitleStreamDto {
                index: raw.index.unwrap_or_default(),
                codec_name: raw.codec_name.clone().unwrap_or_else(|| "未知".to_string()),
                language: raw.tags.get("language").cloned(),
                title: raw.tags.get("title").cloned(),
                default: raw.disposition.default != 0,
                forced: raw.disposition.forced != 0,
            }),
            Some("attachment") => attachments += 1,
            _ => {}
        }
    }

    MediaProbeResult {
        file_path: file_path.display().to_string(),
        format_name: format.and_then(|f| f.format_name.clone()),
        format_long_name: format.and_then(|f| f.format_long_name.clone()),
        duration_secs: format.and_then(|f| parse_f64(&f.duration)),
        size_bytes: format.and_then(|f| parse_u64(&f.size)),
        bit_rate: format.and_then(|f| parse_u64(&f.bit_rate)),
        video_streams: video,
        audio_streams: audio,
        subtitle_streams: subtitles,
        chapter_count: output.chapters.len(),
        attachment_count: attachments,
    }
}

/// 用 FFprobe 探测媒体文件并解析 JSON 结果。
pub fn probe_media(ffprobe_path: &Path, file_path: &Path) -> Result<MediaProbeResult, String> {
    let mut command = Command::new(ffprobe_path);
    crate::process_flags::hide_console_std(&mut command);
    let output = command
        .args([
            "-v",
            "error",
            "-print_format",
            "json",
            "-show_format",
            "-show_streams",
            "-show_chapters",
        ])
        .arg(file_path)
        .output()
        .map_err(|e| format!("无法启动 {}：{e}", ffprobe_path.display()))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let tail = stderr
            .lines()
            .rev()
            .take(3)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join(" | ");
        return Err(format!(
            "FFprobe 探测失败（退出码：{}）：{}",
            output.status,
            if tail.is_empty() {
                "无错误输出"
            } else {
                &tail
            }
        ));
    }
    let parsed: FfprobeOutput = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("解析 FFprobe JSON 失败：{e}"))?;
    Ok(build_result(file_path, &parsed))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_output() -> FfprobeOutput {
        let json = r#"{
  "streams": [
    {
      "index": 0,
      "codec_name": "h264",
      "codec_type": "video",
      "profile": "High",
      "width": 1920,
      "height": 1080,
      "pix_fmt": "yuv420p10le",
      "bits_per_raw_sample": "10",
      "avg_frame_rate": "24000/1001",
      "r_frame_rate": "24000/1001",
      "tags": { "language": "und", "title": "main video" }
    },
    {
      "index": 1,
      "codec_name": "aac",
      "codec_type": "audio",
      "sample_rate": "48000",
      "channels": 2,
      "channel_layout": "stereo",
      "tags": { "language": "jpn" },
      "disposition": { "default": 1, "forced": 0 }
    },
    {
      "index": 2,
      "codec_name": "ass",
      "codec_type": "subtitle",
      "tags": { "language": "chi", "title": "简中" },
      "disposition": { "default": 0, "forced": 1 }
    },
    {
      "index": 3,
      "codec_name": "ttf",
      "codec_type": "attachment"
    }
  ],
  "chapters": [ { "id": 0, "tags": { "title": "OP" } } ],
  "format": {
    "filename": "test.mkv",
    "format_name": "matroska,webm",
    "format_long_name": "Matroska / WebM",
    "duration": "93.456000",
    "size": "234567890",
    "bit_rate": "20076543"
  }
}"#;
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn build_result_extracts_all_stream_kinds() {
        let result = build_result(Path::new("test.mkv"), &sample_output());
        assert_eq!(result.format_name.as_deref(), Some("matroska,webm"));
        assert_eq!(result.duration_secs, Some(93.456));
        assert_eq!(result.size_bytes, Some(234567890));
        assert_eq!(result.chapter_count, 1);
        assert_eq!(result.attachment_count, 1);

        let v = &result.video_streams[0];
        assert_eq!(v.codec_name, "h264");
        assert_eq!(v.width, Some(1920));
        assert_eq!(v.pixel_format.as_deref(), Some("yuv420p10le"));
        assert_eq!(v.bit_depth, Some(10));
        assert_eq!(v.frame_rate.as_deref(), Some("24000/1001"));

        let a = &result.audio_streams[0];
        assert_eq!(a.channels, Some(2));
        assert_eq!(a.language.as_deref(), Some("jpn"));
        assert!(a.default);

        let s = &result.subtitle_streams[0];
        assert_eq!(s.language.as_deref(), Some("chi"));
        assert!(s.forced);
    }

    #[test]
    fn bit_depth_falls_back_to_pix_fmt() {
        let raw = RawStream {
            index: Some(0),
            codec_type: Some("video".into()),
            codec_name: Some("hevc".into()),
            profile: None,
            width: Some(1280),
            height: Some(720),
            pix_fmt: Some("yuv420p12le".into()),
            bits_per_raw_sample: None,
            avg_frame_rate: None,
            r_frame_rate: None,
            sample_rate: None,
            channels: None,
            channel_layout: None,
            tags: HashMap::new(),
            disposition: Disposition::default(),
        };
        assert_eq!(bit_depth(&raw), Some(12));
    }

    /// 实机验证：用真实 ffmpeg 生成短样片（MP4 + MKV），再用真实 ffprobe 探测。
    /// 仅在显式 `--ignored` 时运行，要求本机 PATH 中已安装 ffmpeg / ffprobe。
    #[test]
    #[ignore = "requires ffmpeg and ffprobe on PATH"]
    fn probe_real_generated_media() {
        use crate::media_probe::find_in_path;
        let path_entries: Vec<std::path::PathBuf> = std::env::var_os("PATH")
            .map(|p| std::env::split_paths(&p).collect())
            .unwrap_or_default();
        let ffmpeg = find_in_path("ffmpeg", &path_entries).expect("ffmpeg 未在 PATH 中检测到");
        let ffprobe = find_in_path("ffprobe", &path_entries).expect("ffprobe 未在 PATH 中检测到");

        let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("test-tmp")
            .join(format!("probe-real-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let mp4 = dir.join("sample.mp4");
        let mkv = dir.join("sample.mkv");

        // 生成 1 秒 testsrc 视频 + 正弦音频
        let gen = Command::new(&ffmpeg)
            .args([
                "-y",
                "-f",
                "lavfi",
                "-i",
                "testsrc=size=320x240:rate=25",
                "-f",
                "lavfi",
                "-i",
                "sine=frequency=440:sample_rate=48000",
                "-t",
                "1",
                "-c:v",
                "libx264",
                "-pix_fmt",
                "yuv420p",
                "-ac",
                "2",
                "-c:a",
                "aac",
            ])
            .arg(&mp4)
            .output()
            .expect("生成样片失败");
        assert!(
            gen.status.success(),
            "ffmpeg 生成 MP4 失败：{}",
            String::from_utf8_lossy(&gen.stderr)
                .lines()
                .last()
                .unwrap_or("无错误输出")
        );

        let mp4_info = probe_media(&ffprobe, &mp4).expect("MP4 探测失败");
        assert!(mp4_info
            .format_name
            .as_deref()
            .unwrap_or("")
            .contains("mp4"));
        let dur = mp4_info.duration_secs.expect("缺少时长");
        assert!((0.5..1.6).contains(&dur), "时长异常：{dur}");
        assert!(mp4_info.size_bytes.unwrap_or(0) > 0);
        let v = mp4_info.video_streams.first().expect("缺少视频流");
        assert_eq!(v.codec_name, "h264");
        assert_eq!(v.width, Some(320));
        assert_eq!(v.height, Some(240));
        assert_eq!(v.pixel_format.as_deref(), Some("yuv420p"));
        let a = mp4_info.audio_streams.first().expect("缺少音频流");
        assert_eq!(a.codec_name, "aac");
        assert_eq!(a.channels, Some(2));

        // MP4 转封装为 MKV，验证 MKV 容器探测
        let remux = Command::new(&ffmpeg)
            .args(["-y", "-i"])
            .arg(&mp4)
            .args(["-c", "copy"])
            .arg(&mkv)
            .output()
            .expect("转封装失败");
        assert!(remux.status.success(), "ffmpeg 转封装 MKV 失败");

        let mkv_info = probe_media(&ffprobe, &mkv).expect("MKV 探测失败");
        assert!(
            mkv_info
                .format_name
                .as_deref()
                .unwrap_or("")
                .contains("matroska"),
            "格式名异常：{:?}",
            mkv_info.format_name
        );
        assert_eq!(mkv_info.video_streams.len(), 1);
        assert_eq!(mkv_info.audio_streams.len(), 1);

        let _ = std::fs::remove_dir_all(&dir);
    }
}
