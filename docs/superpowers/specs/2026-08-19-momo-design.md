# Momo — Design Spec

Date: 2026-08-19
Source design: `Momo.dc.html` (Claude Design project `29fe5053-5dac-4265-8c33-dbbfff0f6113`)
Repo: `zhanshuyou/Pomodo`

## 1. Overview

Momo is a Pomodoro timer for macOS whose timer *is* a pixel-art desktop pet. Progress
fills the cells on the pet's belly; the pet nags you to stand up, drink water and rest
your eyes; and it keeps running on the desktop after the main window is closed.

The UI language is Chinese. The app has four surfaces:

1. **主窗口** — tabs 专注 / 统计 / 宠物
2. **桌面** — menu-bar tray popover + a free-floating desktop pet
3. **设置 · 提醒** — three-layer progressive reminder configuration
4. **提醒强度** — three interruption intensities (bubble / pet / fullscreen)

## 2. Decisions taken during brainstorming

| Question | Decision |
| --- | --- |
| Scope | All four surfaces, built as seven sequential plan documents |
| Platform | macOS-first; Linux/Windows must still compile and run with degraded window behaviour so the existing 3-platform CI stays green |
| State ownership | Rust owns everything. Timer, reminders, tasks, pet and stats live in a Rust actor that ticks on a background thread and emits events to every webview. Webviews are pure views. |
| Sprite rendering | Keep the design's 16×16 character maps and palette verbatim as data; rasterise to `<canvas>` via `putImageData` + `image-rendering: pixelated` instead of the design's `box-shadow` technique |
| `accent` and `tone` | Promoted from design-canvas props to real user settings under 设置 · 通用 |

## 3. Design tokens

### 3.1 Colour

All colours are `oklch()`, copied from the design.

| Token | Value | Use |
| --- | --- | --- |
| `--bg` | `oklch(0.95 0.008 70)` | app canvas / desktop backdrop |
| `--card` | `oklch(0.99 0.004 80)` | window and card surfaces |
| `--surface-2` | `oklch(0.965 0.006 70)` | title bars, sidebars |
| `--ink` | `oklch(0.24 0.012 60)` | primary text |
| `--dim` | `oklch(0.5 0.012 60)` | secondary text |
| `--faint` | `oklch(0.6 0.012 60)` | tertiary text, labels |
| `--line` | `oklch(0.88 0.008 70)` | borders |
| `--line-soft` | `oklch(0.93 0.008 70)` | dividers |
| `--track` | `oklch(0.9 0.008 70)` | progress track, empty cells |
| `--good` | `oklch(0.55 0.11 145)` | positive delta text |

`--accent` is user-selectable, default first:

- `oklch(0.63 0.13 40)` (terracotta, default)
- `oklch(0.58 0.11 250)` (blue)
- `oklch(0.58 0.12 150)` (green)
- `oklch(0.55 0.14 320)` (magenta)

Desktop-wallpaper surface (artboard 02) uses
`radial-gradient(120% 90% at 78% 12%, oklch(0.46 0.045 245) 0%, oklch(0.3 0.025 255) 62%, oklch(0.25 0.02 260) 100%)`.

Reminder category colours: 站立 `oklch(0.63 0.13 40)`, 喝水 `oklch(0.66 0.09 195)`,
护眼 `oklch(0.7 0.1 145)`, 复盘/深呼吸 `oklch(0.68 0.1 300)`, 肩颈拉伸 `oklch(0.7 0.12 60)`,
记一句想法 `oklch(0.62 0.07 250)`.

### 3.2 Type

- UI: `"IBM Plex Sans", "PingFang SC", "Helvetica Neue", sans-serif`
- Numerals / timer: `"IBM Plex Mono", monospace`
- Pixel accents (section numbers, pet names, badges): `"Silkscreen", monospace`

**Fonts must be vendored into `src/assets/fonts/` and loaded with local `@font-face`.**
The design pulls them from Google Fonts; a packaged desktop app cannot depend on network
access. Subset IBM Plex Sans to Latin + the Chinese glyphs actually used, or fall back to
`PingFang SC` for CJK and ship only the Latin subset.

