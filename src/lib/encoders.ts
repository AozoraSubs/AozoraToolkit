import type {
  Container,
  EncoderCapabilityKey,
  QualityMode,
  VideoConfig,
} from "./types/encode";

export type EncoderFamily = "software" | "nvenc";

export interface PixelFormatOption {
  /** null = 自动（不传 -pix_fmt）。 */
  value: string | null;
  label: string;
}

export interface EncoderDefinition {
  id: string;
  label: string;
  family: EncoderFamily;
  qualityMode: QualityMode;
  /** 质量参数在 UI 中的名称。 */
  qualityLabel: "CRF" | "CQ";
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

const AUTO_PIXEL_FORMAT: PixelFormatOption = { value: null, label: "自动" };
const PIXEL_8BIT: PixelFormatOption = {
  value: "yuv420p",
  label: "8-bit（yuv420p）",
};
const PIXEL_10BIT_YUV420P10LE: PixelFormatOption = {
  value: "yuv420p10le",
  label: "10-bit（yuv420p10le）",
};
const PIXEL_10BIT_P010LE: PixelFormatOption = {
  value: "p010le",
  label: "10-bit（p010le）",
};

/** 内置编码器元数据（与 Rust `ENCODER_SPECS` 保持一致）。 */
export const ENCODER_DEFINITIONS: EncoderDefinition[] = [
  {
    id: "libx264",
    label: "H.264（libx264，软件）",
    family: "software",
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
    id: "h264_nvenc",
    label: "H.264（h264_nvenc，NVIDIA）",
    family: "nvenc",
    qualityMode: "cq",
    qualityLabel: "CQ",
    qualityRange: [0, 51],
    defaultQuality: 21,
    presets: [...NVENC_PRESETS],
    defaultPreset: "p5",
    pixelFormats: [AUTO_PIXEL_FORMAT, PIXEL_8BIT],
    defaultPixelFormat: null,
    capabilityKey: "h264Nvenc",
  },
  {
    id: "hevc_nvenc",
    label: "HEVC（hevc_nvenc，NVIDIA）",
    family: "nvenc",
    qualityMode: "cq",
    qualityLabel: "CQ",
    qualityRange: [0, 51],
    defaultQuality: 21,
    presets: [...NVENC_PRESETS],
    defaultPreset: "p5",
    pixelFormats: [AUTO_PIXEL_FORMAT, PIXEL_8BIT, PIXEL_10BIT_P010LE],
    defaultPixelFormat: null,
    capabilityKey: "hevcNvenc",
  },
  {
    id: "av1_nvenc",
    label: "AV1（av1_nvenc，NVIDIA）",
    family: "nvenc",
    qualityMode: "cq",
    qualityLabel: "CQ",
    qualityRange: [0, 63],
    defaultQuality: 30,
    presets: [...NVENC_PRESETS],
    defaultPreset: "p5",
    pixelFormats: [AUTO_PIXEL_FORMAT, PIXEL_8BIT, PIXEL_10BIT_P010LE],
    defaultPixelFormat: null,
    capabilityKey: "av1Nvenc",
  },
];

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
    note: "仅支持 VP8 / VP9 / AV1 视频与 Opus / Vorbis 音频",
  },
];

/** 容器是否承载得了当前视频编码器（判断依据：容器的标准编码集合）。 */
export function containerSupportsEncoder(
  container: Container,
  encoderId: string,
): boolean {
  if (container !== "webm") {
    return true;
  }
  // WebM 只接受 VP8 / VP9 / AV1；本应用当前编码器集合里只有 AV1 可用。
  return encoderId.includes("av1") || encoderId.startsWith("libvpx");
}
