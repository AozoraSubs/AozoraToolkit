import type { Preset } from "./types/preset";

/** 测试用预设：默认是一个合法的 HEVC 10-bit 用户预设。 */
export function presetFixture(overrides: Partial<Preset> = {}): Preset {
  return {
    schemaVersion: 1,
    id: "p-1-0",
    name: "我的预设",
    description: null,
    builtin: false,
    video: {
      encoderId: "libx265",
      qualityMode: "crf",
      qualityValue: 16,
      preset: "slow",
      pixelFormat: "yuv420p10le",
    },
    audio: { mode: "copy" },
    subtitles: { mode: "drop", externalPath: null },
    output: { overwrite: false },
    createdAtMs: 1,
    updatedAtMs: 1,
    ...overrides,
  };
}

/** 测试用内置预设。 */
export function builtinPresetFixture(): Preset {
  return presetFixture({
    id: "builtin-hevc-quality",
    name: "HEVC 高画质（CRF 16 / slow / 10-bit）",
    builtin: true,
  });
}
