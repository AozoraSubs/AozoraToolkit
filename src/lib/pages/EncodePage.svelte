<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { open, save } from "@tauri-apps/plugin-dialog";
  import { onDestroy, onMount } from "svelte";
  import {
    ENCODER_DEFINITIONS,
    defaultVideoConfig,
    encoderDefinition,
    suggestOutputPath,
    validateVideoConfig,
  } from "../encoders";
  import type {
    ActiveJobUi,
    EncodeConfig,
    JobFinishedEvent,
    JobLogEvent,
    JobProgressEvent,
    JobStartedEvent,
    VideoConfig,
  } from "../types/encode";
  import type { MediaProbeResult } from "../types/media";
  import type { CapabilityReport } from "../types/tools";
  import { formatBytes, formatSeconds } from "../utils/format";

  const MAX_LOG_LINES = 500;

  let info = $state<MediaProbeResult | null>(null);
  let busy = $state(false);
  let error = $state<string | null>(null);
  let dragging = $state(false);

  // 编码能力与表单状态
  let capabilities = $state<CapabilityReport | null>(null);
  let video = $state<VideoConfig>(defaultVideoConfig("libx264"));
  let outputPath = $state("");
  let overwrite = $state(false);
  let formError = $state<string | null>(null);
  let previewArgs = $state<string[] | null>(null);
  let previewError = $state<string | null>(null);

  // 任务状态
  let job = $state<ActiveJobUi | null>(null);
  let elapsed = $state(0);

  const currentDefinition = $derived(encoderDefinition(video.encoderId));

  function isAvailable(encoderId: string): boolean {
    if (!capabilities) {
      return true; // 能力尚未加载时先允许选择，启动前会再校验
    }
    const def = encoderDefinition(encoderId);
    if (!def) {
      return false;
    }
    return capabilities.encoders[def.capabilityKey];
  }

  function buildConfig(): EncodeConfig {
    return {
      schemaVersion: 1,
      video,
      audio: { mode: "copy" },
      output: { path: outputPath, overwrite },
    };
  }

  function resetJob() {
    job = null;
    elapsed = 0;
    formError = null;
  }

  async function loadFile(path: string) {
    busy = true;
    error = null;
    resetJob();
    previewArgs = null;
    previewError = null;
    try {
      info = await invoke<MediaProbeResult>("probe_media_file", { path });
      outputPath = suggestOutputPath(info.filePath);
      await refreshPreview();
    } catch (err) {
      info = null;
      error = err instanceof Error ? err.message : String(err);
    } finally {
      busy = false;
    }
  }

  async function pickFile() {
    const selected = await open({
      multiple: false,
      directory: false,
      title: "选择视频文件",
      filters: [
        {
          name: "视频文件",
          extensions: [
            "mkv",
            "mp4",
            "mov",
            "avi",
            "webm",
            "flv",
            "ts",
            "m2ts",
            "wmv",
          ],
        },
      ],
    });
    if (typeof selected === "string") {
      await loadFile(selected);
    }
  }

  async function pickOutputPath() {
    const selected = await save({
      title: "选择输出位置",
      defaultPath: outputPath || (info ? suggestOutputPath(info.filePath) : ""),
      filters: [{ name: "视频文件", extensions: ["mkv", "mp4"] }],
    });
    if (typeof selected === "string") {
      outputPath = selected;
      await refreshPreview();
    }
  }

  function onEncoderChange() {
    video = defaultVideoConfig(video.encoderId);
    void refreshPreview();
  }

  async function refreshPreview() {
    previewError = null;
    if (!info || !outputPath.trim()) {
      previewArgs = null;
      return;
    }
    try {
      previewArgs = await invoke<string[]>("preview_job_args", {
        config: buildConfig(),
        inputPath: info.filePath,
      });
    } catch (err) {
      previewArgs = null;
      previewError = err instanceof Error ? err.message : String(err);
    }
  }

  async function startJob() {
    if (!info || !outputPath.trim()) {
      formError = "请先导入输入文件并设置输出路径";
      return;
    }
    const validationError = validateVideoConfig(video);
    if (validationError) {
      formError = validationError;
      return;
    }
    if (!isAvailable(video.encoderId)) {
      formError = `当前 FFmpeg 构建不支持编码器 ${video.encoderId}`;
      return;
    }
    formError = null;
    job = {
      id: "",
      startedAt: Date.now(),
      notice: null,
      percent: 0,
      outTimeSecs: 0,
      durationKnown: info.durationSecs !== null,
      durationSecs: info.durationSecs,
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
    try {
      const jobId = await invoke<string>("start_job", {
        config: buildConfig(),
        inputPath: info.filePath,
        durationSecs: info.durationSecs,
      });
      // job:started 事件通常先到；这里兜底修正 ID
      if (job && !job.id) {
        job.id = jobId;
      }
    } catch (err) {
      resetJob();
      formError = err instanceof Error ? err.message : String(err);
    }
  }

  async function cancelJob() {
    if (!job || job.finished) {
      return;
    }
    try {
      await invoke("cancel_job", { jobId: job.id });
    } catch (err) {
      formError = err instanceof Error ? err.message : String(err);
    }
  }

  const unlisteners: Array<() => void> = [];

  onMount(async () => {
    unlisteners.push(
      await getCurrentWebview().onDragDropEvent((event) => {
        if (event.payload.type === "over") {
          dragging = true;
        } else if (event.payload.type === "drop") {
          dragging = false;
          const path = event.payload.paths[0];
          if (path) {
            void loadFile(path);
          }
        } else {
          dragging = false;
        }
      }),
    );
    unlisteners.push(
      await listen<JobStartedEvent>("job:started", (event) => {
        const payload = event.payload;
        if (job) {
          job.id = payload.jobId;
          job.startedAt = Date.now();
          job.notice = payload.notice;
        }
      }),
    );
    unlisteners.push(
      await listen<JobProgressEvent>("job:progress", (event) => {
        const payload = event.payload;
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
      }),
    );
    unlisteners.push(
      await listen<JobLogEvent>("job:log", (event) => {
        const payload = event.payload;
        if (!job || job.id !== payload.jobId) {
          return;
        }
        if (job.logs.length >= MAX_LOG_LINES) {
          job.logs.shift();
        }
        job.logs.push(payload.line);
      }),
    );
    unlisteners.push(
      await listen<JobFinishedEvent>("job:finished", (event) => {
        const payload = event.payload;
        if (!job || job.id !== payload.jobId) {
          return;
        }
        job.finished = true;
        job.status = payload.status;
        job.error = payload.error;
        job.outputPath = payload.outputPath;
        job.elapsedSecs = payload.elapsedSecs;
      }),
    );

    try {
      capabilities = await invoke<CapabilityReport>("detect_capabilities");
    } catch {
      capabilities = null;
    }
  });

  onDestroy(() => {
    for (const unlisten of unlisteners) {
      unlisten();
    }
  });

  // 运行中每秒刷新已用时间
  $effect(() => {
    if (job && !job.finished) {
      const active = job;
      elapsed = 0;
      const timer = setInterval(() => {
        elapsed = (Date.now() - active.startedAt) / 1000;
      }, 500);
      return () => {
        clearInterval(timer);
        elapsed = (Date.now() - active.startedAt) / 1000;
      };
    }
  });

  const shownElapsed = $derived(
    job && job.finished ? job.elapsedSecs : elapsed,
  );

  function logText(): string {
    return job?.logs.join("\n") ?? "";
  }
</script>

<section class="page">
  <header>
    <h1>压制</h1>
    <p class="hint">选择输入文件、编码器与输出路径，开始压制任务。</p>
  </header>

  <div
    class="dropzone"
    class:dragging
    role="button"
    tabindex="0"
    onclick={pickFile}
    onkeydown={(e) => {
      if (e.key === "Enter" || e.key === " ") {
        e.preventDefault();
        pickFile();
      }
    }}
  >
    <p class="dz-main">{dragging ? "松开以导入文件" : "拖放视频文件到此处"}</p>
    <p class="dz-sub">或点击选择文件（MKV / MP4 / MOV 等）</p>
  </div>

  {#if busy}
    <p class="hint">正在探测媒体信息…</p>
  {/if}

  {#if error}
    <div class="errors" role="alert">
      <strong>探测失败</strong>
      <p>{error}</p>
    </div>
  {/if}

  {#if info}
    <section class="card">
      <h2>编码设置</h2>
      <div class="form-grid">
        <label class="field">
          <span class="field-label">视频编码器</span>
          <select
            bind:value={video.encoderId}
            onchange={onEncoderChange}
            disabled={!!job && !job.finished}
          >
            {#each ENCODER_DEFINITIONS as def (def.id)}
              <option value={def.id} disabled={!isAvailable(def.id)}>
                {def.label}{!isAvailable(def.id)
                  ? "（当前 FFmpeg 不支持）"
                  : ""}
              </option>
            {/each}
          </select>
        </label>

        <label class="field">
          <span class="field-label"
            >质量（{currentDefinition?.qualityLabel}）</span
          >
          <input
            type="number"
            bind:value={video.qualityValue}
            min={currentDefinition?.qualityRange[0]}
            max={currentDefinition?.qualityRange[1]}
            disabled={!!job && !job.finished}
            onchange={() => void refreshPreview()}
          />
          {#if currentDefinition}
            <span class="field-hint">
              {currentDefinition.qualityRange[0]}–{currentDefinition
                .qualityRange[1]}
              ，不同编码器的数值不可直接比较
            </span>
          {/if}
        </label>

        <label class="field">
          <span class="field-label">预设</span>
          <select
            bind:value={video.preset}
            disabled={!!job && !job.finished}
            onchange={() => void refreshPreview()}
          >
            {#each currentDefinition?.presets ?? [] as preset (preset)}
              <option value={preset}>{preset}</option>
            {/each}
          </select>
        </label>

        <label class="field">
          <span class="field-label">像素格式</span>
          <select
            value={video.pixelFormat ?? ""}
            disabled={!!job && !job.finished}
            onchange={(e) => {
              const value = e.currentTarget.value;
              video.pixelFormat = value === "" ? null : value;
              void refreshPreview();
            }}
          >
            {#each currentDefinition?.pixelFormats ?? [] as option (option.label)}
              <option value={option.value ?? ""}>{option.label}</option>
            {/each}
          </select>
        </label>
      </div>

      <div class="form-grid">
        <label class="field wide">
          <span class="field-label">输出路径</span>
          <div class="row">
            <input
              type="text"
              bind:value={outputPath}
              placeholder="选择或输入输出文件路径（.mkv / .mp4）"
              disabled={!!job && !job.finished}
              onchange={() => void refreshPreview()}
            />
            <button
              type="button"
              onclick={pickOutputPath}
              disabled={!!job && !job.finished}
            >
              浏览…
            </button>
          </div>
        </label>

        <label class="field checkbox-field">
          <span class="field-label">覆盖行为</span>
          <label class="checkbox">
            <input type="checkbox" bind:checked={overwrite} />
            输出文件已存在时覆盖
          </label>
        </label>
      </div>

      {#if previewArgs}
        <details class="preview">
          <summary>参数预览（{previewArgs.length} 项）</summary>
          <pre class="preview-args">{previewArgs.join("\n")}</pre>
        </details>
      {/if}
      {#if previewError}
        <p class="form-error">参数预览失败：{previewError}</p>
      {/if}

      {#if formError}
        <div class="errors" role="alert">
          <strong>无法开始压制</strong>
          <p>{formError}</p>
        </div>
      {/if}

      <div class="actions">
        <button
          type="button"
          class="primary"
          onclick={startJob}
          disabled={!!job && !job.finished}
        >
          {job && !job.finished ? "压制进行中…" : "开始压制"}
        </button>
      </div>
    </section>
  {/if}

  {#if job && !job.finished}
    <section class="card">
      <h2>压制进行中</h2>
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
          <dd>{formatSeconds(shownElapsed)}</dd>
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
        <pre class="log-pre">{logText()}</pre>
      </details>
    </section>
  {:else if job?.finished}
    <section class="card">
      <h2>
        {job.status === "completed"
          ? "压制完成"
          : job.status === "cancelled"
            ? "任务已取消"
            : "压制失败"}
      </h2>
      {#if job.status === "completed"}
        <p class="result-ok">
          输出文件：
          <span class="path" title={job.outputPath ?? ""}>{job.outputPath}</span
          >
        </p>
        {#if job.notice}
          <div class="notice" role="status">{job.notice}</div>
        {/if}
      {:else if job.status === "failed"}
        <div class="errors" role="alert">
          <strong>压制失败</strong>
          <p>{job.error}</p>
        </div>
      {/if}
      <dl class="meta">
        <div>
          <dt>已用时间</dt>
          <dd>{formatSeconds(job.elapsedSecs)}</dd>
        </div>
      </dl>
      <div class="actions">
        <button type="button" onclick={resetJob}>清除结果</button>
      </div>
      <details class="log">
        <summary>日志（{job.logs.length} 行）</summary>
        <pre class="log-pre">{logText()}</pre>
      </details>
    </section>
  {/if}

  {#if info}
    <section class="card">
      <h2>文件信息</h2>
      <dl class="meta">
        <div>
          <dt>路径</dt>
          <dd class="path" title={info.filePath}>{info.filePath}</dd>
        </div>
        <div>
          <dt>容器</dt>
          <dd>
            {info.formatName ?? "—"}{info.formatLongName
              ? `（${info.formatLongName}）`
              : ""}
          </dd>
        </div>
        <div>
          <dt>时长</dt>
          <dd>
            {info.durationSecs !== null
              ? formatSeconds(info.durationSecs)
              : "—"}
          </dd>
        </div>
        <div>
          <dt>大小</dt>
          <dd>{info.sizeBytes !== null ? formatBytes(info.sizeBytes) : "—"}</dd>
        </div>
        <div>
          <dt>码率</dt>
          <dd>
            {info.bitRate !== null ? `${formatBytes(info.bitRate)}/s` : "—"}
          </dd>
        </div>
        <div>
          <dt>章节</dt>
          <dd>{info.chapterCount} 个</dd>
        </div>
        <div>
          <dt>附件</dt>
          <dd>{info.attachmentCount} 个</dd>
        </div>
      </dl>

      {#if info.videoStreams.length > 0}
        <h3>视频流</h3>
        <table class="streams">
          <thead>
            <tr>
              <th>#</th><th>编码</th><th>Profile</th><th>分辨率</th>
              <th>帧率</th><th>像素格式</th><th>位深</th><th>语言</th>
            </tr>
          </thead>
          <tbody>
            {#each info.videoStreams as s (s.index)}
              <tr>
                <td>{s.index}</td>
                <td>{s.codecName}</td>
                <td>{s.profile ?? "—"}</td>
                <td
                  >{s.width !== null && s.height !== null
                    ? `${s.width}×${s.height}`
                    : "—"}</td
                >
                <td>{s.frameRate ?? "—"}</td>
                <td>{s.pixelFormat ?? "—"}</td>
                <td>{s.bitDepth !== null ? `${s.bitDepth} bit` : "—"}</td>
                <td>{s.language ?? "—"}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}

      {#if info.audioStreams.length > 0}
        <h3>音频流</h3>
        <table class="streams">
          <thead>
            <tr>
              <th>#</th><th>编码</th><th>采样率</th><th>声道</th>
              <th>声道布局</th><th>语言</th><th>默认</th>
            </tr>
          </thead>
          <tbody>
            {#each info.audioStreams as s (s.index)}
              <tr>
                <td>{s.index}</td>
                <td>{s.codecName}</td>
                <td>{s.sampleRate ? `${s.sampleRate} Hz` : "—"}</td>
                <td>{s.channels ?? "—"}</td>
                <td>{s.channelLayout ?? "—"}</td>
                <td>{s.language ?? "—"}</td>
                <td>{s.default ? "✓" : ""}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}

      {#if info.subtitleStreams.length > 0}
        <h3>字幕流</h3>
        <table class="streams">
          <thead>
            <tr>
              <th>#</th><th>格式</th><th>语言</th><th>标题</th><th>默认</th><th
                >强制</th
              >
            </tr>
          </thead>
          <tbody>
            {#each info.subtitleStreams as s (s.index)}
              <tr>
                <td>{s.index}</td>
                <td>{s.codecName}</td>
                <td>{s.language ?? "—"}</td>
                <td>{s.title ?? "—"}</td>
                <td>{s.default ? "✓" : ""}</td>
                <td>{s.forced ? "✓" : ""}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
      <p class="hint">
        M2 说明：音频全部复制原音轨，字幕与章节暂不处理（M4 实现）。
      </p>
    </section>
  {/if}

  {#if !info}
    <div class="placeholder">
      <strong>单任务压制（M2）</strong>
      <ul>
        <li>x264 / x265 与可用 NVENC 编码器</li>
        <li>质量、预设与像素格式选择</li>
        <li>进度、速度、ETA 与日志</li>
        <li>支持取消与失败诊断</li>
      </ul>
    </div>
  {/if}
</section>

<style>
  .dropzone {
    display: flex;
    flex-direction: column;
    gap: 4px;
    align-items: center;
    justify-content: center;
    min-height: 120px;
    margin-bottom: 16px;
    border: 2px dashed var(--color-border);
    border-radius: 10px;
    background: var(--color-surface);
    cursor: pointer;
    text-align: center;
  }

  .dropzone:hover,
  .dropzone.dragging {
    border-color: var(--color-accent);
    background: var(--color-hover);
  }

  .dz-main {
    margin: 0;
    font-size: 16px;
    font-weight: 600;
  }

  .dz-sub {
    margin: 0;
    font-size: 12px;
    color: var(--color-text-secondary);
  }

  .errors {
    margin: 0 0 16px;
    padding: 10px 12px;
    max-width: 720px;
    border: 1px solid var(--color-danger);
    border-radius: 8px;
    background: var(--color-surface);
    color: var(--color-danger);
    word-break: break-all;
  }

  .errors p {
    margin: 4px 0 0;
  }

  .card {
    margin-bottom: 16px;
    max-width: 860px;
    padding: 14px 16px;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: 10px;
  }

  .card h2 {
    margin: 0 0 10px;
    font-size: 16px;
  }

  .card h3 {
    margin: 14px 0 6px;
    font-size: 14px;
  }

  .form-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
    gap: 10px 16px;
    margin-bottom: 12px;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .field.wide {
    grid-column: 1 / -1;
  }

  .field-label {
    font-size: 12px;
    color: var(--color-text-secondary);
  }

  .field-hint {
    font-size: 11px;
    color: var(--color-text-secondary);
  }

  .field input[type="text"],
  .field input[type="number"],
  .field select {
    padding: 6px 8px;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    background: var(--color-background);
    color: var(--color-text);
    font: inherit;
  }

  .row {
    display: flex;
    gap: 8px;
  }

  .row input {
    flex: 1;
  }

  .checkbox-field {
    justify-content: flex-end;
  }

  .checkbox {
    display: flex;
    gap: 6px;
    align-items: center;
    font-size: 13px;
  }

  .actions {
    display: flex;
    gap: 8px;
    margin-top: 10px;
  }

  button {
    padding: 7px 14px;
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

  .preview,
  .log {
    margin-top: 10px;
  }

  .preview summary,
  .log summary {
    cursor: pointer;
    font-size: 13px;
    color: var(--color-text-secondary);
  }

  .preview-args,
  .log-pre {
    margin: 8px 0 0;
    padding: 10px;
    max-height: 260px;
    overflow: auto;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    background: var(--color-background);
    font-size: 12px;
    white-space: pre-wrap;
    word-break: break-all;
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
    grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
    gap: 8px 16px;
    margin: 0;
  }

  .meta dt {
    font-size: 12px;
    color: var(--color-text-secondary);
  }

  .meta dd {
    margin: 0;
  }

  .meta .path,
  .result-ok .path {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 100%;
  }

  .result-ok {
    color: var(--color-accent);
  }

  .form-error {
    margin: 8px 0 0;
    color: var(--color-danger);
  }

  .notice {
    margin: 0 0 10px;
    padding: 8px 10px;
    border: 1px solid var(--color-warning, var(--color-border));
    border-radius: 6px;
    background: var(--color-surface);
    color: var(--color-text);
    font-size: 13px;
    word-break: break-all;
  }

  .streams {
    width: 100%;
    border-collapse: collapse;
    font-size: 13px;
  }

  .streams th,
  .streams td {
    padding: 4px 8px;
    text-align: left;
    border-bottom: 1px solid var(--color-border);
  }

  .streams th {
    color: var(--color-text-secondary);
    font-weight: 600;
  }

  .placeholder {
    max-width: 720px;
    padding: 14px 16px;
    color: var(--color-text-secondary);
    background: var(--color-surface);
    border: 1px dashed var(--color-border);
    border-radius: 10px;
  }
</style>
