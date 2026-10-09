<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  type AppInfo = {
    name: string;
    version: string;
  };

  let info = $state<AppInfo | null>(null);
  let error = $state<string | null>(null);

  onMount(async () => {
    try {
      info = await invoke<AppInfo>("app_info");
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
    }
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
      <dd>{info ? info.name : "…"}</dd>
    </div>
    <div>
      <dt>版本</dt>
      <dd>{info ? info.version : "…"}</dd>
    </div>
  </dl>

  {#if error}
    <p class="error" role="alert">无法读取应用信息：{error}</p>
  {/if}

  <div class="placeholder">
    <strong>即将实现（M1）</strong>
    <ul>
      <li>FFmpeg / FFprobe 路径配置与能力检测</li>
      <li>主题与日志设置</li>
    </ul>
  </div>
</section>

<style>
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
</style>
