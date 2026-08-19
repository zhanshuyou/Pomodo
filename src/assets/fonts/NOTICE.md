# Vendored fonts

All files here are Latin subsets fetched from the Fontsource CDN and committed so the
packaged app never makes a network request for a font. CJK glyphs are intentionally not
vendored — the `--font-ui` stack falls back to `PingFang SC` on macOS.

| File | Family | Weight | Upstream |
| --- | --- | --- | --- |
| `ibm-plex-sans-400.woff2` | IBM Plex Sans | 400 | <https://github.com/IBM/plex> |
| `ibm-plex-sans-500.woff2` | IBM Plex Sans | 500 | <https://github.com/IBM/plex> |
| `ibm-plex-sans-600.woff2` | IBM Plex Sans | 600 | <https://github.com/IBM/plex> |
| `ibm-plex-mono-400.woff2` | IBM Plex Mono | 400 | <https://github.com/IBM/plex> |
| `ibm-plex-mono-500.woff2` | IBM Plex Mono | 500 | <https://github.com/IBM/plex> |
| `silkscreen-400.woff2` | Silkscreen | 400 | <https://github.com/googlefonts/silkscreen> |

Both families are licensed under the SIL Open Font License 1.1; see `OFL.txt`.
