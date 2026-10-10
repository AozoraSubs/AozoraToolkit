<script lang="ts">
  import { open, save } from "@tauri-apps/plugin-dialog";
  import { onMount } from "svelte";
  import {
    copyName,
    presetEncoderAvailable,
    presetFileName,
    presetSummary,
    presetToInput,
  } from "../presets";
  import { applyPreset } from "../stores/encodeForm.svelte";
  import { navigation } from "../stores/navigation.svelte";
  import {
    builtinPresets,
    createPreset,
    deletePreset,
    exportPresetFile,
    importPresetFile,
    presetsState,
    refreshPresets,
    renamePreset,
    userPresets,
  } from "../stores/presets.svelte";
  import { refreshCapabilities, toolsState } from "../stores/tools.svelte";
  import type { Preset } from "../types/preset";
  import { formatDateTime } from "../utils/job";

  let notice = $state<string | null>(null);
  let actionError = $state<string | null>(null);
  let editingId = $state<string | null>(null);
  let editingName = $state("");
  let confirmDeleteId = $state<string | null>(null);
  let busy = $state(false);

  const capabilities = $derived(toolsState.capabilities);

  onMount(() => {
    void refreshPresets();
    void refreshCapabilities();
  });

  function allPresetNames(): string[] {
    return presetsState.presets.map((preset) => preset.name);
  }

  /** 应用预设到压制页（预设不含路径，输出路径由压制页决定）。 */
  function usePreset(preset: Preset) {
    applyPreset(preset);
    notice = null;
    actionError = null;
    navigation.activePage = "encode";
  }

  async function duplicatePreset(preset: Preset) {
    actionError = null;
    notice = null;
    busy = true;
    try {
      const name = copyName(preset.name, allPresetNames());
      const created = await createPreset(presetToInput(preset, name));
      notice = `已创建副本：${created.name}`;
    } catch (err) {
      actionError = err instanceof Error ? err.message : String(err);
    } finally {
      busy = false;
    }
  }

  async function exportPreset(preset: Preset) {
    actionError = null;
    notice = null;
    const target = await save({
      title: "导出预设",
      defaultPath: presetFileName(preset.name),
      filters: [{ name: "预设 JSON", extensions: ["json"] }],
    });
    if (typeof target !== "string") {
      return;
    }
    busy = true;
    try {
      await exportPresetFile(preset.id, target);
      notice = `已导出预设到：${target}`;
    } catch (err) {
      actionError = err instanceof Error ? err.message : String(err);
    } finally {
      busy = false;
    }
  }

  async function importPreset() {
    actionError = null;
    notice = null;
    const selected = await open({
      multiple: false,
      directory: false,
      title: "导入预设 JSON",
      filters: [{ name: "预设 JSON", extensions: ["json"] }],
    });
    if (typeof selected !== "string") {
      return;
    }
    busy = true;
    try {
      const imported = await importPresetFile(selected);
      notice = `已导入预设：${imported.name}`;
    } catch (err) {
      actionError = err instanceof Error ? err.message : String(err);
    } finally {
      busy = false;
    }
  }

  function startRename(preset: Preset) {
    editingId = preset.id;
    editingName = preset.name;
    confirmDeleteId = null;
    actionError = null;
    notice = null;
  }

  async function commitRename(preset: Preset) {
    const name = editingName.trim();
    if (!name || name === preset.name) {
      editingId = null;
      return;
    }
    busy = true;
    actionError = null;
    try {
      const renamed = await renamePreset(preset.id, name);
      notice = `已重命名为：${renamed.name}`;
      editingId = null;
    } catch (err) {
      actionError = err instanceof Error ? err.message : String(err);
    } finally {
      busy = false;
    }
  }

  async function removePreset(preset: Preset) {
    busy = true;
    actionError = null;
    notice = null;
    try {
      await deletePreset(preset.id);
      notice = `已删除预设：${preset.name}`;
      confirmDeleteId = null;
    } catch (err) {
      actionError = err instanceof Error ? err.message : String(err);
    } finally {
      busy = false;
    }
  }
