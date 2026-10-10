import { beforeEach, describe, expect, it, vi } from "vitest";

const invokeMock = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

import {
  jobState,
  onJobFinished,
  onJobLog,
  onJobProgress,
  onJobQueue,
  onJobStarted,
  refreshHistory,
  tickElapsed,
} from "./jobs.svelte";
import type { JobStartedEvent } from "../types/encode";

function startedEvent(overrides: Partial<JobStartedEvent> = {}): JobStartedEvent {
  return {
    jobId: "e-1",
    name: "样片.mkv",
    inputPath: "D:/影片/样片.mkv",
    outputPath: "D:/影片/样片.encoded.mkv",
    durationSecs: 120,
    notice: null,
    startedAtMs: 1_700_000_000_000,
    ...overrides,
  };
}

beforeEach(() => {
  invokeMock.mockReset();
  invokeMock.mockResolvedValue([]);
  jobState.job = null;
  jobState.elapsed = 0;
  jobState.queue = [];
  jobState.activeMeta = null;
  jobState.history = [];
  jobState.historyError = null;
});

describe("任务事件处理", () => {
  it("任务开始时按事件载荷初始化状态", () => {
    onJobStarted(startedEvent({ notice: "目标目录不可写" }));
    const job = jobState.job;
    expect(job).not.toBeNull();
    expect(job?.id).toBe("e-1");
    expect(job?.name).toBe("样片.mkv");
    expect(job?.durationKnown).toBe(true);
    expect(job?.notice).toBe("目标目录不可写");
    expect(job?.finished).toBe(false);
    expect(jobState.elapsed).toBe(0);
  });

  it("时长未知时不显示百分比", () => {
    onJobStarted(startedEvent({ durationSecs: null }));
    expect(jobState.job?.durationKnown).toBe(false);
  });

  it("进度与日志只作用于当前任务", () => {
    onJobStarted(startedEvent());
    onJobProgress({
      jobId: "e-other",
      percent: 50,
      outTimeSecs: 60,
      durationKnown: true,
      durationSecs: 120,
      speed: 2,
      fps: 25,
      etaSecs: 30,
    });
    expect(jobState.job?.percent).toBe(0);

    onJobProgress({
      jobId: "e-1",
      percent: 25,
      outTimeSecs: 30,
      durationKnown: true,
      durationSecs: 120,
      speed: 2,
      fps: 25,
      etaSecs: 45,
    });
    expect(jobState.job?.percent).toBe(25);
    expect(jobState.job?.etaSecs).toBe(45);

    onJobLog({ jobId: "e-other", line: "无关日志" });
    onJobLog({ jobId: "e-1", line: "frame=1" });
    expect(jobState.job?.logs).toEqual(["frame=1"]);
  });

  it("日志行数有上限，避免长任务无限增长", () => {
    onJobStarted(startedEvent());
    for (let index = 0; index < 600; index += 1) {
      onJobLog({ jobId: "e-1", line: `line ${index}` });
    }
    expect(jobState.job?.logs.length).toBe(500);
    expect(jobState.job?.logs[0]).toBe("line 100");
  });

  it("任务结束写入结果并刷新历史", async () => {
    onJobStarted(startedEvent());
    onJobFinished({
      jobId: "e-1",
      status: "failed",
      error: "FFmpeg 退出码：1",
      outputPath: null,
      elapsedSecs: 12.5,
      historyError: null,
    });
    expect(jobState.job?.finished).toBe(true);
    expect(jobState.job?.status).toBe("failed");
    expect(jobState.job?.elapsedSecs).toBe(12.5);
    await vi.waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith("list_job_history");
    });
  });

  it("已用时间在任务结束后不再增长", () => {
    onJobStarted(startedEvent({ startedAtMs: Date.now() - 5_000 }));
    tickElapsed();
    expect(jobState.elapsed).toBeGreaterThanOrEqual(4.9);
    onJobFinished({
      jobId: "e-1",
      status: "completed",
      error: null,
      outputPath: "D:/out.mkv",
      elapsedSecs: 6,
      historyError: null,
    });
    const frozen = jobState.elapsed;
    tickElapsed();
    expect(jobState.elapsed).toBe(frozen);
  });
});

describe("队列与历史", () => {
  it("队列快照覆盖运行中与排队任务", () => {
    onJobQueue({
      active: {
        id: "e-1",
        name: "a.mkv",
        inputPath: "D:/a.mkv",
        outputPath: "D:/a.out.mkv",
        queuedAtMs: 1,
      },
      queued: [
        {
          id: "e-2",
          name: "b.mkv",
          inputPath: "D:/b.mkv",
          outputPath: "D:/b.out.mkv",
          queuedAtMs: 2,
        },
      ],
    });
    expect(jobState.queue.map((job) => job.id)).toEqual(["e-2"]);
    expect(jobState.activeMeta?.id).toBe("e-1");
  });

  it("历史读取失败时给出可读错误而不是静默清空", async () => {
    invokeMock.mockRejectedValueOnce(new Error("任务历史文件格式无效"));
    await refreshHistory();
    expect(jobState.history).toEqual([]);
    expect(jobState.historyError).toContain("格式无效");
  });
});
