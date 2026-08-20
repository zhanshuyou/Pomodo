import { readFileSync } from "node:fs";

import { flushSync, mount, unmount } from "svelte";
import { afterEach, beforeEach, describe, expect, it } from "vitest";

import TitleBar from "./TitleBar.svelte";

describe("TitleBar", () => {
  let host: HTMLDivElement;
  let component: Record<string, unknown>;

  beforeEach(() => {
    host = document.createElement("div");
    document.body.appendChild(host);
    component = mount(TitleBar, { target: host, props: { title: "Pomodo" } });
    flushSync();
  });

  afterEach(() => {
    void unmount(component);
    host.remove();
  });

  it("does not draw traffic lights — macOS floats the real ones over the bar", () => {
    expect(host.querySelectorAll(".light")).toHaveLength(0);
    // Nothing circular and coloured should be masquerading as a window control.
    const circles = [...host.querySelectorAll("span")].filter(
      (e) => (e as HTMLElement).style.borderRadius === "50%",
    );
    expect(circles).toHaveLength(0);
  });

  it("renders the title and stays draggable", () => {
    expect(host.querySelector(".title")?.textContent).toBe("Pomodo");
    expect(host.querySelector("[data-tauri-drag-region]")).not.toBeNull();
  });

  it("reserves room for the native controls via the inset token", () => {
    // vite-plugin-svelte does not inject scoped styles under vitest, so read the
    // component source: this guards against someone hard-coding the padding again.
    // vitest runs from the project root.
    const source = readFileSync("src/lib/components/TitleBar.svelte", "utf8");
    expect(source).toContain("var(--titlebar-inset, 16px)");
    expect(host.querySelector(".bar")).not.toBeNull();
  });
});
