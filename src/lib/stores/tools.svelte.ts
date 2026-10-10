import { invoke } from "@tauri-apps/api/core";
import type { CapabilityReport, ToolDetectionResult } from "../types/tools";

/**
 * FFmpeg 检测结果缓存：整个应用会话内只探测一次（避免每次进入
 * 设置页都重新跑 ffmpeg -version/-encoders 造成卡顿），
 * 需要时通过「重新检测」按钮强制刷新。
 */
export const toolsState = $state<{
  detection: ToolDetectionResult | null;
  capabilities: CapabilityReport | null;
  detectionError: string | null;
  capabilitiesError: string | null;
  busy: boolean;
}>({
  detection: null,
  capabilities: null,
  detectionError: null,
  capabilitiesError: null,
  busy: false,
});

/** 检测 FFmpeg / FFprobe（结果缓存；force=true 强制重新检测）。 */
export async function refreshDetection(force = false): Promise<void> {
  if (toolsState.detection && !force) {
    return;
  }
  toolsState.busy = true;
  toolsState.detectionError = null;
  try {
    toolsState.detection = await invoke<ToolDetectionResult>("detect_tools");
  } catch (err) {
    toolsState.detection = null;
    toolsState.detectionError =
      err instanceof Error ? err.message : String(err);
  } finally {
    toolsState.busy = false;
  }
}

/** 检测 FFmpeg 能力（结果缓存；force=true 强制重新检测）。 */
export async function refreshCapabilities(force = false): Promise<void> {
  if (toolsState.capabilities && !force) {
    return;
  }
  toolsState.capabilitiesError = null;
  try {
    toolsState.capabilities = await invoke<CapabilityReport>(
      "detect_capabilities",
    );
  } catch (err) {
    toolsState.capabilities = null;
    toolsState.capabilitiesError =
      err instanceof Error ? err.message : String(err);
  }
}
