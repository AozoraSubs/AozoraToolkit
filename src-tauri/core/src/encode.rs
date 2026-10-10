//! 结构化编码配置模型与校验。
//!
//! 本模块只定义数据模型与纯逻辑校验，不涉及文件系统与进程；
//! 路径相关的检查见 [`crate::planner::check_paths`]。

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const ENCODE_CONFIG_SCHEMA_VERSION: u32 = 1;

/// 软件编码器（x264 / x265，CRF 质量控制）。
pub const SOFTWARE_ENCODERS: [&str; 2] = ["libx264", "libx265"];
/// NVIDIA 硬件编码器（CQ 质量控制）。
pub const NVENC_ENCODERS: [&str; 3] = ["h264_nvenc", "hevc_nvenc", "av1_nvenc"];

// ---------- 配置模型 ----------

/// 应用内部的编码配置（PLAN §5.1 的结构化模型），执行前需要校验。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EncodeConfig {
    pub schema_version: u32,
    pub video: VideoConfig,
    pub audio: AudioConfig,
    /// 字幕处理（M4 提前实施的最小选项：丢弃 / 保留全部）。
    /// 旧版配置无此字段时按默认（丢弃）处理。
    #[serde(default)]
    pub subtitles: SubtitleConfig,
    pub output: OutputConfig,
}

