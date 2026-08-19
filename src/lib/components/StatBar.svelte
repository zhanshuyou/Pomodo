<script lang="ts">
  interface Props {
    name: string;
    value: string;
    pct: number;
    color: string;
  }

  let { name, value, pct, color }: Props = $props();
</script>

<div class="stat">
  <div class="row">
    <span>{name}</span>
    <span class="value">{value}</span>
  </div>
  <div class="track">
    <div
      class="fill"
      style:width="{Math.max(0, Math.min(100, pct))}%"
      style:background={color}
    ></div>
  </div>
</div>

<style>
  .stat {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .row {
    display: flex;
    justify-content: space-between;
    font-size: 12.5px;
  }
  .value {
    font-family: var(--font-mono);
    color: var(--dim);
  }
  .track {
    height: 6px;
    border-radius: 3px;
    background: var(--line-soft);
    overflow: hidden;
  }
  .fill {
    height: 100%;
  }
</style>
