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

/** 应用关注的编码器可用性。 */
export interface KnownEncoders {
  libx264: boolean;
  libx265: boolean;
  h264Nvenc: boolean;
  hevcNvenc: boolean;
  av1Nvenc: boolean;
}

/** FFmpeg 能力检测结果。 */
export interface CapabilityReport {
  encoders: KnownEncoders;
  encoderCount: number;
  filterCount: number;
  /** 是否有 subtitles 滤镜（libass）——外部字幕烧录依赖它。 */
  hasSubtitlesFilter: boolean;
  hwAccels: string[];
  warnings: string[];
}
