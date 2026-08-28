<script lang="ts">
  import { onMount } from "svelte";
  import Pet from "../../lib/components/Pet.svelte";
  import { mmss } from "../../lib/format";
  import { snoozeLabel } from "../../lib/copy";
  import {
    type FirePayload,
    SNOOZE_MINUTES,
    dismissOverlay,
    onOverlayShow,
    snoozeOverlay,
  } from "../../lib/ipc";
  import { PETS } from "../../lib/sprites";
  import { app } from "../../lib/state.svelte";

  let fire = $state<FirePayload | null>(null);
  let left = $state(0);
  /** The countdown has finished and Rust has been told; do not tell it again. */
  let finished = false;

  const pet = $derived(PETS[app.pet.selected] ?? PETS[0]);

  onMount(() => {
    void app.init();

    const un = onOverlayShow((payload) => {
      fire = payload;
      // Bubble/pet fires omit the duration; an overlay always carries one.
      left = payload.durationSecs ?? 60;
      finished = false;
    });

    // The window can exist before its payload does (one is created per
    // display, then told what to say); until then it shows nothing rather
    // than a made-up countdown.
    const ticker = setInterval(() => {
      if (!fire) return;
      if (left > 0) {
        left -= 1;
      } else if (fire && !finished && !fire.mustComplete) {
        // Sitting through the whole countdown counts as doing the thing.
        // 必须完成 waits for the button instead.
        finished = true;
        void dismissOverlay(fire.id, true);
      }
    }, 1000);

    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape" && fire && !fire.mustComplete) {
        void dismissOverlay(fire.id, false);
      }
    };
    window.addEventListener("keydown", onKey);

    return () => {
      clearInterval(ticker);
      window.removeEventListener("keydown", onKey);
      void un.then((f) => f());
      app.dispose();
    };
  });

  $effect(() => {
    document.documentElement.dataset.accent = app.settings.accent;
  });
</script>

{#if fire}
  {@const f = fire}
  <div class="mask">
    <Pet scale={3} anim="sway" slot="nag" alt={pet.name} />
    <span class="name">
      <span class="dot" style:background={f.color}></span>
      {f.name}
    </span>
    <span class="count">{mmss(left)}</span>
    <span class="line">{f.message}</span>
    <div class="acts">
      {#if !f.mustComplete}
        <button class="later" type="button" onclick={() => void snoozeOverlay(f.id)}>
          {snoozeLabel(app.tone, SNOOZE_MINUTES)}
        </button>
      {/if}
      <!-- 必须完成 shows the button only once the countdown has run down. -->
      {#if !f.mustComplete || left === 0}
        <button class="done" type="button" onclick={() => void dismissOverlay(f.id, true)}>
          做完了
        </button>
      {/if}
    </div>
    {#if f.mustComplete}
      <span class="escape">这条得做完才能走</span>
    {:else}
      <span class="escape">按 ⎋ 逃跑（它会记着）</span>
    {/if}
  </div>
{/if}

<style>
  :global(html),
  :global(body) {
    margin: 0;
    overflow: hidden;
  }
  .mask {
    position: relative;
    width: 100vw;
    height: 100vh;
    background: oklch(0.29 0.025 258);
    color: oklch(0.97 0.004 80);
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 10px;
  }
  .name {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    font-size: 13px;
    color: oklch(0.97 0.004 80 / 0.75);
  }
  .dot {
    width: 9px;
    height: 9px;
    border-radius: 2px;
    flex: none;
  }
  .count {
    font-family: var(--font-mono);
    font-size: 64px;
    font-weight: 500;
    letter-spacing: -0.03em;
    font-variant-numeric: tabular-nums;
  }
  .line {
    font-size: 15px;
    opacity: 0.85;
  }
  .acts {
    margin-top: 14px;
    display: flex;
    align-items: center;
    gap: 16px;
  }
  .later {
    padding: 12px 18px;
    border: 1px solid oklch(0.97 0.004 80 / 0.3);
    border-radius: 12px;
    background: transparent;
    color: oklch(0.97 0.004 80 / 0.85);
    font-family: inherit;
    font-size: 14px;
    cursor: pointer;
  }
  .later:hover {
    background: oklch(0.97 0.004 80 / 0.1);
  }
  .done {
    padding: 12px 28px;
    border: none;
    border-radius: 12px;
    background: var(--accent);
    color: oklch(0.99 0.004 80);
    font-family: inherit;
    font-size: 15px;
    font-weight: 600;
    cursor: pointer;
    box-shadow: var(--inset-press);
  }
  .escape {
    position: absolute;
    bottom: 18px;
    right: 20px;
    font-size: 11px;
    opacity: 0.5;
  }
</style>
