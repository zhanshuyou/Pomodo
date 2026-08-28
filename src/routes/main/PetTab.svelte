<script lang="ts">
  import { onMount } from "svelte";
  import Chip from "../../lib/components/Chip.svelte";
  import Pet from "../../lib/components/Pet.svelte";
  import PetCanvas from "../../lib/components/PetCanvas.svelte";
  import {
    PET_IMAGE_EXTENSIONS,
    type PetFlags,
    type PetSlot,
    clearCustomPet,
    importCustomPet,
    onFileDrop,
    petImageSrc,
    pickPetImage,
    selectPet,
    setPetFlag,
    setUseCustomPet,
  } from "../../lib/ipc";
  import { LOCKED_BODY, PETS } from "../../lib/sprites";
  import { app } from "../../lib/state.svelte";

  const pet = $derived(PETS[app.pet.selected] ?? PETS[0]);
  // Level, stage and the unlock table all come from Rust (core/pet.rs).
  const unlockAt = (id: number) => app.pet.unlockAt[id] ?? Number.POSITIVE_INFINITY;
  const unlocked = (id: number) => app.pet.lifetimePomodoros >= unlockAt(id);

  let selectError = $state("");

  async function pick(id: number) {
    selectError = "";
    const ok = await selectPet(id);
    if (!ok) selectError = `还没解锁，再攒 ${unlockAt(id) - app.pet.lifetimePomodoros} 个番茄`;
  }

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
  /** A file is being dragged over the window: light the drop slot up. */
  let dragOver = $state(false);
  /** Which of the three states a dropped file (or the slot's click) lands in. */
  let dropSlot = $state<PetSlot>("focus");

  /**
   * Handle one OS drag-and-drop event. Exported so the test can feed events
   * directly instead of standing up Tauri's webview listener.
   */
  export function receiveDrop(e: import("../../lib/ipc").FileDropEvent): void {
    if (e.type === "enter" || e.type === "over") {
      dragOver = true;
      return;
    }
    dragOver = false;
    if (e.type !== "drop") return;
    error = "";
    const [path] = e.paths;
    if (!path || e.paths.length !== 1) {
      error = "一次拖一张就好";
      return;
    }
    const ext = path.split(".").pop()?.toLowerCase() ?? "";
    if (!(PET_IMAGE_EXTENSIONS as readonly string[]).includes(ext)) {
      error = `不支持的图片格式：${ext || "（无扩展名）"}，请拖入 PNG / GIF / APNG / WebP`;
      return;
    }
    importCustomPet(dropSlot, path).catch((err) => (error = String(err)));
  }

  onMount(() => {
    const un = onFileDrop(receiveDrop);
    return () => void un.then((f) => f());
  });

  /** Clearing needs a second click; window.confirm() is unavailable in WKWebView. */
  let pendingClear = $state<PetSlot | null>(null);
  let pendingTimer: ReturnType<typeof setTimeout> | undefined;

  const hasCustom = $derived(
    !!(app.pet.custom.focus || app.pet.custom.rest || app.pet.custom.nag),
  );

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

  function onSlotChip(slot: PetSlot) {
    clearTimeout(pendingTimer);
    if (!app.pet.custom[slot]) {
      pendingClear = null;
      void importSlot(slot);
      return;
    }
    if (pendingClear === slot) {
      pendingClear = null;
      void clearCustomPet(slot);
      return;
    }
    pendingClear = slot;
    pendingTimer = setTimeout(() => (pendingClear = null), 3000);
  }
</script>

<div class="pettab">
  <aside class="hero">
    <Pet scale={9} anim="bob" slot="focus" alt={pet.name} />
    <div class="heroinfo">
      <div class="heroname">
        <span class="pname">{pet.name}</span>
        <span class="plevel">Lv.{app.pet.level} · {app.pet.stage}</span>
      </div>
      <div class="track"><div class="fill" style:width="{app.pet.progressPct}%"></div></div>
      <span class="hint">再专注 {app.pet.toNextLevel} 个番茄升到 Lv.{app.pet.level + 1}</span>
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
            title={open ? p.name : `专注满 ${unlockAt(p.id)} 个番茄解锁`}
            onclick={() => void pick(p.id)}
          >
            <PetCanvas
              map={p.map}
              body={open ? p.body : LOCKED_BODY}
              scale={4}
              alt={p.name}
            />
            <span class="petname">{p.name}</span>
            {#if !open}
              <span class="unlock">{app.pet.lifetimePomodoros}/{unlockAt(p.id)}</span>
            {/if}
          </button>
        {/each}
      </div>
      {#if selectError}<span class="error">{selectError}</span>{/if}
    </section>

    <section class="custom">
      <div class="slotcol">
        <div class="slot" class:dragover={dragOver}>
          {#if app.pet.custom[dropSlot]}
            {@const label = SLOTS.find((s) => s.key === dropSlot)?.label ?? ""}
            <img src={petImageSrc(app.pet.custom[dropSlot] ?? "")} alt="自定义宠物（{label}）" />
          {:else}
            <button class="drop" type="button" onclick={() => void importSlot(dropSlot)}>
              拖入你的宠物 PNG / GIF
            </button>
          {/if}
        </div>
        <div class="droptarget" role="radiogroup" aria-label="拖入到哪个状态">
          {#each SLOTS as slot (slot.key)}
            <button
              class="target"
              class:on={dropSlot === slot.key}
              type="button"
              role="radio"
              aria-checked={dropSlot === slot.key}
              onclick={() => (dropSlot = slot.key)}
            >
              {slot.label}
            </button>
          {/each}
        </div>
      </div>

      <div class="customtext">
        <span class="sectitle">或者养你自己的</span>
        <span class="blurb">
          拖入 PNG / GIF / APNG 就成了你的宠物。可以给「专注」「休息」「催你站起来」三种状态各配一张，Pomodo
          自动换装；像素图会按整数倍放大，不糊。
        </span>
        <div class="chiprow">
          {#each SLOTS as slot (slot.key)}
            {@const path = app.pet.custom[slot.key]}
            <Chip selected={!!path} onclick={() => onSlotChip(slot.key)}>
              {#if path}
                <img class="thumb" src={petImageSrc(path)} alt="" />
              {/if}
              {pendingClear === slot.key ? `再点一次清除「${slot.label}」` : slot.label}
            </Chip>
          {/each}
          {#if hasCustom}
            <Chip
              selected={app.pet.useCustom}
              dot={app.pet.useCustom ? "var(--accent)" : "oklch(0.85 0.008 70)"}
              onclick={() => void setUseCustomPet(!app.pet.useCustom)}
            >
              用自己的形象
            </Chip>
          {/if}
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
  .slotcol {
    display: flex;
    flex-direction: column;
    gap: 8px;
    flex: none;
  }
  .droptarget {
    display: flex;
    gap: 4px;
    justify-content: center;
  }
  .target {
    padding: 3px 8px;
    border: 1px solid transparent;
    border-radius: var(--radius-chip);
    background: transparent;
    color: var(--faint);
    font-size: 11px;
    cursor: pointer;
  }
  .target.on {
    border-color: var(--accent);
    background: oklch(0.975 0.008 70);
    color: var(--ink);
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
  .slot.dragover {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }
  .slot.dragover .drop {
    border-color: var(--accent);
    color: var(--ink);
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
  .unlock {
    font-family: var(--font-mono);
    font-size: 10.5px;
    color: var(--faint);
  }
  .thumb {
    width: 20px;
    height: 20px;
    object-fit: contain;
    image-rendering: pixelated;
  }
</style>