### 3.3 Motion

Five keyframes, copied from the design:

| Name | Definition | Use |
| --- | --- | --- |
| `momo-bob` | `0%,100% translateY(0) rotate(-1.5deg)` / `50% translateY(-6px) rotate(1.5deg)`, 4.2s | idle pet |
| `momo-hop` | `0%,60%,100% translateY(0)` / `25% translateY(-14px)` / `40% translateY(-3px)`, 1.6s | pet reminder |
| `momo-rise` | `opacity 0→1, translateY(6px)→0`, 0.35s | popover entry |
| `momo-pulse` | `0%,100% opacity .4 scale(1)` / `50% opacity .06 scale(1.45)`, 3.6s | focus ring around pet |
| `momo-sway` | `±3deg`, 3s | fullscreen overlay pet |

All animation must respect `prefers-reduced-motion: reduce` — the design does not specify
this, but an always-visible desktop pet makes it necessary. Reduced motion holds the pet
still and cross-fades instead of hopping.

## 4. Sprite system

### 4.1 Data

Six pets, each a 16×16 array of 16-character strings. Character → palette entry:

| Char | Colour | Meaning |
| --- | --- | --- |
| `.` | — | transparent |
| `o` | `oklch(0.26 0.02 60)` | outline |
| `b` | pet body colour | body |
| `s` | `oklch(from <body> calc(l - 0.12) c h)` | body shade |
| `e` | `oklch(0.2 0.015 60)` | eye |
| `w` | `oklch(0.98 0.006 80)` | belly / highlight |
| `p` | `oklch(0.78 0.11 20)` | blush / feet |

The six maps (`CAT`, `SLIME`, `FROG`, `BEAR`, `BIRD`, `GHOST`) are copied verbatim from
the design's script block into `src/lib/sprites.ts`.

| Id | Name | Map | Body | Unlocked initially |
| --- | --- | --- | --- | --- |
| 0 | MOCHI | CAT | `oklch(0.84 0.09 80)` | yes |
| 1 | PUDDING | SLIME | `oklch(0.82 0.08 195)` | yes |
| 2 | TOFU | FROG | `oklch(0.82 0.1 145)` | yes |
| 3 | BEAN | BEAR | `oklch(0.72 0.06 55)` | yes |
| 4 | PEEP | BIRD | `oklch(0.85 0.11 95)` | no |
| 5 | BOO | GHOST | `oklch(0.9 0.02 280)` | no |

Locked pets render with body `oklch(0.86 0.006 70)` at `opacity: 0.5`.
Header copy "6+1 宠物形象" = six built-ins plus one user-supplied custom pet.

### 4.2 Renderer

`PetCanvas.svelte` takes `{ map, body, scale, state }` and draws to a `<canvas>` sized
`16 * scale`. Implementation notes:

- Resolve the seven palette entries to RGBA once per (pet, accent) pair and memoise.
  `oklch(from … )` relative colour syntax must be resolved in JS, not left to CSS —
  compute the shade by subtracting 0.12 from L and converting oklch→sRGB.
- Draw at 1:1 into a 16×16 offscreen canvas with `putImageData`, then upscale with
  `imageSmoothingEnabled = false` and `image-rendering: pixelated`.
- Redraw only when pet, accent or animation frame changes — not per rAF tick. The
  bob/hop motion is a CSS transform on the canvas element, not a redraw.
- Scales used by the design: 8 (main focus tab, desktop pet), 9 (pet tab hero),
  4 (pet picker cards, reminder hop), 3 (tray avatar, small bubbles, overlay).

### 4.3 Custom pet

Users may drop a PNG / GIF / APNG. Requirements from the design:

- Three optional slots — 专注 / 休息 / 催你站起来 — auto-swapped by pet state.
- Pixel art is upscaled by integer factors only, never smoothed.
- Files are copied into the app data directory; the store holds paths, not blobs.

## 5. Copy and tone

`tone` has three values that rewrite every user-facing string:
`克制专业` (a) / `温和陪伴` (b) / `俏皮拟人` (c, default).

A `tone(a, b, c)` helper in `src/lib/theme.ts` selects the variant. Every string below is
product copy and must be preserved exactly.

