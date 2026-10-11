import { describe, expect, it } from "vitest";
import {
  ENCODER_DEFINITIONS,
  RESOLUTION_PRESETS,
  availableEncoderMap,
  containerFromPath,
  containerSupportsAudioCodec,
  containerSupportsEncoder,
  defaultVideoConfig,
  encoderDefinition,
  encoderUsable,
  evenDimension,
  oddDimensions,
  pickDefaultEncoder,
  resolutionIsSet,
  suggestOutputPath,
  swapOutputExtension,
  validateResolution,
  validateVideoConfig,
  vendorLabel,
} from "./encoders";
import type { CapabilityReport } from "./types/tools";

describe("encoderDefinition", () => {
  it("locates known encoders", () => {
    expect(encoderDefinition("libx265")?.family).toBe("software");
    expect(encoderDefinition("hevc_nvenc")?.family).toBe("nvenc");
    expect(encoderDefinition("no-such-encoder")).toBeUndefined();
  });

  it("definitions carry capability keys", () => {
    for (const def of ENCODER_DEFINITIONS) {
      expect(def.qualityRange[0]).toBeLessThan(def.qualityRange[1]);
      expect(def.presets).toContain(def.defaultPreset);
      expect(def.pixelFormats.map((option) => option.value)).toContain(
        def.defaultPixelFormat,
      );
      expect(def.qualityRange[0]).toBeLessThanOrEqual(def.defaultQuality);
      expect(def.defaultQuality).toBeLessThanOrEqual(def.qualityRange[1]);
    }
  });
});

describe("defaultVideoConfig", () => {
  it("uses encoder defaults", () => {
    const video = defaultVideoConfig("libx265");
    expect(video).toEqual({
      encoderId: "libx265",
      qualityMode: "crf",
      qualityValue: 18,
      preset: "slow",
      pixelFormat: "yuv420p10le",
      // 默认保持源分辨率
      resolution: null,
    });
  });

  it("falls back to first definition for unknown id", () => {
    expect(defaultVideoConfig("unknown").encoderId).toBe("libx264");
  });
});

describe("分辨率设置", () => {
  it("校验宽高范围与整数要求", () => {
    expect(validateResolution(null)).toBeNull();
    expect(validateResolution({ width: null, height: null })).toBeNull();
    expect(validateResolution({ width: 1920, height: 1080 })).toBeNull();
    // 只填一边也合法
    expect(validateResolution({ width: 1280, height: null })).toBeNull();
    expect(validateResolution({ width: null, height: 720 })).toBeNull();
    expect(validateResolution({ width: 15, height: null })).toContain("宽度");
    expect(validateResolution({ width: null, height: 99999 })).toContain(
      "高度",
    );
    expect(validateResolution({ width: 1280.5, height: null })).toContain(
      "整数",
    );
  });

  it("识别显式填入的奇数尺寸并向下取偶", () => {
    expect(oddDimensions({ width: 641, height: 361 })).toEqual([641, 361]);
    expect(oddDimensions({ width: 640, height: null })).toEqual([]);
    expect(oddDimensions(null)).toEqual([]);
    expect(evenDimension(641)).toBe(640);
    expect(evenDimension(640)).toBe(640);
    // 不小于尺寸下限
    expect(evenDimension(17)).toBe(16);
    expect(evenDimension(16)).toBe(16);
  });

  it("判断是否设置了分辨率", () => {
    expect(resolutionIsSet(null)).toBe(false);
    expect(resolutionIsSet({ width: null, height: null })).toBe(false);
    expect(resolutionIsSet({ width: 1920, height: null })).toBe(true);
    expect(resolutionIsSet({ width: null, height: 1080 })).toBe(true);
  });

  it("提供常见分辨率档位，含保持源分辨率", () => {
    expect(RESOLUTION_PRESETS[0].resolution).toBeNull();
    const labels = RESOLUTION_PRESETS.map((option) => option.label);
    expect(labels).toContain("4K（3840×2160）");
    expect(labels).toContain("1080p（1920×1080）");
    expect(labels).toContain("720p（1280×720）");
    expect(labels).toContain("480p（854×480）");
    for (const option of RESOLUTION_PRESETS.slice(1)) {
      expect(validateResolution(option.resolution)).toBeNull();
    }
    // 4K 档位是 3840×2160，且在允许范围内（上限 8192），取偶后不变
    const fourK = RESOLUTION_PRESETS.find((option) =>
      option.label.startsWith("4K"),
    );
    expect(fourK?.resolution).toEqual({ width: 3840, height: 2160 });
    expect(evenDimension(3840)).toBe(3840);
    expect(evenDimension(2160)).toBe(2160);
  });
});

