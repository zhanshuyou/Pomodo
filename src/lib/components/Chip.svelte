<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    selected?: boolean;
    dot?: string;
    onclick?: () => void;
    children: Snippet;
  }

  let { selected = false, dot, onclick, children }: Props = $props();
</script>

<button class="chip" class:selected type="button" {onclick}>
  {#if dot}<span class="dot" style:background={dot}></span>{/if}
  {@render children()}
</button>

<style>
  .chip {
    display: flex;
    align-items: center;
    gap: 7px;
    padding: 7px 11px;
    border: 1px solid var(--line);
    border-radius: var(--radius-chip);
    background: var(--card);
    color: var(--dim);
    font-size: 12.5px;
    cursor: pointer;
  }
  .chip:hover {
    background: oklch(0.96 0.006 70);
  }
  .chip.selected {
    border-color: var(--accent);
    background: oklch(0.975 0.008 70);
    color: var(--ink);
  }
  .dot {
    width: 7px;
    height: 7px;
    flex: none;
  }
</style>
