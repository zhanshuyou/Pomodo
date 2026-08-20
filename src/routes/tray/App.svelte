<script lang="ts">
  import { onMount } from "svelte";
  import PetCanvas from "../../lib/components/PetCanvas.svelte";
  import { phaseLabel, runLabel } from "../../lib/copy";
  import { mmss } from "../../lib/format";
  import {
    type TodaySummary,
    type UpNextItem,
    openPrefs,
    pause,
    showMain,
    skipPhase,
    start,
    todaySummary,
    upNext,
  } from "../../lib/ipc";
  import { PETS } from "../../lib/sprites";
  import { app } from "../../lib/state.svelte";
  import { ACCENTS, elapsedPct, ringGradient } from "../../lib/theme";

  let next = $state<UpNextItem[]>([]);
  let today = $state<TodaySummary | null>(null);

  const pet = $derived(PETS[app.pet.selected] ?? PETS[0]);
  const accent = $derived(ACCENTS[app.settings.accent]);

  const totalSecs = $derived(
    app.timer.phase === "focus"
      ? app.settings.focusSecs
      : app.timer.phase === "shortBreak"
        ? app.settings.shortBreakSecs
        : app.settings.longBreakSecs,
  );
  const pct = $derived(elapsedPct(totalSecs, app.timer.remainingSecs));

  async function refreshLists() {
    [next, today] = await Promise.all([upNext(), todaySummary()]);
  }

  onMount(() => {
    void app.init().then(refreshLists);
    // The up-next column counts down in whole minutes; refreshing every 15 s is
    // frequent enough to look live without re-invoking on every timer tick.
    const handle = setInterval(() => void refreshLists(), 15_000);
    return () => {
      clearInterval(handle);
      app.dispose();
    };
  });

  $effect(() => {
    document.documentElement.dataset.accent = app.settings.accent;
  });
</script>

<div class="popover">
  <div class="head">
    <div
      class="ring"
      style:background={ringGradient(accent, pct)}
    >
      <div class="disc"></div>
      <div class="petslot">
        <PetCanvas map={pet.map} body={pet.body} scale={3} alt={pet.name} />
      </div>
    </div>

    <div class="headright">
      <div class="line">
        <span class="mmss">{mmss(app.timer.remainingSecs)}</span>
        <span class="phase">{phaseLabel(app.timer.phase)}</span>
      </div>
      <div class="buttons">
        <button
          class="primary"
          type="button"
          onclick={() => void (app.timer.running ? pause() : start())}
        >
          {runLabel(app.timer.running)}
        </button>
        <button class="secondary" type="button" onclick={() => void skipPhase()}>跳过</button>
      </div>
    </div>
  </div>

  <div class="rule"></div>

  <div class="next">
    <span class="label">接下来轮到</span>
    {#each next as item (item.id)}
      <div class="nextrow">
        <span class="swatch" style:background={item.color}></span>
        <span class="nextname">{item.name}</span>
        <span class="due">{item.due}</span>
      </div>
    {:else}
      <span class="empty">今天没有排队的提醒</span>
    {/each}
  </div>

  <div class="rule"></div>

  <div class="foot">
    <button class="link" type="button" onclick={() => void showMain()}>
      {today?.label ?? "今天 0 个番茄 · 0h00m"}
    </button>
    <button class="link" type="button" onclick={() => void openPrefs()}>设置…</button>
  </div>
</div>

<style>
  :global(html),
  :global(body) {
    background: transparent;
  }
  .popover {
    width: 330px;
    border-radius: 16px;
    background: oklch(0.985 0.004 80 / 0.95);
    backdrop-filter: blur(30px);
    box-shadow: 0 24px 50px -18px oklch(0.2 0.02 260 / 0.55);
    overflow: hidden;
    animation: momo-rise 0.35s ease both;
  }
  .head {
    padding: 18px 20px 16px;
    display: flex;
    align-items: center;
    gap: 16px;
  }
  .ring {
    position: relative;
    width: 76px;
    height: 76px;
    border-radius: 50%;
    flex: none;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .disc {
    position: absolute;
    inset: 6px;
    border-radius: 50%;
    background: var(--card);
  }
  .petslot {
    position: relative;
    width: 48px;
    height: 48px;
    display: grid;
    place-items: center;
  }
  .headright {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 9px;
    min-width: 0;
  }
  .line {
    display: flex;
    align-items: baseline;
    gap: 8px;
  }
  .mmss {
    font-family: var(--font-mono);
    font-size: 22px;
    font-weight: 500;
    font-variant-numeric: tabular-nums;
  }
  .phase {
    font-size: 12px;
    color: var(--accent);
    font-weight: 600;
  }
  .buttons {
    display: flex;
    gap: 7px;
  }
  .primary {
    flex: 1;
    padding: 8px 0;
    border: none;
    border-radius: 9px;
    background: var(--accent);
    color: var(--card);
    font-family: inherit;
    font-size: 12.5px;
    font-weight: 600;
    cursor: pointer;
  }
  .primary:hover {
    filter: brightness(1.07);
  }
  .secondary {
    padding: 8px 11px;
    border: 1px solid var(--line);
    border-radius: 9px;
    background: var(--card);
    font-family: inherit;
    font-size: 12.5px;
    color: oklch(0.42 0.012 60);
    cursor: pointer;
  }
  .rule {
    height: 1px;
    background: oklch(0.9 0.008 70);
  }
  .next {
    padding: 13px 20px;
    display: flex;
    flex-direction: column;
    gap: 11px;
  }
  .label {
    font-size: 11px;
    letter-spacing: 0.14em;
    text-transform: uppercase;
    color: var(--faint);
  }
  .nextrow {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .swatch {
    width: 8px;
    height: 8px;
    flex: none;
  }
  .nextname {
    font-size: 13.5px;
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .due {
    font-family: var(--font-mono);
    font-size: 12.5px;
    color: oklch(0.58 0.012 60);
  }
  .empty {
    font-size: 12.5px;
    color: var(--dim);
  }
  .foot {
    padding: 12px 20px 15px;
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .link {
    border: none;
    background: transparent;
    padding: 0;
    font-family: inherit;
    font-size: 12.5px;
    color: var(--dim);
    cursor: pointer;
  }
  .link:hover {
    color: var(--ink);
  }
</style>
