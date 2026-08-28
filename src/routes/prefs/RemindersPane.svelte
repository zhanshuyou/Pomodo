<script lang="ts">
  import Chip from "../../lib/components/Chip.svelte";
  import PetCanvas from "../../lib/components/PetCanvas.svelte";
  import Toggle from "../../lib/components/Toggle.svelte";
  import {
    type FocusBehavior,
    type Intensity,
    type Rules,
    MESSAGE_PLACEHOLDERS,
    SOUND_TONES,
    type SoundTone,
    addQuietWindow,
    addReminder,
    deleteQuietWindow,
    deleteReminder,
    previewSound,
    soundLabel,
    toggleReminder,
    updateReminder,
  } from "../../lib/ipc";
  import { PETS } from "../../lib/sprites";
  import { app } from "../../lib/state.svelte";
  import {
    WEEKDAY_SHORT,
    escalationLabel,
    minutesToTime,
    quietLabel,
    timeToMinutes,
    weekdaysLabel,
    withWeekday,
  } from "../../lib/rules";
  import { REMINDER_COLORS } from "../../lib/theme";

  const TEMPLATES = [
    { name: "站立", color: REMINDER_COLORS.stand },
    { name: "喝水", color: REMINDER_COLORS.water },
    { name: "护眼", color: REMINDER_COLORS.eyes },
    { name: "深呼吸", color: REMINDER_COLORS.breathe },
    { name: "肩颈拉伸", color: REMINDER_COLORS.stretch },
    { name: "记一句想法", color: REMINDER_COLORS.note },
  ];

  const INTERVALS = [20, 30, 45, 60];
  const MIN_INTERVAL = 5;
  const MAX_INTERVAL = 480;

  const STYLES: { key: Intensity; label: string; hint: string }[] = [
    { key: "bubble", label: "气泡", hint: "角落一闪" },
    { key: "pet", label: "宠物", hint: "它跳给你看" },
    { key: "fullscreen", label: "全屏", hint: "躲不掉" },
  ];

  let editId = $state<number | null>(null);

  /** The 安静时段 add row; defaults to the design's worst hour. */
  let quietFrom = $state("15:00");
  let quietTo = $state("16:00");
  function addQuiet() {
    const from = timeToMinutes(quietFrom);
    const to = timeToMinutes(quietTo);
    if (from === null || to === null || from === to) return;
    void addQuietWindow(from, to);
  }
  let advanced = $state(false);
  /** The one row currently asking whether it should really go. */
  let confirmingId = $state<number | null>(null);

  function select(id: number) {
    editId = id;
    // Moving on abandons the question; a row left mid-confirmation somewhere
    // else in the list is a trap waiting to be clicked.
    confirmingId = null;
  }

  /** A chip should land you in the editor for what it just made. */
  async function add(template: string | null, color: string | null = null) {
    const id = await addReminder(template, color);
    select(id);
  }

  function confirmDelete(id: number) {
    // Editing follows the list, so hand the editor back to whatever survives.
    if (editId === id) editId = null;
    confirmingId = null;
    void deleteReminder(id);
  }

  const editing = $derived(
    app.reminders.find((r) => r.id === editId) ?? app.reminders[0] ?? null,
  );
  const onCount = $derived(app.reminders.filter((r) => r.enabled).length);
  const pet = $derived(PETS[app.pet.selected] ?? PETS[0]);

  const currentInterval = $derived(
    editing?.schedule.kind === "every" ? editing.schedule.minutes : null,
  );
  const dailyTime = $derived(
    editing?.schedule.kind === "dailyAt"
      ? minutesToTime(editing.schedule.hour * 60 + editing.schedule.minute)
      : "",
  );
  const messageBlank = $derived(!editing?.message.trim());

  function setEvery(minutes: number) {
    if (!editing) return;
    const m = Math.round(minutes);
    if (!Number.isFinite(m)) return;
    void updateReminder(editing.id, {
      schedule: {
        kind: "every",
        minutes: Math.max(MIN_INTERVAL, Math.min(MAX_INTERVAL, m)),
      },
    });
  }

  function setDaily(value: string) {
    if (!editing) return;
    const min = timeToMinutes(value);
    if (min === null) return;
    void updateReminder(editing.id, {
      schedule: { kind: "dailyAt", hour: Math.floor(min / 60), minute: min % 60 },
    });
  }

  /** Flipping to 每天定时 keeps the old interval's spirit: default to the review hour. */
  function switchToDaily() {
    if (!editing || editing.schedule.kind === "dailyAt") return;
    setDaily("17:30");
  }

  function switchToEvery() {
    if (!editing || editing.schedule.kind === "every") return;
    setEvery(INTERVALS[2]);
  }

  function onName(value: string) {
    if (!editing) return;
    const name = value.trim();
    if (!name || name === editing.name) return;
    void updateReminder(editing.id, { name });
  }

  const FOCUS_MODES: { key: FocusBehavior; label: string }[] = [
    { key: "defer", label: "推迟到本轮结束" },
    { key: "silence", label: "静默" },
    { key: "interrupt", label: "直接打断" },
  ];
  const MAX_ESCALATE = 10;

  /** Every rule edit sends the whole block; Rust replaces it wholesale. */
  function patchRules(rules: Rules) {
    if (!editing) return;
    void updateReminder(editing.id, { rules });
  }

  function onTime(which: "activeFromMin" | "activeToMin", value: string) {
    if (!editing) return;
    const min = timeToMinutes(value);
    if (min === null) return;
    patchRules({ ...editing.rules, [which]: min });
  }

  function setTone(tone: SoundTone) {
    if (!editing) return;
    const sound = { ...editing.rules.sound, tone };
    patchRules({ ...editing.rules, sound });
    void previewSound(sound);
  }

  function setVolume(value: string) {
    if (!editing) return;
    const volume = Math.max(0, Math.min(100, Math.round(Number(value))));
    if (!Number.isFinite(volume)) return;
    patchRules({ ...editing.rules, sound: { ...editing.rules.sound, volume } });
  }

  function onEscalate(value: string) {
    if (!editing) return;
    const n = Math.round(Number(value));
    if (!Number.isFinite(n)) return;
    patchRules({
      ...editing.rules,
      escalateAfter: Math.max(0, Math.min(MAX_ESCALATE, n)),
    });
  }
