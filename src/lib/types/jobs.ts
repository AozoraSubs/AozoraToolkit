import type { EncodeConfig, JobStatus } from "./encode";

/** 队列中任务的元数据（对应 Rust `aozora_core::queue::JobMeta`）。 */
export interface JobMeta {
  id: string;
  name: string;
  inputPath: string;
  /** 计划写入的最终输出路径。 */
  outputPath: string;
  queuedAtMs: number;
}

export interface QueueSnapshot {
  active: JobMeta | null;
  queued: JobMeta[];
}

/** 一条已完成的任务历史（对应 Rust `aozora_core::history::JobRecord`）。 */
export interface JobRecord {
  id: string;
  name: string;
  inputPath: string;
  outputPath: string | null;
  status: JobStatus;
  error: string | null;
  notice: string | null;
  /** 执行时使用的编码配置，用于"载入配置后重试"。 */
  config: EncodeConfig;
  durationSecs: number | null;
  startedAtMs: number;
  finishedAtMs: number;
  elapsedSecs: number;
  /** 日志尾部（失败诊断用，已截断）。 */
  logTail: string[];
}
