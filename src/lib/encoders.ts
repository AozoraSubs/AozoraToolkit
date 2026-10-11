import type {
  Container,
  EncoderCapabilityKey,
  QualityMode,
  VideoConfig,
} from "./types/encode";
import type { CapabilityReport, EncoderAvailability } from "./types/tools";

/** 参数写法家族：与 Rust `EncoderFamily` 对应，决定 FFmpeg 参数组合。 */
export type EncoderFamily = "software" | "nvenc" | "qsv" | "amf";

/** 硬件归属：与 Rust `EncoderVendor` 对应。 */
export type EncoderVendor = "software" | "nvidia" | "amd" | "intel";

export interface PixelFormatOption {
  /** null = 自动（不传 -pix_fmt）。 */
  value: string | null;
  label: string;
}

export interface EncoderDefinition {
  id: string;
  label: string;
  family: EncoderFamily;
  vendor: EncoderVendor;
  /** 输出视频编码格式：h264 / hevc / av1 / vp9。 */
  codec: string;
  qualityMode: QualityMode;
  /** 质量参数在 UI 中的名称。 */
  qualityLabel: "CRF" | "CQ" | "ICQ" | "QP";
  qualityRange: [number, number];
  defaultQuality: number;
  presets: string[];
  defaultPreset: string;
  pixelFormats: PixelFormatOption[];
  defaultPixelFormat: string | null;
  capabilityKey: EncoderCapabilityKey;
}

const SOFTWARE_PRESETS = [
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
] as const;

const NVENC_PRESETS = ["p1", "p2", "p3", "p4", "p5", "p6", "p7"] as const;
const QSV_PRESETS = [
  "veryfast",
  "faster",
  "fast",
  "medium",
  "slow",
  "slower",
  "veryslow",
] as const;
const AMF_PRESETS = ["speed", "balanced", "quality"] as const;
/** libvpx-vp9 的 `-cpu-used` 档位（越小越慢越好）。 */
const VP9_PRESETS = ["0", "1", "2", "3", "4", "5"] as const;
/** libaom-av1 的 `-cpu-used` 档位。 */
const AOM_PRESETS = ["0", "1", "2", "3", "4", "5", "6", "7", "8"] as const;
/** SVT-AV1 的 `-preset`（数字越小越慢越好）。 */
const SVTAV1_PRESETS = ["0", "2", "4", "6", "8", "10", "12", "13"] as const;
/** librav1e 的 `-speed` 档位。 */
const RAV1E_PRESETS = ["0", "2", "4", "5", "6", "8", "10"] as const;

const AUTO_PIXEL_FORMAT: PixelFormatOption = { value: null, label: "自动" };
const PIXEL_8BIT: PixelFormatOption = {
  value: "yuv420p",
  label: "8-bit（yuv420p）",
};
const PIXEL_NV12: PixelFormatOption = { value: "nv12", label: "8-bit（nv12）" };
const PIXEL_10BIT_YUV420P10LE: PixelFormatOption = {
  value: "yuv420p10le",
  label: "10-bit（yuv420p10le）",
};
const PIXEL_10BIT_P010LE: PixelFormatOption = {
  value: "p010le",
  label: "10-bit（p010le）",
};

/**
 * 内置编码器的 UI 元数据（质量范围、预设、像素格式）。
 *
 * **是否可用不在这里判断**：它由后端能力检测的结果决定
 * （见 `types/tools.ts` 的 `CapabilityReport.encoders`，其中硬件编码器还经过
 * 一次最小实拍）。应用支持的编码器集合与 Rust `ENCODER_SPECS` 保持一致。
 */
