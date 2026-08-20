<script lang="ts">
  import { onMount } from "svelte";
  import TitleBar from "../../lib/components/TitleBar.svelte";
  import { openPrefs } from "../../lib/ipc";
  import { app } from "../../lib/state.svelte";
  import FocusTab from "./FocusTab.svelte";
  import PetTab from "./PetTab.svelte";
  import StatsTab from "./StatsTab.svelte";

  const TABS = ["专注", "统计", "宠物"] as const;
  let tab = $state(0);

  onMount(() => {
    void app.init();
    const onKey = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === ",") {
        e.preventDefault();
        void openPrefs();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => {
      window.removeEventListener("keydown", onKey);
      app.dispose();
    };
  });

  // The accent lives on the root element so every token resolves against it.
  $effect(() => {
    document.documentElement.dataset.accent = app.settings.accent;
  });
</script>

<div class="window">
  <TitleBar title="Momo">
    <div class="tabs">
      {#each TABS as name, i (name)}
        <button class="tab" class:active={tab === i} type="button" onclick={() => (tab = i)}>
          {name}
        </button>
      {/each}
    </div>
    <div class="meta">
      <span>连续 12 天</span>
      <span class="sep"></span>
      <button class="prefslink" type="button" onclick={() => void openPrefs()}>⌘,</button>
    </div>
  </TitleBar>

  {#if tab === 0}
    <FocusTab />
  {:else if tab === 1}
    <StatsTab />
  {:else}
    <PetTab />
  {/if}
</div>

<style>
  .window {
    height: 100vh;
    display: flex;
    flex-direction: column;
    background: var(--card);
    overflow: hidden;
  }
  .tabs {
    margin-left: 20px;
    display: flex;
    gap: 4px;
    padding: 3px;
    border-radius: 9px;
    background: oklch(0.92 0.008 70);
  }
  .tab {
    padding: 5px 14px;
    border: none;
    border-radius: 7px;
    background: transparent;
    color: var(--dim);
    font-size: 12.5px;
    font-weight: 400;
    cursor: pointer;
  }
  .tab.active {
    background: var(--card);
    color: var(--ink);
    font-weight: 600;
  }
  .meta {
    margin-left: auto;
    display: flex;
    align-items: center;
    gap: 14px;
    font-size: 12.5px;
    color: oklch(0.52 0.012 60);
  }
  .prefslink {
    border: none;
    background: transparent;
    padding: 0;
    font: inherit;
    color: inherit;
    cursor: pointer;
  }
  .prefslink:hover {
    color: var(--ink);
  }
  .sep {
    width: 1px;
    height: 14px;
    background: var(--line);
  }
</style>
