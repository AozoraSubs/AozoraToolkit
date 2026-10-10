import { builtinPresetFixture, presetFixture } from "./testFixtures";
import {
  copyName,
  presetEncoderAvailable,
  presetFileName,
  presetSummary,
  presetToInput,
} from "./presets";
import { describe, expect, it } from "vitest";
import type { CapabilityReport } from "./types/tools";

const capabilities: CapabilityReport = {
  encoders: {
    libx264: true,
    libx265: true,
    h264Nvenc: false,
    hevcNvenc: false,
    av1Nvenc: false,
  },
  encoderCount: 10,
  filterCount: 10,
  hasSubtitlesFilter: true,
  hwAccels: [],
  warnings: [],
};

describe("presetSummary", () => {
  it("describes video, pixel format, subtitles and overwrite", () => {
    const summary = presetSummary(builtinPresetFixture());
    expect(summary).toContain("libx265");
    expect(summary).toContain("CRF 16");
    expect(summary).toContain("yuv420p10le");
    expect(summary).toContain("丢弃字幕");
    expect(summary).toContain("不覆盖输出");
  });

  it("falls back to the raw encoder id for unknown encoders", () => {
    const preset = {
      ...presetFixture(),
      video: { ...presetFixture().video, encoderId: "libvpx-vp9" },
    };
    expect(presetSummary(preset)).toContain("libvpx-vp9");
  });
});

describe("presetEncoderAvailable", () => {
  it("treats unknown capabilities as available", () => {
    expect(presetEncoderAvailable(builtinPresetFixture(), null)).toBe(true);
  });

  it("follows the capability report", () => {
    expect(presetEncoderAvailable(builtinPresetFixture(), capabilities)).toBe(
      true,
    );
    const nvenc = {
      ...presetFixture(),
      video: { ...presetFixture().video, encoderId: "hevc_nvenc" },
    };
    expect(presetEncoderAvailable(nvenc, capabilities)).toBe(false);
  });
});

describe("presetToInput", () => {
  it("drops the external subtitle path and copies fields", () => {
    const preset = {
      ...presetFixture(),
      subtitles: { mode: "burn" as const, externalPath: "D:/sub.srt" },
    };
    const input = presetToInput(preset, "副本");
    expect(input.name).toBe("副本");
    expect(input.subtitles).toEqual({ mode: "burn", externalPath: null });
    expect(input.video).toEqual(preset.video);
    expect(input.video).not.toBe(preset.video);
  });
});

describe("copyName", () => {
  it("appends 副本 and avoids collisions", () => {
    expect(copyName("HEVC", [])).toBe("HEVC 副本");
    expect(copyName("HEVC", ["hevc 副本"])).toBe("HEVC 副本 2");
    expect(copyName("HEVC", ["HEVC 副本", "HEVC 副本 2"])).toBe("HEVC 副本 3");
  });
});

describe("presetFileName", () => {
  it("sanitises illegal filename characters", () => {
    expect(presetFileName("HEVC 高画质")).toBe("HEVC 高画质.json");
    expect(presetFileName('a:b*c?d"e<f>g|h\\i/j')).toBe(
      "a_b_c_d_e_f_g_h_i_j.json",
    );
    expect(presetFileName("   ")).toBe("preset.json");
  });
});
