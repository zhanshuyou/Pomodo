<script lang="ts">
  import { SOUND_TONES, type SoundSetting, type SoundTone, previewSound } from "../../lib/ipc";

  interface Props {
    label: string;
    value: SoundSetting;
    /** Shown after the setting text, e.g. 「（部分提醒不同）」. */
    note?: string;
    onchange: (sound: SoundSetting) => void;
  }
  let { label, value, note = "", onchange }: Props = $props();

  function pick(tone: SoundTone) {
    const sound = { ...value, tone };
    onchange(sound);
    void previewSound(sound);
  }

  function onVolume(raw: string) {
    const volume = Math.max(0, Math.min(100, Math.round(Number(raw))));
    if (Number.isFinite(volume)) onchange({ ...value, volume });
  }
</script>

<div class="row" role="group" aria-label={label}>
  <div class="head">
    <span class="label">{label}</span>
    {#if note}<span class="note">{note}</span>{/if}
  </div>
  <div class="tones">
    {#each SOUND_TONES as t (t.key)}
      <button
        class="tone"
        class:on={value.tone === t.key}
        type="button"
        aria-pressed={value.tone === t.key}
        onclick={() => pick(t.key)}
      >
        {t.label}
      </button>
    {/each}
    <input
      type="range"
      min="0"
      max="100"
      step="5"
      aria-label="{label}音量"
      disabled={value.tone === "none"}
      value={value.volume}
      onchange={(e) => onVolume(e.currentTarget.value)}
    />
    <span class="pct">{value.tone === "none" ? "—" : `${value.volume}%`}</span>
    <button
      class="tone"
      type="button"
      aria-label="试听{label}"
      disabled={value.tone === "none"}
      onclick={() => void previewSound(value)}
    >
      ▶ 试听
    </button>
  </div>
</div>

<style>
  .row {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .head {
    display: flex;
    align-items: baseline;
    gap: 8px;
    font-size: 13px;
  }
  .note {
    font-size: 12px;
    color: var(--faint);
  }
  .tones {
    display: flex;
    align-items: center;
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
  input[type="range"] {
    width: 120px;
    margin-left: 8px;
    accent-color: var(--accent);
  }
  .pct {
    min-width: 34px;
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--dim);
  }
</style>
