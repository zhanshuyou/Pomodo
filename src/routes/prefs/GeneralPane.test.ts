import { flushSync, mount, unmount } from "svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import GeneralPane from "./GeneralPane.svelte";

const ipc = vi.hoisted(() => ({
  autostartEnabled: vi.fn(async () => true),
  setAutostart: vi.fn(async (_v: boolean) => {}),
}));
vi.mock("../../lib/ipc", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/ipc")>()),
  ...ipc,
}));

describe("GeneralPane 开机自动启动", () => {
  let host: HTMLDivElement;
  let component: Record<string, unknown>;

  beforeEach(async () => {
    ipc.setAutostart.mockClear();
    host = document.createElement("div");
    document.body.appendChild(host);
    component = mount(GeneralPane, { target: host });
    flushSync();
    await Promise.resolve();
    flushSync();
  });

  afterEach(() => {
    void unmount(component);
    host.remove();
  });

  const toggle = () =>
    host.querySelector<HTMLButtonElement>('[aria-label="开机自动启动"]')!;

  it("reads the current state from the OS rather than from settings", () => {
    expect(toggle().getAttribute("aria-checked")).toBe("true");
  });

  it("asks the plugin to flip it and re-reads the result", async () => {
    ipc.autostartEnabled.mockResolvedValueOnce(false);
    toggle().click();
    await vi.waitFor(() => expect(ipc.setAutostart).toHaveBeenCalledWith(false));
    await vi.waitFor(() => {
      flushSync();
      expect(toggle().getAttribute("aria-checked")).toBe("false");
    });
    expect(toggle().getAttribute("aria-checked")).toBe("false");
  });
});
