import { formatDateTime, jobStatusClass, jobStatusLabel } from "./job";
import { describe, expect, it } from "vitest";

describe("jobStatusLabel", () => {
  it("maps every status to a Chinese label", () => {
    expect(jobStatusLabel("completed")).toBe("已完成");
    expect(jobStatusLabel("failed")).toBe("失败");
    expect(jobStatusLabel("cancelled")).toBe("已取消");
  });

  it("produces a css modifier per status", () => {
    expect(jobStatusClass("failed")).toBe("status-failed");
  });
});

describe("formatDateTime", () => {
  it("formats timestamps and rejects empty values", () => {
    const ms = new Date(2026, 9, 10, 21, 5).getTime();
    expect(formatDateTime(ms)).toBe("2026-10-10 21:05");
    expect(formatDateTime(0)).toBe("—");
    expect(formatDateTime(null)).toBe("—");
    expect(formatDateTime(Number.NaN)).toBe("—");
  });
});
