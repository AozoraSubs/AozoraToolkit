import { describe, expect, it } from "vitest";
import { formatBytes, formatSeconds } from "./format";

describe("formatBytes", () => {
  it("formats values with binary units", () => {
    expect(formatBytes(0)).toBe("0 B");
    expect(formatBytes(1023)).toBe("1023 B");
    expect(formatBytes(1024)).toBe("1.00 KiB");
    expect(formatBytes(1536)).toBe("1.50 KiB");
    expect(formatBytes(5 * 1024 * 1024)).toBe("5.00 MiB");
    expect(formatBytes(3.7 * 1024 ** 3)).toBe("3.70 GiB");
  });

  it("returns a dash for invalid input", () => {
    expect(formatBytes(-1)).toBe("—");
    expect(formatBytes(Number.NaN)).toBe("—");
    expect(formatBytes(Number.POSITIVE_INFINITY)).toBe("—");
  });
});

describe("formatSeconds", () => {
  it("formats minutes and hours", () => {
    expect(formatSeconds(0)).toBe("0:00");
    expect(formatSeconds(65)).toBe("1:05");
    expect(formatSeconds(3725)).toBe("1:02:05");
  });

  it("returns a dash for invalid input", () => {
    expect(formatSeconds(-1)).toBe("—");
    expect(formatSeconds(Number.NaN)).toBe("—");
  });
});
