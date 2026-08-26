# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

Package manager is **pnpm** (pinned via `packageManager` in package.json). Do not use npm/yarn — the README's `npm` examples are stale.

```sh
pnpm install
pnpm tauri dev              # run the desktop app (starts Vite on :1420 itself)
pnpm tauri build            # installers into src-tauri/target/release/bundle
pnpm tauri build --no-bundle  # what CI runs: release binary, no installer

pnpm check                  # svelte-check + tsc -p tsconfig.node.json
pnpm test                   # vitest run
pnpm test:watch
pnpm dev                    # frontend only, in a plain browser (see gallery.html)
```

Single frontend test: `pnpm vitest run src/lib/format.test.ts` (add `-t "name"` for one case).

Rust (all from `src-tauri/`): `cargo test`, `cargo fmt --all --check`, `cargo clippy --all-targets --all-features -- -D warnings`. CI enforces all three plus a three-OS Tauri build, so run fmt and clippy before pushing.

## Architecture

Pomodo is a Chinese-language macOS-first Pomodoro timer whose timer *is* a pixel-art desktop pet. Tauri 2 + Svelte 5 (runes) + TypeScript + Vite.

**Rust owns all state; webviews are pure views.** This is the load-bearing decision. `AppState` (`src-tauri/src/state.rs`) wraps a single `Model` behind a mutex; a background thread in `lib.rs` ticks once per second and calls `AppState::tick`. Elapsed time comes from a monotonic `Instant` delta, not from counting iterations, so a machine sleeping for an hour replays every crossed phase on wake. Frontend never computes timer or reminder state — it reads events and re-invokes `list_model`.

**Data flow.** Frontend calls a `#[tauri::command]` in `commands.rs` → command mutates via `state.with(|m| …)` → emits an event → every window's `AppStore` (`src/lib/state.svelte.ts`) refreshes. Events are declared in `src-tauri/src/events.rs`: `timer:tick` (per-second, patched in place), `timer:phase`, `model:changed` (coarse section hint; the frontend just refetches), `reminder:fire`. `src/lib/ipc.ts` is the single typed boundary — every `invoke` and `listen` lives there, with TS interfaces mirroring the serde `camelCase` shapes.

**Multi-window.** Seven Vite HTML entries, one per window, all registered in `vite.config.ts` `rollupOptions.input` and mirrored by `src/entries/*.ts` → `src/routes/<name>/App.svelte`. Adding a window means touching all of: a root `*.html`, an entry in `vite.config.ts`, `src/entries/`, `src/routes/`, `src-tauri/tauri.conf.json` `app.windows`, and the `windows` list in `src-tauri/capabilities/default.json` — a missing capability entry is the usual cause of a window whose IPC silently does nothing. Overlay windows are the exception: created at runtime as `overlay-0`, `overlay-1`, … (one per monitor) by `windows.rs`, matched by the `overlay-*` glob in the capability file.

Windows: `main` (专注/统计/宠物 tabs), `prefs` (设置), `tray` (menu-bar popover), `pet` (free-floating desktop pet), `bubble`, `mini` (迷你模式 bar, ⌘⌥M), `overlay-*` (fullscreen reminder). `gallery.html` is dev-only — it renders every component against the design artboards and is never opened by the app.

**Reminder engine** (`src-tauri/src/core/reminder.rs`). Each reminder ticks toward a `Schedule` (`every` / `dailyAt`) and fires at one of three `Intensity` tiers — `bubble`, `pet`, `fullscreen` — which decides *which window renders it*, dispatched in `AppState::run_reminders`. `Rules.during_focus` (`defer`/`silence`/`interrupt`) and `silence_in_meeting` gate firing; deferred reminders are released when a focus round ends. In 迷你模式 bubble/pet-tier fires are redirected into the bar (`mini:nudge`) instead of opening a window — fullscreen is deliberately never downgraded.

**Platform layer** (`src-tauri/src/platform/`). macOS-only window behaviour (desktop layer, overlay layer, click-through, fullscreen-app detection, microphone-in-use) sits behind the `PlatformWindows` trait with a no-op `fallback` impl, so Linux and Windows keep compiling and CI stays green. Never call AppKit directly from `windows.rs` or `state.rs`.

**Pure core modules** (`src-tauri/src/core/`): `timer`, `reminder`, `stats`, `task`, `pet`, `desk` (screen clamping / 贴边吸附 snapping). These are free of Tauri types and carry the Rust unit tests; `state.rs` and `windows.rs` are the wiring around them.

**Persistence** (`store.rs`). One `state.json` under the app data dir, written atomically (temp file + rename), debounced to at most once per second and flushed on exit. `SCHEMA_VERSION` is checked on load; an unknown version is backed up to `state.json.bak` and replaced with a fresh model, and every new `Model` field needs a `#[serde(default)]` so older files still load — there is a regression test for this in `state.rs`.

## Conventions

- **The design spec is the source of truth**: `docs/superpowers/specs/2026-08-19-momo-design.md`, implemented by the seven plans in `docs/superpowers/plans/`. Colours, copy, motion and screen layouts are specified there; check it before inventing a value.
- **All colours are `oklch()`**, defined as CSS custom properties in `src/styles/tokens.css`. `--accent` is user-selectable via `:root[data-accent="…"]`. Derived colours use relative-colour CSS (`oklch(from var(--accent) …)`) where the browser can resolve it, and `src/lib/sprites.ts` reimplements the conversion in JS for canvas rendering.
- **UI copy is Chinese and tone-aware.** Every user-facing string goes through `tone(t, professional, gentle, playful)` in `src/lib/theme.ts`; frontend strings live in `src/lib/copy.ts`, reminder strings in `src-tauri/src/core/reminder_copy.rs`. Changing tone re-tones every reminder the user has not edited.
- **Pets are 16×16 sprite data** in `src/lib/sprites.ts`, rasterised to `<canvas>` with `putImageData` + `image-rendering: pixelated`. App icons are generated from the same grids by `tools/generate-icons.py` (stdlib-only PNG writer).
- Frontend tests run under jsdom and outside Tauri. `IS_TAURI` in `ipc.ts` gates the bridge, and `subscribe()` returns a no-op unlisten when absent — keep that guard when adding listeners or every mount leaks an unhandled rejection.
- Comments in this codebase explain *why* a non-obvious choice was made (the blur/click race in `tray.rs`, the mutex-poison recovery in `state.rs`, the browser resolve condition in `vitest.config.ts`). Match that habit rather than narrating what the code does.
