<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { open } from "@tauri-apps/plugin-dialog";
  import { onDestroy, onMount } from "svelte";
  import type { MediaProbeResult } from "../types/media";
  import { formatBytes, formatSeconds } from "../utils/format";

  let info = $state<MediaProbeResult | null>(null);
  let busy = $state(false);
  let error = $state<string | null>(null);
  let dragging = $state(false);

  async function loadFile(path: string) {
    busy = true;
    error = null;
    try {
      info = await invoke<MediaProbeResult>("probe_media_file", { path });
    } catch (err) {
      info = null;
      error = err instanceof Error ? err.message : String(err);
    } finally {
      busy = false;
    }
  }

  async function pickFile() {
    const selected = await open({
      multiple: false,
      directory: false,
      title: "选择视频文件",
      filters: [
        {
          name: "视频文件",
          extensions: [
            "mkv",
            "mp4",
            "mov",
            "avi",
            "webm",
            "flv",
            "ts",
            "m2ts",
            "wmv",
          ],
        },
      ],
    });
    if (typeof selected === "string") {
      await loadFile(selected);
    }
  }

  let unlisten: (() => void) | null = null;
  onMount(async () => {
    unlisten = await getCurrentWebview().onDragDropEvent((event) => {
      if (event.payload.type === "over") {
        dragging = true;
      } else if (event.payload.type === "drop") {
        dragging = false;
        const path = event.payload.paths[0];
        if (path) {
          void loadFile(path);
        }
      } else {
        dragging = false;
      }
    });
  });
  onDestroy(() => {
    unlisten?.();
  });
</script>

