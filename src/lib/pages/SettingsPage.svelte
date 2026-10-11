<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import {
    refreshCapabilities,
    refreshDetection,
    toolsState,
  } from "../stores/tools.svelte";
  import type { ToolInfo } from "../types/tools";
  import { toolSourceLabel } from "../utils/toolSourceLabel";

  type AppInfo = {
    name: string;
    version: string;
    snapshot: string | null;
  };

  let appInfo = $state<AppInfo | null>(null);
  let error = $state<string | null>(null);
  let notice = $state<string | null>(null);
  let customDirInput = $state("");

  // 检测结果来自全局缓存：进入页面直接展示，不再重复探测
  const detection = $derived(toolsState.detection);
  const capabilities = $derived(toolsState.capabilities);
  const capabilitiesError = $derived(toolsState.capabilitiesError);
  const busy = $derived(toolsState.busy);

  const TOOLS: Array<{ id: "ffmpeg" | "ffprobe"; label: string }> = [
    { id: "ffmpeg", label: "FFmpeg" },
    { id: "ffprobe", label: "FFprobe" },
  ];

  /** 显卡厂商标识 → 显示名。 */
  function gpuVendorLabel(vendor: string): string {
    switch (vendor) {
      case "nvidia":
        return "NVIDIA";
      case "amd":
        return "AMD";
      case "intel":
        return "Intel";
      default:
        return "其他";
    }
  }

  async function refresh(force = true) {
    error = null;
    notice = null;
    await refreshDetection(force);
    if (toolsState.detection) {
      customDirInput = toolsState.detection.customToolDir ?? "";
    }
    await refreshCapabilities(force);
  }

  async function saveCustomDir(event: Event) {
    event.preventDefault();
    const dir = customDirInput.trim();
    error = null;
    notice = null;
    try {
      await invoke("set_custom_tool_dir", { dir: dir === "" ? null : dir });
      notice = "自定义路径已保存";
      // 路径变化后强制重新检测（缓存失效）
      await refresh(true);
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
    }
  }

  function toolFor(id: "ffmpeg" | "ffprobe"): ToolInfo | null {
    return detection ? detection[id] : null;
  }

  onMount(async () => {
    try {
      appInfo = await invoke<AppInfo>("app_info");
    } catch {
      appInfo = null;
    }
    // 首次进入使用缓存；没有缓存时才真正检测
    await refreshDetection(false);
    await refreshCapabilities(false);
  });
</script>

