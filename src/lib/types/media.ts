export interface VideoStreamDto {
  index: number;
  codecName: string;
  profile: string | null;
  width: number | null;
  height: number | null;
  pixelFormat: string | null;
  bitDepth: number | null;
  frameRate: string | null;
  language: string | null;
  title: string | null;
}

export interface AudioStreamDto {
  index: number;
  codecName: string;
  sampleRate: string | null;
  channels: number | null;
  channelLayout: string | null;
  language: string | null;
  title: string | null;
  default: boolean;
}

export interface SubtitleStreamDto {
  index: number;
  codecName: string;
  language: string | null;
  title: string | null;
  default: boolean;
  forced: boolean;
}

export interface MediaProbeResult {
  filePath: string;
  formatName: string | null;
  formatLongName: string | null;
  durationSecs: number | null;
  sizeBytes: number | null;
  bitRate: number | null;
  videoStreams: VideoStreamDto[];
  audioStreams: AudioStreamDto[];
  subtitleStreams: SubtitleStreamDto[];
  chapterCount: number;
  attachmentCount: number;
}
