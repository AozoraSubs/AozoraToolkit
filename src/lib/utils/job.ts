import type { JobStatus } from "../types/encode";

/** 任务状态的中文标签。 */
export function jobStatusLabel(status: JobStatus): string {
  switch (status) {
    case "completed":
      return "已完成";
    case "failed":
      return "失败";
    case "cancelled":
      return "已取消";
  }
}

/** 任务状态对应的 CSS 修饰名（用于状态标签配色）。 */
export function jobStatusClass(status: JobStatus): string {
  return `status-${status}`;
}

/** 将毫秒时间戳格式化为本地 "YYYY-MM-DD HH:mm"；无效值返回 "—"。 */
export function formatDateTime(ms: number | null | undefined): string {
  if (ms === null || ms === undefined || !Number.isFinite(ms) || ms <= 0) {
    return "—";
  }
  const date = new Date(ms);
  const pad = (value: number) => String(value).padStart(2, "0");
  return (
    `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}` +
    ` ${pad(date.getHours())}:${pad(date.getMinutes())}`
  );
}
