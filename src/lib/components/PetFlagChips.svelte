<script lang="ts">
  import Chip from "./Chip.svelte";
  import { type PetFlags, setPetFlag } from "../ipc";
  import { app } from "../state.svelte";

  /** 贴边吸附 / 点击互动 / 全屏时隐藏 / 睡眠动画 — shown in both 宠物 tab and 设置 · 宠物. */
  const FLAGS: { key: keyof PetFlags; name: string }[] = [
    { key: "snapEdges", name: "贴边吸附" },
    { key: "clickInteract", name: "点击互动" },
    { key: "hideFullscreen", name: "全屏时隐藏" },
    { key: "sleepAnimation", name: "睡眠动画" },
  ];
</script>

<div class="chiprow">
  {#each FLAGS as flag (flag.key)}
    <Chip
      selected={app.settings.petFlags[flag.key]}
      dot={app.settings.petFlags[flag.key] ? "var(--accent)" : "oklch(0.85 0.008 70)"}
      onclick={() => void setPetFlag(flag.key, !app.settings.petFlags[flag.key])}
    >
      {flag.name}
    </Chip>
  {/each}
</div>

<style>
  .chiprow {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }
</style>
