<script lang="ts">
  import EncodePage from "./lib/pages/EncodePage.svelte";
  import JobsPage from "./lib/pages/JobsPage.svelte";
  import PresetsPage from "./lib/pages/PresetsPage.svelte";
  import SettingsPage from "./lib/pages/SettingsPage.svelte";
  import { navigation, type PageId } from "./lib/stores/navigation.svelte";

  const pages: Array<{ id: PageId; label: string }> = [
    { id: "encode", label: "压制" },
    { id: "jobs", label: "任务" },
    { id: "presets", label: "预设" },
    { id: "settings", label: "设置" },
  ];
</script>

<div class="shell">
  <nav class="sidebar" aria-label="主导航">
    <div class="brand">
      <span class="brand-mark" aria-hidden="true">Ao</span>
      <div>
        <div class="brand-name">Aozora Encode</div>
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
    <div class="sidebar-foot">本地优先 · 离线处理</div>
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
    display: grid;
    place-items: center;
    width: 36px;
    height: 36px;
    border-radius: 8px;
    background: var(--color-accent);
    color: var(--color-accent-contrast);
    font-weight: 700;
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