### 5.1 Tagline

- a: 菜单栏与主窗口双入口的番茄计时器，含可自定义的身体提醒与桌面宠物。
- b: 一个陪你专注的番茄钟：它记得提醒你站立喝水，也记得在你完成时替你高兴。
- c: 它负责计时、催你喝水、盯你站起来，并在你摸鱼时用眼神谴责你。

### 5.2 Pet line (during focus, `{mm}` = remaining minutes)

- a: 本轮剩余 {mm} 分钟。
- b: 再 {mm} 分钟就好，我陪着你。
- c: 还有 {mm} 分钟，我盯着你呢

### 5.3 Stats verdict

- a: 本周专注 14h20m，较上周 +12%，中断率下降。
- b: 这周你比上周多专注了 1 小时 40 分，很稳。
- c: 这周表现不错，我勉为其难地允许你今晚多睡半小时。

### 5.4 Built-in reminders

Four seeded reminders. `message` is what the pet says; `hint` is the explanatory line
shown beside the small pet in the settings editor.

**站起来动一动** — colour `oklch(0.63 0.13 40)`, every 45 min, detail `每 45 分钟 · 宠物提示 · 工作时段`

- message a: 已连续坐着 45 分钟，请起身活动 2 分钟。
- message b: 坐久了，陪我一起站起来伸个懒腰？
- message c: 再坐下去你就要跟椅子长在一起了，起来！
- hint a: 专注进行中时会推迟到本轮结束。
- hint b: 我会等你这轮结束再叫你。
- hint c: 我不打断你，但下课钟一响我就扑上来。

**喝水** — colour `oklch(0.66 0.09 195)`, every 30 min, detail `每 30 分钟 · 轻量气泡 · 计入每日 8 杯`

- message a: 补充 200ml 水，今日 6/8 杯。
- message b: 喝口水吧，今天第 7 杯了。
- message c: 你的杯子在喊你，它说它很空。
- hint a: 菜单栏会累计今日饮水杯数。
- hint b: 我帮你数着杯数。
- hint c: 我偷偷在小本本上记你喝了几杯。

**远眺护眼** — colour `oklch(0.7 0.1 145)`, every 20 min, detail `每 20 分钟 · 轻量气泡 · 20-20-20`

- message a: 看向 6 米外物体并保持 20 秒。
- message b: 抬头看看窗外，20 秒就好。
- message c: 眼睛快冒烟了，看看远方压压火。
- hint a: 遵循 20-20-20 护眼规则。
- hint b: 20 分钟、20 英尺、20 秒。
- hint c: 我数到 20 就放你走，说好了。

**收工前复盘** — colour `oklch(0.68 0.1 300)`, daily 17:30, detail `每天 17:30 · 全屏 · 仅工作日`

- message a: 用 5 分钟复盘今天并规划明天。
- message b: 收工前，和我一起理一理今天？
- message c: 先夸自己一句，再写下明天要干的事。
- hint a: 自定义提醒：时间、文案、方式都可改。
- hint b: 这条完全是你自己写的。
- hint c: 这条是你自己加的，别怪我。

### 5.5 Reminder templates

Chips that seed a new reminder: 站立, 喝水, 护眼, 深呼吸, 肩颈拉伸, 记一句想法, plus `＋ 空白`.

## 6. Rust architecture

```
src-tauri/src/
  main.rs
  lib.rs              builder wiring: plugins, tray, windows, commands, tick thread
  state.rs            AppState — Mutex<Model> + event emitter
  model.rs            Model: timer, tasks, reminders, pet, stats, settings
  store.rs            load/save JSON, atomic write, schema version + migration
  commands.rs         #[tauri::command] surface
  events.rs           typed event payloads + names
  core/timer.rs
  core/reminder.rs
  core/task.rs
  core/pet.rs
  core/stats.rs
  windows.rs          create/show/hide main, tray, pet, overlay
  platform/mod.rs     trait PlatformWindows
  platform/macos.rs   NSPanel level, join-all-spaces, click-through, fullscreen detect
  platform/fallback.rs
```

### 6.1 Model

