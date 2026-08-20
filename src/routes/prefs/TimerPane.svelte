<script lang="ts">
  import { app } from "../../lib/state.svelte";

  const mins = (secs: number) => Math.round(secs / 60);
</script>

<div class="pane">
  <h3>时长</h3>
  <div class="rows">
    <div class="row"><span>专注</span><span class="val">{mins(app.settings.focusSecs)} min</span></div>
    <div class="row"><span>短休息</span><span class="val">{mins(app.settings.shortBreakSecs)} min</span></div>
    <div class="row"><span>长休息</span><span class="val">{mins(app.settings.longBreakSecs)} min</span></div>
    <div class="row"><span>一轮几个番茄</span><span class="val">{app.settings.roundsPerCycle}</span></div>
  </div>
  <p class="note">时长调整将在后续版本开放编辑。</p>
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
  .rows {
    display: flex;
    flex-direction: column;
    gap: 13px;
    max-width: 420px;
  }
  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
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
