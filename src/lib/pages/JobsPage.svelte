<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { revealItemInDir } from "@tauri-apps/plugin-opener";
  import { onMount } from "svelte";
  import { loadFromJobRecord } from "../stores/encodeForm.svelte";
  import {
    clearFinishedJob,
    clearHistory,
    deleteHistoryEntry,
    jobState,
    refreshHistory,
    refreshQueue,
    removeQueuedJob,
  } from "../stores/jobs.svelte";
  import { navigation } from "../stores/navigation.svelte";
  import type { JobRecord } from "../types/jobs";
  import { formatSeconds } from "../utils/format";
  import { formatDateTime, jobStatusClass, jobStatusLabel } from "../utils/job";

  let notice = $state<string | null>(null);
  let actionError = $state<string | null>(null);
  let busy = $state(false);
  let confirmClear = $state(false);
  let confirmDeleteId = $state<string | null>(null);
  let expandedId = $state<string | null>(null);

  const job = $derived(jobState.job);
  const jobRunning = $derived(!!job && !job.finished);
  // 历史文件损坏或版本不受支持时，允许清空（重置）而不是让用户无从下手
  const canClearHistory = $derived(
    jobState.history.length > 0 || jobState.historyError !== null,
  );

  onMount(() => {
    void refreshQueue();
    void refreshHistory();
  });

  async function cancelJob() {
    const current = jobState.job;
    if (!current || current.finished) {
      return;
    }
    try {
      await invoke("cancel_job", { jobId: current.id });
    } catch (err) {
      actionError = err instanceof Error ? err.message : String(err);
    }
  }

  async function removeQueued(jobId: string) {
    actionError = null;
    try {
      await removeQueuedJob(jobId);
    } catch (err) {
      actionError = err instanceof Error ? err.message : String(err);
    }
  }

  /** 载入任务配置到压制页，用户可在修改参数后重新开始。 */
  async function retry(record: JobRecord) {
    actionError = null;
    notice = null;
    busy = true;
    try {
      const message = await loadFromJobRecord(record);
      notice = message;
      navigation.activePage = "encode";
    } catch (err) {
      actionError = err instanceof Error ? err.message : String(err);
    } finally {
      busy = false;
    }
  }

  async function openOutput(record: JobRecord) {
    if (!record.outputPath) {
      return;
    }
    actionError = null;
    try {
      await revealItemInDir(record.outputPath);
    } catch (err) {
      actionError = err instanceof Error ? err.message : String(err);
    }
  }

  async function deleteEntry(record: JobRecord) {
    actionError = null;
    notice = null;
    try {
      await deleteHistoryEntry(record.id);
      confirmDeleteId = null;
      notice = `已删除历史记录：${record.name}`;
    } catch (err) {
      actionError = err instanceof Error ? err.message : String(err);
    }
  }

  async function clearAll() {
    actionError = null;
    notice = null;
    try {
      await clearHistory();
      confirmClear = false;
      notice = "已清空任务历史";
    } catch (err) {
      actionError = err instanceof Error ? err.message : String(err);
    }
  }

  const elapsed = $derived(
    job && job.finished ? job.elapsedSecs : jobState.elapsed,
  );
</script>

