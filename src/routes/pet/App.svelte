<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import Pet from "../../lib/components/Pet.svelte";
  import { petLine } from "../../lib/copy";
  import { minutesLeft } from "../../lib/format";
  import {
    type FirePayload,
    ackReminder,
    hidePet,
    ignoreReminder,
    onPetNudge,
    petInteracted,
    setPetPlacement,
    showMain,
  } from "../../lib/ipc";
  import { PETS } from "../../lib/sprites";
  import { app } from "../../lib/state.svelte";

  const NUDGE_MS = 12_000;
  /** Pointer travel that turns a press into a drag rather than a click. */
  const DRAG_SLOP_PX = 4;
  /** Two clicks inside this window count as a double-click. */
  const DOUBLE_CLICK_MS = 400;

  let nudge = $state<FirePayload | null>(null);
  let dragging = $state(false);
  let nudgeTimer: ReturnType<typeof setTimeout> | undefined;

  let pressOrigin: { x: number; y: number } | null = null;
  let dragStarted = false;
  let lastClickAt = 0;

  const pet = $derived(PETS[app.pet.selected] ?? PETS[0]);
  // Rust decides the mood (pet:state); the local nudge only supplies the words.
  const anim = $derived(
    app.petMood === "nagging"
      ? "hop"
      : app.petMood === "sleeping"
        ? "sleep"
        : "bob",
  );
  const bubbleText = $derived(
    nudge?.message ?? petLine(app.tone, minutesLeft(app.timer.remainingSecs)),
  );

  onMount(() => {
    void app.init();
    const un = onPetNudge((payload) => {
      nudge = payload;
      clearTimeout(nudgeTimer);
      // An unanswered hop is an ignore, for escalation purposes.
      nudgeTimer = setTimeout(() => {
        void ignoreReminder(payload.id);
        nudge = null;
      }, NUDGE_MS);
    });
    return () => {
      clearTimeout(nudgeTimer);
      void un.then((f) => f());
      app.dispose();
    };
  });

  $effect(() => {
    document.documentElement.dataset.accent = app.settings.accent;
  });

  /**
   * The drag is deliberately deferred until the pointer actually moves.
   * `startDragging` hands the mouse loop to macOS, which swallows every
   * subsequent click — so starting it on pointerdown would make clicks and
   * double-clicks impossible to detect.
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
    dragging = true;
    const win = getCurrentWindow();
    await win.startDragging();
    // Read the position back once the native drag finishes; Rust applies edge
    // snapping and stores it.
    const pos = await win.outerPosition();
    const scale = await win.scaleFactor();
    dragging = false;
    pressOrigin = null;
    await setPetPlacement(pos.x / scale, pos.y / scale);
  }

  function onPointerUp() {
    const wasPress = pressOrigin !== null && !dragStarted;
    pressOrigin = null;
    if (!wasPress) return;
    // Any poke wakes a dozing pet, whether or not clicks do anything else.
    void petInteracted();
    if (!app.settings.petFlags.clickInteract) return;

    // A pending nudge is answered by a single click — it is a direct reply to a
    // prompt, not something you hit by accident.
    if (nudge) {
      clearTimeout(nudgeTimer);
      void ackReminder(nudge.id);
      nudge = null;
      lastClickAt = 0;
      return;
    }

    const now = Date.now();
    if (now - lastClickAt < DOUBLE_CLICK_MS) {
      lastClickAt = 0;
      void showMain();
    } else {
      lastClickAt = now;
    }
  }

  function onKey(event: KeyboardEvent) {
    if (event.key === "Enter") void showMain();
  }
</script>

<div class="stage">
  <div
    class="petwrap"
    role="button"
    tabindex="0"
    aria-label="{pet.name}（双击打开 Pomodo）"
    onpointerdown={onPointerDown}
    onpointermove={onPointerMove}
    onpointerup={onPointerUp}
    onkeydown={onKey}
  >
    <div class="pet" class:dragging>
      <Pet scale={8} {anim} alt={pet.name} />
      <div class="shadow"></div>
    </div>

    <button
      class="close"
      type="button"
      aria-label="隐藏桌面宠物"
      title="隐藏桌面宠物（可从菜单栏找回）"
      onpointerdown={(e) => e.stopPropagation()}
      onpointerup={(e) => e.stopPropagation()}
      onclick={() => void hidePet()}
    >
      ×
    </button>
  </div>

  <div class="bubble" class:nudging={!!nudge} class:dozing={anim === "sleep"}>
    {anim === "sleep" && !nudge ? "zzz…" : bubbleText}
  </div>
</div>

<style>
  :global(html),
  :global(body) {
    background: transparent;
    overflow: hidden;
  }
  .stage {
    display: flex;
    align-items: flex-end;
    gap: 12px;
    padding: 8px;
    height: 100vh;
  }
  .petwrap {
    position: relative;
    width: 128px;
    height: 128px;
    flex: none;
    background: transparent;
    border: none;
    padding: 0;
  }
  .pet {
    position: relative;
    width: 100%;
    height: 100%;
    cursor: grab;
  }
  .pet.dragging {
    cursor: grabbing;
  }
  .shadow {
    position: absolute;
    bottom: -6px;
    left: 8px;
    width: 112px;
    height: 10px;
    border-radius: 50%;
    background: oklch(0.2 0.02 260 / 0.4);
    filter: blur(4px);
  }
  /* Stays out of the way until you go looking for it. */
  .close {
    position: absolute;
    top: -2px;
    right: -2px;
    width: 20px;
    height: 20px;
    border-radius: 50%;
    border: none;
    background: oklch(0.24 0.012 60 / 0.55);
    color: oklch(0.99 0.004 80);
    font-size: 13px;
    line-height: 1;
    cursor: pointer;
    opacity: 0;
    transition: opacity 0.15s ease;
    display: grid;
    place-items: center;
    padding: 0;
  }
  .petwrap:hover .close,
  .close:focus-visible {
    opacity: 1;
  }
  .close:hover {
    background: oklch(0.24 0.012 60 / 0.8);
  }
  .bubble {
    margin-bottom: 4px;
    padding: 10px 14px;
    border-radius: 13px 13px 13px 4px;
    background: oklch(0.985 0.004 80 / 0.95);
    box-shadow: 0 12px 28px -12px oklch(0.2 0.02 260 / 0.6);
    font-size: 13.5px;
    line-height: 1.45;
    max-width: 220px;
    color: var(--ink);
  }
  .bubble.nudging {
    border: 1.5px solid var(--accent);
  }
  .bubble.dozing {
    color: var(--dim);
    font-family: "Silkscreen", monospace;
    letter-spacing: 0.08em;
  }
</style>
