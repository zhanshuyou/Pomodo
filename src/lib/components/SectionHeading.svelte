<script lang="ts">
  interface Props {
    index: string;
    title: string;
    caption?: string;
  }

  let { index, title, caption }: Props = $props();
</script>

<div class="heading">
  <span class="index">{index}</span>
  <h2>{title}</h2>
  {#if caption}<span class="caption">{caption}</span>{/if}
</div>

<style>
  .heading {
    display: flex;
    align-items: baseline;
    gap: 16px;
  }
  .index {
    font-family: var(--font-pixel);
    font-size: 12px;
    letter-spacing: 0.14em;
    color: var(--accent);
  }
  h2 {
    margin: 0;
    font-size: 22px;
    font-weight: 600;
  }
  .caption {
    font-size: 14px;
    color: var(--dim);
  }
</style>
