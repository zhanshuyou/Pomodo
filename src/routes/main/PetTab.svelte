<script lang="ts">
  import Chip from "../../lib/components/Chip.svelte";
  import PetCanvas from "../../lib/components/PetCanvas.svelte";
  import {
    type PetFlags,
    type PetSlot,
    clearCustomPet,
    convertFileSrc,
    importCustomPet,
    pickPetImage,
    selectPet,
    setPetFlag,
  } from "../../lib/ipc";
  import { LOCKED_BODY, PETS } from "../../lib/sprites";
  import { app } from "../../lib/state.svelte";

  const POMODOROS_PER_LEVEL = 13;

  const pet = $derived(PETS[app.pet.selected] ?? PETS[0]);
  const level = $derived(Math.floor(app.pet.lifetimePomodoros / POMODOROS_PER_LEVEL) + 1);
  const progressPct = $derived(
    ((app.pet.lifetimePomodoros % POMODOROS_PER_LEVEL) / POMODOROS_PER_LEVEL) * 100,
  );
  const toNext = $derived(
    POMODOROS_PER_LEVEL - (app.pet.lifetimePomodoros % POMODOROS_PER_LEVEL),
  );

  const UNLOCK_AT: Record<number, number> = { 0: 0, 1: 0, 2: 0, 3: 0, 4: 150, 5: 300 };
  const unlocked = (id: number) =>
    app.pet.lifetimePomodoros >= (UNLOCK_AT[id] ?? Number.POSITIVE_INFINITY);

  const FLAGS: { key: keyof PetFlags; name: string }[] = [
    { key: "snapEdges", name: "贴边吸附" },
    { key: "clickInteract", name: "点击互动" },
    { key: "hideFullscreen", name: "全屏时隐藏" },
    { key: "sleepAnimation", name: "睡眠动画" },
  ];

  const SLOTS: { key: PetSlot; label: string }[] = [
    { key: "focus", label: "专注" },
    { key: "rest", label: "休息" },
    { key: "nag", label: "催你站起来" },
  ];

  let error = $state("");

  async function importSlot(slot: PetSlot) {
    error = "";
    const source = await pickPetImage();
    if (!source) return;
    try {
      await importCustomPet(slot, source);
    } catch (e) {
      error = String(e);
    }
  }
</script>

