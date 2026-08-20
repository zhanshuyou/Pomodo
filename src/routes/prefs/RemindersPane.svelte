<script lang="ts">
  import Chip from "../../lib/components/Chip.svelte";
  import PetCanvas from "../../lib/components/PetCanvas.svelte";
  import Toggle from "../../lib/components/Toggle.svelte";
  import {
    type Intensity,
    addReminder,
    toggleReminder,
    updateReminder,
  } from "../../lib/ipc";
  import { PETS } from "../../lib/sprites";
  import { app } from "../../lib/state.svelte";
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

  const STYLES: { key: Intensity; label: string; hint: string }[] = [
    { key: "bubble", label: "气泡", hint: "角落一闪" },
    { key: "pet", label: "宠物", hint: "它跳给你看" },
    { key: "fullscreen", label: "全屏", hint: "躲不掉" },
  ];

  let editId = $state<number | null>(null);
  let advanced = $state(false);

  const editing = $derived(
    app.reminders.find((r) => r.id === editId) ?? app.reminders[0] ?? null,
  );
  const onCount = $derived(app.reminders.filter((r) => r.enabled).length);
  const pet = $derived(PETS[app.pet.selected] ?? PETS[0]);

  const currentInterval = $derived(
    editing?.schedule.kind === "every" ? editing.schedule.minutes : null,
  );

  function minutesLabel(min: number): string {
    return `${String(Math.floor(min / 60)).padStart(2, "0")}:${String(min % 60).padStart(2, "0")}`;
  }

  const ruleRows = $derived(
    editing
      ? [
          {
            name: "生效时段",
            value: `${minutesLabel(editing.rules.activeFromMin)} – ${minutesLabel(editing.rules.activeToMin)}`,
          },
          {
            name: "生效日期",
            value: editing.rules.weekdays.slice(0, 5).every(Boolean)
              ? "周一 – 周五"
              : "自定义",
          },
          {
            name: "专注中",
            value:
              editing.rules.duringFocus === "defer"
                ? "推迟到本轮结束"
                : editing.rules.duringFocus === "silence"
                  ? "静默"
                  : "直接打断",
          },
          {
            name: "检测到会议 / 通话",
            value: editing.rules.silenceInMeeting ? "静默" : "照常提醒",
          },
          {
            name: `连续忽略 ${editing.rules.escalateAfter} 次`,
            value: "升级为全屏",
          },
          { name: "声音", value: editing.rules.sound },
        ]
      : [],
  );
</script>

<div class="col2">
  <section>
    <div class="sechead">
      <span class="num">01</span>
      <span class="sectitle">从模板抓一个</span>
    </div>
    <div class="chips">
      {#each TEMPLATES as t (t.name)}
        <Chip dot={t.color} onclick={() => void addReminder(t.name)}>{t.name}</Chip>
      {/each}
      <button class="blank" type="button" onclick={() => void addReminder(null)}>
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
        role="button"
        tabindex="0"
        onclick={() => (editId = r.id)}
        onkeydown={(e) => e.key === "Enter" && (editId = r.id)}
      >
        <span class="tile" style:background={r.color} style:opacity={r.enabled ? 1 : 0.28}
        ></span>
        <div class="remtext">
          <span class="remname">{r.name}</span>
          <span class="remdetail">{r.detail}</span>
        </div>
        <Toggle
          checked={r.enabled}
          onchange={() => void toggleReminder(r.id)}
          label="{r.name} 开关"
        />
      </div>
    {/each}
  </section>
</div>

<div class="col3">
  {#if editing}
    <div class="sechead">
      <span class="num">03</span>
      <span class="sectitle">编辑「{editing.name}」</span>
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
    </div>

    <div class="field">
      <span class="flabel">多久一次</span>
      <div class="chips">
        {#each INTERVALS as min (min)}
          <Chip
            selected={currentInterval === min}
            onclick={() => void updateReminder(editing.id, { intervalMinutes: min })}
          >
            <span class="mono">{min} min</span>
          </Chip>
        {/each}
      </div>
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
    </div>

    <button class="disclose" type="button" onclick={() => (advanced = !advanced)}>
      <span>{advanced ? "收起精细规则" : "还要更精细？展开规则"}</span>
      <span class="arrow">{advanced ? "▲" : "▼"}</span>
    </button>

    {#if advanced}
      <div class="rules">
        {#each ruleRows as row (row.name)}
          <div class="rrow">
            <span class="rname">{row.name}</span>
            <span class="rvalue">{row.value}</span>
          </div>
        {/each}
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
  .rvalue {
    padding: 5px 10px;
    border: 1px solid var(--line);
    border-radius: 8px;
    background: var(--card);
    font-family: var(--font-mono);
    font-size: 12px;
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
