<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import Pet from "../../lib/components/Pet.svelte";
  import { mmss } from "../../lib/format";
  import {
    type FirePayload,
    IS_TAURI,
    ackReminder,
    onMiniNudge,
    pause,
    setMiniHeight,
    setMiniMode,
    setMiniPlacement,
    skipPhase,
    start,
  } from "../../lib/ipc";
  import { PETS } from "../../lib/sprites";
  import { app } from "../../lib/state.svelte";
  import { elapsedPct } from "../../lib/theme";

  /** How long a swelled bar stays swelled, matching the desktop pet's nudge. */
  const NUDGE_MS = 12_000;
  /** Pointer travel that turns a press into a drag rather than a click. */
  const DRAG_SLOP_PX = 4;
  /** Two clicks inside this window count as a double-click. */
  const DOUBLE_CLICK_MS = 400;

  let hovered = $state(false);
  let nudge = $state<FirePayload | null>(null);
  let nudgeTimer: ReturnType<typeof setTimeout> | undefined;
  let bar = $state<HTMLElement | null>(null);

  let pressOrigin: { x: number; y: number } | null = null;
  let dragStarted = false;
  let lastClickAt = 0;

  const pet = $derived(PETS[app.pet.selected] ?? PETS[0]);
  const totalSecs = $derived(
    app.timer.phase === "focus"
      ? app.settings.focusSecs
      : app.timer.phase === "shortBreak"
        ? app.settings.shortBreakSecs
        : app.settings.longBreakSecs,
  );
  const pctStr = $derived(
    `${elapsedPct(totalSecs, app.timer.remainingSecs).toFixed(1)}%`,
  );

  /**
   * Swell the bar around a reminder rather than opening a second window: the
   * screen is already the other app's, and covering it again would take it back.
   * Exported so the component can be driven directly in tests.
   */
  export function receiveNudge(payload: FirePayload): void {
    nudge = payload;
    clearTimeout(nudgeTimer);
    nudgeTimer = setTimeout(collapse, NUDGE_MS);
  }

  function collapse(): void {
    clearTimeout(nudgeTimer);
    nudge = null;
  }

  /**
   * The window is exactly as tall as the bar rendered. How tall a reminder
   * comes out depends on the message, the font and the user's text size, none
   * of which Rust can measure — so the bar reports it rather than Rust guessing
   * a constant that a longer message would overflow. Resizing is a native
   * operation, so outside Tauri there is nothing to tell.
   */
  $effect(() => {
    // Re-runs whenever the nudge appears or goes away.
    void nudge;
    const el = bar;
    if (!IS_TAURI || !el) return;
    void setMiniHeight(el.offsetHeight);
  });

  onMount(() => {
    void app.init();
    const un = onMiniNudge(receiveNudge);
    return () => {
      clearTimeout(nudgeTimer);
      void un.then((f) => f());
      app.dispose();
    };
  });

  $effect(() => {
    document.documentElement.dataset.accent = app.settings.accent;
  });

  function exitMini() {
    void setMiniMode(false);
  }

  function toggleRun() {
    void (app.timer.running ? pause() : start());
  }

  function answerNudge() {
    if (!nudge) return;
    void ackReminder(nudge.id);
    collapse();
  }

  /**
   * Same deferral as the desktop pet: `startDragging` hands the mouse loop to
   * macOS, which swallows every later click, so it must not start until the
   * pointer has actually moved.
   */
  function onPointerDown(event: PointerEvent) {
    if (event.button !== 0) return;
    pressOrigin = { x: event.clientX, y: event.clientY };
    dragStarted = false;
  }

  function onPointerMove(event: PointerEvent) {
    if (!pressOrigin || dragStarted) return;
    const dx = event.clientX - pressOrigin.x;
    const dy = event.clientY - pressOrigin.y;
    if (Math.hypot(dx, dy) < DRAG_SLOP_PX) return;
    dragStarted = true;
    void beginDrag();
  }

  async function beginDrag() {
    const win = getCurrentWindow();
    await win.startDragging();
    // Read the position back once the native drag ends; Rust snaps and stores it.
    const pos = await win.outerPosition();
    const scale = await win.scaleFactor();
    pressOrigin = null;
    await setMiniPlacement(pos.x / scale, pos.y / scale);
  }

  function onPointerUp() {
    const wasPress = pressOrigin !== null && !dragStarted;
    pressOrigin = null;
    if (!wasPress) return;

    const now = Date.now();
    if (now - lastClickAt < DOUBLE_CLICK_MS) {
      lastClickAt = 0;
      exitMini();
    } else {
      lastClickAt = now;
    }
  }
</script>