</script>

<div class="col2">
  <section>
    <div class="sechead">
      <span class="num">01</span>
      <span class="sectitle">从模板抓一个</span>
    </div>
    <div class="chips">
      {#each TEMPLATES as t (t.name)}
        <Chip dot={t.color} onclick={() => void add(t.name, t.color)}>{t.name}</Chip>
      {/each}
      <button class="blank" type="button" onclick={() => void add(null)}>
        ＋ 空白
      </button>
    </div>
  </section>

  <div class="divider"></div>

  <section>
    <div class="sechead">
      <span class="num">02</span>
      <span class="sectitle">你的提醒</span>
      <span class="oncount">{onCount} 条开启</span>
    </div>

    {#each app.reminders as r (r.id)}
      <div
        class="rem"
        class:sel={editing?.id === r.id}
        class:confirming={confirmingId === r.id}
        role="button"
        tabindex="0"
        onclick={() => select(r.id)}
        onkeydown={(e) => {
          if (e.key === "Enter") select(r.id);
          if (e.key === "Escape") confirmingId = null;
        }}
      >
        <span class="tile" style:background={r.color} style:opacity={r.enabled ? 1 : 0.28}
        ></span>

        {#if confirmingId === r.id}
          <span class="confirm-ask">删掉「{r.name}」？</span>
          <button
            class="confirm-yes"
            type="button"
            onclick={(e) => {
              e.stopPropagation();
              confirmDelete(r.id);
            }}
          >
            删除
          </button>
          <button
            class="confirm-no"
            type="button"
            onclick={(e) => {
              e.stopPropagation();
              confirmingId = null;
            }}
          >
            算了
          </button>
        {:else}
          <div class="remtext">
            <span class="remname">{r.name}</span>
            <span class="remdetail">{r.detail}</span>
          </div>
          <Toggle
            checked={r.enabled}
            onchange={() => void toggleReminder(r.id)}
            label="{r.name} 开关"
          />
          <button
            class="del"
            type="button"
            aria-label="删除{r.name}"
            title="删除"
            onclick={(e) => {
              e.stopPropagation();
              confirmingId = r.id;
            }}
          >
            ×
          </button>
        {/if}
      </div>
    {/each}

    {#if app.reminders.length === 0}
      <p class="remempty">还没有提醒，从上面抓一个模板</p>
    {/if}
  </section>

  <div class="divider"></div>

  <section class="quiet">
    <div class="sechead">
      <span class="num">04</span>
      <span class="sectitle">安静时段</span>
    </div>
    <span class="seccaption">
      这段时间里专注不会被打断（直接打断改为推迟到本轮结束），最响也只到宠物提示。
    </span>
    {#each app.quietHours as w (w.id)}
      <div class="quietrow">
        <span class="quiettime">{quietLabel(w.fromMin, w.toMin)}</span>
        <button
          class="del quietdel"
          type="button"
          aria-label="删除安静时段 {quietLabel(w.fromMin, w.toMin)}"
          onclick={() => void deleteQuietWindow(w.id)}
        >
          ×
        </button>
      </div>
    {/each}
    <div class="quietadd">
      <input class="qtime" type="time" aria-label="安静时段开始" bind:value={quietFrom} />
      <span class="dash">–</span>
      <input class="qtime" type="time" aria-label="安静时段结束" bind:value={quietTo} />
      <button class="blank" type="button" onclick={addQuiet}>＋ 添加</button>
    </div>
  </section>
</div>

<div class="col3">
  {#if editing}
    <div class="sechead">
      <span class="num">03</span>
      <span class="sectitle">编辑「{editing.name}」</span>
    </div>

    <div class="field">
      <span class="flabel">叫什么</span>
      <input
        class="name"
        type="text"
        aria-label="提醒名称"
        value={editing.name}
        onchange={(e) => onName(e.currentTarget.value)}
      />
    </div>

    <div class="field">
      <span class="flabel">什么颜色</span>
      <div class="swatches" role="radiogroup" aria-label="提醒颜色">
        {#each Object.values(REMINDER_COLORS) as color (color)}
          <button
            class="swatch"
            class:on={editing.color === color}
            type="button"
            role="radio"
            aria-checked={editing.color === color}
            aria-label={color}
            style:background={color}
            onclick={() => void updateReminder(editing.id, { color })}
          ></button>
        {/each}
      </div>
    </div>

    <div class="field">
      <span class="flabel">列表里的备注</span>
      <input
        class="name"
        type="text"
        aria-label="提醒备注"
        placeholder="例如：计入每日 8 杯"
        value={editing.note}
        onchange={(e) => void updateReminder(editing.id, { note: e.currentTarget.value })}
      />
    </div>

    <div class="field">
      <span class="flabel">它会怎么说</span>
      <textarea
        class="message"
        rows="2"
        value={editing.message}
        onchange={(e) =>
          void updateReminder(editing.id, {
            message: (e.currentTarget as HTMLTextAreaElement).value,
          })}
      ></textarea>
      {#if messageBlank}
        <span class="warn">先写一句它会怎么说，否则这条不会响。</span>
      {:else if editing.message.includes("{")}
        <span class="caption">
          响的时候会填上真实数字：{MESSAGE_PLACEHOLDERS.map((p) => `${p.key} ${p.label}`).join(
            " · ",
          )}
        </span>
      {/if}
    </div>

    <div class="field">
      <span class="flabel">多久一次</span>
      <div class="modes">
        <button
          class="mode"
          class:on={editing.schedule.kind === "every"}
          type="button"
          aria-pressed={editing.schedule.kind === "every"}
          onclick={switchToEvery}
        >
          每隔
        </button>
        <button
          class="mode"
          class:on={editing.schedule.kind === "dailyAt"}
          type="button"
          aria-pressed={editing.schedule.kind === "dailyAt"}
          onclick={switchToDaily}
        >
          每天定时
        </button>
      </div>
      {#if editing.schedule.kind === "every"}
        <div class="chips">
          {#each INTERVALS as min (min)}
            <Chip selected={currentInterval === min} onclick={() => setEvery(min)}>
              <span class="mono">{min} min</span>
            </Chip>
          {/each}
          <label class="custom-interval">
            <input
              class="minutes"
              type="number"
              min={MIN_INTERVAL}
              max={MAX_INTERVAL}
              aria-label="自定义间隔（分钟）"
              value={currentInterval ?? ""}
              onchange={(e) => setEvery(Number(e.currentTarget.value))}
            />
            <span class="mono">min</span>
          </label>
        </div>
      {:else}
        <input
          class="time daily"
          type="time"
          aria-label="每天几点"
          value={dailyTime}
          onchange={(e) => setDaily(e.currentTarget.value)}
        />
      {/if}
    </div>

    <div class="field">
      <span class="flabel">怎么打扰你</span>
      <div class="styles">
        {#each STYLES as s (s.key)}
          <button
            class="style"
            class:sel={editing.intensity === s.key}
            type="button"
            onclick={() => void updateReminder(editing.id, { intensity: s.key })}
          >
            <span class="slabel">{s.label}</span>
            <span class="shint">{s.hint}</span>
          </button>
        {/each}
      </div>
      {#if editing.intensity !== "bubble"}
        <label class="duration">
          <span class="flabel">持续多久</span>
          <input
            class="minutes"
            type="number"
            min="5"
            max="3600"
            step="5"
            aria-label="持续多久（秒）"
            value={editing.durationSecs}
            onchange={(e) => {
              const secs = Math.round(Number(e.currentTarget.value));
              if (Number.isFinite(secs))
                void updateReminder(editing.id, {
                  durationSecs: Math.max(5, Math.min(3600, secs)),
                });
            }}
          />
          <span class="mono">秒</span>
        </label>
      {/if}
      {#if editing.intensity === "fullscreen"}
        <label class="must">
          <Toggle
            checked={editing.rules.mustComplete}
            onchange={(v) => patchRules({ ...editing.rules, mustComplete: v })}
            label="必须完成"
          />
          <span>必须完成（不能按 ⎋ 逃跑，也不能稍后）</span>
        </label>
      {/if}
    </div>

    <button class="disclose" type="button" onclick={() => (advanced = !advanced)}>
      <span>{advanced ? "收起精细规则" : "还要更精细？展开规则"}</span>
      <span class="arrow">{advanced ? "▲" : "▼"}</span>
    </button>

    {#if advanced}
      <div class="rules">
        <div class="rrow">
          <span class="rname">生效时段</span>
          <span class="rctl">
            <input
              class="time"
              type="time"
              aria-label="生效开始"
              value={minutesToTime(editing.rules.activeFromMin)}
              onchange={(e) => onTime("activeFromMin", e.currentTarget.value)}
            />
            <span class="dash">–</span>
            <input
              class="time"
              type="time"
              aria-label="生效结束"
              value={minutesToTime(editing.rules.activeToMin)}
              onchange={(e) => onTime("activeToMin", e.currentTarget.value)}
            />
          </span>
        </div>

        <div class="rrow">
          <span class="rname">
            生效日期
            <span class="rsub">{weekdaysLabel(editing.rules.weekdays)}</span>
          </span>
          <span class="rctl days">
            {#each WEEKDAY_SHORT as d, i (d)}
              <button
                class="day"
                class:on={editing.rules.weekdays[i]}
                type="button"
                aria-pressed={editing.rules.weekdays[i]}
                aria-label="周{d}"
                onclick={() =>
                  patchRules(withWeekday(editing.rules, i, !editing.rules.weekdays[i]))}
              >
                {d}
              </button>
            {/each}
          </span>
        </div>

        <div class="rrow">
          <span class="rname">专注中</span>
          <span class="rctl seg">
            {#each FOCUS_MODES as mode (mode.key)}
              <button
                class="segbtn"
                class:on={editing.rules.duringFocus === mode.key}
                type="button"
                aria-pressed={editing.rules.duringFocus === mode.key}
                onclick={() => patchRules({ ...editing.rules, duringFocus: mode.key })}
              >
                {mode.label}
              </button>
            {/each}
          </span>
        </div>

        <div class="rrow">
          <span class="rname">检测到会议 / 通话时静默</span>
          <Toggle
            checked={editing.rules.silenceInMeeting}
            onchange={(v) => patchRules({ ...editing.rules, silenceInMeeting: v })}
            label="检测到会议 / 通话时静默"
          />
        </div>

        <div class="rrow">
          <span class="rname">
            连续忽略
            <span class="rsub">{escalationLabel(editing.rules.escalateAfter)}</span>
          </span>
          <input
            class="esc"
            type="number"
            min="0"
            max={MAX_ESCALATE}
            aria-label="连续忽略次数"
            value={editing.rules.escalateAfter}
            onchange={(e) => onEscalate(e.currentTarget.value)}
          />
        </div>

        <div class="rrow">
          <span class="rname">
            声音
            <span class="rsub">{soundLabel(editing.rules.sound)}</span>
          </span>
          <span class="rctl">
            {#each SOUND_TONES as t (t.key)}
              <button
                class="segbtn"
                class:on={editing.rules.sound.tone === t.key}
                type="button"
                aria-pressed={editing.rules.sound.tone === t.key}
                onclick={() => setTone(t.key)}
              >
                {t.label}
              </button>
            {/each}
            <input
              class="vol"
              type="range"
              min="0"
              max="100"
              step="5"
              aria-label="音量"
              disabled={editing.rules.sound.tone === "none"}
              value={editing.rules.sound.volume}
              onchange={(e) => setVolume(e.currentTarget.value)}
            />
            <button
              class="segbtn"
              type="button"
              aria-label="试听"
              title="试听"
              disabled={editing.rules.sound.tone === "none"}
              onclick={() => void previewSound(editing.rules.sound)}
            >
              ▶
            </button>
          </span>
        </div>
      </div>
    {/if}

    <div class="hintcard">
      <PetCanvas map={pet.map} body={pet.body} scale={3} alt={pet.name} />
      <span>{editing.hint}</span>
    </div>
  {/if}
</div>

<style>
  .col2 {
    width: 396px;
    flex: none;
    padding: 22px;
    border-right: 1px solid oklch(0.91 0.008 70);
    display: flex;
    flex-direction: column;
    gap: 18px;
    overflow-y: auto;
  }
  .col3 {
    flex: 1;
    padding: 22px 26px;
    display: flex;
    flex-direction: column;
    gap: 16px;
    overflow-y: auto;
    min-width: 0;
  }
  section {
    display: flex;
    flex-direction: column;
    gap: 11px;
  }
  .sechead {
    display: flex;
    align-items: baseline;
    gap: 8px;
  }
  .num {
    font-family: var(--font-pixel);
    font-size: 10px;
    color: var(--accent);
  }
  .sectitle {
    font-size: 13.5px;
    font-weight: 600;
  }
  .oncount {
    margin-left: auto;
    font-size: 12px;
    color: var(--faint);
  }
  .chips {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }
  .blank {
    padding: 7px 11px;
    border: 1px dashed oklch(0.84 0.008 70);
    border-radius: var(--radius-chip);
    background: transparent;
    font-size: 12.5px;
    color: var(--dim);
    cursor: pointer;
  }
  .divider {
    height: 1px;
    background: var(--line-soft);
  }
  .rem {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 13px;
    border: 1px solid var(--line);
    border-radius: var(--radius-control);
    background: var(--card);
    cursor: pointer;
  }
  .rem.sel {
    border-color: var(--accent);
    background: oklch(0.975 0.008 70);
  }
  .rem.confirming {
    border-color: oklch(0.62 0.16 25);
    background: oklch(0.975 0.012 30);
  }
  /* Stays out of the way until you go looking for it, like the pet's own
     dismiss button — a row of permanent × turns a list into a minefield. */
  .del {
    flex: none;
    width: 22px;
    height: 22px;
    padding: 0;
    border: none;
    border-radius: 50%;
    background: transparent;
    color: var(--faint);
    font-size: 16px;
    line-height: 1;
    cursor: pointer;
    opacity: 0;
    transition:
      opacity 0.15s ease,
      background 0.15s ease;
    display: grid;
    place-items: center;
  }
  .rem:hover .del,
  .del:focus-visible {
    opacity: 1;
  }
  .del:hover {
    background: oklch(0.93 0.02 25);
    color: oklch(0.5 0.16 25);
  }
  .confirm-ask {
    flex: 1;
    min-width: 0;
    font-size: 13px;
    color: var(--ink);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .confirm-yes,
  .confirm-no {
    flex: none;
    padding: 5px 11px;
    border-radius: var(--radius-chip);
    border: 1px solid var(--line);
    background: var(--card);
    font-size: 12.5px;
    cursor: pointer;
  }
  .confirm-yes {
    border-color: oklch(0.62 0.16 25);
    background: oklch(0.62 0.16 25);
    color: oklch(0.99 0.004 80);
  }
  .confirm-yes:hover {
    background: oklch(0.55 0.16 25);
  }
  .confirm-no:hover {
    background: var(--surface-2);
  }
  .remempty {
    margin: 0;
    padding: 14px 13px;
    border: 1px dashed var(--line);
    border-radius: var(--radius-control);
    font-size: 12.5px;
    color: var(--faint);
  }
  .tile {
    width: 26px;
    height: 26px;
    border-radius: 8px;
    flex: none;
  }
  .remtext {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 3px;
    min-width: 0;
  }
  .remname {
    font-size: 13.5px;
    font-weight: 600;
  }
  .remdetail {
    font-size: 12px;
    color: var(--dim);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .field {
    display: flex;
    flex-direction: column;
    gap: 7px;
  }
  .flabel {
    font-size: 12px;
    color: var(--dim);
  }
  .message {
    padding: 11px 13px;
    border: 1px solid var(--line);
    border-radius: 10px;
    background: oklch(0.985 0.004 80);
    font-family: inherit;
    font-size: 13.5px;
    line-height: 1.5;
    color: var(--ink);
    resize: vertical;
  }
  .mono {
    font-family: var(--font-mono);
  }
  .styles {
    display: flex;
    gap: 7px;
  }
  .style {
    flex: 1;
    padding: 11px 12px;
    border: 1px solid var(--line);
    border-radius: 10px;
    background: var(--card);
    cursor: pointer;
    display: flex;
    flex-direction: column;
    gap: 3px;
    text-align: left;
  }
  .style.sel {
    border-color: var(--accent);
    background: oklch(0.975 0.008 70);
  }
  .slabel {
    font-size: 12.5px;
    font-weight: 600;
    color: var(--ink);
  }
  .shint {
    font-size: 11.5px;
    color: var(--faint);
  }
  .disclose {
    display: flex;
    align-items: center;
    gap: 8px;
    border: none;
    background: transparent;
    padding: 0;
    font-family: inherit;
    font-size: 12.5px;
    color: var(--accent);
    cursor: pointer;
  }
  .arrow {
    font-size: 11px;
  }
  .rules {
    padding: 16px 18px;
    border: 1px solid oklch(0.9 0.008 70);
    border-radius: 12px;
    background: oklch(0.975 0.006 70);
    display: flex;
    flex-direction: column;
    gap: 13px;
  }
  .rrow {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }
  .rname {
    font-size: 12.5px;
    color: oklch(0.42 0.012 60);
  }
  .duration {
    margin-top: 10px;
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
    color: var(--dim);
  }
  .must {
    margin-top: 10px;
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 12.5px;
    color: var(--dim);
  }
  .name {
    padding: 8px 10px;
    border: 1px solid var(--line);
    border-radius: 8px;
    background: var(--card);
    font: inherit;
    font-size: 13px;
    color: var(--ink);
  }
  .quiet {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .quietrow {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 10px;
    border: 1px solid var(--line);
    border-radius: 10px;
    font-family: var(--font-mono);
    font-size: 12.5px;
  }
  .quietdel {
    opacity: 0.6;
  }
  .quietadd {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .qtime {
    padding: 4px 6px;
    border: 1px solid var(--line);
    border-radius: 8px;
    background: var(--card);
    font: inherit;
    font-size: 12px;
  }
  .dash {
    color: var(--faint);
  }
  .swatches {
    display: flex;
    gap: 8px;
  }
  .swatch {
    width: 22px;
    height: 22px;
    border: 2px solid transparent;
    border-radius: 6px;
    cursor: pointer;
    padding: 0;
  }
  .swatch.on {
    border-color: var(--ink);
    box-shadow: 0 0 0 2px var(--card) inset;
  }
  .warn {
    font-size: 12px;
    color: oklch(0.55 0.15 25);
  }
  .caption {
    font-size: 11.5px;
    color: var(--faint);
    line-height: 1.5;
  }
  .modes {
    display: flex;
    gap: 6px;
    margin-bottom: 8px;
  }
  .mode {
    padding: 4px 10px;
    border: 1px solid var(--line);
    border-radius: 8px;
    background: var(--card);
    color: var(--dim);
    font-size: 12px;
    cursor: pointer;
  }
  .mode.on {
    border-color: var(--accent);
    background: oklch(0.975 0.008 70);
    color: var(--ink);
  }
  .custom-interval {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 12px;
    color: var(--dim);
  }
  .minutes {
    width: 58px;
    padding: 5px 8px;
    border: 1px solid var(--line);
    border-radius: 8px;
    background: var(--card);
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--ink);
  }
  .daily {
    align-self: flex-start;
  }
  .rsub {
    display: block;
    font-size: 11px;
    color: var(--faint);
  }
  .rctl {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .time,
  .esc {
    padding: 4px 8px;
    border: 1px solid var(--line);
    border-radius: 8px;
    background: var(--card);
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--ink);
  }
  .esc {
    width: 56px;
  }
  .dash {
    color: var(--faint);
  }
  .vol {
    width: 72px;
    accent-color: var(--accent);
  }
  .day,
  .segbtn {
    padding: 4px 8px;
    border: 1px solid var(--line);
    border-radius: 8px;
    background: var(--card);
    color: var(--dim);
    font-size: 12px;
    cursor: pointer;
  }
  .day {
    width: 28px;
    padding: 4px 0;
  }
  .day.on,
  .segbtn.on {
    border-color: var(--accent);
    background: oklch(0.975 0.008 70);
    color: var(--ink);
  }
  .hintcard {
    margin-top: auto;
    padding: 14px 16px;
    border-radius: 12px;
    background: oklch(0.96 0.012 70);
    display: flex;
    gap: 12px;
    align-items: center;
  }
  .hintcard span {
    font-size: 12.5px;
    color: oklch(0.45 0.012 60);
    line-height: 1.5;
  }
</style>
