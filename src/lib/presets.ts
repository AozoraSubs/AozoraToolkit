import {
  CONTAINER_OPTIONS,
  RESOLUTION_RANGE,
  encoderDefinition,
  encoderUsable,
  evenDimension,
  resolutionIsSet,
} from "./encoders";
import type {
  Container,
  Resolution,
  SubtitleMode,
  VideoConfig,
} from "./types/encode";
import type { Preset, PresetInput } from "./types/preset";
import type { CapabilityReport } from "./types/tools";

/** 分辨率摘要（列表展示用）。 */
export function resolutionSummary(resolution: Resolution | null): string {
  if (!resolutionIsSet(resolution)) {
    return "源分辨率";
  }
  const width = resolution?.width ?? null;
  const height = resolution?.height ?? null;
  if (width !== null && height !== null) {
    return `${evenDimension(width)}×${evenDimension(height)} 以内`;
  }
  return width !== null
    ? `宽 ${evenDimension(width)}`
    : `高 ${evenDimension(height ?? RESOLUTION_RANGE[0])}`;
}

/** 预设参数摘要（列表展示用）。 */
export function presetSummary(preset: Preset): string {
  const def = encoderDefinition(preset.video.encoderId);
  const label = def?.label ?? preset.video.encoderId;
  const quality = `${def?.qualityLabel ?? "质量"} ${preset.video.qualityValue}`;
  const pixel = preset.video.pixelFormat ?? "自动位深";
  const resolution = resolutionSummary(preset.video.resolution);
  const subtitle =
    preset.subtitles.mode === "copy"
      ? "保留全部字幕轨"
      : preset.subtitles.mode === "burn"
        ? "烧录外部字幕（使用时选择文件）"
        : "丢弃字幕";
  const container = CONTAINER_OPTIONS.find(
    (option) => option.id === preset.output.container,
  );
  const overwrite = preset.output.overwrite ? "覆盖已有输出" : "不覆盖输出";
  return `${label} · ${quality} · ${preset.video.preset} · ${pixel} · ${resolution} · ${subtitle} · ${
    container?.label ?? preset.output.container
  } · ${overwrite}`;
}

/** 预设要求的编码器在当前 FFmpeg 与显卡上是否可用（能力尚未检测时视为可用）。 */
export function presetEncoderAvailable(
  preset: Preset,
  capabilities: CapabilityReport | null,
): boolean {
  return encoderUsable(capabilities, preset.video.encoderId);
}

/** 由预设生成"另存为我的预设"的提交内容（不携带外部字幕路径）。 */
export function presetToInput(preset: Preset, name: string): PresetInput {
  return {
    name,
    description: preset.description,
    video: { ...preset.video },
    audio: { ...preset.audio },
    subtitles: { mode: preset.subtitles.mode, externalPath: null },
    output: {
      container: preset.output.container,
      overwrite: preset.output.overwrite,
    },
  };
}

/**
 * 当前表单参数是否仍与预设一致（用于在下拉框旁标注"参数已修改"）。
 *
 * 只比较预设实际保存的字段：输入 / 输出路径与外部字幕文件不属于预设。
 */
export function matchesPreset(
  preset: Preset,
  video: VideoConfig,
  subtitleMode: SubtitleMode,
  container: Container,
  overwrite: boolean,
): boolean {
  return (
    video.encoderId === preset.video.encoderId &&
    video.qualityMode === preset.video.qualityMode &&
    video.qualityValue === preset.video.qualityValue &&
    video.preset === preset.video.preset &&
    video.pixelFormat === preset.video.pixelFormat &&
    sameResolution(video.resolution, preset.video.resolution) &&
    subtitleMode === preset.subtitles.mode &&
    container === preset.output.container &&
    overwrite === preset.output.overwrite
  );
}

/** 两个分辨率设置是否等价（null 与"两边都空"视为同一件事）。 */
function sameResolution(a: Resolution | null, b: Resolution | null): boolean {
  if (!resolutionIsSet(a) && !resolutionIsSet(b)) {
    return true;
  }
  return (
    (a?.width ?? null) === (b?.width ?? null) &&
    (a?.height ?? null) === (b?.height ?? null)
  );
}

/**
 * 生成不与 `existing` 冲突的副本名称：`原名 副本`、
 * 冲突时递增为 `原名 副本 2`、`原名 副本 3`…
 */
export function copyName(base: string, existing: string[]): string {
  const taken = new Set(existing.map((name) => name.trim().toLowerCase()));
  let index = 1;
  for (;;) {
    const candidate = index === 1 ? `${base} 副本` : `${base} 副本 ${index}`;
    if (!taken.has(candidate.toLowerCase())) {
      return candidate;
    }
    index += 1;
  }
}

/** 导出预设时的建议文件名（去掉 Windows 文件名非法字符）。 */
export function presetFileName(name: string): string {
  const cleaned = name
    .replace(/[\\/:*?"<>|]/g, "_")
    .replace(/\s+/g, " ")
    .trim();
  return `${cleaned || "preset"}.json`;
}
