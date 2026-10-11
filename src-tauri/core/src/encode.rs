//! 结构化编码配置模型与校验。
//!
//! 本模块只定义数据模型与纯逻辑校验，不涉及文件系统与进程；
//! 路径相关的检查见 [`crate::planner::check_paths`]。

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const ENCODE_CONFIG_SCHEMA_VERSION: u32 = 1;

/// 应用内置的软件编码器（常数仅用于文档与测试断言）。
pub const SOFTWARE_ENCODERS: [&str; 3] = ["libx264", "libx265", "libvpx-vp9"];
/// 应用内置的 NVIDIA 硬件编码器。
pub const NVENC_ENCODERS: [&str; 3] = ["h264_nvenc", "hevc_nvenc", "av1_nvenc"];
/// 应用内置的 Intel Quick Sync 编码器。
pub const QSV_ENCODERS: [&str; 3] = ["h264_qsv", "hevc_qsv", "av1_qsv"];
/// 应用内置的 AMD AMF 编码器。
pub const AMF_ENCODERS: [&str; 3] = ["h264_amf", "hevc_amf", "av1_amf"];

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
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VideoConfig {
    pub encoder_id: String,
    pub quality_mode: QualityMode,
    pub quality_value: u32,
    pub preset: String,
    /// 像素格式；`None` 表示自动（不传 `-pix_fmt`，由编码器选择）。
    pub pixel_format: Option<String>,
    /// 输出分辨率；`None` 表示保持源分辨率（不插入 scale 滤镜）。
    /// 旧配置 / 旧预设无此字段时按"保持源分辨率"处理。
    #[serde(default)]
    pub resolution: Option<Resolution>,
}

/// 输出分辨率：宽高都可单独留空。
///
/// 只填一边时另一边按源画面宽高比自动计算（并保证偶数）；两边都填时
/// 画面按宽高比缩放到"不超出给定框"的尺寸，不做拉伸或裁剪（见
/// [`crate::planner::scale_filter`]）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Resolution {
    pub width: Option<u32>,
    pub height: Option<u32>,
}

impl Resolution {
    /// 是否指定了任何一边（两边都为空等于保持源分辨率）。
    pub fn is_set(&self) -> bool {
        self.width.is_some() || self.height.is_some()
    }
}

/// 分辨率允许的范围：下限 16（编码器普遍要求 ≥ 16），上限 8192。
pub const RESOLUTION_RANGE: (u32, u32) = (16, 8192);

impl Default for VideoConfig {
    fn default() -> Self {
        let spec = &ENCODER_SPECS[0];
        Self {
            encoder_id: spec.encoder_id.to_string(),
            quality_mode: quality_mode_of(spec),
            quality_value: spec.default_quality,
            preset: spec.default_preset.to_string(),
            pixel_format: None,
            resolution: None,
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
#[serde(rename_all = "camelCase", deny_unknown_fields)]
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
#[serde(rename_all = "camelCase", deny_unknown_fields)]
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

/// 输出封装格式（容器）。用户可在压制页显式选择，不再依赖输出路径扩展名。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Container {
    /// Matroska：兼容性最好，支持全部常见字幕格式（默认）。
    #[default]
    Mkv,
    /// MP4：兼容消费设备，字幕仅支持 mov_text。
    Mp4,
    /// WebM：仅支持 VP8 / VP9 / AV1 视频与 Vorbis / Opus 音频。
    Webm,
}

impl Container {
    /// 该容器在输出路径中使用的扩展名。
    pub fn extension(self) -> &'static str {
        match self {
            Container::Mkv => "mkv",
            Container::Mp4 => "mp4",
            Container::Webm => "webm",
        }
    }

    /// 全部容器扩展名（路径校验用）。
    pub fn extensions() -> [&'static str; 3] {
        ["mkv", "mp4", "webm"]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OutputConfig {
    pub path: PathBuf,
    /// 封装格式；旧配置无此字段时默认 MKV。
    #[serde(default)]
    pub container: Container,
    /// 输出文件已存在时是否覆盖。
    pub overwrite: bool,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            path: PathBuf::new(),
            container: Container::default(),
            overwrite: false,
        }
    }
}

// ---------- 编码器规格 ----------

/// 编码器家族：决定 FFmpeg 参数写法（预设 / 质量控制方式）。
///
/// - `Software`：libx264 / libx265 / libvpx-vp9，用 `-preset` + `-crf`；
/// - `Nvenc`（NVIDIA）：`-preset p1…p7 -rc vbr -cq N`；
/// - `Qsv`（Intel Quick Sync）：`-preset … -global_quality N`（ICQ）；
/// - `Amf`（AMD）：`-quality … -rc cqp -qp_i N -qp_p N`。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncoderFamily {
    Software,
    Nvenc,
    Qsv,
    Amf,
}