<div class="pettab">
  <aside class="hero">
    <PetCanvas map={pet.map} body={pet.body} scale={9} anim="bob" alt={pet.name} />
    <div class="heroinfo">
      <div class="heroname">
        <span class="pname">{pet.name}</span>
        <span class="plevel">Lv.{level} · 好奇期</span>
      </div>
      <div class="track"><div class="fill" style:width="{progressPct}%"></div></div>
      <span class="hint">再专注 {toNext} 个番茄升到 Lv.{level + 1}，解锁「披风」</span>
    </div>
  </aside>

  <div class="right">
    <section>
      <div class="sechead">
        <span class="sectitle">选一只</span>
        <span class="seccaption">灰色的还锁着，专注攒够就解锁</span>
      </div>
      <div class="grid">
        {#each PETS as p (p.id)}
          {@const open = unlocked(p.id)}
          <button
            class="petcard"
            class:sel={app.pet.selected === p.id}
            class:locked={!open}
            type="button"
            disabled={!open}
            onclick={() => void selectPet(p.id)}
          >
            <PetCanvas
              map={p.map}
              body={open ? p.body : LOCKED_BODY}
              scale={4}
              alt={p.name}
            />
            <span class="petname">{p.name}</span>
          </button>
        {/each}
      </div>
    </section>

    <section class="custom">
      <div class="slot">
        {#if app.pet.custom.focus}
          <img src={convertFileSrc(app.pet.custom.focus)} alt="自定义宠物" />
        {:else}
          <button class="drop" type="button" onclick={() => void importSlot("focus")}>
            拖入你的宠物 PNG / GIF
          </button>
        {/if}
      </div>

      <div class="customtext">
        <span class="sectitle">或者养你自己的</span>
        <span class="blurb">
          拖入 PNG / GIF / APNG 就成了你的宠物。可以给「专注」「休息」「催你站起来」三种状态各配一张，Momo
          自动换装；像素图会按整数倍放大，不糊。
        </span>
        <div class="chiprow">
          {#each SLOTS as slot (slot.key)}
            <Chip
              selected={!!app.pet.custom[slot.key]}
              onclick={() =>
                app.pet.custom[slot.key]
                  ? void clearCustomPet(slot.key)
                  : void importSlot(slot.key)}
            >
              {slot.label}
            </Chip>
          {/each}
        </div>
        {#if error}<span class="error">{error}</span>{/if}
      </div>
    </section>

    <section>
      <div class="chiprow">
        {#each FLAGS as flag (flag.key)}
          <Chip
            selected={app.settings.petFlags[flag.key]}
            dot={app.settings.petFlags[flag.key]
              ? "var(--accent)"
              : "oklch(0.85 0.008 70)"}
            onclick={() => void setPetFlag(flag.key, !app.settings.petFlags[flag.key])}
          >
            {flag.name}
          </Chip>
        {/each}
      </div>
    </section>
  </div>
</div>

<style>
  .pettab {
    flex: 1;
    padding: 30px 40px 38px;
    display: flex;
    gap: 36px;
    overflow-y: auto;
  }
  .hero {
    width: 300px;
    flex: none;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 18px;
    padding: 26px 20px;
    border: 1px solid oklch(0.9 0.008 70);
    border-radius: 14px;
    background: linear-gradient(180deg, oklch(0.975 0.012 75) 0%, oklch(0.99 0.004 80) 70%);
    align-self: flex-start;
  }
  .heroinfo {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    width: 100%;
  }
  .heroname {
    display: flex;
    align-items: baseline;
    gap: 8px;
  }
  .pname {
    font-family: var(--font-pixel);
    font-size: 18px;
  }
  .plevel {
    font-size: 12.5px;
    color: var(--dim);
  }
  .track {
    width: 100%;
    height: 8px;
    border-radius: 4px;
    background: var(--track);
    overflow: hidden;
  }
  .fill {
    height: 100%;
    background: var(--accent);
  }
  .hint {
    font-size: 12px;
    color: oklch(0.53 0.012 60);
    text-align: center;
  }
  .right {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 20px;
    min-width: 0;
  }
  section {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .sechead {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
  }
  .sectitle {
    font-size: 14px;
    font-weight: 600;
  }
  .seccaption {
    font-size: 12.5px;
    color: var(--dim);
  }
  .grid {
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
  }
  .petcard.sel {
    border-color: var(--accent);
    background: oklch(0.975 0.008 70);
  }
  .petcard.locked {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .petname {
    font-family: var(--font-pixel);
    font-size: 11px;
  }
  .custom {
    flex-direction: row;
    gap: 16px;
    align-items: stretch;
  }
  .slot {
    width: 148px;
    height: 148px;
    flex: none;
    border-radius: 12px;
    overflow: hidden;
    background: var(--surface-2);
  }
  .slot img {
    width: 100%;
    height: 100%;
    object-fit: contain;
    image-rendering: pixelated;
  }
  .drop {
    width: 100%;
    height: 100%;
    border: 1px dashed oklch(0.85 0.008 70);
    border-radius: 12px;
    background: transparent;
    color: var(--dim);
    font-size: 12.5px;
    cursor: pointer;
    padding: 12px;
  }
  .customtext {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 10px;
    justify-content: center;
  }
  .blurb {
    font-size: 13px;
    line-height: 1.55;
    color: oklch(0.52 0.012 60);
  }
  .chiprow {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }
  .error {
    font-size: 12px;
    color: oklch(0.55 0.15 25);
  }
</style>
