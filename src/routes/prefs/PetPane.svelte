<script lang="ts">
  import Chip from "../../lib/components/Chip.svelte";
  import { type PetFlags, setPetFlag } from "../../lib/ipc";
  import { app } from "../../lib/state.svelte";

  const FLAGS: { key: keyof PetFlags; name: string }[] = [
    { key: "snapEdges", name: "贴边吸附" },
    { key: "clickInteract", name: "点击互动" },
    { key: "hideFullscreen", name: "全屏时隐藏" },
    { key: "sleepAnimation", name: "睡眠动画" },
  ];
</script>

<div class="pane">
  <h3>桌面行为</h3>
  <div class="chips">
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
  <p class="note">宠物形象与自定义图片请在主窗口的「宠物」标签页设置。</p>
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
  .chips {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }
  .note {
    margin: 0;
    font-size: 12px;
    color: var(--dim);
  }
</style>