<section class="page">
  <header>
    <h1>压制</h1>
    <p class="hint">选择输入文件、编码器与输出路径，开始压制任务。</p>
  </header>

  <div
    class="dropzone"
    class:dragging
    role="button"
    tabindex="0"
    onclick={pickFile}
    onkeydown={(e) => {
      if (e.key === "Enter" || e.key === " ") {
        e.preventDefault();
        pickFile();
      }
    }}
  >
    <p class="dz-main">{dragging ? "松开以导入文件" : "拖放视频文件到此处"}</p>
    <p class="dz-sub">或点击选择文件（MKV / MP4 / MOV 等）</p>
  </div>

  {#if busy}
    <p class="hint">正在探测媒体信息…</p>
  {/if}

  {#if error}
    <div class="errors" role="alert">
      <strong>探测失败</strong>
      <p>{error}</p>
    </div>
  {/if}

  {#if info}
    <section class="file-card">
      <h2>文件信息</h2>
      <dl class="meta">
        <div>
          <dt>路径</dt>
          <dd class="path" title={info.filePath}>{info.filePath}</dd>
        </div>
        <div>
          <dt>容器</dt>
          <dd>
            {info.formatName ?? "—"}{info.formatLongName
              ? `（${info.formatLongName}）`
              : ""}
          </dd>
        </div>
        <div>
          <dt>时长</dt>
          <dd>
            {info.durationSecs !== null
              ? formatSeconds(info.durationSecs)
              : "—"}
          </dd>
        </div>
        <div>
          <dt>大小</dt>
          <dd>{info.sizeBytes !== null ? formatBytes(info.sizeBytes) : "—"}</dd>
        </div>
        <div>
          <dt>码率</dt>
          <dd>
            {info.bitRate !== null ? `${formatBytes(info.bitRate)}/s` : "—"}
          </dd>
        </div>
        <div>
          <dt>章节</dt>
          <dd>{info.chapterCount} 个</dd>
        </div>
        <div>
          <dt>附件</dt>
          <dd>{info.attachmentCount} 个</dd>
        </div>
      </dl>

      {#if info.videoStreams.length > 0}
        <h3>视频流</h3>
        <table class="streams">
          <thead>
            <tr>
              <th>#</th><th>编码</th><th>Profile</th><th>分辨率</th>
              <th>帧率</th><th>像素格式</th><th>位深</th><th>语言</th>
            </tr>
          </thead>
          <tbody>
            {#each info.videoStreams as s (s.index)}
              <tr>
                <td>{s.index}</td>
                <td>{s.codecName}</td>
                <td>{s.profile ?? "—"}</td>
                <td
                  >{s.width !== null && s.height !== null
                    ? `${s.width}×${s.height}`
                    : "—"}</td
                >
                <td>{s.frameRate ?? "—"}</td>
                <td>{s.pixelFormat ?? "—"}</td>
                <td>{s.bitDepth !== null ? `${s.bitDepth} bit` : "—"}</td>
                <td>{s.language ?? "—"}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}

      {#if info.audioStreams.length > 0}
        <h3>音频流</h3>
        <table class="streams">
          <thead>
            <tr>
              <th>#</th><th>编码</th><th>采样率</th><th>声道</th>
              <th>声道布局</th><th>语言</th><th>默认</th>
            </tr>
          </thead>
          <tbody>
            {#each info.audioStreams as s (s.index)}
              <tr>
                <td>{s.index}</td>
                <td>{s.codecName}</td>
                <td>{s.sampleRate ? `${s.sampleRate} Hz` : "—"}</td>
                <td>{s.channels ?? "—"}</td>
                <td>{s.channelLayout ?? "—"}</td>
                <td>{s.language ?? "—"}</td>
                <td>{s.default ? "✓" : ""}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}

      {#if info.subtitleStreams.length > 0}
        <h3>字幕流</h3>
        <table class="streams">
          <thead>
            <tr>
              <th>#</th><th>格式</th><th>语言</th><th>标题</th><th>默认</th><th
                >强制</th
              >
            </tr>
          </thead>
          <tbody>
            {#each info.subtitleStreams as s (s.index)}
              <tr>
                <td>{s.index}</td>
                <td>{s.codecName}</td>
                <td>{s.language ?? "—"}</td>
                <td>{s.title ?? "—"}</td>
                <td>{s.default ? "✓" : ""}</td>
                <td>{s.forced ? "✓" : ""}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
    </section>
  {/if}

  <div class="placeholder">
    <strong>即将实现（M2）</strong>
    <ul>
      <li>编码器、质量参数与预设选择</li>
      <li>输出路径与命令预览</li>
      <li>任务进度、日志与取消</li>
    </ul>
  </div>
</section>

<style>
  .dropzone {
    display: flex;
    flex-direction: column;
    gap: 4px;
    align-items: center;
    justify-content: center;
    min-height: 120px;
    margin-bottom: 16px;
    border: 2px dashed var(--color-border);
    border-radius: 10px;
    background: var(--color-surface);
    cursor: pointer;
    text-align: center;
  }

  .dropzone:hover,
  .dropzone.dragging {
    border-color: var(--color-accent);
    background: var(--color-hover);
  }

  .dz-main {
    margin: 0;
    font-size: 16px;
    font-weight: 600;
  }

  .dz-sub {
    margin: 0;
    font-size: 12px;
    color: var(--color-text-secondary);
  }

  .errors {
    margin: 0 0 16px;
    padding: 10px 12px;
    max-width: 720px;
    border: 1px solid var(--color-danger);
    border-radius: 8px;
    background: var(--color-surface);
    color: var(--color-danger);
    word-break: break-all;
  }

  .errors p {
    margin: 4px 0 0;
  }

  .file-card {
    margin-bottom: 16px;
    max-width: 860px;
    padding: 14px 16px;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: 10px;
  }

  .file-card h2 {
    margin: 0 0 10px;
    font-size: 16px;
  }

  .file-card h3 {
    margin: 14px 0 6px;
    font-size: 14px;
  }

  .meta {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
    gap: 8px 16px;
    margin: 0;
  }

  .meta dt {
    font-size: 12px;
    color: var(--color-text-secondary);
  }

  .meta dd {
    margin: 0;
  }

  .meta .path {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 100%;
    direction: rtl;
    text-align: left;
  }

  .streams {
    width: 100%;
    border-collapse: collapse;
    font-size: 13px;
  }

  .streams th,
  .streams td {
    padding: 4px 8px;
    text-align: left;
    border-bottom: 1px solid var(--color-border);
  }

  .streams th {
    color: var(--color-text-secondary);
    font-weight: 600;
  }
</style>