describe("validateVideoConfig", () => {
  it("accepts defaults for every encoder", () => {
    for (const def of ENCODER_DEFINITIONS) {
      expect(validateVideoConfig(defaultVideoConfig(def.id))).toBeNull();
    }
  });

  it("rejects unknown encoder and out-of-range quality", () => {
    const video = defaultVideoConfig("libx264");
    video.encoderId = "libvpx-vp9";
    expect(validateVideoConfig(video)).not.toBeNull();

    const invalid = defaultVideoConfig("libx264");
    invalid.qualityValue = 99;
    expect(validateVideoConfig(invalid)).toMatch(/0–51/);
  });

  it("rejects unsupported pixel format", () => {
    const video = defaultVideoConfig("libx264");
    video.pixelFormat = "yuv420p10le";
    expect(validateVideoConfig(video)).not.toBeNull();
  });

  it("enforces av1_nvenc wider cq range", () => {
    const video = defaultVideoConfig("av1_nvenc");
    video.qualityValue = 63;
    expect(validateVideoConfig(video)).toBeNull();
    video.qualityValue = 64;
    expect(validateVideoConfig(video)).not.toBeNull();
  });
});

describe("suggestOutputPath", () => {
  it("uses the selected container extension in the same directory", () => {
    expect(suggestOutputPath("C:\\视频\\样片.mkv", "mkv")).toBe(
      "C:/视频/样片.encoded.mkv",
    );
    expect(suggestOutputPath("/media/videos/sample.mp4", "mkv")).toBe(
      "/media/videos/sample.encoded.mkv",
    );
    expect(suggestOutputPath("/media/videos/sample.mkv", "mp4")).toBe(
      "/media/videos/sample.encoded.mp4",
    );
    expect(suggestOutputPath("D:\\影片\\样片.mkv", "webm")).toBe(
      "D:/影片/样片.encoded.webm",
    );
  });

  it("handles paths with spaces and dots", () => {
    expect(suggestOutputPath("D:\\目录 (1)\\my file.v2.mkv", "mkv")).toBe(
      "D:/目录 (1)/my file.v2.encoded.mkv",
    );
  });

  it("handles paths without extension or directory", () => {
    expect(suggestOutputPath("video", "mkv")).toBe("video.encoded.mkv");
    expect(suggestOutputPath("video", "mp4")).toBe("video.encoded.mp4");
  });
});

describe("swapOutputExtension", () => {
  it("replaces the container extension", () => {
    expect(swapOutputExtension("D:/影片/样片.encoded.mkv", "mp4")).toBe(
      "D:/影片/样片.encoded.mp4",
    );
    expect(swapOutputExtension("D:\\影片\\样片.mkv", "webm")).toBe(
      "D:/影片/样片.webm",
    );
  });

  it("appends an extension when missing", () => {
    expect(swapOutputExtension("D:/影片/样片", "mkv")).toBe("D:/影片/样片.mkv");
    expect(swapOutputExtension("样片.v2", "webm")).toBe("样片.webm");
  });

  it("leaves an empty path empty", () => {
    expect(swapOutputExtension("   ", "mkv")).toBe("");
  });
});

