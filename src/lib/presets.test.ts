import { builtinPresetFixture, presetFixture } from "./testFixtures";
import {
  copyName,
  matchesPreset,
  presetEncoderAvailable,
  presetFileName,
  presetSummary,
  presetToInput,
} from "./presets";
import { describe, expect, it } from "vitest";
import type { CapabilityReport } from "./types/tools";

const capabilities: CapabilityReport = {
  encoders: [
    {
      id: "libx264",
      label: "H.264（libx264，软件）",
      vendor: "software",
      codec: "h264",
      hardware: false,
      listed: true,
      usable: true,
      reason: null,
    },
    {
      id: "libx265",
      label: "HEVC（libx265，软件）",
      vendor: "software",
      codec: "hevc",
      hardware: false,
      listed: true,
      usable: true,
      reason: null,
    },
    {
      id: "hevc_nvenc",
      label: "HEVC（hevc_nvenc，NVIDIA）",
      vendor: "nvidia",
      codec: "hevc",
      hardware: true,
      listed: true,
      usable: false,
      reason: "NVIDIA 硬件编码不可用：No capable devices found",
    },
  ],
  encoderCount: 10,
  filterCount: 10,
  hasSubtitlesFilter: true,
  hwAccels: [],
  gpus: [],
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

describe("matchesPreset", () => {
  it("detects an unmodified preset application", () => {
    const preset = presetFixture();
    expect(
      matchesPreset(preset, { ...preset.video }, "drop", "mkv", false),
    ).toBe(true);
  });

  it("reports modifications of any preset field", () => {
    const preset = presetFixture();
    expect(
      matchesPreset(
        preset,
        { ...preset.video, qualityValue: 20 },
        "drop",
        "mkv",
        false,
      ),
    ).toBe(false);
    expect(
      matchesPreset(
        preset,
        { ...preset.video, encoderId: "libx264" },
        "drop",
        "mkv",
        false,
      ),
    ).toBe(false);
    expect(
      matchesPreset(preset, { ...preset.video }, "copy", "mkv", false),
    ).toBe(false);
    // 封装格式也属于预设保存的内容
    expect(
      matchesPreset(preset, { ...preset.video }, "drop", "mp4", false),
    ).toBe(false);
    expect(
      matchesPreset(preset, { ...preset.video }, "drop", "mkv", true),
    ).toBe(false);
  });

  it("ignores fields a preset does not store", () => {
    const preset = presetFixture();
    // 像素格式在预设中有值，必须参与比较；外部字幕文件路径不参与
    expect(
      matchesPreset(
        preset,
        { ...preset.video, pixelFormat: null },
        "drop",
        "mkv",
        false,
      ),
    ).toBe(false);
  });
});