```rust
struct Model {
    timer: Timer,
    tasks: Vec<Task>,
    reminders: Vec<Reminder>,
    pet: PetState,
    stats: Stats,
    settings: Settings,
}

struct Timer {
    phase: Phase,          // Focus | ShortBreak | LongBreak
    remaining_secs: u32,
    running: bool,
    round: u8,             // 1..=4
    active_task: Option<TaskId>,
}

struct Settings {
    accent: Accent,        // 4 variants
    tone: Tone,            // 3 variants
    focus_secs: u32,       // default 1500
    short_break_secs: u32, // default 300
    long_break_secs: u32,  // default 900
    rounds_per_cycle: u8,  // default 4
    pet_flags: PetFlags,   // 贴边吸附 / 点击互动 / 全屏时隐藏 / 睡眠动画
}
```

### 6.2 Timer semantics

- Focus 25:00, short break 5:00, long break 15:00, four rounds per cycle.
  The design's copy "再 2 轮就能哄它去睡长觉（15 分钟）" confirms the long break follows round 4.
- The tick thread decrements once per second while `running`. It must derive elapsed time
  from a monotonic instant rather than counting ticks, so that sleep/wake does not drift.
- Phase completion advances the round on break→focus, emits a phase-change event, records
  a session in stats, and triggers any reminder deferred to round end.
- `skip` jumps to the next phase without recording a completed session.

### 6.3 Reminder engine

A `Reminder` carries: name, colour, message (tone-aware), interval or daily time,
intensity (`Bubble` | `Pet` | `Fullscreen`), enabled flag, and an advanced rule block:

| Rule | Default |
| --- | --- |
| 生效时段 | 09:30 – 18:30 |
| 生效日期 | 周一 – 周五 |
| 专注中 | 推迟到本轮结束 |
| 检测到会议 / 通话 | 静默 |
| 连续忽略 3 次 | 升级为全屏 |
| 声音 | 木鱼 · 30% |

Escalation: three consecutive ignores promote the next firing to `Fullscreen` once, then
reset. "Deep work" mode demotes every reminder to `Bubble` (design: "深度工作时全部自动降到最轻那档").
Meeting/call detection is macOS-only (microphone-in-use check); elsewhere it is a no-op.

### 6.4 Events

Rust → all webviews, emitted on change (not per tick where avoidable):

- `timer:tick` `{ remaining_secs, phase, running, round }` — once per second
- `timer:phase` `{ from, to, round }`
- `reminder:fire` `{ id, name, message, intensity, colour }`
- `model:changed` `{ section }` — tasks / pet / stats / settings / reminders
- `pet:state` `{ state }` — Idle | Focus | Break | Nagging | Sleeping

### 6.5 Commands

`start`, `pause`, `skip_phase`, `set_active_task`, `add_task`, `toggle_task`,
`delete_task`, `list_model`, `set_setting`, `select_pet`, `import_custom_pet`,
`upsert_reminder`, `toggle_reminder`, `delete_reminder`, `ack_reminder`,
`snooze_reminder`, `set_pet_position`, `toggle_mini_mode`.

### 6.6 Persistence

Single JSON document at `$APPDATA/momo/state.json` (`app_data_dir()`), written
atomically (temp file + rename), debounced to at most one write per second. Top-level
`schema_version` field; unknown-version files are backed up rather than overwritten.
Custom pet images live in `$APPDATA/momo/pets/`.

## 7. Frontend architecture

Svelte 5 with runes. Five Vite entry points, one per window (see §9). `overlay.html` is
built once and instantiated per display.

```
src/
  lib/sprites.ts        char maps, palette, oklch→rgba
  lib/theme.ts          tokens, accent, tone()
  lib/ipc.ts            typed invoke + listen wrappers
  lib/state.svelte.ts   runes store fed by events
  lib/components/       PetCanvas, PixelButton, Toggle, Chip, StatBar,
                        SpeechBubble, TitleBar, SectionHeading
  routes/main/          App.svelte, FocusTab, StatsTab, PetTab
  routes/prefs/         App.svelte, TimerPane, RemindersPane, PetPane, SoundPane, GeneralPane
  routes/tray/          App.svelte
  routes/pet/           App.svelte
  routes/overlay/       App.svelte
index.html  prefs.html  tray.html  pet.html  overlay.html
```