export const ENCODER_DEFINITIONS: EncoderDefinition[] = [
  {
    id: "libx264",
    label: "H.264（libx264，软件）",
    family: "software",
    vendor: "software",
    codec: "h264",
    qualityMode: "crf",
    qualityLabel: "CRF",
    qualityRange: [0, 51],
    defaultQuality: 18,
    presets: [...SOFTWARE_PRESETS],
    defaultPreset: "slow",
    pixelFormats: [AUTO_PIXEL_FORMAT, PIXEL_8BIT],
    defaultPixelFormat: null,
    capabilityKey: "libx264",
  },
  {
    id: "libx265",
    label: "HEVC（libx265，软件）",
    family: "software",
    vendor: "software",
    codec: "hevc",
    qualityMode: "crf",
    qualityLabel: "CRF",
    qualityRange: [0, 51],
    defaultQuality: 18,
    presets: [...SOFTWARE_PRESETS],
    defaultPreset: "slow",
    pixelFormats: [AUTO_PIXEL_FORMAT, PIXEL_8BIT, PIXEL_10BIT_YUV420P10LE],
    defaultPixelFormat: "yuv420p10le",
    capabilityKey: "libx265",
  },
  {
    id: "libvpx-vp9",
    label: "VP9（libvpx-vp9，软件）",
    family: "software",
    vendor: "software",
    codec: "vp9",
    qualityMode: "crf",
    qualityLabel: "CRF",
    qualityRange: [0, 63],
    defaultQuality: 32,
    presets: [...VP9_PRESETS],
    defaultPreset: "2",
    pixelFormats: [AUTO_PIXEL_FORMAT, PIXEL_8BIT],
    defaultPixelFormat: null,
    capabilityKey: "libvpx-vp9",
  },
  {
    id: "libsvtav1",
    label: "AV1（libsvtav1，软件）",
    family: "software",
    vendor: "software",
    codec: "av1",
    qualityMode: "crf",
    qualityLabel: "CRF",
    qualityRange: [0, 63],
    defaultQuality: 35,
    presets: [...SVTAV1_PRESETS],
    defaultPreset: "8",
    pixelFormats: [AUTO_PIXEL_FORMAT, PIXEL_8BIT, PIXEL_10BIT_YUV420P10LE],
    defaultPixelFormat: null,
    capabilityKey: "libsvtav1",
  },
  {
    id: "libaom-av1",
    label: "AV1（libaom-av1，软件）",
    family: "software",
    vendor: "software",
    codec: "av1",
    qualityMode: "crf",
    qualityLabel: "CRF",
    qualityRange: [0, 63],
    defaultQuality: 32,
    presets: [...AOM_PRESETS],
    defaultPreset: "6",
    pixelFormats: [AUTO_PIXEL_FORMAT, PIXEL_8BIT, PIXEL_10BIT_YUV420P10LE],
    defaultPixelFormat: null,
    capabilityKey: "libaom-av1",
  },
  {
    id: "librav1e",
    label: "AV1（librav1e，软件）",
    family: "software",
    vendor: "software",
    codec: "av1",
    qualityMode: "crf",
    qualityLabel: "CRF",
    qualityRange: [0, 63],
    defaultQuality: 32,
    presets: [...RAV1E_PRESETS],
    defaultPreset: "6",
    pixelFormats: [AUTO_PIXEL_FORMAT, PIXEL_8BIT, PIXEL_10BIT_YUV420P10LE],
    defaultPixelFormat: null,
    capabilityKey: "librav1e",
  },
  {
    id: "h264_nvenc",
    label: "H.264（h264_nvenc，NVIDIA）",
    family: "nvenc",
    vendor: "nvidia",
    codec: "h264",
    qualityMode: "cq",
    qualityLabel: "CQ",
    qualityRange: [0, 51],
    defaultQuality: 21,
    presets: [...NVENC_PRESETS],
    defaultPreset: "p5",
    pixelFormats: [AUTO_PIXEL_FORMAT, PIXEL_8BIT],
    defaultPixelFormat: null,
    capabilityKey: "h264_nvenc",
  },
  {
    id: "hevc_nvenc",
    label: "HEVC（hevc_nvenc，NVIDIA）",
    family: "nvenc",
    vendor: "nvidia",
    codec: "hevc",
    qualityMode: "cq",
    qualityLabel: "CQ",
    qualityRange: [0, 51],
    defaultQuality: 21,
    presets: [...NVENC_PRESETS],
    defaultPreset: "p5",
    pixelFormats: [AUTO_PIXEL_FORMAT, PIXEL_8BIT, PIXEL_10BIT_P010LE],
    defaultPixelFormat: null,
    capabilityKey: "hevc_nvenc",
  },
  {
    id: "av1_nvenc",
    label: "AV1（av1_nvenc，NVIDIA）",
    family: "nvenc",
    vendor: "nvidia",
    codec: "av1",
    qualityMode: "cq",
    qualityLabel: "CQ",
    qualityRange: [0, 63],
    defaultQuality: 30,
    presets: [...NVENC_PRESETS],
    defaultPreset: "p5",
    pixelFormats: [AUTO_PIXEL_FORMAT, PIXEL_8BIT, PIXEL_10BIT_P010LE],
    defaultPixelFormat: null,
    capabilityKey: "av1_nvenc",
  },
  {
    id: "h264_qsv",
    label: "H.264（h264_qsv，Intel Quick Sync）",
    family: "qsv",
    vendor: "intel",
    codec: "h264",
    qualityMode: "cq",
    qualityLabel: "ICQ",
    qualityRange: [1, 51],
    defaultQuality: 23,
    presets: [...QSV_PRESETS],
    defaultPreset: "medium",
    pixelFormats: [AUTO_PIXEL_FORMAT, PIXEL_NV12],
    defaultPixelFormat: null,
    capabilityKey: "h264_qsv",
  },
  {
    id: "hevc_qsv",
    label: "HEVC（hevc_qsv，Intel Quick Sync）",
    family: "qsv",
    vendor: "intel",
    codec: "hevc",
    qualityMode: "cq",
    qualityLabel: "ICQ",
    qualityRange: [1, 51],
    defaultQuality: 23,
    presets: [...QSV_PRESETS],
    defaultPreset: "medium",
    pixelFormats: [AUTO_PIXEL_FORMAT, PIXEL_NV12, PIXEL_10BIT_P010LE],
    defaultPixelFormat: null,
    capabilityKey: "hevc_qsv",
  },
  {
    id: "av1_qsv",
    label: "AV1（av1_qsv，Intel Quick Sync）",
    family: "qsv",
    vendor: "intel",
    codec: "av1",
    qualityMode: "cq",
    qualityLabel: "ICQ",
    qualityRange: [1, 63],
    defaultQuality: 32,
    presets: [...QSV_PRESETS],
    defaultPreset: "medium",
    pixelFormats: [AUTO_PIXEL_FORMAT, PIXEL_NV12, PIXEL_10BIT_P010LE],
    defaultPixelFormat: null,
    capabilityKey: "av1_qsv",
  },
  {
    id: "h264_amf",
    label: "H.264（h264_amf，AMD）",
    family: "amf",
    vendor: "amd",
    codec: "h264",
    qualityMode: "cq",
    qualityLabel: "QP",
    qualityRange: [0, 51],
    defaultQuality: 24,
    presets: [...AMF_PRESETS],
    defaultPreset: "balanced",
    pixelFormats: [AUTO_PIXEL_FORMAT, PIXEL_8BIT, PIXEL_NV12],
    defaultPixelFormat: null,
    capabilityKey: "h264_amf",
  },
  {
    id: "hevc_amf",
    label: "HEVC（hevc_amf，AMD）",
    family: "amf",
    vendor: "amd",
    codec: "hevc",
    qualityMode: "cq",
    qualityLabel: "QP",
    qualityRange: [0, 51],
    defaultQuality: 24,
    presets: [...AMF_PRESETS],
    defaultPreset: "balanced",
    pixelFormats: [
      AUTO_PIXEL_FORMAT,
      PIXEL_8BIT,
      PIXEL_NV12,
      PIXEL_10BIT_P010LE,
    ],
    defaultPixelFormat: null,
    capabilityKey: "hevc_amf",
  },
  {
    id: "av1_amf",
    label: "AV1（av1_amf，AMD）",
    family: "amf",
    vendor: "amd",
    codec: "av1",
    qualityMode: "cq",
    qualityLabel: "QP",
    qualityRange: [0, 63],
    defaultQuality: 30,
    presets: [...AMF_PRESETS],
    defaultPreset: "balanced",
    pixelFormats: [
      AUTO_PIXEL_FORMAT,
      PIXEL_8BIT,
      PIXEL_NV12,
      PIXEL_10BIT_P010LE,
    ],
    defaultPixelFormat: null,
    capabilityKey: "av1_amf",
  },
];

