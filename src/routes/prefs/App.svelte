<script lang="ts">
  import { onMount } from "svelte";
  import TitleBar from "../../lib/components/TitleBar.svelte";
  import { app } from "../../lib/state.svelte";
  import GeneralPane from "./GeneralPane.svelte";
  import PetPane from "./PetPane.svelte";
  import RemindersPane from "./RemindersPane.svelte";
  import SoundPane from "./SoundPane.svelte";
  import TimerPane from "./TimerPane.svelte";

  const PANES = ["计时", "提醒", "宠物", "声音", "通用"] as const;
  let active = $state(1);

  onMount(() => {
    void app.init();
    return () => app.dispose();
  });

  $effect(() => {
    document.documentElement.dataset.accent = app.settings.accent;
  });
</script>

<div class="window">
  <TitleBar title="设置 — {PANES[active]}" />
  <div class="body">
    <nav>
      {#each PANES as name, i (name)}
        <button class="nav" class:active={active === i} type="button" onclick={() => (active = i)}>
          {name}
        </button>
      {/each}
    </nav>

    {#if active === 0}
      <TimerPane />
    {:else if active === 1}
      <RemindersPane />
    {:else if active === 2}
      <PetPane />
    {:else if active === 3}
      <SoundPane />
    {:else}
      <GeneralPane />
    {/if}
  </div>
</div>

<style>
  .window {
    height: 100vh;
    display: flex;
    flex-direction: column;
    background: var(--card);
    overflow: hidden;
  }
  .body {
    flex: 1;
    display: flex;
    min-height: 0;
  }
  nav {
    width: 172px;
    flex: none;
    padding: 16px 12px;
    background: var(--surface-2);
    border-right: 1px solid oklch(0.91 0.008 70);
    display: flex;
    flex-direction: column;
    gap: 3px;
  }
  .nav {
    padding: 8px 12px;
    border: none;
    border-radius: 8px;
    background: transparent;
    color: var(--dim);
    font-size: 13.5px;
    font-weight: 400;
    text-align: left;
    cursor: pointer;
  }
  .nav.active {
    background: var(--card);
    color: var(--ink);
    font-weight: 600;
  }
</style>