describe("containerFromPath / containerSupportsEncoder", () => {
  it("reads a supported container from the path", () => {
    expect(containerFromPath("D:/影片/样片.MKV")).toBe("mkv");
    expect(containerFromPath("样片.mp4")).toBe("mp4");
    expect(containerFromPath("样片.webm")).toBe("webm");
    expect(containerFromPath("样片.avi")).toBeNull();
    expect(containerFromPath("无扩展名")).toBeNull();
  });

  it("only WebM restricts the video encoder", () => {
    expect(containerSupportsEncoder("mkv", "libx264")).toBe(true);
    expect(containerSupportsEncoder("mp4", "libx265")).toBe(true);
    // WebM 只接受 VP9 / AV1
    expect(containerSupportsEncoder("webm", "libx264")).toBe(false);
    expect(containerSupportsEncoder("webm", "hevc_nvenc")).toBe(false);
    expect(containerSupportsEncoder("webm", "libvpx-vp9")).toBe(true);
    expect(containerSupportsEncoder("webm", "libsvtav1")).toBe(true);
    expect(containerSupportsEncoder("webm", "libaom-av1")).toBe(true);
    expect(containerSupportsEncoder("webm", "librav1e")).toBe(true);
    expect(containerSupportsEncoder("webm", "av1_nvenc")).toBe(true);
    expect(containerSupportsEncoder("webm", "av1_qsv")).toBe(true);
    expect(containerSupportsEncoder("webm", "av1_amf")).toBe(true);
  });

  it("only WebM restricts the audio codec (app copies audio, never transcodes)", () => {
    expect(containerSupportsAudioCodec("mkv", "aac")).toBe(true);
    expect(containerSupportsAudioCodec("mp4", "flac")).toBe(true);
    expect(containerSupportsAudioCodec("webm", "opus")).toBe(true);
    expect(containerSupportsAudioCodec("webm", "VORBIS")).toBe(true);
    // 实测：AAC 音轨 + WebM 会被 FFmpeg 拒绝（Only VP8/VP9/AV1 + Vorbis/Opus）
    expect(containerSupportsAudioCodec("webm", "aac")).toBe(false);
    expect(containerSupportsAudioCodec("webm", null)).toBe(true);
  });
});

