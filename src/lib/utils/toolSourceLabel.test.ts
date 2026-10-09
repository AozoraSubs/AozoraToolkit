import { describe, expect, it } from "vitest";
import { toolSourceLabel } from "./toolSourceLabel";

describe("toolSourceLabel", () => {
  it("maps every detection source to a Chinese label", () => {
    expect(toolSourceLabel("path")).toBe("运行环境 PATH");
    expect(toolSourceLabel("app_bin")).toBe("安装目录 bin");
    expect(toolSourceLabel("custom")).toBe("自定义路径");
  });
});
