export type ToolSource = "path" | "app_bin" | "custom";

export interface ToolInfo {
  path: string;
  version: string | null;
  source: ToolSource;
}

export interface ToolDetectionResult {
  ffmpeg: ToolInfo | null;
  ffprobe: ToolInfo | null;
  customToolDir: string | null;
  errors: string[];
}
