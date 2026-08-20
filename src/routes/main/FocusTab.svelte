<script lang="ts">
  import PetCanvas from "../../lib/components/PetCanvas.svelte";
  import PixelButton from "../../lib/components/PixelButton.svelte";
  import SpeechBubble from "../../lib/components/SpeechBubble.svelte";
  import { miniLabel, petLine, phaseLabel, runLabel } from "../../lib/copy";
  import { endsAt, minutesLeft, mmss } from "../../lib/format";
  import { pause, skipPhase, start } from "../../lib/ipc";
  import { PETS } from "../../lib/sprites";
  import { app } from "../../lib/state.svelte";
  import TaskSidebar from "./TaskSidebar.svelte";

  const pet = $derived(PETS[app.pet.selected] ?? PETS[0]);

  let mini = $state(false);

  const remaining = $derived(app.timer.remainingSecs);
  const cells = $derived(app.bellyCells);
  const roundsTotal = $derived(app.settings.roundsPerCycle);

  function toggleRun() {
    void (app.timer.running ? pause() : start());
  }
</script>

<div class="body">
  <section class="stage">
    <div class="status">
      <span class="dot"></span>
      <span>
        {phaseLabel(app.timer.phase)} · 第 {app.timer.round}/{roundsTotal} 轮 · {app.activeTaskName}
      </span>
    </div>

    <div class="petwrap">
      <div class="ring"></div>
      <PetCanvas map={pet.map} body={pet.body} scale={8} anim="bob" alt={pet.name} />
      <div class="shadow"></div>
      <div class="belly">
        {#each Array.from({ length: 10 }, (_, i) => i) as i (i)}
          <span class="cell" class:filled={i < cells}></span>
        {/each}
      </div>
    </div>

    <div class="clock">
      <span class="mmss">{mmss(remaining)}</span>
      <span class="ends">{endsAt(remaining)}</span>
    </div>

    <SpeechBubble maxWidth={340}>
      {petLine(app.tone, minutesLeft(remaining))}
    </SpeechBubble>

    <div class="actions">
      <PixelButton onclick={toggleRun}>{runLabel(app.timer.running)}</PixelButton>
      <PixelButton variant="secondary" onclick={() => void skipPhase()}>跳过</PixelButton>
      <PixelButton variant="secondary" onclick={() => (mini = !mini)}>
        {miniLabel(mini)}
      </PixelButton>
    </div>

    <div class="rounds">
      {#each Array.from({ length: roundsTotal }, (_, i) => i) as i (i)}
        <span class="pip" class:on={i < app.timer.round}></span>
      {/each}
      <span class="hint">再 2 轮就能哄它去睡长觉（15 分钟）</span>
    </div>
  </section>

  <TaskSidebar />
</div>

<style>
  .body {
    flex: 1;
    display: flex;
    min-height: 0;
  }
  .stage {
    flex: 1;
    padding: 40px 44px 34px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 24px;
    background: linear-gradient(
      180deg,
      oklch(0.975 0.012 75) 0%,
      oklch(0.99 0.004 80) 70%
    );
    overflow-y: auto;
  }
  .status {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 6px 14px;
    border-radius: 20px;
    background: oklch(0.99 0.004 80 / 0.8);
    font-size: 13px;
    color: oklch(0.4 0.012 60);
  }
  .dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--accent);
  }
  .petwrap {
    position: relative;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
  }
  .ring {
    position: absolute;
    inset: -18px -34px;
    border-radius: 50%;
    border: 2px solid var(--accent);
    animation: momo-pulse 3.6s ease-in-out infinite;
    pointer-events: none;
  }
  .shadow {
    width: 124px;
    height: 11px;
    border-radius: 50%;
    background: oklch(0.24 0.012 60 / 0.14);
    filter: blur(4px);
  }
  .belly {
    display: flex;
    gap: 4px;
    margin-top: 4px;
  }
  .cell {
    width: 11px;
    height: 11px;
    background: var(--track);
  }
  .cell.filled {
    background: var(--accent);
  }
  .clock {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
  }
  .mmss {
    font-family: var(--font-mono);
    font-size: 78px;
    font-weight: 500;
    letter-spacing: -0.05em;
    line-height: 1;
    font-variant-numeric: tabular-nums;
  }
  .ends {
    font-size: 13px;
    color: var(--dim);
  }
  .actions {
    display: flex;
    gap: 10px;
    width: 100%;
    max-width: 420px;
  }
  .actions :global(.btn--primary) {
    flex: 1;
    padding-left: 0;
    padding-right: 0;
  }
  .rounds {
    display: flex;
    gap: 8px;
    align-items: center;
    font-size: 12.5px;
    color: var(--dim);
  }
  .pip {
    width: 10px;
    height: 10px;
    background: var(--line);
  }
  .pip.on {
    background: var(--accent);
  }
  .hint {
    margin-left: 6px;
  }
</style>
