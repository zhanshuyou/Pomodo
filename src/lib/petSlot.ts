import type { Phase } from "./copy";
import type { CustomPet, PetMood, PetSlot } from "./ipc";

/**
 * Which of the three custom-sprite slots (专注 / 休息 / 催你站起来) the pet
 * should wear right now. Nagging wins, then anything that is not a running
 * focus phase counts as resting; a dozing pet is resting too.
 */
export function petSlotFor(mood: PetMood, phase: Phase): PetSlot {
  if (mood === "nagging") return "nag";
  if (mood === "break" || mood === "sleeping" || phase !== "focus") return "rest";
  return "focus";
}

/**
 * The image to show for a slot, falling back through the others so a user
 * who only imported one picture still sees it everywhere. `null` means
 * nothing is imported and the built-in sprite should be drawn.
 */
export function resolveSlot(custom: CustomPet, slot: PetSlot): string | null {
  return custom[slot] ?? custom.focus ?? custom.rest ?? custom.nag ?? null;
}

/** Largest integer factor that fits a `w`×`h` image inside a `box` square, never below 1. */
export function integerScale(w: number, h: number, box: number): number {
  return Math.max(1, Math.floor(box / Math.max(1, w, h)));
}
