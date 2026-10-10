<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { open, save } from "@tauri-apps/plugin-dialog";
  import { onDestroy, onMount } from "svelte";
  import {
    ENCODER_DEFINITIONS,
    encoderDefinition,
    suggestOutputPath,
    validateVideoConfig,
  } from "../encoders";
  import { copyName, presetEncoderAvailable } from "../presets";
  import {
    applyPreset,
    buildEncodeConfig,
    encodeForm,
    loadInputFile,
    resetVideoForEncoder,
  } from "../stores/encodeForm.svelte";
  import {
    clearFinishedJob,
    jobState,
    removeQueuedJob,
  } from "../stores/jobs.svelte";
  import {
    builtinPresets,
    createPreset,
    presetById,
    refreshPresets,
    userPresets,
  } from "../stores/presets.svelte";
  import { refreshCapabilities, toolsState } from "../stores/tools.svelte";
  import type { EnqueueResult } from "../types/encode";
  import { copyText, formatCommandLine } from "../utils/command";
  import { formatBytes, formatSeconds } from "../utils/format";

  let dragging = $state(false);
  let formError = $state<string | null>(null);
  let previewArgs = $state<string[] | null>(null);
  let previewError = $state<string | null>(null);
  let enqueueBusy = $state(false);
  let notice = $state<string | null>(null);

  // 预设：应用已有预设 / 保存当前配置为新预设
  let presetSelectId = $state("");
  let saveOpen = $state(false);
  let saveName = $state("");
  let saveDescription = $state("");
  let saveBusy = $state(false);
  let saveError = $state<string | null>(null);

  const info = $derived(encodeForm.info);
  const job = $derived(jobState.job);
  const jobRunning = $derived(!!job && !job.finished);
  const capabilities = $derived(toolsState.capabilities);
  const currentDefinition = $derived(
    encoderDefinition(encodeForm.video.encoderId),
  );
  const queuedCount = $derived(jobState.queue.length);
  const commandLine = $derived(
    previewArgs
      ? formatCommandLine(
          toolsState.detection?.ffmpeg?.path ?? null,
          previewArgs,
        )
      : "",
  );

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

  async function pickSubtitleFile() {
    const selected = await open({
      multiple: false,
      directory: false,
      title: "选择要烧录的字幕文件",
      filters: [{ name: "字幕文件", extensions: ["srt", "ass"] }],
    });
    if (typeof selected === "string") {
      encodeForm.subtitlePath = selected;
      await refreshPreview();
    }
  }

  async function loadFile(path: string) {
    notice = null;
    clearFinishedJob();
    previewArgs = null;
    previewError = null;
    const error = await loadInputFile(path);
    if (!error) {
      await refreshPreview();
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
      defaultPath:
        encodeForm.outputPath || (info ? suggestOutputPath(info.filePath) : ""),
      filters: [{ name: "视频文件", extensions: ["mkv", "mp4"] }],
    });
    if (typeof selected === "string") {
      encodeForm.outputPath = selected;
      await refreshPreview();
    }
  }

  function onEncoderChange() {
    resetVideoForEncoder(encodeForm.video.encoderId);
    void refreshPreview();
  }

  async function refreshPreview() {
    previewError = null;
    if (!info || !encodeForm.outputPath.trim()) {
      previewArgs = null;
      return;
    }
    try {
      previewArgs = await invoke<string[]>("preview_job_args", {
        config: buildEncodeConfig(),
        inputPath: info.filePath,
      });
    } catch (err) {
      previewArgs = null;
      previewError = err instanceof Error ? err.message : String(err);
    }
  }

  async function copyCommand() {
    const ok = await copyText(commandLine);
    notice = ok ? "已复制命令到剪贴板" : "复制失败：剪贴板不可用";
  }

  async function startEnqueue() {
    if (!info || !encodeForm.outputPath.trim()) {
      formError = "请先导入输入文件并设置输出路径";
      return;
    }
    const validationError = validateVideoConfig(encodeForm.video);
    if (validationError) {
      formError = validationError;
      return;
    }
    if (!isAvailable(encodeForm.video.encoderId)) {
      formError = `当前 FFmpeg 构建不支持编码器 ${encodeForm.video.encoderId}`;
      return;
    }
    if (encodeForm.subtitleMode === "burn" && !encodeForm.subtitlePath.trim()) {
      formError = "烧录字幕需要先选择外部字幕文件（srt / ass）";
      return;
    }
    formError = null;
    notice = null;
    enqueueBusy = true;
    try {
      const result = await invoke<EnqueueResult>("enqueue_job", {
        config: buildEncodeConfig(),
        inputPath: info.filePath,
        durationSecs: info.durationSecs,
      });
      if (!result.started) {
        notice =
          result.position !== null
            ? `已加入队列（第 ${result.position} 位），当前任务完成后自动开始`
            : "已加入队列，当前任务完成后自动开始";
      }
    } catch (err) {
      formError = err instanceof Error ? err.message : String(err);
    } finally {
      enqueueBusy = false;
    }
  }

  async function cancelJob() {
    const current = jobState.job;
    if (!current || current.finished) {
      return;
    }
    try {
      await invoke("cancel_job", { jobId: current.id });
    } catch (err) {
      formError = err instanceof Error ? err.message : String(err);
    }
  }

  async function removeQueued(jobId: string) {
    try {
      await removeQueuedJob(jobId);
    } catch (err) {
      formError = err instanceof Error ? err.message : String(err);
    }
  }

  async function applySelectedPreset() {
    const preset = presetById(presetSelectId);
    presetSelectId = "";
    if (!preset) {
      return;
    }
    applyPreset(preset);
    notice = `已应用预设：${preset.name}`;
    await refreshPreview();
  }

  async function savePreset() {
    saveError = null;
    const name = saveName.trim();
    if (!name) {
      saveError = "请填写预设名称";
      return;
    }
    saveBusy = true;
    try {
      const preset = await createPreset({
        name,
        description: saveDescription.trim() || null,
        video: { ...encodeForm.video },
        audio: { mode: "copy" },
        subtitles: {
          mode: encodeForm.subtitleMode,
          // 预设不保存字幕文件路径（与机器相关）
          externalPath: null,
        },
        output: { overwrite: encodeForm.overwrite },
      });
      notice = `已保存预设：${preset.name}（可在「预设」页管理）`;
      saveName = "";
      saveDescription = "";
      saveOpen = false;
    } catch (err) {
      saveError = err instanceof Error ? err.message : String(err);
    } finally {
      saveBusy = false;
    }
  }

  let unlistenDragDrop: (() => void) | null = null;

  onMount(async () => {
    unlistenDragDrop = await getCurrentWebview().onDragDropEvent((event) => {
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
    });
    // 能力检测与预设列表都在全局缓存，只读取一次
    void refreshCapabilities();
    void refreshPresets();
  });

  onDestroy(() => {
    unlistenDragDrop?.();
  });

  const shownElapsed = $derived(
    job && job.finished ? job.elapsedSecs : jobState.elapsed,
  );

  function logText(lines: string[]): string {
    return lines.join("\n");
  }
