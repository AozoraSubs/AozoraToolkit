import type {
  AudioConfig,
  Container,
  SubtitleConfig,
  VideoConfig,
} from "./encode";

/** 预设中的输出选项：只保留与机器无关的输出意图，不含输出路径。 */
export interface PresetOutput {
  container: Container;
  overwrite: boolean;
}

/** 一条编码预设（对应 Rust `aozora_core::preset::Preset`）。 */
export interface Preset {
  schemaVersion: number;
  id: string;
  name: string;
  description: string | null;
  /** 内置示例预设：只读，不可重命名或删除。 */
  builtin: boolean;
  video: VideoConfig;
  audio: AudioConfig;
  subtitles: SubtitleConfig;
  output: PresetOutput;
  createdAtMs: number;
  updatedAtMs: number;
}

/** 新建 / 复制预设时提交的输入（ID 与时间戳由后端生成）。 */
export interface PresetInput {
  name: string;
  description: string | null;
  video: VideoConfig;
  audio: AudioConfig;
  subtitles: SubtitleConfig;
  output: PresetOutput;
}

export interface PresetListResult {
  presets: Preset[];
  /** 单个预设文件加载失败的可读错误（不影响其他预设）。 */
  errors: string[];
}
