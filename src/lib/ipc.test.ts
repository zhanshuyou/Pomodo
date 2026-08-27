import { describe, expect, it } from "vitest";

import {
  IS_TAURI,
  onBubbleShow,
  onChanged,
  onMiniNudge,
  onOverlayShow,
  onPetNudge,
  onPetState,
  onPhase,
  onReminderFire,
  onTick,
} from "./ipc";

/**
 * Every window mounts its listeners in onMount, including the ones vitest and
 * gallery.html render outside Tauri. Without a bridge to subscribe to, that has
 * to be a no-op rather than a rejection nobody is positioned to catch.
 */
describe("event subscriptions outside Tauri", () => {
  it("runs these tests outside Tauri, which is what makes them meaningful", () => {
    expect(IS_TAURI).toBe(false);
  });

  const subscriptions = {
    onTick,
    onPhase,
    onChanged,
    onReminderFire,
    onPetNudge,
    onPetState,
    onMiniNudge,
    onBubbleShow,
    onOverlayShow,
  };

  for (const [name, subscribe] of Object.entries(subscriptions)) {
    it(`${name} resolves to a callable unlisten instead of rejecting`, async () => {
      const unlisten = await subscribe(() => {});
      expect(typeof unlisten).toBe("function");
      expect(() => unlisten()).not.toThrow();
    });
  }
});
