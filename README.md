# Pomodo

A cross-platform desktop app built with [Tauri 2](https://tauri.app), React, TypeScript and Vite.

## Stack

| Layer     | Tech                                  |
| --------- | ------------------------------------- |
| Shell     | Tauri 2 (Rust)                        |
| Frontend  | React 19 + TypeScript, bundled by Vite |
| Dev URL   | `http://localhost:1420`               |

## Layout

```
├── index.html          # Vite entry point
├── src/                # React frontend
├── src-tauri/          # Rust backend
│   ├── src/main.rs     # Binary entry point
│   ├── src/lib.rs      # App setup + #[tauri::command] handlers
│   ├── capabilities/   # Permissions granted to windows
│   └── tauri.conf.json # App/bundle configuration
└── vite.config.ts
```

## Prerequisites

- [Node.js](https://nodejs.org) 18+
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
npm install
npm run tauri dev     # run the desktop app with hot reload
```

## Building

```sh
npm run tauri build   # produce platform installers in src-tauri/target/release/bundle
```

Frontend-only commands are also available: `npm run dev`, `npm run build`, `npm run preview`.

## Recommended IDE setup

[VS Code](https://code.visualstudio.com/) with the [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) and [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) extensions.

## License

Apache-2.0 — see [LICENSE](LICENSE).
