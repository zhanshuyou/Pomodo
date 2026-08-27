<script lang="ts">
  import { onMount } from "svelte";
  import Pet from "../../lib/components/Pet.svelte";
  import {
    type FirePayload,
    ackReminder,
    hideBubble,
    SNOOZE_MINUTES,
    ignoreReminder,
    onBubbleShow,
    snoozeReminder,
  } from "../../lib/ipc";
  import { snoozeLabel } from "../../lib/copy";
  import { PETS } from "../../lib/sprites";
  import { app } from "../../lib/state.svelte";

  /** 右上角滑入，6 秒自动收起。 */
  const AUTO_DISMISS_MS = 6000;

  let fire = $state<FirePayload | null>(null);
  let timer: ReturnType<typeof setTimeout> | undefined;

  const pet = $derived(PETS[app.pet.selected] ?? PETS[0]);

  onMount(() => {
    void app.init();
    const un = onBubbleShow((payload) => {
      fire = payload;
      clearTimeout(timer);
      // Letting it slide off counts as ignoring it — that is what feeds the
      // 连续忽略 N 次 escalation. Pressing 好 acknowledges instead.
      timer = setTimeout(() => {
        void ignoreReminder(payload.id);
        void hideBubble();
      }, AUTO_DISMISS_MS);
    });
    return () => {
      clearTimeout(timer);
      void un.then((f) => f());
      app.dispose();
    };
  });
</script>

{#if fire}
  <div class="toast">
    <Pet scale={3} slot="nag" alt={pet.name} />
    <div class="text">
      <span class="title">{fire.name}</span>
      <span class="body">{fire.message}</span>
    </div>
    <button
      class="later"
      type="button"
      onclick={() => {
        clearTimeout(timer);
        if (fire) void snoozeReminder(fire.id);
        void hideBubble();
      }}
    >
      {snoozeLabel(app.tone, SNOOZE_MINUTES)}
    </button>
    <button
      class="ack"
      type="button"
      onclick={() => {
        clearTimeout(timer);
        if (fire) void ackReminder(fire.id);
        void hideBubble();
      }}
    >
      好
    </button>
  </div>
{/if}

<style>
  :global(html),
  :global(body) {
    background: transparent;
    overflow: hidden;
  }
  .toast {
    margin: 8px;
    padding: 15px;
    border-radius: 13px;
    background: oklch(0.31 0.025 258);
    color: oklch(0.97 0.004 80);
    display: flex;
    gap: 12px;
    align-items: center;
    box-shadow: 0 18px 40px -16px oklch(0.2 0.02 260 / 0.7);
    animation: momo-rise 0.35s ease both;
  }
  .text {
    display: flex;
    flex-direction: column;
    gap: 3px;
    flex: 1;
    min-width: 0;
  }
  .title {
    font-size: 13.5px;
    font-weight: 600;
  }
  .body {
    font-size: 12.5px;
    opacity: 0.8;
    line-height: 1.4;
  }
  .ack {
    border: 1px solid oklch(0.97 0.004 80 / 0.3);
    border-radius: 8px;
    background: transparent;
    color: inherit;
    font-family: inherit;
    font-size: 12.5px;
    padding: 6px 12px;
    cursor: pointer;
    flex: none;
  }
  .later {
    border: none;
    background: transparent;
    color: oklch(0.97 0.004 80 / 0.65);
    font-family: inherit;
    font-size: 12px;
    padding: 6px 4px;
    cursor: pointer;
    flex: none;
    text-decoration: underline;
    text-underline-offset: 2px;
  }
  .later:hover {
    color: oklch(0.97 0.004 80);
  }
  .ack:hover {
    background: oklch(0.97 0.004 80 / 0.14);
  }
</style>
