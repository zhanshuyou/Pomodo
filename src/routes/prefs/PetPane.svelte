<script lang="ts">
  import PetFlagChips from "../../lib/components/PetFlagChips.svelte";
  import Toggle from "../../lib/components/Toggle.svelte";
  import { setPetVisible, setUseCustomPet } from "../../lib/ipc";
  import { app } from "../../lib/state.svelte";

  const hasCustom = $derived(
    !!(app.pet.custom.focus || app.pet.custom.rest || app.pet.custom.nag),
  );

</script>

<div class="pane">
  <h3>桌面宠物</h3>
  <div class="row">
    <span>在桌面上显示宠物</span>
    <Toggle
      checked={app.settings.petVisible}
      onchange={(v) => void setPetVisible(v)}
      label="显示桌面宠物"
    />
  </div>

  <h3>桌面行为</h3>
  <PetFlagChips />
  {#if hasCustom}
    <div class="row">
      <span>用自己导入的形象</span>
      <Toggle
        checked={app.pet.useCustom}
        onchange={(v) => void setUseCustomPet(v)}
        label="用自己导入的形象"
      />
    </div>
  {/if}
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
  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    max-width: 480px;
    font-size: 12.5px;
    color: oklch(0.42 0.012 60);
  }
  .note {
    margin: 0;
    font-size: 12px;
    color: var(--dim);
  }
</style>
