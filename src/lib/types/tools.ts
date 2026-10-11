export type ToolSource = "path" | "app_bin" | "custom";

export interface ToolInfo {
  path: string;
  version: string | null;
  source: ToolSource;
}

export interface ToolDetectionResult {
  ffmpeg: ToolInfo | null;
  ffprobe: ToolInfo | null;
  customToolDir: string | null;
  errors: string[];
}

/** 编码器厂商：与 Rust `EncoderVendor` 对应。 */
export type EncoderVendor = "software" | "nvidia" | "amd" | "intel";

/** 单个编码器的实际可用性（由后端能力检测得出）。 */
export interface EncoderAvailability {
  id: string;
  label: string;
  vendor: EncoderVendor;
  /** 输出视频编码格式：h264 / hevc / av1 / vp9。 */
  codec: string;
  /** 是否为硬件（显卡）编码器。 */
  hardware: boolean;
  /** 是否出现在 `ffmpeg -encoders` 中（该构建是否编译了它）。 */
  listed: boolean;
  /** 是否真正可用（编译进来了 + 本机实拍通过）。 */
  usable: boolean;
  /** 不可用原因。 */
  reason: string | null;
}

/** 显示适配器（显卡）信息。 */
export interface GpuAdapter {
  name: string;
  /** 归一化厂商：nvidia / amd / intel / other。 */
  vendor: string;
  driver: string | null;
}

/** FFmpeg 能力检测结果。 */
export interface CapabilityReport {
  encoders: EncoderAvailability[];
  encoderCount: number;
  filterCount: number;
  /** 是否有 subtitles 滤镜（libass）——外部字幕烧录依赖它。 */
  hasSubtitlesFilter: boolean;
  hwAccels: string[];
  /** 检测到的显卡，用于解释硬件编码器可用性。 */
  gpus: GpuAdapter[];
  warnings: string[];
}
