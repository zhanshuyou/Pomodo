<script lang="ts">
  import { type PetSlot, petImageSrc } from "../ipc";
  import { integerScale, petSlotFor, resolveSlot } from "../petSlot";
  import { PETS, SPRITE_SIZE } from "../sprites";
  import { app } from "../state.svelte";
  import PetCanvas from "./PetCanvas.svelte";

  interface Props {
    scale?: number;
    anim?: "none" | "bob" | "hop" | "sway" | "sleep";
    /** Which custom sprite to wear; `auto` follows the pet's mood. */
    slot?: PetSlot | "auto";
    alt?: string;
  }

  let { scale = 8, anim = "none", slot = "auto", alt }: Props = $props();

  const builtin = $derived(PETS[app.pet.selected] ?? PETS[0]);
  const box = $derived(SPRITE_SIZE * scale);
  const slotKey = $derived(
    slot === "auto" ? petSlotFor(app.petMood, app.timer.phase) : slot,
  );
  const path = $derived(
    app.pet.useCustom ? resolveSlot(app.pet.custom, slotKey) : null,
  );

  // Custom pictures are upscaled by whole multiples only — never smoothed —
  // so a 16px sprite at scale 8 is exactly as crisp as the built-in canvas.
  let natural = $state<{ w: number; h: number } | null>(null);
  const factor = $derived(natural ? integerScale(natural.w, natural.h, box) : 1);

  function onLoad(event: Event) {
    const img = event.currentTarget as HTMLImageElement;
    natural = { w: img.naturalWidth, h: img.naturalHeight };
  }
</script>

{#if path}
  <div
    class="frame frame--{anim}"
    style:width="{box}px"
    style:height="{box}px"
    role="img"
    aria-label={alt ?? builtin.name}
  >
    <img
      src={petImageSrc(path)}
      alt=""
      onload={onLoad}
      style:width={natural ? `${natural.w * factor}px` : undefined}
      style:height={natural ? `${natural.h * factor}px` : undefined}
    />
  </div>
{:else}
  <PetCanvas
    map={builtin.map}
    body={builtin.body}
    {scale}
    {anim}
    alt={alt ?? builtin.name}
  />
{/if}

<style>
  .frame {
    display: flex;
    align-items: flex-end;
    justify-content: center;
    overflow: hidden;
  }
  img {
    image-rendering: pixelated;
    max-width: 100%;
    max-height: 100%;
  }
  .frame--bob {
    animation: momo-bob 4.2s ease-in-out infinite;
  }
  .frame--hop {
    animation: momo-hop 1.6s ease-in-out infinite;
  }
  .frame--sway {
    animation: momo-sway 3s ease-in-out infinite;
  }
  .frame--sleep {
    animation: momo-breathe 5s ease-in-out infinite;
  }
</style>
