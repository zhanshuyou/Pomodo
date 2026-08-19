<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    tail?: "bottom-left" | "none";
    maxWidth?: number;
    children: Snippet;
  }

  let { tail = "bottom-left", maxWidth, children }: Props = $props();
</script>

<div
  class="bubble"
  class:tailed={tail === "bottom-left"}
  style:max-width={maxWidth ? `${maxWidth}px` : undefined}
>
  {@render children()}
</div>

<style>
  .bubble {
    padding: 11px 16px;
    border-radius: 13px;
    background: var(--card);
    box-shadow: var(--shadow-bubble);
    font-size: 14px;
    line-height: 1.45;
  }
  .bubble.tailed {
    border-bottom-left-radius: 4px;
  }
</style>