impl Default for EncodeConfig {
    fn default() -> Self {
        Self {
            schema_version: ENCODE_CONFIG_SCHEMA_VERSION,
            video: VideoConfig::default(),
            audio: AudioConfig::default(),
            subtitles: SubtitleConfig::default(),
            output: OutputConfig::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoConfig {
    pub encoder_id: String,
    pub quality_mode: QualityMode,
    pub quality_value: u32,
    pub preset: String,
    /// 像素格式；`None` 表示自动（不传 `-pix_fmt`，由编码器选择）。
    pub pixel_format: Option<String>,
}

impl Default for VideoConfig {
    fn default() -> Self {
        let spec = &ENCODER_SPECS[0];
        Self {
            encoder_id: spec.encoder_id.to_string(),
            quality_mode: default_quality_mode(spec.family),
            quality_value: spec.default_quality,
            preset: spec.default_preset.to_string(),
            pixel_format: None,
        }
    }
}

/// 质量控制模式。不同编码器的 CRF / CQ 标尺不等价（PLAN §5.2）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum QualityMode {
    /// 恒定质量（CRF），x264 / x265。
    Crf,
    /// 恒定质量（CQ），NVENC。
    Cq,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioConfig {
    /// M2 仅支持 `copy`（复制原音轨，`-c:a copy`）。
    pub mode: AudioMode,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            mode: AudioMode::Copy,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AudioMode {
    Copy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubtitleConfig {
    /// `copy`：保留全部字幕轨（`-map 0:s? -c:s copy`）；
    /// `drop`：丢弃字幕（`-sn`）；
    /// `burn`：把外部字幕文件（srt/ass）烧录进画面（libass 滤镜）。
    pub mode: SubtitleMode,
    /// `burn` 模式下的外部字幕文件路径。旧版配置无此字段时按 None 处理。
    #[serde(default)]
    pub external_path: Option<PathBuf>,
}

impl Default for SubtitleConfig {
    fn default() -> Self {
        Self {
            mode: SubtitleMode::Drop,
            external_path: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SubtitleMode {
    Copy,
    Drop,
    Burn,
}

/// 烧录模式支持的外部字幕扩展名。
pub const BURN_SUBTITLE_EXTENSIONS: [&str; 2] = ["srt", "ass"];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputConfig {
    pub path: PathBuf,
    /// 输出文件已存在时是否覆盖。
    pub overwrite: bool,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            path: PathBuf::new(),
            overwrite: false,
        }
    }
}

// ---------- 编码器规格 ----------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncoderFamily {
    Software,
    Nvenc,
}

/// 单个编码器的静态规格：预设、质量范围、像素格式等（与能力检测无关的固有属性）。
#[derive(Debug, Clone)]
pub struct EncoderSpec {
    pub encoder_id: &'static str,
    pub family: EncoderFamily,
    /// 可用预设列表。
    pub presets: &'static [&'static str],
    /// 默认预设。
    pub default_preset: &'static str,
    /// 质量参数范围（含端点）。
    pub quality_range: (u32, u32),
    /// 默认质量值。
    pub default_quality: u32,
    /// 支持的像素格式列表；`None` 条目代表"自动"。
    pub pixel_formats: &'static [Option<&'static str>],
}

pub const X264_PRESETS: &[&str] = &[
    "ultrafast",
    "superfast",
    "veryfast",
    "faster",
    "fast",
    "medium",
    "slow",
    "slower",
    "veryslow",
    "placebo",
];
pub const X265_PRESETS: &[&str] = &[
    "ultrafast",
    "superfast",
    "veryfast",
    "faster",
    "fast",
    "medium",
    "slow",
    "slower",
    "veryslow",
    "placebo",
];
pub const NVENC_PRESETS: &[&str] = &["p1", "p2", "p3", "p4", "p5", "p6", "p7"];

/// 内置编码器规格。UI 按此展示参数，并在能力检测基础上过滤可用项。
pub static ENCODER_SPECS: &[EncoderSpec] = &[
    EncoderSpec {
        encoder_id: "libx264",
        family: EncoderFamily::Software,
        presets: X264_PRESETS,
        default_preset: "slow",
        quality_range: (0, 51),
        default_quality: 18,
        // 常见 libx264 构建为 8-bit，不支持 10-bit 像素格式。
        pixel_formats: &[None, Some("yuv420p")],
    },
    EncoderSpec {
        encoder_id: "libx265",
        family: EncoderFamily::Software,
        presets: X265_PRESETS,
        default_preset: "slow",
        quality_range: (0, 51),
        default_quality: 18,
        pixel_formats: &[None, Some("yuv420p"), Some("yuv420p10le")],
    },
    EncoderSpec {
        encoder_id: "h264_nvenc",
        family: EncoderFamily::Nvenc,
        presets: NVENC_PRESETS,
        default_preset: "p5",
        quality_range: (0, 51),
        default_quality: 21,
        pixel_formats: &[None, Some("yuv420p")],
    },
    EncoderSpec {
        encoder_id: "hevc_nvenc",
        family: EncoderFamily::Nvenc,
        presets: NVENC_PRESETS,
        default_preset: "p5",
        quality_range: (0, 51),
        default_quality: 21,
        pixel_formats: &[None, Some("yuv420p"), Some("p010le")],
    },
    EncoderSpec {
        encoder_id: "av1_nvenc",
        family: EncoderFamily::Nvenc,
        presets: NVENC_PRESETS,
        default_preset: "p5",
        // av1_nvenc 的 -cq 标尺为 0–63，与 h264/hevc_nvenc 的 0–51 不同。
        quality_range: (0, 63),
        default_quality: 30,
        pixel_formats: &[None, Some("yuv420p"), Some("p010le")],
    },
];

/// 按编码器 ID 查找规格。
pub fn encoder_spec(encoder_id: &str) -> Option<&'static EncoderSpec> {
    ENCODER_SPECS.iter().find(|s| s.encoder_id == encoder_id)
}

fn default_quality_mode(family: EncoderFamily) -> QualityMode {
    match family {
        EncoderFamily::Software => QualityMode::Crf,
        EncoderFamily::Nvenc => QualityMode::Cq,
    }
}

// ---------- 校验 ----------

/// 校验编码配置的字段与范围（不涉及文件系统）。
pub fn validate_config(config: &EncodeConfig) -> Result<(), String> {
    if config.schema_version != ENCODE_CONFIG_SCHEMA_VERSION {
        return Err(format!(
            "编码配置版本不受支持：{}（当前支持版本 {}）",
            config.schema_version, ENCODE_CONFIG_SCHEMA_VERSION
        ));
    }
    let spec = encoder_spec(&config.video.encoder_id).ok_or_else(|| {
        format!(
            "未知编码器：{}（支持：{}）",
            config.video.encoder_id,
            SOFTWARE_ENCODERS
                .iter()
                .chain(NVENC_ENCODERS.iter())
                .copied()
                .collect::<Vec<_>>()
                .join("、")
        )
    })?;

    let expected_mode = default_quality_mode(spec.family);
    if config.video.quality_mode != expected_mode {
        return Err(format!(
            "编码器 {} 不支持质量模式 {:?}（应使用 {:?}）",
            spec.encoder_id, config.video.quality_mode, expected_mode
        ));
    }
    let (min, max) = spec.quality_range;
    if config.video.quality_value < min || config.video.quality_value > max {
        return Err(format!(
            "质量参数 {} 超出 {} 支持范围（{min}–{max}）",
            config.video.quality_value, spec.encoder_id
        ));
    }
    if !spec.presets.contains(&config.video.preset.as_str()) {
        return Err(format!(
            "预设 {} 不受 {} 支持（可用：{}）",
            config.video.preset,
            spec.encoder_id,
            spec.presets.join("、")
        ));
    }
    if let Some(pf) = &config.video.pixel_format {
        if !spec.pixel_formats.iter().flatten().any(|p| p == pf) {
            return Err(format!(
                "像素格式 {pf} 不受 {} 支持（可用：{}）",
                spec.encoder_id,
                spec.pixel_formats
                    .iter()
                    .map(|p| p.unwrap_or("自动"))
                    .collect::<Vec<_>>()
                    .join("、")
            ));
        }
    }
    match config.audio.mode {
        AudioMode::Copy => {}
    }
    // 字幕模式：burn 必须携带外部字幕文件且扩展名受支持
    match config.subtitles.mode {
        SubtitleMode::Copy | SubtitleMode::Drop => {}
        SubtitleMode::Burn => {
            let path = config
                .subtitles
                .external_path
                .as_ref()
                .ok_or_else(|| "烧录字幕需要选择外部字幕文件（srt / ass）".to_string())?;
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_ascii_lowercase());
            match ext {
                Some(e) if BURN_SUBTITLE_EXTENSIONS.contains(&e.as_str()) => {}
                _ => {
                    return Err(format!(
                        "字幕文件扩展名必须是 .srt 或 .ass：{}",
                        path.display()
                    ));
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_valid() {
        assert!(validate_config(&EncodeConfig::default()).is_ok());
    }

    #[test]
    fn unknown_encoder_is_rejected() {
        let mut config = EncodeConfig::default();
        config.video.encoder_id = "libvpx-vp9".into();
        let err = validate_config(&config).unwrap_err();
        assert!(err.contains("未知编码器"), "error: {err}");
    }

    #[test]
    fn wrong_quality_mode_is_rejected() {
        let mut config = EncodeConfig::default();
        config.video.encoder_id = "hevc_nvenc".into();
        config.video.quality_mode = QualityMode::Crf;
        assert!(validate_config(&config).is_err());
    }

    #[test]
    fn quality_range_is_enforced() {
        let mut config = EncodeConfig::default();
        config.video.quality_value = 52;
        assert!(validate_config(&config).is_err());

        config.video.encoder_id = "av1_nvenc".into();
        config.video.quality_mode = QualityMode::Cq;
        config.video.preset = "p5".into();
        config.video.quality_value = 64;
        assert!(validate_config(&config).is_err());
        config.video.quality_value = 63;
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn unknown_preset_is_rejected() {
        let mut config = EncodeConfig::default();
        config.video.preset = "ultraplus".into();
        assert!(validate_config(&config).is_err());
    }

    #[test]
    fn unsupported_pixel_format_is_rejected() {
        let mut config = EncodeConfig::default();
        // libx264 8-bit 构建不支持 10-bit 像素格式
        config.video.pixel_format = Some("yuv420p10le".into());
        assert!(validate_config(&config).is_err());

        config.video.encoder_id = "libx265".into();
        config.video.pixel_format = Some("yuv420p10le".into());
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn schema_version_mismatch_is_rejected() {
        let config = EncodeConfig {
            schema_version: 99,
            ..EncodeConfig::default()
        };
        assert!(validate_config(&config).is_err());
    }

    #[test]
    fn json_roundtrip_uses_camel_case() {
        let config = EncodeConfig {
            schema_version: ENCODE_CONFIG_SCHEMA_VERSION,
            video: VideoConfig {
                encoder_id: "libx265".into(),
                quality_mode: QualityMode::Crf,
                quality_value: 16,
                preset: "slow".into(),
                pixel_format: Some("yuv420p10le".into()),
            },
            audio: AudioConfig::default(),
            subtitles: SubtitleConfig::default(),
            output: OutputConfig {
                path: PathBuf::from(r"D:\out.mkv"),
                overwrite: true,
            },
        };
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"encoderId\":\"libx265\""), "json: {json}");
        assert!(json.contains("\"qualityMode\":\"crf\""), "json: {json}");
        assert!(
            json.contains("\"pixelFormat\":\"yuv420p10le\""),
            "json: {json}"
        );
        assert!(
            json.contains(r#""subtitles":{"mode":"drop","externalPath":null}"#),
            "json: {json}"
        );
        let restored: EncodeConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(restored, config);
    }

    #[test]
    fn json_without_subtitles_field_defaults_to_drop() {
        let json = r#"{
          "schemaVersion": 1,
          "video": {
            "encoderId": "libx264",
            "qualityMode": "crf",
            "qualityValue": 18,
            "preset": "slow",
            "pixelFormat": null
          },
          "audio": { "mode": "copy" },
          "output": { "path": "out.mkv", "overwrite": false }
        }"#;
        let config: EncodeConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.subtitles.mode, SubtitleMode::Drop);
    }

    #[test]
    fn burn_requires_external_path_and_supported_extension() {
        let mut config = EncodeConfig::default();
        config.subtitles.mode = SubtitleMode::Burn;
        let err = validate_config(&config).unwrap_err();
        assert!(err.contains("外部字幕文件"), "error: {err}");

        config.subtitles.external_path = Some(PathBuf::from("sub.vtt"));
        let err = validate_config(&config).unwrap_err();
        assert!(err.contains(".srt 或 .ass"), "error: {err}");

        config.subtitles.external_path = Some(PathBuf::from(r"C:\字幕\sub.ass"));
        assert!(validate_config(&config).is_ok());

        config.subtitles.external_path = Some(PathBuf::from("sub.SRT"));
        assert!(validate_config(&config).is_ok());
    }
}
