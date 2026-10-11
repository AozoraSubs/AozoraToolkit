import { invoke } from "@tauri-apps/api/core";
import {
  containerFromPath,
  defaultVideoConfig,
  suggestOutputPath,
  swapOutputExtension,
} from "../encoders";
import type {
  Container,
  EncodeConfig,
  SubtitleMode,
  VideoConfig,
} from "../types/encode";
import type { JobRecord } from "../types/jobs";
import type { MediaProbeResult } from "../types/media";
import type { Preset } from "../types/preset";

/**
 * 压制表单状态（全局）：压制页与预设页 / 任务页共用。
 *
 * 页面切换不会丢失已导入的文件与参数；预设应用与失败重试都直接写入这里，
 * 再跳转到压制页展示。
 */
export const encodeForm = $state<{
  info: MediaProbeResult | null;
  busy: boolean;
  error: string | null;
  video: VideoConfig;
  subtitleMode: SubtitleMode;
  subtitlePath: string;
  outputPath: string;
  /** 输出封装格式；来自用户选择或输入文件扩展名，默认 MKV。 */
  container: Container;
  overwrite: boolean;
  /** 当前表单来源的预设 ID；参数被手动修改后仍然保留，由界面标注"已修改"。 */
  appliedPresetId: string | null;
}>({
  info: null,
  busy: false,
  error: null,
  video: defaultVideoConfig("libx264"),
  subtitleMode: "drop",
  subtitlePath: "",
  outputPath: "",
  container: "mkv",
  overwrite: false,
  appliedPresetId: null,
});

/** 由当前表单构造编码配置。 */
export function buildEncodeConfig(): EncodeConfig {
  return {
    schemaVersion: 1,
    video: { ...encodeForm.video },
    audio: { mode: "copy" },
    subtitles: {
      mode: encodeForm.subtitleMode,
      externalPath:
        encodeForm.subtitleMode === "burn"
          ? encodeForm.subtitlePath.trim() || null
          : null,
    },
    output: {
      path: encodeForm.outputPath.trim(),
      container: encodeForm.container,
      overwrite: encodeForm.overwrite,
    },
  };
}

/**
 * 切换封装格式：同步替换输出路径的扩展名，保证两者始终一致
 * （后端会拒绝扩展名与容器不一致的输出路径）。
 */
export function setOutputContainer(container: Container): void {
  encodeForm.container = container;
  if (encodeForm.outputPath.trim()) {
    encodeForm.outputPath = swapOutputExtension(
      encodeForm.outputPath,
      container,
    );
  }
}

/**
 * 导入（或重新导入）输入文件：探测媒体信息并建议输出路径。
 * 返回可读错误信息，成功时返回 null。
 */
export async function loadInputFile(path: string): Promise<string | null> {
  encodeForm.busy = true;
  encodeForm.error = null;
  try {
    const info = await invoke<MediaProbeResult>("probe_media_file", { path });
    encodeForm.info = info;
    // 输入文件本身就是 MKV / MP4 / WebM 时沿用其容器，其余输入保持当前选择
    encodeForm.container =
      containerFromPath(info.filePath) ?? encodeForm.container;
    encodeForm.outputPath = suggestOutputPath(
      info.filePath,
      encodeForm.container,
    );
    return null;
  } catch (err) {
    encodeForm.info = null;
    encodeForm.outputPath = "";
    encodeForm.error = err instanceof Error ? err.message : String(err);
    return encodeForm.error;
  } finally {
    encodeForm.busy = false;
  }
}

/** 切换编码器时重置该编码器的默认参数。 */
export function resetVideoForEncoder(encoderId: string) {
  encodeForm.video = defaultVideoConfig(encoderId);
}

/**
 * 应用预设：覆盖视频参数、字幕模式、封装格式与覆盖策略；输出路径保持
 * 用户已选值，但扩展名跟随预设的容器（保证路径与容器一致）。
 *
 * 预设不保存外部字幕文件路径（与机器相关）：烧录模式的预设保留用户当前
 * 已选择的字幕文件，避免"应用预设后字幕文件被清空"。
 */
export function applyPreset(preset: Preset): void {
  encodeForm.video = { ...preset.video };
  encodeForm.subtitleMode = preset.subtitles.mode;
  if (preset.subtitles.mode !== "burn") {
    encodeForm.subtitlePath = "";
  }
  setOutputContainer(preset.output.container);
  encodeForm.overwrite = preset.output.overwrite;
  encodeForm.appliedPresetId = preset.id;
  if (!encodeForm.outputPath.trim() && encodeForm.info) {
    encodeForm.outputPath = suggestOutputPath(
      encodeForm.info.filePath,
      encodeForm.container,
    );
  }
}

/**
 * 载入历史任务配置用于重试：探测输入文件并恢复当时的编码参数。
 * 返回需要提示用户的信息（例如输入文件已不存在）。
 */
export async function loadFromJobRecord(
  record: JobRecord,
): Promise<string | null> {
  const probeError = await loadInputFile(record.inputPath);
  const config = record.config;
  encodeForm.video = { ...config.video };
  encodeForm.subtitleMode = config.subtitles.mode;
  encodeForm.subtitlePath = config.subtitles.externalPath ?? "";
  encodeForm.outputPath = config.output.path;
  encodeForm.container = config.output.container;
  encodeForm.overwrite = config.output.overwrite;
  // 参数来自任务历史，不是某个预设
  encodeForm.appliedPresetId = null;
  if (probeError) {
    return `输入文件无法读取，请重新选择：${probeError}`;
  }
  return null;
}

/** 清空表单（保留编码参数，便于连续压制同一批片源）。 */
export function clearInputFile() {
  encodeForm.info = null;
  encodeForm.error = null;
  encodeForm.outputPath = "";
}