/** 厂商显示名（界面标注编码器来源）。 */
export function vendorLabel(vendor: EncoderVendor): string {
  switch (vendor) {
    case "software":
      return "软件（CPU）";
    case "nvidia":
      return "NVIDIA";
    case "amd":
      return "AMD";
    case "intel":
      return "Intel";
  }
}

/** 能力报告中的可用编码器集合（按 ID 索引）。 */
export function availableEncoderMap(
  capabilities: CapabilityReport | null,
): Map<string, EncoderAvailability> {
  const map = new Map<string, EncoderAvailability>();
  for (const item of capabilities?.encoders ?? []) {
    map.set(item.id, item);
  }
  return map;
}

/**
 * 编码器是否可用。
 *
 * 能力报告尚未加载时返回 `true`（先让用户能操作，任务启动前后端会再校验）；
 * 已加载时**完全按后端检测结果**——硬件编码器还要通过实拍探测。
 */
export function encoderUsable(
  capabilities: CapabilityReport | null,
  encoderId: string,
): boolean {
  if (!capabilities) {
    return true;
  }
  return capabilities.encoders.some(
    (item) => item.id === encoderId && item.usable,
  );
}

/** 在能力报告范围内挑选可用的默认编码器（优先软件 H.264）。 */
export function pickDefaultEncoder(
  capabilities: CapabilityReport | null,
): string | undefined {
  const usable = ENCODER_DEFINITIONS.filter((def) =>
    encoderUsable(capabilities, def.id),
  );
  const preferred = usable.find((def) => def.id === "libx264") ?? usable[0];
  return preferred?.id;
}

