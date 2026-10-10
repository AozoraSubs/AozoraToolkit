import { formatCommandLine, quoteArg } from "./command";
import { describe, expect, it } from "vitest";

describe("quoteArg", () => {
  it("keeps simple arguments unchanged", () => {
    expect(quoteArg("-c:v")).toBe("-c:v");
    expect(quoteArg("libx265")).toBe("libx265");
    expect(quoteArg("D:/plain/out.mkv")).toBe("D:/plain/out.mkv");
  });

  it("quotes arguments containing spaces or special characters", () => {
    expect(quoteArg("D:/影片 目录/样片 [demo].mkv")).toBe(
      '"D:/影片 目录/样片 [demo].mkv"',
    );
    expect(quoteArg("testsrc=size=320x240:rate=25")).toBe(
      "testsrc=size=320x240:rate=25",
    );
    expect(quoteArg("0:a?")).toBe("0:a?");
    expect(quoteArg("a&b")).toBe('"a&b"');
  });

  it("escapes embedded quotes and empty arguments", () => {
    expect(quoteArg('say "hi"')).toBe('"say ""hi"""');
    expect(quoteArg("")).toBe('""');
  });
});

describe("formatCommandLine", () => {
  it("prefixes the executable and joins arguments with spaces", () => {
    const line = formatCommandLine("C:/ffmpeg/bin/ffmpeg.exe", [
      "-hide_banner",
      "-i",
      "D:/影片 目录/in.mkv",
      "-crf",
      "18",
    ]);
    expect(line).toBe(
      'C:/ffmpeg/bin/ffmpeg.exe -hide_banner -i "D:/影片 目录/in.mkv" -crf 18',
    );
  });

  it("omits the executable when it is unknown", () => {
    expect(formatCommandLine(null, ["-i", "in.mkv"])).toBe("-i in.mkv");
    expect(formatCommandLine("  ", ["-i"])).toBe("-i");
  });
});
