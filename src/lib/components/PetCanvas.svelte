<script lang="ts">
  import { SPRITE_SIZE, rasterize } from "../sprites";

  interface Props {
    map: readonly string[];
    body: string;
    scale?: number;
    anim?: "none" | "bob" | "hop" | "sway";
    alt?: string;
  }

  let { map, body, scale = 8, anim = "none", alt = "" }: Props = $props();

  let canvas = $state<HTMLCanvasElement | null>(null);

  const size = $derived(SPRITE_SIZE * scale);

  // Redraw only when the pixels or the size actually change — never per frame.
  // The bob/hop/sway motion is a CSS transform on the element, not a repaint.
  $effect(() => {
    const el = canvas;
    if (!el) return;
    const buf = rasterize(map, body);
    const px = size;

    const off = document.createElement("canvas");
    off.width = SPRITE_SIZE;
    off.height = SPRITE_SIZE;
    const octx = off.getContext("2d");
    if (!octx) return;
    octx.putImageData(new ImageData(buf, SPRITE_SIZE, SPRITE_SIZE), 0, 0);

    const ctx = el.getContext("2d");
    if (!ctx) return;
    ctx.clearRect(0, 0, px, px);
    ctx.imageSmoothingEnabled = false;
    ctx.drawImage(off, 0, 0, px, px);
  });
</script>

<canvas
  bind:this={canvas}
  width={size}
  height={size}
  style:width="{size}px"
  style:height="{size}px"
  class="pet pet--{anim}"
  aria-label={alt}
></canvas>

<style>
  .pet {
    display: block;
    image-rendering: pixelated;
  }
  .pet--bob {
    animation: momo-bob 4.2s ease-in-out infinite;
  }
  .pet--hop {
    animation: momo-hop 1.6s ease-in-out infinite;
  }
  .pet--sway {
    animation: momo-sway 3s ease-in-out infinite;
  }
</style>