/// 编码器的硬件归属：用于在检测结果里说明"哪个编码器来自哪块显卡"。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum EncoderVendor {
    /// 纯软件编码（CPU）。
    Software,
    Nvidia,
    Amd,
    Intel,
}

impl EncoderVendor {
    pub fn label(self) -> &'static str {
        match self {
            EncoderVendor::Software => "软件（CPU）",
            EncoderVendor::Nvidia => "NVIDIA",
            EncoderVendor::Amd => "AMD",
            EncoderVendor::Intel => "Intel",
        }
    }

    /// 是否为硬件编码器（需要对应的显卡）。
    pub fn is_hardware(self) -> bool {
        !matches!(self, EncoderVendor::Software)
    }
}

/// 视频质量/速度参数写法：不同编码器用不同的选项名。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresetKind {
    /// `-preset`（libx264 / libx265 / NVENC 的 p1–p7 / SVT-AV1 的 0–13）
    Preset,
    /// `-deadline good -cpu-used N`（libvpx-vp9、libaom-av1）
    CpuUsed,
    /// `-speed N`（librav1e）
    Speed,
    /// Intel QSV 的 `-preset` 名称
    QsvPreset,
    /// AMD AMF 的 `-quality` 档位
    AmfQuality,
}

/// 单个编码器的静态规格：预设、质量范围、像素格式等（与能力检测无关的固有属性）。
#[derive(Debug, Clone)]
pub struct EncoderSpec {
    pub encoder_id: &'static str,
    /// 用户可见名称（含厂商，例如 "H.264（h264_nvenc，NVIDIA）"）。
    pub label: &'static str,
    /// 硬件归属。
    pub vendor: EncoderVendor,
    /// 参数写法家族。
    pub family: EncoderFamily,
    /// 输出视频编码格式（H.264 / HEVC / AV1 / VP9），用于容器兼容性提示。
    pub codec: &'static str,
    /// 速度/质量预设的选项写法。
    pub preset_kind: PresetKind,
    /// UI 中质量参数的名称（CRF / CQ / ICQ / QP）。
    pub quality_label: &'static str,
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
    /// 恒定质量是否需要显式 `-b:v 0`（libvpx-vp9 / libaom-av1 / libsvtav1
    /// 不加会退化为受限 VBR，`-crf` 的表现不符合预期）。
    pub zero_bitrate: bool,
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
/// Intel Quick Sync 预设（FFmpeg 接受名称，内部映射为 0–7）。
pub const QSV_PRESETS: &[&str] = &[
    "veryfast", "faster", "fast", "medium", "slow", "slower", "veryslow",
];
/// AMD AMF 质量档位（`-quality`）。
pub const AMF_PRESETS: &[&str] = &["speed", "balanced", "quality"];
/// libvpx-vp9 的速度档位（`-cpu-used`，越小越慢越好）。
pub const VP9_PRESETS: &[&str] = &["0", "1", "2", "3", "4", "5"];
/// libaom-av1 的速度档位（`-cpu-used`，0 最慢最好、8 最快）。
pub const AOM_PRESETS: &[&str] = &["0", "1", "2", "3", "4", "5", "6", "7", "8"];
/// SVT-AV1 预设（`-preset`，数字越小越慢越好；0–13）。
pub const SVTAV1_PRESETS: &[&str] = &["0", "2", "4", "6", "8", "10", "12", "13"];
/// librav1e 速度档位（`-speed`，0 最慢最好、10 最快）。
pub const RAV1E_PRESETS: &[&str] = &["0", "2", "4", "5", "6", "8", "10"];

/// 编码器规格表。**这是"应用支持的编码器集合"，是否真正可用由能力检测决定**
/// （见 `crate::capabilities`：先用 `ffmpeg -encoders` 判断是否编译进来，
/// 再对硬件编码器做一次最小实拍探测，避免"编译进来了但机器上跑不起来"）。
pub static ENCODER_SPECS: &[EncoderSpec] = &[
    EncoderSpec {
        encoder_id: "libx264",
        preset_kind: PresetKind::Preset,
        label: "H.264（libx264，软件）",
        vendor: EncoderVendor::Software,
        family: EncoderFamily::Software,
        codec: "h264",
        quality_label: "CRF",
        presets: X264_PRESETS,
        default_preset: "slow",
        quality_range: (0, 51),
        default_quality: 18,
        // 常见 libx264 构建为 8-bit，不支持 10-bit 像素格式。
        pixel_formats: &[None, Some("yuv420p")],
        zero_bitrate: false,
    },
    EncoderSpec {
        encoder_id: "libx265",
        preset_kind: PresetKind::Preset,
        label: "HEVC（libx265，软件）",
        vendor: EncoderVendor::Software,
        family: EncoderFamily::Software,
        codec: "hevc",
        quality_label: "CRF",
        presets: X265_PRESETS,
        default_preset: "slow",
        quality_range: (0, 51),
        default_quality: 18,
        pixel_formats: &[None, Some("yuv420p"), Some("yuv420p10le")],
        zero_bitrate: false,
    },
    EncoderSpec {
        encoder_id: "libvpx-vp9",
        preset_kind: PresetKind::CpuUsed,
        label: "VP9（libvpx-vp9，软件）",
        vendor: EncoderVendor::Software,
        family: EncoderFamily::Software,
        codec: "vp9",
        quality_label: "CRF",
        presets: VP9_PRESETS,
        default_preset: "2",
        quality_range: (0, 63),
        default_quality: 32,
        pixel_formats: &[None, Some("yuv420p")],
        // VP9 恒定质量需要显式 -b:v 0，否则 -crf 会退化为受限 VBR。
        zero_bitrate: true,
    },
    EncoderSpec {
        encoder_id: "libsvtav1",
        preset_kind: PresetKind::Preset,
        label: "AV1（libsvtav1，软件）",
        vendor: EncoderVendor::Software,
        family: EncoderFamily::Software,
        codec: "av1",
        quality_label: "CRF",
        presets: SVTAV1_PRESETS,
        default_preset: "8",
        quality_range: (0, 63),
        default_quality: 35,
        pixel_formats: &[None, Some("yuv420p"), Some("yuv420p10le")],
        // 恒定质量需要显式 -b:v 0（否则退化为受限 VBR）
        zero_bitrate: true,
    },
    EncoderSpec {
        encoder_id: "libaom-av1",
        preset_kind: PresetKind::CpuUsed,
        label: "AV1（libaom-av1，软件）",
        vendor: EncoderVendor::Software,
        family: EncoderFamily::Software,
        codec: "av1",
        quality_label: "CRF",
        presets: AOM_PRESETS,
        default_preset: "6",
        quality_range: (0, 63),
        default_quality: 32,
        pixel_formats: &[None, Some("yuv420p"), Some("yuv420p10le")],
        zero_bitrate: true,
    },
    EncoderSpec {
        encoder_id: "librav1e",
        preset_kind: PresetKind::Speed,
        label: "AV1（librav1e，软件）",
        vendor: EncoderVendor::Software,
        family: EncoderFamily::Software,
        codec: "av1",
        quality_label: "CRF",
        presets: RAV1E_PRESETS,
        default_preset: "6",
        quality_range: (0, 63),
        default_quality: 32,
        pixel_formats: &[None, Some("yuv420p"), Some("yuv420p10le")],
        zero_bitrate: true,
    },
    EncoderSpec {
        encoder_id: "h264_nvenc",
        preset_kind: PresetKind::Preset,
        label: "H.264（h264_nvenc，NVIDIA）",
        vendor: EncoderVendor::Nvidia,
        family: EncoderFamily::Nvenc,
        codec: "h264",
        quality_label: "CQ",
        presets: NVENC_PRESETS,
        default_preset: "p5",
        quality_range: (0, 51),
        default_quality: 21,
        pixel_formats: &[None, Some("yuv420p")],
        zero_bitrate: false,
    },
    EncoderSpec {
        encoder_id: "hevc_nvenc",
        preset_kind: PresetKind::Preset,
        label: "HEVC（hevc_nvenc，NVIDIA）",
        vendor: EncoderVendor::Nvidia,
        family: EncoderFamily::Nvenc,
        codec: "hevc",
        quality_label: "CQ",
        presets: NVENC_PRESETS,
        default_preset: "p5",
        quality_range: (0, 51),
        default_quality: 21,
        pixel_formats: &[None, Some("yuv420p"), Some("p010le")],
        zero_bitrate: false,
    },
    EncoderSpec {
        encoder_id: "av1_nvenc",
        preset_kind: PresetKind::Preset,
        label: "AV1（av1_nvenc，NVIDIA）",
        vendor: EncoderVendor::Nvidia,
        family: EncoderFamily::Nvenc,
        codec: "av1",
        quality_label: "CQ",
        presets: NVENC_PRESETS,
        default_preset: "p5",
        // av1_nvenc 的 -cq 标尺为 0–63，与 h264/hevc_nvenc 的 0–51 不同。
        quality_range: (0, 63),
        default_quality: 30,
        pixel_formats: &[None, Some("yuv420p"), Some("p010le")],
        zero_bitrate: false,
    },
    EncoderSpec {
        encoder_id: "h264_qsv",
        preset_kind: PresetKind::QsvPreset,
        label: "H.264（h264_qsv，Intel Quick Sync）",
        vendor: EncoderVendor::Intel,
        family: EncoderFamily::Qsv,
        codec: "h264",
        quality_label: "ICQ",
        presets: QSV_PRESETS,
        default_preset: "medium",
        quality_range: (1, 51),
        default_quality: 23,
        pixel_formats: &[None, Some("nv12")],
        zero_bitrate: false,
    },
    EncoderSpec {
        encoder_id: "hevc_qsv",
        preset_kind: PresetKind::QsvPreset,
        label: "HEVC（hevc_qsv，Intel Quick Sync）",
        vendor: EncoderVendor::Intel,
        family: EncoderFamily::Qsv,
        codec: "hevc",
        quality_label: "ICQ",
        presets: QSV_PRESETS,
        default_preset: "medium",
        quality_range: (1, 51),
        default_quality: 23,
        pixel_formats: &[None, Some("nv12"), Some("p010le")],
        zero_bitrate: false,
    },
    EncoderSpec {
        encoder_id: "av1_qsv",
        preset_kind: PresetKind::QsvPreset,
        label: "AV1（av1_qsv，Intel Quick Sync）",
        vendor: EncoderVendor::Intel,
        family: EncoderFamily::Qsv,
        codec: "av1",
        quality_label: "ICQ",
        presets: QSV_PRESETS,
        default_preset: "medium",
        quality_range: (1, 63),
        default_quality: 32,
        pixel_formats: &[None, Some("nv12"), Some("p010le")],
        zero_bitrate: false,
    },
    EncoderSpec {
        encoder_id: "h264_amf",
        preset_kind: PresetKind::AmfQuality,
        label: "H.264（h264_amf，AMD）",
        vendor: EncoderVendor::Amd,
        family: EncoderFamily::Amf,
        codec: "h264",
        quality_label: "QP",
        presets: AMF_PRESETS,
        default_preset: "balanced",
        quality_range: (0, 51),
        default_quality: 24,
        pixel_formats: &[None, Some("yuv420p"), Some("nv12")],
        zero_bitrate: false,
    },
    EncoderSpec {
        encoder_id: "hevc_amf",
        preset_kind: PresetKind::AmfQuality,
        label: "HEVC（hevc_amf，AMD）",
        vendor: EncoderVendor::Amd,
        family: EncoderFamily::Amf,
        codec: "hevc",
        quality_label: "QP",
        presets: AMF_PRESETS,
        default_preset: "balanced",
        quality_range: (0, 51),
        default_quality: 24,
        pixel_formats: &[None, Some("yuv420p"), Some("nv12"), Some("p010le")],
        zero_bitrate: false,
    },
    EncoderSpec {
        encoder_id: "av1_amf",
        preset_kind: PresetKind::AmfQuality,
        label: "AV1（av1_amf，AMD）",
        vendor: EncoderVendor::Amd,
        family: EncoderFamily::Amf,
        codec: "av1",
        quality_label: "QP",
        presets: AMF_PRESETS,
        default_preset: "balanced",
        quality_range: (0, 63),
        default_quality: 30,
        pixel_formats: &[None, Some("yuv420p"), Some("nv12"), Some("p010le")],
        zero_bitrate: false,
    },
];

/// 按编码器 ID 查找规格。
pub fn encoder_spec(encoder_id: &str) -> Option<&'static EncoderSpec> {
    ENCODER_SPECS.iter().find(|s| s.encoder_id == encoder_id)
}

