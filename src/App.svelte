<script lang="ts">
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";
  import brandMark from "./lib/assets/brand-mark.png";
  import EncodePage from "./lib/pages/EncodePage.svelte";
  import JobsPage from "./lib/pages/JobsPage.svelte";
  import PresetsPage from "./lib/pages/PresetsPage.svelte";
  import SettingsPage from "./lib/pages/SettingsPage.svelte";
  import { navigation, type PageId } from "./lib/stores/navigation.svelte";
  import {
    jobState,
    onJobFinished,
    onJobLog,
    onJobProgress,
    onJobQueue,
    onJobStarted,
    refreshHistory,
    refreshQueue,
    tickElapsed,
  } from "./lib/stores/jobs.svelte";
  import type {
    JobFinishedEvent,
    JobLogEvent,
    JobProgressEvent,
    JobStartedEvent,
  } from "./lib/types/encode";
  import type { QueueSnapshot } from "./lib/types/jobs";

  const pages: Array<{ id: PageId; label: string }> = [
    { id: "encode", label: "压制" },
    { id: "jobs", label: "任务" },
    { id: "presets", label: "预设" },
    { id: "settings", label: "设置" },
  ];

  // 任务事件在常驻的 App 组件中统一监听并写入全局状态，
  // 页面切换不会丢失压制进度、队列与历史。
  onMount(() => {
    let unlisteners: Array<() => void> = [];
    void (async () => {
      unlisteners = [
        await listen<JobStartedEvent>("job:started", (event) => {
          onJobStarted(event.payload);
        }),
        await listen<JobProgressEvent>("job:progress", (event) => {
          onJobProgress(event.payload);
        }),
        await listen<JobLogEvent>("job:log", (event) => {
          onJobLog(event.payload);
        }),
        await listen<JobFinishedEvent>("job:finished", (event) => {
          onJobFinished(event.payload);
        }),
        await listen<QueueSnapshot>("job:queue", (event) => {
          onJobQueue(event.payload);
        }),
      ];
      // 补齐监听建立之前的状态（历史与队列）
      void refreshQueue();
      void refreshHistory();
    })();
    return () => {
      for (const unlisten of unlisteners) {
        unlisten();
      }
    };
  });

  // 运行中每 500ms 刷新已用时间
  $effect(() => {
    if (jobState.job && !jobState.job.finished) {
      jobState.elapsed = 0;
      const timer = setInterval(tickElapsed, 500);
      return () => {
        clearInterval(timer);
        tickElapsed();
      };
    }
  });
</script>

<div class="shell">
  <nav class="sidebar" aria-label="主导航">
    <div class="brand">
      <img class="brand-mark" src={brandMark} alt="" />
      <div>
        <div class="brand-name">Aozora Toolbox</div>
        <div class="brand-sub">本地视频压制工具箱</div>
      </div>
    </div>
    <ul class="nav-list">
      {#each pages as page (page.id)}
        <li>
          <button
            type="button"
            class="nav-item"
            class:active={navigation.activePage === page.id}
            aria-current={navigation.activePage === page.id
              ? "page"
              : undefined}
            onclick={() => (navigation.activePage = page.id)}
          >
            {page.label}
          </button>
        </li>
      {/each}
    </ul>
    <div class="sidebar-foot">Copyright 2026 AozoraSubs</div>
  </nav>

  <main class="content">
    {#if navigation.activePage === "encode"}
      <EncodePage />
    {:else if navigation.activePage === "jobs"}
      <JobsPage />
    {:else if navigation.activePage === "presets"}
      <PresetsPage />
    {:else}
      <SettingsPage />
    {/if}
  </main>
</div>

<style>
  .shell {
    display: flex;
    height: 100vh;
  }

  .sidebar {
    width: 200px;
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding: 14px;
    background: var(--color-surface);
    border-right: 1px solid var(--color-border);
  }

  .brand {
    display: flex;
    gap: 10px;
    align-items: center;
    padding: 4px;
  }

  .brand-mark {
    width: 36px;
    height: 36px;
    border-radius: 8px;
  }

  .brand-name {
    font-weight: 650;
  }

  .brand-sub {
    font-size: 12px;
    color: var(--color-text-secondary);
  }

  .nav-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .nav-item {
    width: 100%;
    padding: 8px 10px;
    border: none;
    border-radius: 6px;
    background: transparent;
    color: var(--color-text);
    text-align: left;
    cursor: pointer;
  }

  .nav-item:hover {
    background: var(--color-hover);
  }

  .nav-item.active {
    background: var(--color-active);
    font-weight: 600;
  }

  .sidebar-foot {
    margin-top: auto;
    font-size: 12px;
    color: var(--color-text-secondary);
  }

  .content {
    flex: 1;
    overflow: auto;
    padding: 24px;
  }
</style>
