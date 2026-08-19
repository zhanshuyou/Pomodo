import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { svelte } from "@sveltejs/vite-plugin-svelte";
import { defineConfig } from "vite";

const host = process.env.TAURI_DEV_HOST;
const root = fileURLToPath(new URL(".", import.meta.url));

// https://vite.dev/config/
export default defineConfig(async () => ({
  plugins: [svelte()],

  // One HTML entry per window. `gallery` is a development-only surface for
  // eyeballing components against the artboards; it ships but is never opened
  // by the app itself.
  build: {
    rollupOptions: {
      input: {
        main: resolve(root, "index.html"),
        prefs: resolve(root, "prefs.html"),
        tray: resolve(root, "tray.html"),
        pet: resolve(root, "pet.html"),
        overlay: resolve(root, "overlay.html"),
        gallery: resolve(root, "gallery.html"),
      },
    },
  },

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell Vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
}));
