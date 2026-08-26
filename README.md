# Pomodo

[![CI](https://github.com/zhanshuyou/Pomodo/actions/workflows/ci.yml/badge.svg)](https://github.com/zhanshuyou/Pomodo/actions/workflows/ci.yml)

A cross-platform desktop app built with [Tauri 2](https://tauri.app), Svelte, TypeScript and Vite.

## Stack

| Layer     | Tech                                  |
| --------- | ------------------------------------- |
| Shell     | Tauri 2 (Rust)                        |
| Frontend  | Svelte 5 + TypeScript, bundled by Vite |
| Dev URL   | `http://localhost:1420`               |

## Layout

```
├── index.html          # Vite entry point
├── src/                # Svelte frontend
├── src-tauri/          # Rust backend
│   ├── src/main.rs     # Binary entry point
│   ├── src/lib.rs      # App setup + #[tauri::command] handlers
│   ├── capabilities/   # Permissions granted to windows
│   └── tauri.conf.json # App/bundle configuration
└── vite.config.ts
```

## Prerequisites

- [Node.js](https://nodejs.org) 18+ (CI builds on 22)
- [pnpm](https://pnpm.io) — the version in `package.json`'s `packageManager` field,
  which `corepack enable` installs for you
- [Rust](https://rustup.rs) stable
- Platform dependencies listed at <https://tauri.app/start/prerequisites/>

On Debian/Ubuntu:

```sh
sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev \
  libayatana-appindicator3-dev librsvg2-dev build-essential \
  curl wget file libssl-dev libxdo-dev
```

## Getting started

```sh
pnpm install
pnpm tauri dev     # run the desktop app with hot reload
```

## Building

```sh
pnpm tauri build   # produce platform installers in src-tauri/target/release/bundle
```

Frontend-only commands are also available: `pnpm dev`, `pnpm build`, `pnpm preview`,
`pnpm check` for Svelte/TypeScript diagnostics, and `pnpm test` for the vitest suite.

## Recommended IDE setup

[VS Code](https://code.visualstudio.com/) with the [Svelte](https://marketplace.visualstudio.com/items?itemName=svelte.svelte-vscode), [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) and [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) extensions.

## CI

[`.github/workflows/ci.yml`](.github/workflows/ci.yml) runs on every push to
`main` and on every pull request:

- **Frontend** — `pnpm check` (svelte-check + tsc), `pnpm test` and `pnpm build`
- **Rust** — `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test`
- **Tauri build** — `pnpm tauri build --no-bundle` on Linux, macOS and
  Windows, which compiles the release binary against the built frontend
  without producing installers

## License

Apache-2.0 — see [LICENSE](LICENSE).