export function encoderDefinition(id: string): EncoderDefinition | undefined {
  return ENCODER_DEFINITIONS.find((def) => def.id === id);
}

/** 由编码器 ID 生成默认视频配置（切换编码器时重置参数）。 */
export function defaultVideoConfig(encoderId: string): VideoConfig {
  const def = encoderDefinition(encoderId) ?? ENCODER_DEFINITIONS[0];
  return {
    encoderId: def.id,
    qualityMode: def.qualityMode,
    qualityValue: def.defaultQuality,
    preset: def.defaultPreset,
    pixelFormat: def.defaultPixelFormat,
  };
}

/** 校验视频配置；返回可读错误信息，合法时返回 null。 */
export function validateVideoConfig(video: VideoConfig): string | null {
  const def = encoderDefinition(video.encoderId);
  if (!def) {
    return `未知编码器：${video.encoderId}`;
  }
  if (video.qualityMode !== def.qualityMode) {
    return `编码器 ${def.id} 不支持该质量模式`;
  }
  const [min, max] = def.qualityRange;
  if (
    !Number.isInteger(video.qualityValue) ||
    video.qualityValue < min ||
    video.qualityValue > max
  ) {
    return `质量参数需为 ${min}–${max} 之间的整数（当前：${video.qualityValue}）`;
  }
  if (!def.presets.includes(video.preset)) {
    return `预设 ${video.preset} 不受 ${def.id} 支持`;
  }
  const allowed = def.pixelFormats.map((option) => option.value);
  if (video.pixelFormat !== null && !allowed.includes(video.pixelFormat)) {
    return `像素格式 ${video.pixelFormat} 不受 ${def.id} 支持`;
  }
  return null;
}

/**
 * 由输入路径建议输出路径：同目录、`<文件名>.encoded.<容器扩展名>`。
 *
 * 扩展名由显式选择的封装格式决定（不再跟随输入文件）；输入本身是
 * MKV / MP4 / WebM 时用其扩展名作为初始容器，其余输入按容器默认值。
 */
