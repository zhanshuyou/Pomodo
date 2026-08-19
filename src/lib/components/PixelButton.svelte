<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    variant?: "primary" | "secondary";
    onclick?: () => void;
    children: Snippet;
  }

  let { variant = "primary", onclick, children }: Props = $props();
</script>

<button class="btn btn--{variant}" type="button" {onclick}>{@render children()}</button>

<style>
  .btn {
    padding: 14px 20px;
    border-radius: 12px;
    font-size: 15px;
    cursor: pointer;
    border: none;
  }
  .btn--primary {
    background: var(--accent);
    color: var(--card);
    font-weight: 600;
    box-shadow: var(--inset-press);
  }
  .btn--primary:hover {
    filter: brightness(1.07);
  }
  .btn--secondary {
    background: var(--card);
    color: oklch(0.4 0.012 60);
    border: 1px solid var(--line);
  }
  .btn--secondary:hover {
    background: oklch(0.96 0.006 70);
  }
</style>
