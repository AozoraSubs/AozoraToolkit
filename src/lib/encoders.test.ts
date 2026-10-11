import { describe, expect, it } from "vitest";
import {
  ENCODER_DEFINITIONS,
  containerFromPath,
  containerSupportsEncoder,
  defaultVideoConfig,
  encoderDefinition,
  suggestOutputPath,
  swapOutputExtension,
  validateVideoConfig,
} from "./encoders";

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
    });
  });

  it("falls back to first definition for unknown id", () => {
    expect(defaultVideoConfig("unknown").encoderId).toBe("libx264");
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
    expect(containerSupportsEncoder("webm", "libx264")).toBe(false);
    expect(containerSupportsEncoder("webm", "av1_nvenc")).toBe(true);
  });
});
