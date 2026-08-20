import { mount } from "svelte";
import type { Component } from "svelte";
import "../styles/fonts.css";
import "../styles/tokens.css";
import "../styles/base.css";

/** macOS floats its window controls over our title bar; other platforms do not. */
function markPlatform(): void {
  const ua = navigator.userAgent;
  if (/Mac|iPhone|iPad/.test(ua)) {
    document.documentElement.dataset.platform = "macos";
  }
}

export function mountApp(App: Component): unknown {
  markPlatform();
  return mount(App, { target: document.getElementById("app")! });
}