</script>

<section class="page">
  <header>
    <h1>预设</h1>
    <p class="hint">
      内置示例预设可直接使用或复制为自己的预设；自定义预设保存在应用配置目录。
    </p>
  </header>

  <div class="toolbar">
    <button type="button" class="primary" onclick={importPreset} disabled={busy}
      >导入预设…</button
    >
    <button
      type="button"
      onclick={() => void refreshPresets()}
      disabled={presetsState.loading}
    >
      刷新列表
    </button>
    <span class="toolbar-hint"> 预设只包含编码参数，不含输入 / 输出路径 </span>
  </div>

  {#if notice}
    <div class="notice" role="status">{notice}</div>
  {/if}
  {#if actionError ?? presetsState.error}
    <div class="errors" role="alert">
      <strong>操作失败</strong>
      <p>{actionError ?? presetsState.error}</p>
    </div>
  {/if}
  {#if presetsState.errors.length > 0}
    <div class="errors" role="alert">
      <strong>有 {presetsState.errors.length} 个预设文件无法读取</strong>
      <ul>
        {#each presetsState.errors as error (error)}
          <li>{error}</li>
        {/each}
      </ul>
    </div>
  {/if}

  <section class="group">
    <h2>内置预设（{builtinPresets().length}）</h2>
    <ul class="preset-list">
      {#each builtinPresets() as preset (preset.id)}
        <li class="preset">
          <div class="preset-main">
            <div class="preset-name">
              {preset.name}
              <span class="badge">内置</span>
              {#if !presetEncoderAvailable(preset, capabilities)}
                <span class="badge warn">当前 FFmpeg 不支持该编码器</span>
              {/if}
            </div>
            {#if preset.description}
              <p class="preset-desc">{preset.description}</p>
            {/if}
            <p class="preset-summary">{presetSummary(preset)}</p>
          </div>
          <div class="preset-actions">
            <button
              type="button"
              class="primary"
              onclick={() => usePreset(preset)}
              disabled={!presetEncoderAvailable(preset, capabilities)}
            >
              应用
            </button>
            <button
              type="button"
              onclick={() => void duplicatePreset(preset)}
              disabled={busy}
            >
              复制为我的预设
            </button>
            <button
              type="button"
              onclick={() => void exportPreset(preset)}
              disabled={busy}
            >
              导出
            </button>
          </div>
        </li>
      {/each}
    </ul>
  </section>

  <section class="group">
    <h2>我的预设（{userPresets().length}）</h2>
    {#if presetsState.loading && presetsState.presets.length === 0}
      <p class="hint">正在读取预设…</p>
    {:else if userPresets().length === 0}
      <div class="placeholder">
        <strong>还没有自定义预设</strong>
        <p>
          在「压制」页配置好编码参数后，点击「保存为预设…」即可保存；也可以导入他人分享的
          JSON 预设。
        </p>
      </div>
    {:else}
      <ul class="preset-list">
        {#each userPresets() as preset (preset.id)}
          <li class="preset">
            <div class="preset-main">
              {#if editingId === preset.id}
                <div class="rename-row">
                  <input
                    type="text"
                    bind:value={editingName}
                    maxlength="60"
                    aria-label="预设名称"
                    onkeydown={(e) => {
                      if (e.key === "Enter") {
                        e.preventDefault();
                        void commitRename(preset);
                      } else if (e.key === "Escape") {
                        editingId = null;
                      }
                    }}
                  />
                  <button
                    type="button"
                    class="primary"
                    onclick={() => void commitRename(preset)}
                    disabled={busy}>保存</button
                  >
                  <button type="button" onclick={() => (editingId = null)}
                    >取消</button
                  >
                </div>
              {:else}
                <div class="preset-name">
                  {preset.name}
                  {#if !presetEncoderAvailable(preset, capabilities)}
                    <span class="badge warn">当前 FFmpeg 不支持该编码器</span>
                  {/if}
                </div>
              {/if}
              {#if preset.description}
                <p class="preset-desc">{preset.description}</p>
              {/if}
              <p class="preset-summary">{presetSummary(preset)}</p>
              <p class="preset-time">
                更新于 {formatDateTime(preset.updatedAtMs)}
              </p>
            </div>
            <div class="preset-actions">
              <button
                type="button"
                class="primary"
                onclick={() => usePreset(preset)}
                disabled={!presetEncoderAvailable(preset, capabilities)}
              >
                应用
              </button>
              <button
                type="button"
                onclick={() => startRename(preset)}
                disabled={busy || editingId === preset.id}
              >
                重命名
              </button>
              <button
                type="button"
                onclick={() => void duplicatePreset(preset)}
                disabled={busy}
              >
                复制
              </button>
              <button
                type="button"
                onclick={() => void exportPreset(preset)}
                disabled={busy}
              >
                导出
              </button>
              {#if confirmDeleteId === preset.id}
                <button
                  type="button"
                  class="danger"
                  onclick={() => void removePreset(preset)}
                  disabled={busy}>确认删除</button
                >
                <button type="button" onclick={() => (confirmDeleteId = null)}
                  >取消</button
                >
              {:else}
                <button
                  type="button"
                  class="danger"
                  onclick={() => {
                    confirmDeleteId = preset.id;
                    editingId = null;
                  }}>删除</button
                >
              {/if}
            </div>
          </li>
        {/each}
      </ul>
    {/if}
  </section>
</section>

<style>
  .toolbar {
    display: flex;
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

  .preset-list {
    display: flex;
    flex-direction: column;
    gap: 10px;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .preset {
    display: flex;
    gap: 12px;
    align-items: flex-start;
    justify-content: space-between;
    padding: 12px 14px;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: 10px;
  }

  .preset-main {
    min-width: 0;
    flex: 1;
  }

  .preset-name {
    display: flex;
    gap: 8px;
    align-items: center;
    font-weight: 600;
    word-break: break-all;
  }

  .preset-desc {
    margin: 4px 0 0;
    font-size: 13px;
    color: var(--color-text);
  }

  .preset-summary {
    margin: 4px 0 0;
    font-size: 12px;
    color: var(--color-text-secondary);
    word-break: break-all;
  }

  .preset-time {
    margin: 4px 0 0;
    font-size: 11px;
    color: var(--color-text-secondary);
  }

  .badge {
    padding: 1px 6px;
    border: 1px solid var(--color-border);
    border-radius: 999px;
    font-size: 11px;
    font-weight: 400;
    color: var(--color-text-secondary);
  }

  .badge.warn {
    border-color: var(--color-warning, var(--color-danger));
    color: var(--color-danger);
  }

  .preset-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    justify-content: flex-end;
    max-width: 320px;
  }

  .rename-row {
    display: flex;
    gap: 6px;
    align-items: center;
  }

  .rename-row input {
    flex: 1;
    padding: 6px 8px;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    background: var(--color-background);
    color: var(--color-text);
    font: inherit;
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

  .errors {
    margin: 0 0 12px;
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

  .errors ul {
    margin: 4px 0 0;
    padding-left: 18px;
  }

  .notice {
    margin: 0 0 12px;
    padding: 8px 10px;
    border: 1px solid var(--color-warning, var(--color-border));
    border-radius: 6px;
    background: var(--color-surface);
    font-size: 13px;
    word-break: break-all;
  }

  .placeholder {
    max-width: 720px;
    padding: 14px 16px;
    color: var(--color-text-secondary);
    background: var(--color-surface);
    border: 1px dashed var(--color-border);
    border-radius: 10px;
  }

  .placeholder p {
    margin: 6px 0 0;
  }
</style>
