/**
 * 命令展示工具：把参数数组渲染成可读、可复制的命令行文本。
 *
 * 仅用于**展示与复制**：应用始终以参数数组启动 FFmpeg，不经过 Shell
 * （PLAN §4 安全执行原则）。这里不做 Shell 转义，只做最小限度的
 * 引号包裹，避免带空格或引号的参数在阅读时产生歧义。
 */

/**
 * 需要引号包裹的字符：空白，以及会在 cmd / PowerShell 中被解释的字符。
 * `=`、`:`、`?` 等常见于 FFmpeg 参数（滤镜串、流说明符）的字符不加引号，
 * 保持命令可读。
 */
const NEEDS_QUOTING = /[\s"&|<>^()%!,;]/;

/** 为展示目的给单个参数加引号（不需要时原样返回）。 */
export function quoteArg(arg: string): string {
  if (arg === "") {
    return '""';
  }
  if (!NEEDS_QUOTING.test(arg)) {
    return arg;
  }
  return `"${arg.replaceAll('"', '""')}"`;
}

/**
 * 渲染完整命令行文本。
 *
 * `executable` 为当前生效的 FFmpeg 路径（未知时传 null，只展示参数）。
 */
export function formatCommandLine(
  executable: string | null,
  args: string[],
): string {
  const parts = args.map(quoteArg);
  if (executable && executable.trim() !== "") {
    parts.unshift(quoteArg(executable));
  }
  return parts.join(" ");
}

/** 复制文本到剪贴板；返回是否复制成功。 */
export async function copyText(text: string): Promise<boolean> {
  try {
    if (navigator.clipboard?.writeText) {
      await navigator.clipboard.writeText(text);
      return true;
    }
  } catch {
    // 剪贴板 API 不可用（非安全上下文或权限被拒）时回退到旧接口
  }
  try {
    const area = document.createElement("textarea");
    area.value = text;
    area.setAttribute("readonly", "");
    area.style.position = "fixed";
    area.style.opacity = "0";
    document.body.appendChild(area);
    area.select();
    const ok = document.execCommand("copy");
    area.remove();
    return ok;
  } catch {
    return false;
  }
}
