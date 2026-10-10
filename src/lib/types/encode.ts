import type { KnownEncoders } from "./tools";

export type QualityMode = "crf" | "cq";
export type AudioMode = "copy";
export type SubtitleMode = "copy" | "drop";

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
  /** copy：保留全部字幕轨；drop：丢弃字幕。 */
  mode: SubtitleMode;
}

export interface OutputConfig {
  path: string;
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
  /** 需要提醒用户的事项（如目标目录不可写、已回退保存位置）。 */
  notice: string | null;
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
}

/** 前端维护的当前任务 UI 状态。 */
export interface ActiveJobUi {
  id: string;
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
}

/** 编码器能力键（与 Rust `KnownEncoders` 对应）。 */
export type EncoderCapabilityKey = keyof KnownEncoders;