/// 按编码器 ID 返回其质量控制模式（未知编码器返回 `None`）。
pub fn quality_mode_for(encoder_id: &str) -> Option<QualityMode> {
    encoder_spec(encoder_id).map(quality_mode_of)
}

/// 编码器家族对应的质量控制模式。
///
/// 只有软件编码器用 CRF；NVENC 的 CQ、QSV 的 ICQ 与 AMD 的 QP 都走
/// [`QualityMode::Cq`]（具体参数写法由 `planner::build_args` 按家族决定）。
fn quality_mode_of(spec: &EncoderSpec) -> QualityMode {
    match spec.family {
        EncoderFamily::Software => QualityMode::Crf,
        EncoderFamily::Nvenc | EncoderFamily::Qsv | EncoderFamily::Amf => QualityMode::Cq,
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
            ENCODER_SPECS
                .iter()
                .map(|spec| spec.encoder_id)
                .collect::<Vec<_>>()
                .join("、")
        )
    })?;

    let expected_mode = quality_mode_of(spec);
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
    if let Some(resolution) = &config.video.resolution {
        let (min, max) = RESOLUTION_RANGE;
        for (label, value) in [("宽度", resolution.width), ("高度", resolution.height)] {
            if let Some(value) = value {
                if !(min..=max).contains(&value) {
                    return Err(format!(
                        "输出分辨率{label} {value} 超出支持范围（{min}–{max}）"
                    ));
                }
            }
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
        config.video.encoder_id = "libvpx-vp8".into();
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
                resolution: Some(Resolution {
                    width: Some(1920),
                    height: Some(1080),
                }),
            },
            audio: AudioConfig::default(),
            subtitles: SubtitleConfig::default(),
            output: OutputConfig {
                path: PathBuf::from(r"D:\out.mkv"),
                container: Container::Mkv,
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
        assert!(json.contains("\"container\":\"mkv\""), "json: {json}");
        assert!(
            json.contains(r#""resolution":{"width":1920,"height":1080}"#),
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
    fn resolution_accepts_partial_and_empty_values() {
        // 只给一边：另一边按源宽高比自动计算
        let partial = Resolution {
            width: Some(1280),
            height: None,
        };
        assert!(partial.is_set());
        assert!(validate_config(&config_with_resolution(Some(partial))).is_ok());

        // 两边都空 = 保持源分辨率（不插入 scale 滤镜）
        let empty = Resolution {
            width: None,
            height: None,
        };
        assert!(!empty.is_set());
        assert!(validate_config(&config_with_resolution(Some(empty))).is_ok());

        // 完全没有分辨率设置
        assert!(validate_config(&config_with_resolution(None)).is_ok());
    }

    #[test]
    fn resolution_out_of_range_is_rejected() {
        for resolution in [
            Resolution {
                width: Some(0),
                height: None,
            },
            Resolution {
                width: None,
                height: Some(8),
            },
            Resolution {
                width: Some(99999),
                height: None,
            },
        ] {
            let err = validate_config(&config_with_resolution(Some(resolution))).unwrap_err();
            assert!(err.contains("输出分辨率"), "error: {err}");
        }
    }

    fn config_with_resolution(resolution: Option<Resolution>) -> EncodeConfig {
        let mut config = EncodeConfig::default();
        config.video.resolution = resolution;
        config
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
        // 旧配置（加容器选择之前保存的 JSON）没有 container 字段：默认 MKV
        assert_eq!(config.output.container, Container::Mkv);
        // 旧配置也没有 resolution 字段：保持源分辨率
        assert_eq!(config.video.resolution, None);
    }

    #[test]
    fn container_variants_round_trip_and_expose_extensions() {
        for (container, ext, json) in [
            (Container::Mkv, "mkv", "\"mkv\""),
            (Container::Mp4, "mp4", "\"mp4\""),
            (Container::Webm, "webm", "\"webm\""),
        ] {
            assert_eq!(container.extension(), ext);
            assert_eq!(serde_json::to_string(&container).unwrap(), json);
            assert_eq!(serde_json::from_str::<Container>(json).unwrap(), container);
        }
        assert_eq!(Container::default(), Container::Mkv);
        assert_eq!(Container::extensions(), ["mkv", "mp4", "webm"]);
    }

    #[test]
    fn unknown_container_is_rejected() {
        let json = r#""avi""#;
        assert!(serde_json::from_str::<Container>(json).is_err());
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
