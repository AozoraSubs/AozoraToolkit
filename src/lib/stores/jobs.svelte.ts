import { invoke } from "@tauri-apps/api/core";
import type {
  ActiveJobUi,
  JobFinishedEvent,
  JobLogEvent,
  JobProgressEvent,
  JobStartedEvent,
} from "../types/encode";
import type { JobMeta, JobRecord, QueueSnapshot } from "../types/jobs";

const MAX_LOG_LINES = 500;

/**
 * 全局任务状态：当前任务、队列与历史，独立于页面组件，
 * 切换页面后进度、日志与队列不丢失。
 * 事件监听在 App.svelte（常驻组件）中统一接线。
 */
export const jobState = $state<{
  job: ActiveJobUi | null;
  elapsed: number;
  /** 排队中的任务（不含正在运行的任务）。 */
  queue: JobMeta[];
  /** 正在运行任务的元数据（列表展示用）。 */
  activeMeta: JobMeta | null;
  history: JobRecord[];
  historyError: string | null;
  historyLoading: boolean;
  /** 队列事件序号：用于避免较旧的快照覆盖较新的事件状态。 */
  queueSeq: number;
}>({
  job: null,
  elapsed: 0,
  queue: [],
  activeMeta: null,
  history: [],
  historyError: null,
  historyLoading: false,
  queueSeq: 0,
});

/** 由队列元数据构造运行中任务的占位状态（事件丢失或界面重载时兜底）。 */
function placeholderJob(meta: JobMeta): ActiveJobUi {
  return {
    id: meta.id,
    name: meta.name,
    inputPath: meta.inputPath,
    startedAt: Date.now(),
    notice: null,
    percent: 0,
    outTimeSecs: 0,
    // 时长未知，不显示误导性的百分比
    durationKnown: false,
    durationSecs: null,
    speed: null,
    fps: null,
    etaSecs: null,
    logs: [],
    finished: false,
    status: null,
    error: null,
    outputPath: null,
    elapsedSecs: 0,
    historyError: null,
  };
}

/** 队列中是否还有等待运行的任务。 */
export function hasQueuedJobs(): boolean {
  return jobState.queue.length > 0;
}

/** 任务开始（由 `job:started` 事件驱动，界面不预先创建状态，避免竞态）。 */
export function onJobStarted(payload: JobStartedEvent) {
  jobState.job = {
    id: payload.jobId,
    name: payload.name,
    inputPath: payload.inputPath,
    startedAt: payload.startedAtMs || Date.now(),
    notice: payload.notice,
    percent: 0,
    outTimeSecs: 0,
    durationKnown: payload.durationSecs !== null,
    durationSecs: payload.durationSecs,
    speed: null,
    fps: null,
    etaSecs: null,
    logs: [],
    finished: false,
    status: null,
    error: null,
    outputPath: null,
    elapsedSecs: 0,
    historyError: null,
  };
  jobState.elapsed = 0;
}

export function onJobProgress(payload: JobProgressEvent) {
  const job = jobState.job;
  if (!job || job.id !== payload.jobId) {
    return;
  }
  job.percent = payload.percent;
  job.outTimeSecs = payload.outTimeSecs;
  job.durationKnown = payload.durationKnown;
  job.durationSecs = payload.durationSecs;
  job.speed = payload.speed;
  job.fps = payload.fps;
  job.etaSecs = payload.etaSecs;
}

export function onJobLog(payload: JobLogEvent) {
  const job = jobState.job;
  if (!job || job.id !== payload.jobId) {
    return;
  }
  if (job.logs.length >= MAX_LOG_LINES) {
    job.logs.shift();
  }
  job.logs.push(payload.line);
}

export function onJobFinished(payload: JobFinishedEvent) {
  const job = jobState.job;
  if (job && job.id === payload.jobId) {
    job.finished = true;
    job.status = payload.status;
    job.error = payload.error;
    job.outputPath = payload.outputPath;
    job.elapsedSecs = payload.elapsedSecs;
    job.historyError = payload.historyError;
  }
  // 任务结束后历史文件已写入，刷新历史列表
  void refreshHistory();
}

/** 队列快照（入队 / 启动 / 结束 / 移除时由后端推送）。 */
export function onJobQueue(payload: QueueSnapshot) {
  jobState.queueSeq += 1;
  jobState.queue = payload.queued;
  jobState.activeMeta = payload.active;
  // 事件丢失或界面重载时兜底：队列显示有运行中任务，但界面没有对应状态
  if (payload.active && jobState.job?.id !== payload.active.id) {
    if (!jobState.job || jobState.job.finished) {
      jobState.job = placeholderJob(payload.active);
      jobState.elapsed = 0;
    }
  }
}

/** 由外部定时器调用：刷新运行中任务的已用时间。 */
export function tickElapsed() {
  const job = jobState.job;
  if (job && !job.finished) {
    jobState.elapsed = Math.max(0, (Date.now() - job.startedAt) / 1000);
  }
}

/** 清除已结束任务的结果面板（运行中的任务不受影响）。 */
export function clearFinishedJob() {
  if (jobState.job?.finished) {
    jobState.job = null;
    jobState.elapsed = 0;
  }
}

/** 主动同步队列状态（界面初始化时调用一次）。 */
export async function refreshQueue(): Promise<void> {
  try {
    // 记录请求前的序号：await 期间若有更新的队列事件到达，丢弃这份较旧的快照
    const seq = jobState.queueSeq;
    const snapshot = await invoke<QueueSnapshot>("queue_snapshot");
    if (jobState.queueSeq !== seq) {
      return;
    }
    onJobQueue(snapshot);
  } catch {
    // 队列快照失败不阻塞界面；后续事件仍会更新
  }
}

/**
 * 读取任务历史。
 *
 * 读取失败（文件损坏或版本不受支持）时保留上一次已知良好的列表，
 * 只设置错误信息——避免界面看起来像"历史被清空了"。
 */
export async function refreshHistory(): Promise<void> {
  jobState.historyLoading = true;
  try {
    jobState.history = await invoke<JobRecord[]>("list_job_history");
    jobState.historyError = null;
  } catch (err) {
    jobState.historyError = err instanceof Error ? err.message : String(err);
  } finally {
    jobState.historyLoading = false;
  }
}

/** 删除单条历史记录。 */
export async function deleteHistoryEntry(jobId: string): Promise<void> {
  jobState.history = await invoke<JobRecord[]>("delete_job_history", { jobId });
  jobState.historyError = null;
}

/** 清空历史记录。 */
export async function clearHistory(): Promise<void> {
  await invoke<void>("clear_job_history");
  jobState.history = [];
  jobState.historyError = null;
}

/** 移除排队中的任务。 */
export async function removeQueuedJob(jobId: string): Promise<void> {
  await invoke<void>("remove_queued_job", { jobId });
}
