export type QualityMode = "crf" | "cq";
export type AudioMode = "copy";
export type SubtitleMode = "copy" | "drop" | "burn";
/** 输出封装格式（与 Rust `Container` 对应）。 */
export type Container = "mkv" | "mp4" | "webm";

export interface VideoConfig {
  encoderId: string;
  qualityMode: QualityMode;
  qualityValue: number;
  preset: string;
  pixelFormat: string | null;
}

export interface AudioConfig {
  mode: AudioMode;
}

export interface SubtitleConfig {
  /** copy：保留全部字幕轨；drop：丢弃字幕；burn：烧录外部字幕文件。 */
  mode: SubtitleMode;
  /** burn 模式下要烧录的外部字幕文件路径（srt / ass）。 */
  externalPath: string | null;
}

export interface OutputConfig {
  path: string;
  container: Container;
  overwrite: boolean;
}

/** 应用内部的编码配置（对应 Rust `EncodeConfig`，字段为 camelCase）。 */
export interface EncodeConfig {
  schemaVersion: 1;
  video: VideoConfig;
  audio: AudioConfig;
  subtitles: SubtitleConfig;
  output: OutputConfig;
}

export type JobStatus = "completed" | "failed" | "cancelled";

export interface JobStartedEvent {
  jobId: string;
  /** 任务显示名（输入文件名）。 */
  name: string;
  inputPath: string;
  /** 计划写入的最终输出路径（可能因目标目录不可写而回退）。 */
  outputPath: string;
  /** 输入媒体时长（秒），用于进度百分比与 ETA。 */
  durationSecs: number | null;
  /** 需要提醒用户的事项（如目标目录不可写、已回退保存位置）。 */
  notice: string | null;
  startedAtMs: number;
}

export interface JobProgressEvent {
  jobId: string;
  percent: number;
  outTimeSecs: number;
  durationKnown: boolean;
  durationSecs: number | null;
  speed: number | null;
  fps: number | null;
  etaSecs: number | null;
}

export interface JobLogEvent {
  jobId: string;
  line: string;
}

export interface JobFinishedEvent {
  jobId: string;
  status: JobStatus;
  error: string | null;
  outputPath: string | null;
  elapsedSecs: number;
  /** 写入任务历史时的错误（例如历史文件版本不受支持）。 */
  historyError: string | null;
}

/** 入队结果（对应 Rust `EnqueueResultDto`）。 */
export interface EnqueueResult {
  jobId: string;
  started: boolean;
  /** 排队位置（1 起算）；已开始运行时为 null。 */
  position: number | null;
}

/** 前端维护的当前任务 UI 状态。 */
export interface ActiveJobUi {
  id: string;
  name: string;
  inputPath: string;
  startedAt: number;
  /** 需要提醒用户的事项（如目标目录不可写、已回退保存位置）。 */
  notice: string | null;
  percent: number;
  outTimeSecs: number;
  durationKnown: boolean;
  durationSecs: number | null;
  speed: number | null;
  fps: number | null;
  etaSecs: number | null;
  logs: string[];
  finished: boolean;
  status: JobStatus | null;
  error: string | null;
  outputPath: string | null;
  elapsedSecs: number;
  historyError: string | null;
}

/** 编码器能力键（编码器 ID，与后端能力报告中的 `id` 对应）。 */
export type EncoderCapabilityKey = string;