describe("能力检测驱动的编码器可用性", () => {
  const report: CapabilityReport = {
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
        id: "hevc_nvenc",
        label: "HEVC（hevc_nvenc，NVIDIA）",
        vendor: "nvidia",
        codec: "hevc",
        hardware: true,
        listed: true,
        usable: false,
        reason: "NVIDIA 硬件编码不可用：No capable devices found",
      },
      {
        id: "av1_qsv",
        label: "AV1（av1_qsv，Intel Quick Sync）",
        vendor: "intel",
        codec: "av1",
        hardware: true,
        listed: false,
        usable: false,
        reason: "当前 FFmpeg 构建未包含该编码器（av1_qsv）",
      },
    ],
    encoderCount: 400,
    filterCount: 500,
    hasSubtitlesFilter: true,
    hwAccels: ["cuda", "qsv"],
    gpus: [{ name: "NVIDIA GeForce RTX 4080", vendor: "nvidia", driver: null }],
    warnings: [],
  };

  it("trusts the backend result instead of a hardcoded list", () => {
    expect(encoderUsable(report, "libx264")).toBe(true);
    // 编译进来了但没有对应硬件 → 不可用
    expect(encoderUsable(report, "hevc_nvenc")).toBe(false);
    // 干脆没编译进来 → 不可用
    expect(encoderUsable(report, "av1_qsv")).toBe(false);
    // 报告里没有的编码器 → 不可用（不再默认放行）
    expect(encoderUsable(report, "av1_amf")).toBe(false);
    // 检测尚未完成时先放行，任务启动前后端会再校验
    expect(encoderUsable(null, "hevc_nvenc")).toBe(true);
  });

  it("picks a usable default encoder", () => {
    expect(pickDefaultEncoder(report)).toBe("libx264");
    expect(pickDefaultEncoder(null)).toBe("libx264");
    const onlyNvenc: CapabilityReport = {
      ...report,
      encoders: [
        { ...report.encoders[0], usable: false },
        { ...report.encoders[1], usable: true },
      ],
    };
    expect(pickDefaultEncoder(onlyNvenc)).toBe("hevc_nvenc");
    const none: CapabilityReport = { ...report, encoders: [] };
    expect(pickDefaultEncoder(none)).toBeUndefined();
  });

  it("indexes available encoders by id and labels vendors", () => {
    const map = availableEncoderMap(report);
    expect(map.get("libx264")?.usable).toBe(true);
    expect(map.get("hevc_nvenc")?.vendor).toBe("nvidia");
    expect(availableEncoderMap(null).size).toBe(0);

    expect(vendorLabel("software")).toBe("软件（CPU）");
    expect(vendorLabel("nvidia")).toBe("NVIDIA");
    expect(vendorLabel("amd")).toBe("AMD");
    expect(vendorLabel("intel")).toBe("Intel");
  });

  it("keeps the UI metadata in sync with the Rust encoder specs", () => {
    // 每个编码器的质量范围 / 默认质量 / 预设首项都应存在且合法
    for (const def of ENCODER_DEFINITIONS) {
      const [min, max] = def.qualityRange;
      expect(def.defaultQuality).toBeGreaterThanOrEqual(min);
      expect(def.defaultQuality).toBeLessThanOrEqual(max);
      expect(def.presets.length).toBeGreaterThan(0);
      expect(def.presets).toContain(def.defaultPreset);
      expect(def.capabilityKey).toBe(def.id);
    }
    // 三大显卡厂商 + 软件都有条目
    const vendors = new Set(ENCODER_DEFINITIONS.map((def) => def.vendor));
    expect(vendors).toEqual(new Set(["software", "nvidia", "amd", "intel"]));
  });

  it("maps a real detection report (RTX 4080 + Radeon 610M) to the shown list", () => {
    // 本机实测结果（NVIDIA 4080 Laptop + AMD Radeon 610M，gyan full build）：
    // NVENC H.264/HEVC 与 AMF H.264/HEVC 可用；AV1 硬件编码、全部 QSV 不可用。
    const real: CapabilityReport = {
      encoders: [
        ["libx264", "software", true],
        ["libx265", "software", true],
        ["libvpx-vp9", "software", true],
        ["h264_nvenc", "nvidia", true],
        ["hevc_nvenc", "nvidia", true],
        ["av1_nvenc", "nvidia", false],
        ["h264_qsv", "intel", false],
        ["hevc_qsv", "intel", false],
        ["av1_qsv", "intel", false],
        ["h264_amf", "amd", true],
        ["hevc_amf", "amd", true],
        ["av1_amf", "amd", false],
      ].map(([id, vendor, usable]) => ({
        id: id as string,
        label: id as string,
        vendor: vendor as "software" | "nvidia" | "amd" | "intel",
        codec: "h264",
        hardware: vendor !== "software",
        listed: true,
        usable: usable as boolean,
        reason: usable ? null : "探测失败",
      })),
      encoderCount: 460,
      filterCount: 520,
      hasSubtitlesFilter: true,
      hwAccels: ["cuda", "d3d11va", "qsv", "amf"],
      gpus: [
        {
          name: "NVIDIA GeForce RTX 4080 Laptop GPU",
          vendor: "nvidia",
          driver: null,
        },
        { name: "AMD Radeon(TM) 610M", vendor: "amd", driver: null },
      ],
      warnings: [],
    };

    const shown = ENCODER_DEFINITIONS.filter((def) =>
      encoderUsable(real, def.id),
    );
    const ids = shown.map((def) => def.id);
    expect(ids).toEqual([
      "libx264",
      "libx265",
      "libvpx-vp9",
      "h264_nvenc",
      "hevc_nvenc",
      "h264_amf",
      "hevc_amf",
    ]);
    // 硬件编码器都通过了实拍探测才会出现，且都带厂商归属
    expect(shown.filter((def) => def.vendor !== "software")).toHaveLength(4);
    expect(pickDefaultEncoder(real)).toBe("libx264");
  });
});
