<script lang="ts">
  import { type SoundSetting, setAllSounds, setPhaseSound } from "../../lib/ipc";
  import { app } from "../../lib/state.svelte";
  import SoundRow from "./SoundRow.svelte";

  // Every reminder carries its own sound. This pane edits them all at once
  // and shows the first one's as "the" setting; per-reminder tweaks live in
  // the reminder editor's 声音 row.
  const reminderSound = $derived<SoundSetting | null>(app.reminders[0]?.rules.sound ?? null);
  const mixed = $derived(
    reminderSound !== null &&
      app.reminders.some(
        (r) =>
          r.rules.sound.tone !== reminderSound.tone ||
          r.rules.sound.volume !== reminderSound.volume,
      ),
  );
</script>

<div class="pane">
  <section>
    <h3>计时</h3>
    <SoundRow
      label="专注结束"
      value={app.settings.phaseSounds.focusEnd}
      onchange={(s) => void setPhaseSound("focusEnd", s)}
    />
    <SoundRow
      label="休息结束"
      value={app.settings.phaseSounds.breakEnd}
      onchange={(s) => void setPhaseSound("breakEnd", s)}
    />
    <p class="note">只有自然走完的一轮才会响，「跳过」不出声。</p>
  </section>

  <section>
    <h3>提醒</h3>
    {#if reminderSound}
      <SoundRow
        label="所有提醒"
        value={reminderSound}
        note={mixed ? "（部分提醒不同）" : ""}
        onchange={(s) => void setAllSounds(s)}
      />
    {:else}
      <p class="empty">还没有提醒。在「提醒」里加一条，它的提示音就会出现在这里。</p>
    {/if}
    <p class="note">提示音由 Pomodo 自己合成，不依赖系统音效；单条提醒可在「提醒」里单独改。</p>
  </section>
</div>

<style>
  .pane {
    flex: 1;
    padding: 22px 26px;
    display: flex;
    flex-direction: column;
    gap: 24px;
  }
  section {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }
  h3 {
    margin: 0;
    font-size: 13.5px;
    font-weight: 600;
  }
  .note,
  .empty {
    margin: 0;
    font-size: 12px;
    color: var(--faint);
  }
  .empty {
    color: var(--dim);
  }
</style>
