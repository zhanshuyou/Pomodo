<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import PetCanvas from "../../lib/components/PetCanvas.svelte";
  import { petLine } from "../../lib/copy";
  import { minutesLeft } from "../../lib/format";
  import {
    type FirePayload,
    ackReminder,
    onPetNudge,
    setPetPlacement,
    showMain,
  } from "../../lib/ipc";
  import { PETS } from "../../lib/sprites";
  import { app } from "../../lib/state.svelte";

  const NUDGE_MS = 12_000;

  let nudge = $state<FirePayload | null>(null);
  let dragging = $state(false);
  let nudgeTimer: ReturnType<typeof setTimeout> | undefined;

  const pet = $derived(PETS[app.pet.selected] ?? PETS[0]);
  const bubbleText = $derived(
    nudge?.message ?? petLine(app.tone, minutesLeft(app.timer.remainingSecs)),
  );

  onMount(() => {
    void app.init();
    const un = onPetNudge((payload) => {
      nudge = payload;
      clearTimeout(nudgeTimer);
      nudgeTimer = setTimeout(() => (nudge = null), NUDGE_MS);
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
   * startDragging moves the window natively; when it finishes we read the final
   * position back and hand it to Rust, which applies edge snapping and stores it.
   */
  async function onPointerDown(event: PointerEvent) {
    if (event.button !== 0) return;
    dragging = true;
    const win = getCurrentWindow();
    await win.startDragging();
    const pos = await win.outerPosition();
    const scale = await win.scaleFactor();
    dragging = false;
    await setPetPlacement(pos.x / scale, pos.y / scale);
  }

  function onPoke() {
    if (!app.settings.petFlags.clickInteract) return;
    if (nudge) {
      void ackReminder(nudge.id);
      nudge = null;
      return;
    }
    void showMain();
  }
</script>

<div class="stage">
  <div
    class="pet"
    class:dragging
    role="button"
    tabindex="0"
    aria-label={pet.name}
    onpointerdown={onPointerDown}
    onclick={onPoke}
    onkeydown={(e) => e.key === "Enter" && onPoke()}
  >
    <PetCanvas
      map={pet.map}
      body={pet.body}
      scale={8}
      anim={nudge ? "hop" : "bob"}
      alt={pet.name}
    />
    <div class="shadow"></div>
  </div>

  <div class="bubble" class:nudging={!!nudge}>{bubbleText}</div>
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
  .pet {
    position: relative;
    width: 128px;
    height: 128px;
    flex: none;
    cursor: grab;
    background: transparent;
    border: none;
    padding: 0;
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
</style>
