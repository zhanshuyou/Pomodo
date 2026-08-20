<script lang="ts">
  import Chip from "../../lib/components/Chip.svelte";
  import Toggle from "../../lib/components/Toggle.svelte";
  import { setAccent, setDeepWork, setTone } from "../../lib/ipc";
  import { app } from "../../lib/state.svelte";
  import { ACCENTS, type Accent, type Tone } from "../../lib/theme";

  const TONES: { key: Tone; label: string }[] = [
    { key: "professional", label: "克制专业" },
    { key: "gentle", label: "温和陪伴" },
    { key: "playful", label: "俏皮拟人" },
  ];
</script>

<div class="pane">
  <section>
    <h3>强调色</h3>
    <div class="chips">
      {#each Object.entries(ACCENTS) as [key, css] (key)}
        <Chip
          selected={app.settings.accent === key}
          dot={css}
          onclick={() => void setAccent(key as Accent)}
        >
          {key}
        </Chip>
      {/each}
    </div>
  </section>

  <section>
    <h3>说话的口气</h3>
    <div class="chips">
      {#each TONES as t (t.key)}
        <Chip selected={app.settings.tone === t.key} onclick={() => void setTone(t.key)}>
          {t.label}
        </Chip>
      {/each}
    </div>
    <p class="note">改口气会重写所有没被你编辑过的提醒文案。</p>
  </section>

  <section>
    <h3>深度工作</h3>
    <div class="row">
      <span>开启后所有提醒自动降到最轻那档</span>
      <Toggle checked={app.deepWork} onchange={(v) => void setDeepWork(v)} label="深度工作" />
    </div>
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
  h3 {
    margin: 0 0 10px;
    font-size: 13.5px;
    font-weight: 600;
  }
  .chips {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }
  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    max-width: 480px;
    font-size: 12.5px;
    color: oklch(0.42 0.012 60);
  }
  .note {
    margin: 8px 0 0;
    font-size: 12px;
    color: var(--dim);
  }
</style>
