<script lang="ts">
  import {
    SOUND_TONES,
    type SoundSetting,
    type SoundTone,
    previewSound,
    setAllSounds,
    soundLabel,
  } from "../../lib/ipc";
  import { app } from "../../lib/state.svelte";

  // Every reminder carries its own sound. This pane edits them all at once
  // and shows the first one's as "the" setting; per-reminder tweaks live in
  // the reminder editor's 声音 row.
  const current = $derived<SoundSetting>(
    app.reminders[0]?.rules.sound ?? { tone: "woodblock", volume: 30 },
  );
  const mixed = $derived(
    app.reminders.some(
      (r) => r.rules.sound.tone !== current.tone || r.rules.sound.volume !== current.volume,
    ),
  );

  function apply(sound: SoundSetting) {
    void setAllSounds(sound);
  }

  function pick(tone: SoundTone) {
    const sound = { ...current, tone };
    apply(sound);
    void previewSound(sound);
  }

  function onVolume(value: string) {
    const volume = Math.max(0, Math.min(100, Math.round(Number(value))));
    if (Number.isFinite(volume)) apply({ ...current, volume });
  }
</script>

<div class="pane">
  <h3>提示音</h3>
  <div class="row">
    <span>所有提醒</span>
    <span class="val">{soundLabel(current)}{mixed ? "（部分提醒不同）" : ""}</span>
  </div>

  <div class="tones">
    {#each SOUND_TONES as t (t.key)}
      <button
        class="tone"
        class:on={current.tone === t.key}
        type="button"
        aria-pressed={current.tone === t.key}
        onclick={() => pick(t.key)}
      >
        {t.label}
      </button>
    {/each}
  </div>

  <div class="row">
    <span>音量</span>
    <span class="volrow">
      <input
        type="range"
        min="0"
        max="100"
        step="5"
        aria-label="音量"
        disabled={current.tone === "none"}
        value={current.volume}
        onchange={(e) => onVolume(e.currentTarget.value)}
      />
      <button
        class="tone"
        type="button"
        aria-label="试听"
        disabled={current.tone === "none"}
        onclick={() => void previewSound(current)}
      >
        ▶ 试听
      </button>
    </span>
  </div>
  <p class="note">提示音由 Pomodo 自己合成，不依赖系统音效；单条提醒可在「提醒」里单独改。</p>
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
    font-size: 13px;
  }
  .val {
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--dim);
  }
  .tones {
    display: flex;
    gap: 8px;
  }
  .tone {
    padding: 6px 12px;
    border: 1px solid var(--line);
    border-radius: var(--radius-chip);
    background: var(--card);
    color: var(--dim);
    font-size: 12.5px;
    cursor: pointer;
  }
  .tone.on {
    border-color: var(--accent);
    background: oklch(0.975 0.008 70);
    color: var(--ink);
  }
  .tone:disabled {
    opacity: 0.4;
    cursor: default;
  }
  .volrow {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  input[type="range"] {
    width: 160px;
    accent-color: var(--accent);
  }
  .note {
    margin: 0;
    font-size: 12px;
    color: var(--faint);
  }
</style>
