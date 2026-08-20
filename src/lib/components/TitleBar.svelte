<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    title: string;
    children?: Snippet;
  }

  let { title, children }: Props = $props();
</script>

<!--
  The window uses macOS's Overlay title bar style, so the real traffic lights
  float over this bar — we must not draw our own. `--titlebar-inset` reserves
  room for them on macOS and collapses to normal padding elsewhere, where the
  platform draws its own decorations above us instead.
-->
<div class="bar" data-tauri-drag-region>
  <span class="title">{title}</span>
  {#if children}{@render children()}{/if}
</div>

<style>
  .bar {
    height: 46px;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 0 16px 0 var(--titlebar-inset, 16px);
    border-bottom: 1px solid oklch(0.91 0.008 70);
    background: var(--surface-2);
    flex: none;
  }
  .title {
    font-size: 13px;
    font-weight: 600;
    color: oklch(0.42 0.012 60);
  }
</style>