<div class="wrap">
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    bind:this={bar}
    class="bar"
    onpointerenter={() => (hovered = true)}
    onpointerleave={() => (hovered = false)}
    onpointerdown={onPointerDown}
    onpointermove={onPointerMove}
    onpointerup={onPointerUp}
  >
    <div class="row">
      <div class="sprite">
        <Pet scale={2} anim={app.petMood === "sleeping" ? "sleep" : "none"} alt={pet.name} />
      </div>

      <div class="text">
        <span class="mmss">{mmss(app.timer.remainingSecs)}</span>
        <span class="task">{app.activeTaskName}</span>
      </div>

      <div class="acts" class:shown={hovered}>
        <button
          class="act"
          type="button"
          aria-label={app.timer.running ? "暂停" : "继续"}
          onpointerdown={(e) => e.stopPropagation()}
          onpointerup={(e) => e.stopPropagation()}
          onclick={toggleRun}
        >
          {#if app.timer.running}
            <span class="i-pause"></span>
          {:else}
            <span class="i-play"></span>
          {/if}
        </button>
        <button
          class="act"
          type="button"
          aria-label="跳过"
          onpointerdown={(e) => e.stopPropagation()}
          onpointerup={(e) => e.stopPropagation()}
          onclick={() => void skipPhase()}
        >
          <span class="i-skip"><span class="i-skip-bar"></span></span>
        </button>
        <button
          class="act"
          type="button"
          aria-label="回主窗口"
          onpointerdown={(e) => e.stopPropagation()}
          onpointerup={(e) => e.stopPropagation()}
          onclick={exitMini}
        >
          <span class="i-window"></span>
        </button>
      </div>
    </div>

    {#if nudge}
      <button class="nudge" type="button" onclick={answerNudge}>
        <span class="nudge-name">{nudge.name}</span>
        <span class="nudge-text">{nudge.message}</span>
      </button>
    {/if}

    <div class="track">
      <div class="fill" style:width={pctStr}></div>
    </div>
  </div>
</div>

<style>
  :global(html),
  :global(body) {
    background: transparent;
    overflow: hidden;
  }
  .wrap {
    height: 100vh;
    display: flex;
    align-items: flex-start;
  }
  /* 条形 — 260 x 52 at rest, growing downward around a reminder. */
  .bar {
    width: 260px;
    display: flex;
    flex-direction: column;
    border-radius: 16px;
    background: oklch(0.15 0.015 260 / 0.86);
    backdrop-filter: blur(24px);
    box-shadow: 0 16px 34px -14px oklch(0.12 0.02 260 / 0.8);
    overflow: hidden;
    cursor: grab;
    user-select: none;
  }
  /* 7 + 35 + 7 = 49, plus the 3px track = the artboard's 52. Fixed rather than
     content-derived so a long task name can never push the track out of the
     window, which is not resizable. */
  .row {
    height: 49px;
    box-sizing: border-box;
    padding: 7px 14px 7px 11px;
    display: flex;
    align-items: center;
    gap: 11px;
  }
  .sprite {
    width: 35px;
    height: 35px;
    flex: none;
    display: grid;
    place-items: center;
  }
  .text {
    display: flex;
    flex-direction: column;
    gap: 3px;
    flex: 1;
    min-width: 0;
  }
  .mmss {
    font-family: var(--font-mono);
    font-size: 21px;
    font-weight: 500;
    letter-spacing: -0.03em;
    line-height: 1;
    color: oklch(0.98 0.004 80);
  }
  .task {
    font-size: 11px;
    line-height: 1;
    color: oklch(0.75 0.01 80);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  /* Nothing but the time and the progress until you go looking. */
  .acts {
    display: flex;
    gap: 5px;
    flex: none;
    opacity: 0;
    transition: opacity 0.14s ease;
  }
  .acts.shown,
  .acts:focus-within {
    opacity: 1;
  }
  .act {
    width: 26px;
    height: 26px;
    padding: 0;
    border: none;
    border-radius: 8px;
    background: oklch(0.98 0.004 80 / 0.14);
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
  }
  .act:hover {
    background: oklch(0.98 0.004 80 / 0.26);
  }
  .i-pause {
    width: 9px;
    height: 10px;
    background: linear-gradient(
      to right,
      oklch(0.98 0.004 80) 0 3px,
      transparent 3px 6px,
      oklch(0.98 0.004 80) 6px 9px
    );
  }
  .i-play {
    width: 9px;
    height: 9px;
    background: oklch(0.98 0.004 80);
    clip-path: polygon(0% 0%, 100% 50%, 0% 100%);
  }
  /* The artboard draws skip as a bare triangle, but it only ever draws the
     running bar. Paused, that triangle would be identical to play — so skip
     carries the trailing bar that tells the two apart. */
  .i-skip {
    display: flex;
    align-items: center;
    gap: 2px;
    height: 9px;
  }
  .i-skip::before {
    content: "";
    width: 7px;
    height: 9px;
    background: oklch(0.98 0.004 80);
    clip-path: polygon(0% 0%, 100% 50%, 0% 100%);
  }
  .i-skip-bar {
    width: 2px;
    height: 9px;
    background: oklch(0.98 0.004 80);
  }
  .i-window {
    width: 11px;
    height: 9px;
    border: 1.5px solid oklch(0.98 0.004 80);
    border-top-width: 4px;
    border-radius: 2px;
  }
  .nudge {
    margin: 0 11px 9px;
    padding: 8px 10px;
    border: none;
    border-radius: 10px;
    background: oklch(0.98 0.004 80 / 0.1);
    display: flex;
    flex-direction: column;
    gap: 3px;
    text-align: left;
    cursor: pointer;
  }
  .nudge:hover {
    background: oklch(0.98 0.004 80 / 0.18);
  }
  .nudge-name {
    font-size: 12px;
    font-weight: 600;
    color: oklch(0.98 0.004 80);
  }
  .nudge-text {
    font-size: 11.5px;
    line-height: 1.4;
    color: oklch(0.78 0.01 80);
  }
  .track {
    margin-top: auto;
    height: 3px;
    background: oklch(0.98 0.004 80 / 0.14);
  }
  .fill {
    height: 100%;
    background: var(--accent);
  }
</style>