`state.svelte.ts` holds one `$state` object hydrated by `list_model` on mount and patched
by events. Components never `invoke` directly; they call typed functions from `ipc.ts`.

## 8. Screen specifications

### 8.1 主窗口 (artboard 01)

1180 × ~660, radius 16, border `--line`, shadow `0 28px 56px -28px oklch(0.24 0.012 60 / 0.45)`.

**Title bar** — 46px, `--surface-2`, bottom border. Three traffic lights
(`oklch(0.72 0.15 25)`, `oklch(0.82 0.13 85)`, `oklch(0.78 0.14 145)`), title `Momo`,
segmented tab control (专注 / 统计 / 宠物) in a `oklch(0.92 0.008 70)` pill, and right-aligned
`连续 12 天` · divider · `⌘,`.

**专注 tab** — two columns.

*Left (flex 1, padding 40/44/34, background
`linear-gradient(180deg, oklch(0.975 0.012 75) 0%, oklch(0.99 0.004 80) 70%)`):*

- Status pill: accent dot + `{phase} · 第 {round}/4 轮 · {task}`
- Pet at scale 8 with `momo-bob`, wrapped in a pulsing accent ring (`momo-pulse`),
  elliptical blurred shadow beneath
- Belly progress: 10 cells of 11×11px, filled with accent up to `round(percent / 10)`
- Timer: 78px IBM Plex Mono, `letter-spacing: -0.05em`; beneath it `预计 HH:MM 结束`
- Speech bubble: radius `13px 13px 13px 4px`, max-width 340, centred
- Buttons: primary `{让它歇会儿 | 开始专注}` with `inset 0 -3px 0 oklch(0.24 0.012 60 / 0.18)`,
  then 跳过, then `{迷你模式 | 退出迷你模式}`
- Round dots: four 10×10 squares + `再 2 轮就能哄它去睡长觉（15 分钟）`

*Right (372px, left border):*

- Header `今天要啃的` / `{done} / {total} 完成`
- Task rows: checkbox (17px, radius 5, accent when done), name (strikethrough + `oklch(0.62 0.012 60)`
  when done), meta line, and up to 3 pomodoro pips (7×7)
- `＋ 加一件事（⌘N）` dashed row
- Footer pinned to bottom: `身体这边的账` label + three `StatBar`s —
  喝水 `6 / 8 杯` 75%, 站立 `4 / 6 次` 66%, 久坐最长 `68 分钟` 76%

Seeded tasks: 写产品需求文档 (进行中 · 已投入 3 个番茄, 3 pips), 回 Sarah 的邮件 (预计 1 个番茄, 1),
整理用研笔记 (预计 2 个番茄, 2), 改登录页文案 (已完成 · 1 个番茄, 1, done),
周会前更新看板 (已完成 · 1 个番茄, 1, done).

**统计 tab**

- Four stat cards: 本周专注 `14h20m` / `较上周 +12%` (green); 完成番茄 `43` / `日均 6.1 个`;
  中断次数 `9` / `较上周 −4` (green); 连续天数 `12` / `个人最佳 18`
- `最近两周的专注分布` — 14 stacked bars, one 13px cell per pomodoro, cell colour
  `oklch(from <accent> calc(l + (0.16 - index * 0.035)) c h)`; empty days show one
  `oklch(0.93 0.008 70)` cell. Sample data `[3,5,2,6,4,0,1,5,7,4,5,6,2,5]`,
  labels 一二三四五六日 ×2. Caption `每格 = 一个番茄，颜色越深越连贯`.
- Two insight cards: 被打断最多的时段 (`15:00–16:00，平均每轮被打断 1.8 次。要不要把这段设成「勿扰 + 只留宠物提示」？`)
  and Momo 的评价 (tone-aware verdict)

**宠物 tab**

- Left 300px hero card: pet at scale 9, name in Silkscreen, `Lv.7 · 好奇期`,
  62% progress bar, `再专注 5 个番茄升到 Lv.8，解锁「披风」`
