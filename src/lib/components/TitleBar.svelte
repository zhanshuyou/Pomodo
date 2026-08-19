<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    title: string;
    children?: Snippet;
  }

  let { title, children }: Props = $props();
</script>

<div class="bar" data-tauri-drag-region>
  <span class="light" style:background="oklch(0.72 0.15 25)"></span>
  <span class="light" style:background="oklch(0.82 0.13 85)"></span>
  <span class="light" style:background="oklch(0.78 0.14 145)"></span>
  <span class="title">{title}</span>
  {#if children}{@render children()}{/if}
</div>

<style>
  .bar {
    height: 46px;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 0 16px;
    border-bottom: 1px solid oklch(0.91 0.008 70);
    background: var(--surface-2);
    flex: none;
  }
  .light {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    flex: none;
  }
  .title {
    margin-left: 14px;
    font-size: 13px;
    font-weight: 600;
    color: oklch(0.42 0.012 60);
  }
</style>
