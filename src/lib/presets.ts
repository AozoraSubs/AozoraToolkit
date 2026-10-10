import { encoderDefinition } from "./encoders";
import type { Preset, PresetInput } from "./types/preset";
import type { CapabilityReport } from "./types/tools";

/** 预设参数摘要（列表展示用）。 */
export function presetSummary(preset: Preset): string {
  const def = encoderDefinition(preset.video.encoderId);
  const label = def?.label ?? preset.video.encoderId;
  const quality = `${def?.qualityLabel ?? "质量"} ${preset.video.qualityValue}`;
  const pixel = preset.video.pixelFormat ?? "自动位深";
  const subtitle =
    preset.subtitles.mode === "copy"
      ? "保留全部字幕轨"
      : preset.subtitles.mode === "burn"
        ? "烧录外部字幕（使用时选择文件）"
        : "丢弃字幕";
  const overwrite = preset.output.overwrite ? "覆盖已有输出" : "不覆盖输出";
  return `${label} · ${quality} · ${preset.video.preset} · ${pixel} · ${subtitle} · ${overwrite}`;
}

/** 预设要求的编码器在当前 FFmpeg 中是否可用（能力尚未检测时视为可用）。 */
export function presetEncoderAvailable(
  preset: Preset,
  capabilities: CapabilityReport | null,
): boolean {
  if (!capabilities) {
    return true;
  }
  const def = encoderDefinition(preset.video.encoderId);
  if (!def) {
    return false;
  }
  return capabilities.encoders[def.capabilityKey];
}

/** 由预设生成"另存为我的预设"的提交内容（不携带外部字幕路径）。 */
export function presetToInput(preset: Preset, name: string): PresetInput {
  return {
    name,
    description: preset.description,
    video: { ...preset.video },
    audio: { ...preset.audio },
    subtitles: { mode: preset.subtitles.mode, externalPath: null },
    output: { overwrite: preset.output.overwrite },
  };
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