<section class="page">
  <header>
    <h1>任务</h1>
    <p class="hint">
      运行中的任务、等待队列与历史记录；失败任务可查看日志并载入配置重试。
    </p>
  </header>

  <div class="toolbar">
    <button
      type="button"
      onclick={() => {
        void refreshQueue();
        void refreshHistory();
      }}
    >
      刷新
    </button>
    {#if confirmClear}
      <button
        type="button"
        class="danger"
        onclick={() => void clearAll()}
        disabled={!canClearHistory}
        >确认清空历史（{jobState.history.length} 条）</button
      >
      <button type="button" onclick={() => (confirmClear = false)}>取消</button>
    {:else}
      <button
        type="button"
        class="danger"
        onclick={() => (confirmClear = true)}
        disabled={!canClearHistory}
      >
        {jobState.historyError ? "重置历史文件" : "清空历史"}
      </button>
    {/if}
    <span class="toolbar-hint"
      >历史保存在应用配置目录，最多保留最近 50 条记录</span
    >
  </div>

  {#if notice}
    <div class="notice" role="status">{notice}</div>
  {/if}
  {#if actionError}
    <div class="errors" role="alert">
      <strong>操作失败</strong>
      <p>{actionError}</p>
    </div>
  {/if}
  {#if jobState.historyError}
    <div class="errors" role="alert">
      <strong>任务历史读取失败</strong>
      <p>{jobState.historyError}</p>
    </div>
  {/if}

  <section class="group">
    <h2>运行中</h2>
    {#if jobRunning && job}
      <div class="card">
        <div class="card-head">
          <strong>{job.name}</strong>
          <span class="path" title={job.inputPath}>{job.inputPath}</span>
        </div>
        {#if job.notice}
          <div class="notice" role="status">{job.notice}</div>
        {/if}
        <div class="progress-track">
          <div class="progress-fill" style:width="{job.percent}%"></div>
        </div>
        <dl class="meta">
          <div>
            <dt>进度</dt>
            <dd>
              {job.durationKnown ? `${job.percent.toFixed(1)}%` : "—"}
              {job.durationKnown && job.durationSecs !== null
                ? `（${formatSeconds(job.outTimeSecs)} / ${formatSeconds(job.durationSecs)}）`
                : ""}
            </dd>
          </div>
          <div>
            <dt>速度</dt>
            <dd>{job.speed !== null ? `${job.speed.toFixed(2)}×` : "—"}</dd>
          </div>
          <div>
            <dt>已用时间</dt>
            <dd>{formatSeconds(elapsed)}</dd>
          </div>
          <div>
            <dt>剩余时间</dt>
            <dd>{job.etaSecs !== null ? formatSeconds(job.etaSecs) : "—"}</dd>
          </div>
        </dl>
        <div class="actions">
          <button type="button" class="danger" onclick={cancelJob}
            >取消任务</button
          >
        </div>
        <details class="log">
          <summary>日志（{job.logs.length} 行）</summary>
          <pre class="log-pre">{job.logs.join("\n")}</pre>
        </details>
      </div>
    {:else if job?.finished}
      <div class="card">
        <div class="card-head">
          <strong>{job.name}</strong>
          <span class={jobStatusClass(job.status ?? "failed")}
            >{jobStatusLabel(job.status ?? "failed")}</span
          >
        </div>
        <p class="hint">
          最近结束的任务（已用 {formatSeconds(
            job.elapsedSecs,
          )}）已写入历史，可在下方查看。
        </p>
        <div class="actions">
          <button type="button" onclick={clearFinishedJob}>清除结果面板</button>
        </div>
      </div>
    {:else}
      <p class="hint">当前没有正在运行的任务。</p>
    {/if}
  </section>

  <section class="group">
    <h2>排队中（{jobState.queue.length}）</h2>
    {#if jobState.queue.length === 0}
      <p class="hint">
        队列为空。压制页在已有任务运行时点击「加入队列」即可排队，任务会按顺序自动开始。
      </p>
    {:else}
      <ul class="queue-list">
        {#each jobState.queue as queued, index (queued.id)}
          <li>
            <span class="queue-index">{index + 1}</span>
            <span class="queue-name" title={queued.inputPath}
              >{queued.name}</span
            >
            <span class="path queue-target" title={queued.outputPath}
              >→ {queued.outputPath}</span
            >
            <span class="queue-time"
              >加入于 {formatDateTime(queued.queuedAtMs)}</span
            >
            <button
              type="button"
              class="danger"
              onclick={() => void removeQueued(queued.id)}>移除</button
            >
          </li>
        {/each}
      </ul>
    {/if}
  </section>

  <section class="group">
    <h2>历史记录（{jobState.history.length}）</h2>
    {#if jobState.historyLoading && jobState.history.length === 0}
      <p class="hint">正在读取历史…</p>
    {:else if jobState.history.length === 0}
      <p class="hint">暂无历史记录。完成、失败或取消的任务都会记录在这里。</p>
    {:else}
      <ul class="history-list">
        {#each jobState.history as record (record.id)}
          <li class="history-item">
            <div class="history-head">
              <span class={jobStatusClass(record.status)}
                >{jobStatusLabel(record.status)}</span
              >
              <strong class="history-name" title={record.inputPath}
                >{record.name}</strong
              >
              <span class="history-time"
                >{formatDateTime(record.finishedAtMs)} · 用时 {formatSeconds(
                  record.elapsedSecs,
                )}</span
              >
            </div>
            <dl class="meta">
              <div>
                <dt>输入</dt>
                <dd class="path" title={record.inputPath}>
                  {record.inputPath}
                </dd>
              </div>
              <div>
                <dt>输出</dt>
                <dd class="path" title={record.outputPath ?? ""}>
                  {record.outputPath ?? "—"}
                </dd>
              </div>
              <div>
                <dt>编码参数</dt>
                <dd>
                  {record.config.video.encoderId} ·
                  {record.config.video.qualityMode.toUpperCase()}
                  {record.config.video.qualityValue} ·
                  {record.config.video.preset}{record.config.video.pixelFormat
                    ? ` · ${record.config.video.pixelFormat}`
                    : ""}
                </dd>
              </div>
            </dl>
            {#if record.notice}
              <div class="notice" role="status">{record.notice}</div>
            {/if}
            {#if record.status === "failed" && record.error}
              <div class="errors" role="alert">
                <strong>失败原因</strong>
                <p>{record.error}</p>
              </div>
            {/if}
            <div class="actions">
              <button
                type="button"
                class="primary"
                onclick={() => void retry(record)}
                disabled={busy}>载入配置并重试</button
              >
              <button
                type="button"
                onclick={() =>
                  (expandedId = expandedId === record.id ? null : record.id)}
              >
                {expandedId === record.id
                  ? "收起日志"
                  : `查看日志（${record.logTail.length} 行）`}
              </button>
              {#if record.outputPath}
                <button type="button" onclick={() => void openOutput(record)}
                  >打开输出目录</button
                >
              {/if}
              {#if confirmDeleteId === record.id}
                <button
                  type="button"
                  class="danger"
                  onclick={() => void deleteEntry(record)}>确认删除</button
                >
                <button type="button" onclick={() => (confirmDeleteId = null)}
                  >取消</button
                >
              {:else}
                <button
                  type="button"
                  class="danger"
                  onclick={() => (confirmDeleteId = record.id)}>删除记录</button
                >
              {/if}
            </div>
            {#if expandedId === record.id}
              <pre class="log-pre">{record.logTail.join("\n") ||
                  "（没有日志）"}</pre>
            {/if}
          </li>
        {/each}
      </ul>
    {/if}
  </section>
</section>

<style>
  .toolbar {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    align-items: center;
    margin-bottom: 12px;
  }

  .toolbar-hint {
    font-size: 12px;
    color: var(--color-text-secondary);
  }

  .group {
    margin-bottom: 20px;
    max-width: 900px;
  }

  .group h2 {
    margin: 0 0 8px;
    font-size: 15px;
  }

  .card {
    padding: 12px 14px;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: 10px;
  }

  .card-head {
    display: flex;
    gap: 10px;
    align-items: baseline;
    margin-bottom: 8px;
  }

  .progress-track {
    height: 10px;
    margin-bottom: 10px;
    border-radius: 5px;
    background: var(--color-hover);
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    border-radius: 5px;
    background: var(--color-accent);
    transition: width 0.25s ease;
  }

  .meta {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
    gap: 6px 16px;
    margin: 0;
  }

  .meta dt {
    font-size: 12px;
    color: var(--color-text-secondary);
  }

  .meta dd {
    margin: 0;
    font-size: 13px;
    word-break: break-all;
  }

  .path {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 100%;
  }

  .queue-list,
  .history-list {
    display: flex;
    flex-direction: column;
    gap: 10px;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .queue-list li {
    display: flex;
    gap: 10px;
    align-items: center;
    padding: 8px 12px;
    font-size: 13px;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: 8px;
  }

  .queue-index {
    display: grid;
    place-items: center;
    width: 20px;
    height: 20px;
    border-radius: 50%;
    background: var(--color-hover);
    font-size: 12px;
  }

  .queue-name {
    font-weight: 600;
  }

  .queue-target {
    flex: 1;
    color: var(--color-text-secondary);
  }

  .queue-time {
    font-size: 12px;
    color: var(--color-text-secondary);
  }

  .history-item {
    padding: 12px 14px;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: 10px;
  }

  .history-head {
    display: flex;
    gap: 10px;
    align-items: center;
    margin-bottom: 8px;
  }

  .history-name {
    word-break: break-all;
  }

  .history-time {
    font-size: 12px;
    color: var(--color-text-secondary);
  }

  .status-completed,
  .status-failed,
  .status-cancelled {
    padding: 1px 8px;
    border: 1px solid var(--color-border);
    border-radius: 999px;
    font-size: 12px;
    white-space: nowrap;
  }

  .status-completed {
    color: var(--color-accent);
    border-color: var(--color-accent);
  }

  .status-failed {
    color: var(--color-danger);
    border-color: var(--color-danger);
  }

  .status-cancelled {
    color: var(--color-text-secondary);
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin-top: 10px;
  }

  button {
    padding: 6px 12px;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    background: var(--color-surface);
    color: var(--color-text);
    cursor: pointer;
  }

  button.primary {
    background: var(--color-accent);
    border-color: var(--color-accent);
    color: var(--color-accent-contrast);
    font-weight: 600;
  }

  button.danger {
    border-color: var(--color-danger);
    color: var(--color-danger);
  }

  button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .log {
    margin-top: 10px;
  }

  .log summary {
    cursor: pointer;
    font-size: 13px;
    color: var(--color-text-secondary);
  }

  .log-pre {
    margin: 8px 0 0;
    padding: 10px;
    max-height: 240px;
    overflow: auto;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    background: var(--color-background);
    font-size: 12px;
    white-space: pre-wrap;
    word-break: break-all;
  }

  .errors {
    margin: 0 0 10px;
    padding: 10px 12px;
    border: 1px solid var(--color-danger);
    border-radius: 8px;
    background: var(--color-surface);
    color: var(--color-danger);
    word-break: break-all;
  }

  .errors p {
    margin: 4px 0 0;
  }

  .notice {
    margin: 0 0 10px;
    padding: 8px 10px;
    border: 1px solid var(--color-warning, var(--color-border));
    border-radius: 6px;
    background: var(--color-surface);
    font-size: 13px;
    word-break: break-all;
  }
</style>
