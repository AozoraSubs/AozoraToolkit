import { invoke } from "@tauri-apps/api/core";
import { defaultVideoConfig, suggestOutputPath } from "../encoders";
import type { EncodeConfig, SubtitleMode, VideoConfig } from "../types/encode";
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
  overwrite: boolean;
}>({
  info: null,
  busy: false,
  error: null,
  video: defaultVideoConfig("libx264"),
  subtitleMode: "drop",
  subtitlePath: "",
  outputPath: "",
  overwrite: false,
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
      overwrite: encodeForm.overwrite,
    },
  };
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
    encodeForm.outputPath = suggestOutputPath(info.filePath);
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

/** 应用预设：覆盖视频参数、字幕模式与覆盖策略；输出路径保持用户已选值。 */
export function applyPreset(preset: Preset): void {
  encodeForm.video = { ...preset.video };
  encodeForm.subtitleMode = preset.subtitles.mode;
  encodeForm.subtitlePath = preset.subtitles.externalPath ?? "";
  encodeForm.overwrite = preset.output.overwrite;
  if (!encodeForm.outputPath.trim() && encodeForm.info) {
    encodeForm.outputPath = suggestOutputPath(encodeForm.info.filePath);
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
  encodeForm.overwrite = config.output.overwrite;
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
