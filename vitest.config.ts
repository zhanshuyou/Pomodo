import { svelte } from "@sveltejs/vite-plugin-svelte";
import { defineConfig } from "vitest/config";

export default defineConfig({
  plugins: [svelte()],
  // Without the browser condition, Node resolves Svelte's server build and
  // `mount()` throws lifecycle_function_unavailable inside component tests.
  resolve: {
    conditions: ["browser"],
  },
  test: {
    environment: "jsdom",
    include: ["src/**/*.test.ts"],
  },
});
