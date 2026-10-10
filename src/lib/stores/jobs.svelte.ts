import type {
  ActiveJobUi,
  JobFinishedEvent,
  JobLogEvent,
  JobProgressEvent,
  JobStartedEvent,
} from "../types/encode";

const MAX_LOG_LINES = 500;

/**
 * 全局任务状态：独立于页面组件，切换页面后进度与日志不丢失。
 * 事件监听在 App.svelte（常驻组件）中统一接线。
 */
export const jobState = $state<{ job: ActiveJobUi | null; elapsed: number }>({
  job: null,
  elapsed: 0,
});

/** 开始新任务时初始化 UI 状态（在调用 start_job 前调用）。 */
export function startJobUi(durationSecs: number | null) {
  jobState.job = {
    id: "",
    startedAt: Date.now(),
    notice: null,
    percent: 0,
    outTimeSecs: 0,
    durationKnown: durationSecs !== null,
    durationSecs,
    speed: null,
    fps: null,
    etaSecs: null,
    logs: [],
    finished: false,
    status: null,
    error: null,
    outputPath: null,
    elapsedSecs: 0,
  };
  jobState.elapsed = 0;
}

/** 清除任务 UI 状态。 */
export function resetJobUi() {
  jobState.job = null;
  jobState.elapsed = 0;
}

export function onJobStarted(payload: JobStartedEvent) {
  const job = jobState.job;
  if (job) {
    job.id = payload.jobId;
    job.startedAt = Date.now();
    job.notice = payload.notice;
  }
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
  if (!job || job.id !== payload.jobId) {
    return;
  }
  job.finished = true;
  job.status = payload.status;
  job.error = payload.error;
  job.outputPath = payload.outputPath;
  job.elapsedSecs = payload.elapsedSecs;
}

/** 由外部定时器调用：刷新运行中任务的已用时间。 */
export function tickElapsed() {
  const job = jobState.job;
  if (job && !job.finished) {
    jobState.elapsed = (Date.now() - job.startedAt) / 1000;
  }
}
