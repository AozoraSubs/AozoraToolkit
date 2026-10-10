import { invoke } from "@tauri-apps/api/core";
import type { Preset, PresetInput, PresetListResult } from "../types/preset";

/**
 * 预设列表状态（全局）：会话内缓存，保存 / 导入 / 重命名 / 删除后刷新。
 */
export const presetsState = $state<{
  presets: Preset[];
  /** 单个预设文件加载失败的可读错误（不影响其他预设）。 */
  errors: string[];
  loading: boolean;
  busy: boolean;
  error: string | null;
}>({
  presets: [],
  errors: [],
  loading: false,
  busy: false,
  error: null,
});

/** 内置示例预设。 */
export function builtinPresets(): Preset[] {
  return presetsState.presets.filter((preset) => preset.builtin);
}

/** 用户自定义预设。 */
export function userPresets(): Preset[] {
  return presetsState.presets.filter((preset) => !preset.builtin);
}

export function presetById(id: string): Preset | undefined {
  return presetsState.presets.find((preset) => preset.id === id);
}

/** 重新读取预设列表（force 为兼容保留：列表总是重新读取磁盘）。 */
export async function refreshPresets(): Promise<void> {
  presetsState.loading = true;
  presetsState.error = null;
  try {
    const result = await invoke<PresetListResult>("list_presets");
    presetsState.presets = result.presets;
    presetsState.errors = result.errors;
  } catch (err) {
    presetsState.presets = [];
    presetsState.errors = [];
    presetsState.error = err instanceof Error ? err.message : String(err);
  } finally {
    presetsState.loading = false;
  }
}

/** 执行一次写操作并刷新列表；失败时抛出可读错误。 */
async function mutate<T>(operation: () => Promise<T>): Promise<T> {
  presetsState.busy = true;
  presetsState.error = null;
  try {
    const result = await operation();
    await refreshPresets();
    return result;
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    presetsState.error = message;
    throw new Error(message);
  } finally {
    presetsState.busy = false;
  }
}

/** 保存新的用户预设。 */
export function createPreset(input: PresetInput): Promise<Preset> {
  return mutate(() => invoke<Preset>("create_preset", { preset: input }));
}

/** 重命名用户预设。 */
export function renamePreset(id: string, name: string): Promise<Preset> {
  return mutate(() => invoke<Preset>("rename_preset", { id, name }));
}

/** 删除用户预设。 */
export function deletePreset(id: string): Promise<void> {
  return mutate(() => invoke<void>("delete_preset", { id }));
}

/** 从 JSON 文件导入预设。 */
export function importPresetFile(path: string): Promise<Preset> {
  return mutate(() => invoke<Preset>("import_preset_file", { path }));
}

/** 导出预设到 JSON 文件（不刷新列表）。 */
export async function exportPresetFile(
  id: string,
  path: string,
): Promise<void> {
  presetsState.busy = true;
  presetsState.error = null;
  try {
    await invoke<void>("export_preset_file", { id, path });
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    presetsState.error = message;
    throw new Error(message);
  } finally {
    presetsState.busy = false;
  }
}
