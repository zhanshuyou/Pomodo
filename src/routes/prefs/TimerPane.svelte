<script lang="ts">
  import { setBodyGoals, setTimerDurations } from "../../lib/ipc";
  import { app } from "../../lib/state.svelte";

  const secsToMin = (secs: number) => Math.round(secs / 60);

  let focusMin = $state(secsToMin(app.settings.focusSecs));
  let shortMin = $state(secsToMin(app.settings.shortBreakSecs));
  let longMin = $state(secsToMin(app.settings.longBreakSecs));
  let rounds = $state(app.settings.roundsPerCycle);

  // 身体这边的账 — the sidebar bars' denominators. Same edit/commit dance,
  // but a separate flag: a half-typed water goal must not block a timer
  // change made in another window from showing up here, and vice versa.
  let waterGoal = $state(app.body.waterGoal);
  let standGoal = $state(app.body.standGoal);
  let sitGoalMins = $state(app.body.sitGoalMins);
  let bodyUnsaved = $state(false);

  $effect(() => {
    const b = app.body;
    if (bodyUnsaved) return;
    waterGoal = b.waterGoal;
    standGoal = b.standGoal;
    sitGoalMins = b.sitGoalMins;
  });

  function editBody() {
    bodyUnsaved = true;
  }

  function commitBody() {
    bodyUnsaved = false;
    void setBodyGoals({ waterGoal, standGoal, sitGoalMins });
  }

  /** True from the first keystroke on any field until its change commits. */
  let unsaved = $state(false);

  /**
   * Keep the fields in sync with the backend — needed for the fallback model
   * loading in before app.init() resolves, and to reflect the server's
   * clamped value after a save. Skipped while unsaved so an edit in progress
   * doesn't get overwritten by a settings change made elsewhere, e.g. in
   * another window.
   */
  $effect(() => {
    const s = app.settings;
    if (unsaved) return;
    focusMin = secsToMin(s.focusSecs);
    shortMin = secsToMin(s.shortBreakSecs);
    longMin = secsToMin(s.longBreakSecs);
    rounds = s.roundsPerCycle;
  });

  function edit() {
    unsaved = true;
  }

  function commit() {
    unsaved = false;
    void setTimerDurations({
      focusSecs: focusMin * 60,
      shortBreakSecs: shortMin * 60,
      longBreakSecs: longMin * 60,
      roundsPerCycle: rounds,
    });
  }
</script>

<div class="pane">
  <h3>时长</h3>
  <div class="rows">
    <div class="row">
      <span>专注</span>
      <span class="field">
        <input
          class="val"
          type="number"
          min="1"
          max="240"
          bind:value={focusMin}
          oninput={edit}
          onchange={commit}
        />
        <span class="unit">min</span>
      </span>
    </div>
    <div class="row">
      <span>短休息</span>
      <span class="field">
        <input
          class="val"
          type="number"
          min="1"
          max="240"
          bind:value={shortMin}
          oninput={edit}
          onchange={commit}
        />
        <span class="unit">min</span>
      </span>
    </div>
    <div class="row">
      <span>长休息</span>
      <span class="field">
        <input
          class="val"
          type="number"
          min="1"
          max="240"
          bind:value={longMin}
          oninput={edit}
          onchange={commit}
        />
        <span class="unit">min</span>
      </span>
    </div>
    <div class="row">
      <span>一轮几个番茄</span>
      <span class="field">
        <input
          class="val"
          type="number"
          min="1"
          max="12"
          bind:value={rounds}
          oninput={edit}
          onchange={commit}
        />
      </span>
    </div>
  </div>
  <p class="note">改动会在下一轮生效；进行中的这一轮按原时长走完。</p>

  <h3>身体这边的账</h3>
  <div class="rows">
    <div class="row">
      <span>每天喝水</span>
      <span class="field">
        <input
          class="val"
          type="number"
          min="1"
          max="30"
          aria-label="每天喝水目标"
          bind:value={waterGoal}
          oninput={editBody}
          onchange={commitBody}
        />
        <span class="unit">杯</span>
      </span>
    </div>
    <div class="row">
      <span>每天站起来</span>
      <span class="field">
        <input
          class="val"
          type="number"
          min="1"
          max="30"
          aria-label="每天站起目标"
          bind:value={standGoal}
          oninput={editBody}
          onchange={commitBody}
        />
        <span class="unit">次</span>
      </span>
    </div>
    <div class="row">
      <span>久坐上限</span>
      <span class="field">
        <input
          class="val"
          type="number"
          min="10"
          max="600"
          aria-label="久坐上限"
          bind:value={sitGoalMins}
          oninput={editBody}
          onchange={commitBody}
        />
        <span class="unit">min</span>
      </span>
    </div>
  </div>
  <p class="note">这三个数是专注页右下角三根条的满格；喝水和站起来分别由对应提醒的「做到了」计数。</p>
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
  .field {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .val {
    width: 56px;
    padding: 5px 10px;
    border: 1px solid var(--line);
    border-radius: 8px;
    background: var(--card);
    color: inherit;
    font-family: var(--font-mono);
    font-size: 12px;
    text-align: right;
  }
  .val:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 1px;
  }
  .unit {
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--dim);
  }
  .note {
    margin: 0;
    font-size: 12px;
    color: var(--dim);
  }
</style>