<section class="page">
  <header>
    <h1>设置</h1>
    <p class="hint">配置工具路径、外观与输出行为。</p>
  </header>

  <dl class="app-info">
    <div>
      <dt>应用</dt>
      <dd>{appInfo ? appInfo.name : "…"}</dd>
    </div>
    <div>
      <dt>版本</dt>
      <dd>
        {appInfo ? appInfo.version : "…"}
        {appInfo?.snapshot ? `（快照 ${appInfo.snapshot}）` : ""}
      </dd>
    </div>
  </dl>

  <h2>FFmpeg / FFprobe</h2>

  {#if detection}
    <ul class="tool-list">
      {#each TOOLS as tool (tool.id)}
        {@const info = toolFor(tool.id)}
        <li class="tool-item">
          <span class="status-mark" class:ok={info !== null} aria-hidden="true">
            {info ? "✓" : "✗"}
          </span>
          <div class="tool-detail">
            <div class="tool-head">
              <strong>{tool.label}</strong>
              {#if info}
                <span class="badge">{toolSourceLabel(info.source)}</span>
              {/if}
            </div>
            {#if info}
              <code class="path">{info.path}</code>
              {#if info.version}
                <p class="version">{info.version}</p>
              {/if}
            {:else}
              <p class="missing">未检测到可用的 {tool.label}</p>
            {/if}
          </div>
        </li>
      {/each}
    </ul>
  {:else if !error}
    <p class="hint">正在检测…</p>
  {/if}

  {#if detection?.errors.length}
    <div class="errors" role="alert">
      <strong>检测问题</strong>
      <ul>
        {#each detection.errors as problem (problem)}
          <li>{problem}</li>
        {/each}
      </ul>
    </div>
  {/if}

  <h2>显卡</h2>

  {#if capabilities}
    {#if capabilities.gpus.length > 0}
      <ul class="enc-list">
        {#each capabilities.gpus as gpu (gpu.name)}
          <li>
            <span class="badge">{gpuVendorLabel(gpu.vendor)}</span>
            <span>{gpu.name}</span>
            {#if gpu.driver}
              <span class="hint small">驱动 {gpu.driver}</span>
            {/if}
          </li>
        {/each}
      </ul>
      <p class="hint small">
        硬件编码器以「检测时实拍一次」的结果为准：显卡存在但驱动/编码单元不支持时，
        对应编码器不会出现在压制页的列表里。
      </p>
    {:else}
      <p class="hint small">未检测到显示适配器。</p>
    {/if}
  {:else}
    <p class="hint small">正在检测…</p>
  {/if}

  <h2>编码器与滤镜能力</h2>

  {#if capabilities}
    <ul class="enc-list">
      {#each capabilities.encoders as enc (enc.id)}
        <li>
          <span class="status-mark" class:ok={enc.usable} aria-hidden="true">
            {enc.usable ? "✓" : "✗"}
          </span>
          <span>{enc.label}</span>
          {#if !enc.usable && enc.reason}
            <span class="hint small">{enc.reason}</span>
          {/if}
        </li>
      {/each}
    </ul>
    <p class="hint small">
      共 {capabilities.encoderCount} 个编码器、{capabilities.filterCount} 个滤镜可用。
      {#if capabilities.hwAccels.length > 0}
        硬件加速方法：{capabilities.hwAccels.join("、")}。
      {/if}
    </p>
    {#each capabilities.warnings as warning (warning)}
      <p class="error small">{warning}</p>
    {/each}
  {:else if capabilitiesError}
    <p class="error" role="alert">{capabilitiesError}</p>
  {:else}
    <p class="hint small">能力检测尚未完成。</p>
  {/if}

  <form class="custom-dir" onsubmit={saveCustomDir}>
    <label for="custom-dir">自定义 FFmpeg 目录（Windows）</label>
    <div class="row">
      <input
        id="custom-dir"
        type="text"
        placeholder="例如 C:\tools\ffmpeg\bin"
        bind:value={customDirInput}
      />
      <button type="submit" disabled={busy}>保存</button>
      <button type="button" onclick={() => refresh(true)} disabled={busy}>
        {busy ? "检测中…" : "重新检测"}
      </button>
    </div>
    <p class="hint small">
      指向包含 ffmpeg / ffprobe 可执行文件的文件夹；留空保存即清除。
    </p>
  </form>

  {#if notice}
    <p class="notice" role="status">{notice}</p>
  {/if}

  {#if error}
    <p class="error" role="alert">{error}</p>
  {/if}

  <div class="placeholder">
    <strong>即将实现（后续里程碑）</strong>
    <ul>
      <li>主题与日志设置</li>
    </ul>
  </div>
</section>

<style>
  h2 {
    margin: 20px 0 10px;
    font-size: 16px;
  }

  .app-info {
    display: flex;
    gap: 24px;
    margin: 0 0 20px;
    padding: 12px 16px;
    max-width: 560px;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: 8px;
  }

  .app-info dt {
    font-size: 12px;
    color: var(--color-text-secondary);
  }

  .app-info dd {
    margin: 0;
    font-weight: 600;
  }

  .tool-list {
    margin: 0 0 16px;
    padding: 0;
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: 8px;
    max-width: 640px;
  }

  .tool-item {
    display: flex;
    gap: 10px;
    padding: 10px 12px;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: 8px;
  }

  .status-mark {
    width: 20px;
    height: 20px;
    flex-shrink: 0;
    display: grid;
    place-items: center;
    border-radius: 50%;
    color: var(--color-danger);
    font-weight: 700;
  }

  .status-mark.ok {
    color: #1f9d55;
  }

  .tool-head {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .badge {
    font-size: 12px;
    padding: 1px 8px;
    border-radius: 999px;
    background: var(--color-active);
  }

  .path {
    display: block;
    margin-top: 2px;
    font-size: 12px;
    word-break: break-all;
    color: var(--color-text-secondary);
  }

  .version {
    margin: 2px 0 0;
    font-size: 12px;
    color: var(--color-text-secondary);
    word-break: break-all;
  }

  .missing {
    margin: 2px 0 0;
    color: var(--color-danger);
  }

  .errors {
    margin: 0 0 16px;
    padding: 10px 12px;
    max-width: 640px;
    border: 1px solid var(--color-danger);
    border-radius: 8px;
    color: var(--color-danger);
    background: var(--color-surface);
  }

  .errors ul {
    margin: 4px 0 0;
    padding-left: 18px;
  }

  .custom-dir {
    margin-bottom: 16px;
    max-width: 640px;
  }

  .custom-dir label {
    display: block;
    margin-bottom: 6px;
    font-weight: 600;
  }

  .row {
    display: flex;
    gap: 8px;
  }

  input {
    flex: 1;
    padding: 8px 10px;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    background: var(--color-surface);
    color: var(--color-text);
    font: inherit;
  }

  button {
    padding: 8px 14px;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    background: var(--color-surface);
    color: var(--color-text);
    font: inherit;
    cursor: pointer;
  }

  button:hover:not(:disabled) {
    background: var(--color-hover);
  }

  button:disabled {
    opacity: 0.6;
    cursor: default;
  }

  .hint.small {
    margin: 6px 0 0;
    font-size: 12px;
  }

  .notice {
    color: #1f9d55;
  }

  .enc-list {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
    gap: 6px;
    margin: 0 0 10px;
    padding: 0;
    list-style: none;
    max-width: 720px;
  }

  .enc-list li {
    display: flex;
    gap: 8px;
    align-items: center;
    padding: 6px 10px;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: 6px;
  }

  .error.small {
    font-size: 12px;
    margin: 4px 0;
  }
</style>