export function suggestOutputPath(
  inputPath: string,
  container: Container,
): string {
  const normalized = inputPath.replaceAll("\\", "/");
  const slash = normalized.lastIndexOf("/");
  const dir = slash >= 0 ? normalized.slice(0, slash) : "";
  const file = slash >= 0 ? normalized.slice(slash + 1) : normalized;
  const dot = file.lastIndexOf(".");
  const stem = dot > 0 ? file.slice(0, dot) : file;
  const output = `${stem}.encoded.${container}`;
  return dir ? `${dir}/${output}` : output;
}

/** 输入文件扩展名对应的容器；不是受支持的容器时返回 null。 */
export function containerFromPath(path: string): Container | null {
  const file = path.replaceAll("\\", "/").split("/").pop() ?? "";
  const dot = file.lastIndexOf(".");
  if (dot <= 0) {
    return null;
  }
  const ext = file.slice(dot + 1).toLowerCase();
  return isContainer(ext) ? ext : null;
}

/** 把路径的扩展名替换为所选容器的扩展名（无扩展名时追加）。 */
export function swapOutputExtension(
  path: string,
  container: Container,
): string {
  const trimmed = path.trim();
  if (!trimmed) {
    return trimmed;
  }
  const normalized = trimmed.replaceAll("\\", "/");
  const slash = normalized.lastIndexOf("/");
  const dir = slash >= 0 ? normalized.slice(0, slash) : "";
  const file = slash >= 0 ? normalized.slice(slash + 1) : normalized;
  const dot = file.lastIndexOf(".");
  const stem = dot > 0 ? file.slice(0, dot) : file;
  const output = `${stem}.${container}`;
  return dir ? `${dir}/${output}` : output;
}

/** 类型守卫：字符串是否为受支持的容器。 */
export function isContainer(value: string): value is Container {
  return (CONTAINER_OPTIONS as ReadonlyArray<{ id: string }>).some(
    (option) => option.id === value,
  );
}

/**
 * 封装格式选项：`note` 是容器的固定说明，界面在下方展示。
 * 这些限制来自 FFmpeg 的封装器本身，应用不代为改写编码器或字幕处理方式。
 */
export const CONTAINER_OPTIONS: ReadonlyArray<{
  id: Container;
  label: string;
  note: string;
}> = [
  {
    id: "mkv",
    label: "MKV（Matroska）",
    note: "兼容性最好，支持全部常见字幕格式与多轨",
  },
  {
    id: "mp4",
    label: "MP4",
    note: "消费设备兼容性好；字幕仅支持 mov_text",
  },
  {
    id: "webm",
    label: "WebM",
    note: "仅支持 VP8 / VP9 / AV1 视频与 Vorbis / Opus 音频（音轨为 AAC 等格式时会被拒绝）",
  },
];

/** 容器标准允许的音频编码（应用目前固定复制原音轨，不转码）。 */
export function containerSupportsAudioCodec(
  container: Container,
  audioCodec: string | null,
): boolean {
  if (container !== "webm") {
    return true;
  }
  if (!audioCodec) {
    return true;
  }
  const codec = audioCodec.toLowerCase();
  return codec === "opus" || codec === "vorbis";
}

/**
 * 容器是否承载得了该视频编码器（判断依据：编码格式 + 容器的标准编码集合）。
 *
 * WebM 只接受 VP8 / VP9 / AV1，其中 VP8 本应用未提供，因此实际可用的是
 * VP9（libvpx-vp9）与全部 AV1 编码器（软件 libsvtav1 / libaom-av1 / librav1e
 * 与硬件 av1_nvenc / av1_qsv / av1_amf）。
 */
export function containerSupportsEncoder(
  container: Container,
  encoderId: string,
): boolean {
  if (container !== "webm") {
    return true;
  }
  const def = encoderDefinition(encoderId);
  return def?.codec === "vp9" || def?.codec === "av1";
}
