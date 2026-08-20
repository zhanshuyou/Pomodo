<script lang="ts">
  import { app } from "../../lib/state.svelte";

  // Every reminder carries its own sound; they all start on the design's default.
  const sound = $derived(app.reminders[0]?.rules.sound ?? "木鱼 · 30%");
</script>

<div class="pane">
  <h3>提示音</h3>
  <div class="row"><span>默认提示音</span><span class="val">{sound}</span></div>
  <p class="note">声音库将在后续版本开放。</p>
</div>

<style>
  .pane {
    flex: 1;
    padding: 22px 26px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
  h3 {
    margin: 0;
    font-size: 13.5px;
    font-weight: 600;
  }
  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    max-width: 420px;
    font-size: 12.5px;
    color: oklch(0.42 0.012 60);
  }
  .val {
    padding: 5px 10px;
    border: 1px solid var(--line);
    border-radius: 8px;
    background: var(--card);
    font-family: var(--font-mono);
    font-size: 12px;
  }
  .note {
    margin: 0;
    font-size: 12px;
    color: var(--dim);
  }
</style>