</script>

<section class="page">
  <header>
    <h1>压制</h1>
    <p class="hint">
      选择输入文件、编码器与输出路径；任务按队列逐个运行（同时只跑一个）。
    </p>
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

  {#if encodeForm.busy}
    <p class="hint">正在探测媒体信息…</p>
  {/if}

  {#if encodeForm.error}
    <div class="errors" role="alert">
      <strong>探测失败</strong>
      <p>{encodeForm.error}</p>
    </div>
  {/if}

  {#if notice}
    <div class="notice" role="status">{notice}</div>
  {/if}

  <section class="card">
    <h2>编码设置</h2>

    <div class="form-grid">
      <label class="field wide">
        <span class="field-label">应用预设</span>
        <div class="row">
          <select bind:value={presetSelectId} onchange={applySelectedPreset}>
            <option value="">（不使用预设）</option>
            {#if builtinPresets().length > 0}
              <optgroup label="内置预设">
                {#each builtinPresets() as preset (preset.id)}
                  <option
                    value={preset.id}
                    disabled={!presetEncoderAvailable(preset, capabilities)}
                  >
                    {preset.name}{!presetEncoderAvailable(preset, capabilities)
                      ? "（当前 FFmpeg 不支持）"
                      : ""}
                  </option>
                {/each}
              </optgroup>
            {/if}
            {#if userPresets().length > 0}
              <optgroup label="我的预设">
                {#each userPresets() as preset (preset.id)}
                  <option
                    value={preset.id}
                    disabled={!presetEncoderAvailable(preset, capabilities)}
                  >
                    {preset.name}
                  </option>
                {/each}
              </optgroup>
            {/if}
          </select>
          <button
            type="button"
            onclick={() => {
              saveOpen = !saveOpen;
              if (saveOpen && !saveName) {
                saveName = copyName(
                  encodeForm.video.encoderId,
                  builtinPresets()
                    .concat(userPresets())
                    .map((preset) => preset.name),
                );
              }
            }}
          >
            保存为预设…
          </button>
        </div>
        <span class="field-hint">
          预设只包含编码参数，不含输入 /
          输出路径；在「预设」页可以重命名、删除与导入导出。
        </span>
      </label>
    </div>

    {#if saveOpen}
      <div class="form-grid save-preset">
        <label class="field">
          <span class="field-label">预设名称</span>
          <input type="text" bind:value={saveName} maxlength="60" />
        </label>
        <label class="field">
          <span class="field-label">描述（可选）</span>
          <input type="text" bind:value={saveDescription} maxlength="200" />
        </label>
        <div class="field save-actions">
          <button
            type="button"
            class="primary"
            onclick={savePreset}
            disabled={saveBusy}
          >
            {saveBusy ? "保存中…" : "保存预设"}
          </button>
          <button type="button" onclick={() => (saveOpen = false)}>取消</button>
        </div>
      </div>
      {#if saveError}
        <p class="form-error">{saveError}</p>
      {/if}
    {/if}

    <div class="form-grid">
      <label class="field">
        <span class="field-label">视频编码器</span>
        <select
          bind:value={encodeForm.video.encoderId}
          onchange={onEncoderChange}
        >
          {#each ENCODER_DEFINITIONS as def (def.id)}
            <option value={def.id} disabled={!isAvailable(def.id)}>
              {def.label}{!isAvailable(def.id) ? "（当前 FFmpeg 不支持）" : ""}
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
          bind:value={encodeForm.video.qualityValue}
          min={currentDefinition?.qualityRange[0]}
          max={currentDefinition?.qualityRange[1]}
          onchange={() => void refreshPreview()}
        />
        {#if currentDefinition}
          <span class="field-hint">
            {currentDefinition.qualityRange[0]}–{currentDefinition
              .qualityRange[1]}，不同编码器的数值不可直接比较
          </span>
        {/if}
      </label>

      <label class="field">
        <span class="field-label">预设</span>
        <select
          bind:value={encodeForm.video.preset}
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
          value={encodeForm.video.pixelFormat ?? ""}
          onchange={(e) => {
            const value = e.currentTarget.value;
            encodeForm.video.pixelFormat = value === "" ? null : value;
            void refreshPreview();
          }}
        >
          {#each currentDefinition?.pixelFormats ?? [] as option (option.label)}
            <option value={option.value ?? ""}>{option.label}</option>
          {/each}
        </select>
      </label>

      <label class="field">
        <span class="field-label">字幕</span>
        <select
          bind:value={encodeForm.subtitleMode}
          onchange={() => void refreshPreview()}
        >
          <option value="drop">丢弃字幕</option>
          <option value="copy">保留全部字幕轨</option>
          <option
            value="burn"
            disabled={capabilities !== null && !capabilities.hasSubtitlesFilter}
          >
            烧录外部字幕（srt / ass）
            {capabilities !== null && !capabilities.hasSubtitlesFilter
              ? "（当前 FFmpeg 无 libass）"
              : ""}
          </option>
        </select>
        {#if encodeForm.subtitleMode === "burn"}
          <span class="field-hint">字幕将被渲染进画面，成为视频的一部分</span>
        {:else if info && info.subtitleStreams.length === 0}
          <span class="field-hint">源文件无字幕轨</span>
        {:else if encodeForm.subtitleMode === "copy" && /\.mp4$/i.test(encodeForm.outputPath)}
          <span class="field-hint">
            MP4 对字幕格式支持有限（仅 mov_text），建议使用 MKV
          </span>
        {/if}
      </label>
    </div>

    {#if encodeForm.subtitleMode === "burn"}
      <div class="form-grid">
        <label class="field wide">
          <span class="field-label">外部字幕文件</span>
          <div class="row">
            <input
              type="text"
              bind:value={encodeForm.subtitlePath}
              placeholder="选择要烧录的字幕文件（.srt / .ass）"
              onchange={() => void refreshPreview()}
            />
            <button type="button" onclick={pickSubtitleFile}>浏览…</button>
          </div>
          {#if !encodeForm.subtitlePath.trim()}
            <span class="field-hint">请选择字幕文件后再开始压制</span>
          {/if}
        </label>
      </div>
    {/if}

    <div class="form-grid">
      <label class="field wide">
        <span class="field-label">输出路径</span>
        <div class="row">
          <input
            type="text"
            bind:value={encodeForm.outputPath}
            placeholder="选择或输入输出文件路径（.mkv / .mp4）"
            onchange={() => void refreshPreview()}
          />
          <button type="button" onclick={pickOutputPath}>浏览…</button>
        </div>
        {#if !info}
          <span class="field-hint">导入输入文件后会自动建议输出路径</span>
        {/if}
      </label>

      <label class="field checkbox-field">
        <span class="field-label">覆盖行为</span>
        <label class="checkbox">
          <input type="checkbox" bind:checked={encodeForm.overwrite} />
          输出文件已存在时覆盖
        </label>
      </label>
    </div>

    {#if previewArgs}
      <details class="preview">
        <summary>命令预览（{previewArgs.length} 项参数）</summary>
        <div class="command-row">
          <pre class="command-line">{commandLine}</pre>
          <button type="button" onclick={copyCommand}>复制命令</button>
        </div>
        <p class="field-hint">
          全部参数由应用自动生成（无用户自定义参数）；实际执行时 FFmpeg
          先写入应用临时目录，成功后移动到输出路径。
        </p>
        <details class="args">
          <summary>逐项参数</summary>
          <pre class="preview-args">{previewArgs.join("\n")}</pre>
        </details>
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
        onclick={startEnqueue}
        disabled={enqueueBusy || !info || !encodeForm.outputPath.trim()}
      >
        {enqueueBusy ? "正在提交…" : jobRunning ? "加入队列" : "开始压制"}
      </button>
      {#if !info}
        <span class="action-hint">请先导入输入文件</span>
      {/if}
    </div>
  </section>

  {#if job && !job.finished}
    <section class="card">
      <h2>压制进行中：{job.name}</h2>
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
        <div>
          <dt>排队中</dt>
          <dd>{queuedCount} 个任务</dd>
        </div>
      </dl>
      <div class="actions">
        <button type="button" class="danger" onclick={cancelJob}
          >取消任务</button
        >
      </div>
      <details class="log">
        <summary>日志（{job.logs.length} 行）</summary>
        <pre class="log-pre">{logText(job.logs)}</pre>
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
      {#if job.historyError}
        <div class="notice" role="status">
          任务历史写入失败：{job.historyError}
        </div>
      {/if}
      <dl class="meta">
        <div>
          <dt>已用时间</dt>
          <dd>{formatSeconds(job.elapsedSecs)}</dd>
        </div>
      </dl>
      <div class="actions">
        <button type="button" onclick={clearFinishedJob}>清除结果</button>
      </div>
      <details class="log">
        <summary>日志（{job.logs.length} 行）</summary>
        <pre class="log-pre">{logText(job.logs)}</pre>
      </details>
    </section>
  {/if}

  {#if queuedCount > 0}
    <section class="card">
      <h2>队列（{queuedCount} 个任务等待运行）</h2>
      <ol class="queue-list">
        {#each jobState.queue as queued (queued.id)}
          <li>
            <span class="queue-name" title={queued.inputPath}
              >{queued.name}</span
            >
            <span class="queue-target path" title={queued.outputPath}
              >→ {queued.outputPath}</span
            >
            <button type="button" onclick={() => removeQueued(queued.id)}
              >移除</button
            >
          </li>
        {/each}
      </ol>
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
        说明：音频全部复制原音轨；字幕可选保留（复制全部字幕轨）、丢弃或烧录外部
        srt/ass 文件；章节与附件暂不处理（M4 实现）。
      </p>
    </section>
  {/if}

  {#if !info}
    <div class="placeholder">
      <strong>M3：预设与队列</strong>
      <ul>
        <li>预设保存 / 重命名 / 删除 / 导入导出（含内置示例预设）</li>
        <li>任务队列：同时只运行一个任务，其余排队等待</li>
        <li>任务历史持久化，失败任务可查看日志并载入配置重试</li>
        <li>命令预览与一键复制</li>
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

  .save-preset {
    padding: 10px;
    border: 1px dashed var(--color-border);
    border-radius: 8px;
  }

  .save-actions {
    flex-direction: row;
    align-items: flex-end;
    gap: 8px;
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

  .row input,
  .row select {
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
    align-items: center;
    margin-top: 10px;
  }

  .action-hint {
    font-size: 12px;
    color: var(--color-text-secondary);
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
  .log summary,
  .args summary {
    cursor: pointer;
    font-size: 13px;
    color: var(--color-text-secondary);
  }

  .command-row {
    display: flex;
    gap: 8px;
    align-items: flex-start;
    margin-top: 8px;
  }

  .command-line {
    flex: 1;
    margin: 0;
    padding: 10px;
    max-height: 140px;
    overflow: auto;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    background: var(--color-background);
    font-size: 12px;
    white-space: pre-wrap;
    word-break: break-all;
  }

  .args {
    margin-top: 8px;
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

  .queue-list {
    margin: 0;
    padding-left: 20px;
    font-size: 13px;
  }

  .queue-list li {
    display: flex;
    gap: 8px;
    align-items: center;
    margin-bottom: 4px;
  }

  .queue-name {
    font-weight: 600;
  }

  .queue-target {
    flex: 1;
    color: var(--color-text-secondary);
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