- `选一只` — 6-column grid of pet cards at scale 4, note `灰色的还锁着，专注攒够就解锁`
- Custom pet: 148×148 drop slot (`拖入你的宠物 PNG / GIF`) beside `或者养你自己的` and the
  explanatory paragraph, plus four toggle chips — 贴边吸附, 点击互动, 全屏时隐藏, 睡眠动画
  (first three on by default)

### 8.2 桌面 (artboard 02)

**Menu bar** — 30px, `oklch(0.22 0.015 260 / 0.55)` + `backdrop-filter: blur(20px)`.
Left: apple dot, `Momo`, menus 文件 / 专注 / 宠物 / 窗口. Right: a pill containing an accent
square and the live `mm:ss`, then `100%`, then the clock.

**Tray popover** — 330px wide, `oklch(0.985 0.004 80 / 0.95)`, `blur(30px)`, radius 16,
`momo-rise` entry, anchored below the tray item.

- Row 1: 76px conic-gradient ring `conic-gradient(<accent> <pct>%, oklch(0.9 0.008 70) 0)`
  with a `--card` disc inset 6px and the pet at scale 3 inside; beside it the 22px timer,
  the phase label in accent, and 开始/暂停 + 跳过 buttons
- Divider, then `接下来轮到` and three up-next rows (colour square, name, due):
  喝水 `4 分钟后`, 站起来动一动 `12 分钟后`, 远眺护眼 `18 分钟后`
- Divider, then `今天 5 个番茄 · 2h05m` and `设置…`

**Desktop pet** — pet at scale 8 with `momo-bob`, blurred ellipse shadow, and a speech
bubble to its right. Behaviour: draggable anywhere, snaps to screen edges when 贴边吸附
is on, hides when a fullscreen app is frontmost when 全屏时隐藏 is on, ignores clicks
except on the pet itself.

### 8.3 设置 · 提醒 (artboard 03)

1180 wide, min-height 560. Title bar reads `设置 — 提醒`.

- **Sidebar** 172px, `--surface-2`: 计时 / 提醒 / 宠物 / 声音 / 通用
- **Column 2** (396px) — layer 1 and 2:
  - `01 从模板抓一个` — template chips + `＋ 空白`
  - divider
  - `02 你的提醒` with `{n} 条开启` — reminder rows: 26px colour tile (opacity 0.28 when off),
    name, detail line, and a 38×22 switch
- **Column 3** (flex 1) — layer 3, `03 编辑「{name}」`:
  - `它会怎么说` — message field
  - `多久一次` — interval chips 20 / 30 / 45 / 60 min
  - `怎么打扰你` — three cards: 气泡 (角落一闪), 宠物 (它跳给你看), 全屏 (躲不掉)
  - Disclosure `还要更精细？展开规则 ▼` / `收起精细规则 ▲` revealing the six rule rows
  - Pinned to the bottom: an `oklch(0.96 0.012 70)` card with the pet at scale 3 and the
    tone-aware hint

Selected chips and rows use border `--accent` and background `oklch(0.975 0.008 70)`.

### 8.4 提醒强度 (artboard 04)

Three 400px cards, the middle one bordered in accent and badged `DEFAULT`.

1. **轻量气泡** — 右上角滑入，6 秒自动收起。Dark `oklch(0.31 0.025 258)` toast with the pet
   at scale 3, title 该喝水了 and the water message. 适合：喝水、护眼、深呼吸
2. **宠物来闹你** — 它蹦起来说话，一个窗口都不遮。Pet at scale 4 with `momo-hop` beside a
   speech bubble carrying the stand-up message. 适合：站立、肩颈活动、日常打气
3. **全屏遮罩** — 盖住所有屏幕，可设成「必须完成」。`oklch(0.29 0.025 258)` panel, pet at scale 3
   with `momo-sway`, a 30px countdown, `站起来走走，看点远的东西`, and the corner note
   `按 ⎋ 逃跑（它会记着）`. 适合：长休息、久坐超 90 分钟

The fullscreen variant must cover every connected display and record an escape as an
"ignore" for escalation purposes.

## 9. Windows

