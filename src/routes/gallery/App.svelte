<script lang="ts">
  import Chip from "../../lib/components/Chip.svelte";
  import PetCanvas from "../../lib/components/PetCanvas.svelte";
  import PixelButton from "../../lib/components/PixelButton.svelte";
  import SectionHeading from "../../lib/components/SectionHeading.svelte";
  import SpeechBubble from "../../lib/components/SpeechBubble.svelte";
  import StatBar from "../../lib/components/StatBar.svelte";
  import TitleBar from "../../lib/components/TitleBar.svelte";
  import Toggle from "../../lib/components/Toggle.svelte";
  import { app } from "../../lib/state.svelte";
  import { LOCKED_BODY, PETS } from "../../lib/sprites";
  import { ACCENTS, type Accent, type Tone, tone } from "../../lib/theme";

  const TONES: [Tone, string][] = [
    ["professional", "克制专业"],
    ["gentle", "温和陪伴"],
    ["playful", "俏皮拟人"],
  ];

  let accent = $state<Accent>("terracotta");
  let activeTone = $state<Tone>("playful");
  let switched = $state(true);
  let picked = $state(0);

  $effect(() => {
    document.documentElement.dataset.accent = accent;
  });
</script>

<div class="page">
  <SectionHeading index="00" title="组件画廊" caption="仅用于开发时比对设计稿" />

  <div class="controls">
    {#each Object.keys(ACCENTS) as key (key)}
      <Chip
        selected={accent === key}
        dot={ACCENTS[key as Accent]}
        onclick={() => (accent = key as Accent)}
      >
        {key}
      </Chip>
    {/each}
    {#each TONES as [key, label] (key)}
      <Chip selected={activeTone === key} onclick={() => (activeTone = key)}>
        {label}
      </Chip>
    {/each}
  </div>

  <p class="tagline">
    {tone(
      activeTone,
      "菜单栏与主窗口双入口的番茄计时器，含可自定义的身体提醒与桌面宠物。",
      "一个陪你专注的番茄钟：它记得提醒你站立喝水，也记得在你完成时替你高兴。",
      "它负责计时、催你喝水、盯你站起来，并在你摸鱼时用眼神谴责你。",
    )}
  </p>

  <div class="pets">
    {#each PETS as pet (pet.id)}
      <button class="petcard" class:sel={picked === pet.id} onclick={() => (picked = pet.id)}>
        <PetCanvas
          map={pet.map}
          body={app.pet.lifetimePomodoros >= (app.pet.unlockAt[pet.id] ?? Infinity)
            ? pet.body
            : LOCKED_BODY}
          scale={4}
          alt={pet.name}
        />
        <span>{pet.name}</span>
      </button>
    {/each}
  </div>

  <div class="row">
    <PetCanvas map={PETS[picked].map} body={PETS[picked].body} scale={8} anim="bob" />
    <PetCanvas map={PETS[picked].map} body={PETS[picked].body} scale={4} anim="hop" />
    <PetCanvas map={PETS[picked].map} body={PETS[picked].body} scale={3} anim="sway" />
    <PetCanvas map={PETS[picked].map} body={PETS[picked].body} scale={9} />
  </div>

  <div class="row">
    <PixelButton>让它歇会儿</PixelButton>
    <PixelButton variant="secondary">跳过</PixelButton>
    <Toggle checked={switched} onchange={(v) => (switched = v)} label="示例开关" />
  </div>

  <SpeechBubble maxWidth={340}>还有 12 分钟，我盯着你呢</SpeechBubble>

  <div class="bars">
    <StatBar name="喝水" value="6 / 8 杯" pct={75} color="oklch(0.66 0.09 195)" />
    <StatBar name="站立" value="4 / 6 次" pct={66} color="oklch(0.63 0.13 40)" />
    <StatBar name="久坐最长" value="68 分钟" pct={76} color="oklch(0.7 0.12 60)" />
  </div>

  <div class="window">
    <TitleBar title="Pomodo" />
  </div>
</div>

<style>
  .page {
    padding: 40px;
    display: flex;
    flex-direction: column;
    gap: 28px;
    max-width: 900px;
  }
  .controls,
  .row {
    display: flex;
    gap: 10px;
    align-items: center;
    flex-wrap: wrap;
  }
  .tagline {
    margin: 0;
    color: var(--dim);
    font-size: 17px;
    line-height: 1.6;
  }
  .pets {
    display: grid;
    grid-template-columns: repeat(6, 1fr);
    gap: 10px;
  }
  .petcard {
    padding: 18px 8px 11px;
    border: 1.5px solid var(--line);
    border-radius: 12px;
    background: var(--card);
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
    cursor: pointer;
    font-family: var(--font-pixel);
    font-size: 11px;
  }
  .petcard.sel {
    border-color: var(--accent);
    background: oklch(0.975 0.008 70);
  }
  .bars {
    display: flex;
    flex-direction: column;
    gap: 12px;
    max-width: 320px;
  }
  .window {
    border: 1px solid var(--line);
    border-radius: var(--radius-window);
    overflow: hidden;
    background: var(--card);
  }
</style>
