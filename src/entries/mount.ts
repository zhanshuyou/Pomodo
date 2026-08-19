import { mount } from "svelte";
import type { Component } from "svelte";
import "../styles/fonts.css";
import "../styles/tokens.css";
import "../styles/base.css";

export function mountApp(App: Component): unknown {
  return mount(App, { target: document.getElementById("app")! });
}