| Window | Entry | Behaviour |
| --- | --- | --- |
| `main` | `index.html` | 1180×660, standard, closable to tray |
| `prefs` | `prefs.html` | 1180×620, opened by ⌘, and 设置… |
| `tray` | `tray.html` | 330×~330, undecorated, transparent, hidden on blur, positioned under the tray icon |
| `pet` | `pet.html` | ~360×200, transparent, always-on-top, no shadow, click-through except on the pet |
| `overlay` | `overlay.html` | one per display, fullscreen, always-on-top, created on demand |

macOS specifics live in `platform/macos.rs`: promote `tray`/`pet`/`overlay` to
`NSPanel`-like behaviour (`NSWindowCollectionBehaviorCanJoinAllSpaces`, non-activating),
detect frontmost-fullscreen to hide the pet, and set `ignoresMouseEvents` regions.
`platform/fallback.rs` implements the same trait with plain always-on-top windows so the
Linux and Windows CI builds keep compiling.

## 10. Testing

**Rust (`cargo test`)**

- `core/timer.rs` — phase transitions, round advance, long break after round 4, skip
  semantics, monotonic-clock drift under a simulated sleep
- `core/reminder.rs` — interval and daily scheduling, active-window/day filters,
  defer-to-round-end, three-ignore escalation and its reset, deep-work demotion
- `core/stats.rs` — daily aggregation, streak calculation, week-over-week delta
- `store.rs` — round-trip, atomic write, unknown schema version backs up rather than
  overwrites

**Frontend (vitest)**

- `sprites.ts` — char map → RGBA buffer, palette resolution, shade computation,
  locked-pet greying
- `theme.ts` — `tone()` selection across all three values, accent token resolution,
  the stats bar `calc(l + …)` colour ramp

UI layout is verified visually against the artboards; there is no snapshot testing.

**CI** — the existing `npm run check`, `npm run build`, `cargo fmt --check`,
`cargo clippy -D warnings`, `cargo test`, and the 3-platform `tauri build --no-bundle`
must stay green after every plan.

## 11. Risks

| Risk | Mitigation |
| --- | --- |
| Multi-entry Vite + Tauri window wiring is fiddly | Plan 1 establishes and proves the multi-page build before any window depends on it |
| `NSPanel` behaviour needs raw `objc2` handles | Isolated behind the `PlatformWindows` trait; a plain always-on-top window is an acceptable interim result |
| macOS notification and accessibility permissions | Request lazily with an in-app explanation; every reminder intensity must still work if denied |
| Fullscreen overlay across multiple displays | One window per monitor, created on demand and torn down after; verify on a two-display setup |
| Font licensing and CJK subset size | IBM Plex and Silkscreen are OFL; ship the Latin subset only and let `PingFang SC` cover CJK |
| Always-on-top pet costing battery | Redraw only on state change; pause animation when the pet is occluded or on battery saver |

## 12. Implementation plan sequence

Seven plan documents, executed one at a time. Plans 1 and 2 are independent; 3 depends on
both; 4–7 each depend on 3.

1. **Foundation** — remove the scaffold, vendor fonts, establish design tokens, port the
   sprite maps, build `PetCanvas` and the pixel component kit, prove the multi-entry Vite
   build. Verifiable in a plain browser.
2. **Rust timer core** — `model.rs`, `core/timer.rs`, `store.rs`, `state.rs`, the tick
   thread, events and commands. `cargo test` green. No UI.
3. **主窗口 · 专注 tab** — wire `FocusTab` to the Rust timer; task list; body stats. The
   first genuinely usable Pomodoro.
4. **主窗口 · 统计 + 宠物 tabs** — `core/stats.rs` aggregation, the 14-day bars, pet picker
   with unlock rules, custom sprite import.
5. **Reminder engine + 设置 window** — `core/reminder.rs`, the three-layer settings pane,
   the advanced rule block, escalation.
6. **Tray popover** — tray icon, the `tray` window, positioning, the conic ring and
   up-next list.
7. **Desktop pet + overlays** — the `pet` window, drag and edge-snap, the three
   intensities including per-display fullscreen, and `platform/macos.rs`.
