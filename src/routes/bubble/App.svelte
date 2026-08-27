<script lang="ts">
  import { onMount } from "svelte";
  import Pet from "../../lib/components/Pet.svelte";
  import { type FirePayload, ackReminder, hideBubble, onBubbleShow } from "../../lib/ipc";
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
      timer = setTimeout(() => void hideBubble(), AUTO_DISMISS_MS);
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
      class="ack"
      type="button"
      onclick={() => {
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
  .ack:hover {
    background: oklch(0.97 0.004 80 / 0.14);
  }
</style>
