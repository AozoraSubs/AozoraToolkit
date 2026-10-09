import type { ToolSource } from "../types/tools";

/** 将检测来源转换为设置页展示的中文标签。 */
export function toolSourceLabel(source: ToolSource): string {
  switch (source) {
    case "path":
      return "运行环境 PATH";
    case "app_bin":
      return "安装目录 bin";
    case "custom":
      return "自定义路径";
  }
}
