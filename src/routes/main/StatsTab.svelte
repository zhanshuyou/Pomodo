<script lang="ts">
  import { petVerdict } from "../../lib/copy";
  import { app } from "../../lib/state.svelte";
  import { ACCENTS, barCellColor } from "../../lib/theme";

  const s = $derived(app.summary);
  const accent = $derived(ACCENTS[app.settings.accent]);

  function hoursMinutes(secs: number): string {
    const h = Math.floor(secs / 3600);
    const m = Math.floor((secs % 3600) / 60);
    return `${h}h${String(m).padStart(2, "0")}m`;
  }

  /** `+12%` / `−4` use U+2212, matching the design. */
  function signed(n: number, suffix = ""): string {
    if (n > 0) return `+${n}${suffix}`;
    if (n < 0) return `−${Math.abs(n)}${suffix}`;
    return `${n}${suffix}`;
  }

  const cards = $derived(
    s
      ? [
          {
            name: "本周专注",
            value: hoursMinutes(s.weekFocusSecs),
            delta: `较上周 ${signed(s.weekDeltaPct, "%")}`,
            good: s.weekDeltaPct > 0,
          },
          {
            name: "完成番茄",
            value: String(s.pomodoros),
            delta: `日均 ${s.dailyAverage.toFixed(1)} 个`,
            good: false,
          },
          {
            name: "中断次数",
            value: String(s.interruptions),
            delta: `较上周 ${signed(s.interruptionsDelta)}`,
            good: s.interruptionsDelta < 0,
          },
          {
            name: "连续天数",
            value: String(s.streak),
            delta: `个人最佳 ${s.bestStreak}`,
            good: false,
          },
        ]
      : [],
  );
</script>

<div class="stats">
  <div class="cards">
    {#each cards as card (card.name)}
      <div class="card">
        <span class="cname">{card.name}</span>
        <span class="cvalue">{card.value}</span>
        <span class="cdelta" class:good={card.good}>{card.delta}</span>
      </div>
    {/each}
  </div>

  <div class="chart">
    <div class="chart-head">
      <span class="ctitle">最近两周的专注分布</span>
      <span class="ccaption">每格 = 一个番茄，颜色越深越连贯</span>
    </div>
    <div class="bars">
      {#each s?.bars ?? [] as bar, i (i)}
        <div class="bar">
          <div class="stack">
            {#if bar.count === 0}
              <span class="cell empty"></span>
            {:else}
              {#each Array.from({ length: bar.count }, (_, k) => k) as k (k)}
                <span class="cell" style:background={barCellColor(accent, k)}></span>
              {/each}
            {/if}
          </div>
          <span class="blabel">{bar.label}</span>
        </div>
      {/each}
    </div>
  </div>

  <div class="insights">
    <div class="insight">
      <span class="ititle">被打断最多的时段</span>
      <span class="ibody">
        15:00–16:00，平均每轮被打断 1.8 次。要不要把这段设成「勿扰 + 只留宠物提示」？
      </span>
    </div>
    <div class="insight">
      <span class="ititle">Momo 的评价</span>
      <span class="ibody">{petVerdict(app.tone)}</span>
    </div>
  </div>
</div>

<style>
  .stats {
    flex: 1;
    padding: 32px 40px 38px;
    display: flex;
    flex-direction: column;
    gap: 28px;
    overflow-y: auto;
  }
  .cards {
    display: flex;
    gap: 40px;
  }
  .card {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .cname {
    font-size: 12px;
    color: var(--dim);
  }
  .cvalue {
    font-family: var(--font-mono);
    font-size: 30px;
    font-weight: 500;
    letter-spacing: -0.02em;
  }
  .cdelta {
    font-size: 12px;
    color: var(--dim);
  }
  .cdelta.good {
    color: var(--good);
  }
  .chart {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .chart-head {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
  }
  .ctitle {
    font-size: 14px;
    font-weight: 600;
  }
  .ccaption {
    font-size: 12.5px;
    color: var(--dim);
  }
  .bars {
    display: flex;
    gap: 5px;
    align-items: flex-end;
    height: 148px;
  }
  .bar {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 7px;
  }
  .stack {
    width: 100%;
    height: 118px;
    display: flex;
    flex-direction: column;
    justify-content: flex-end;
    gap: 3px;
    overflow: hidden;
  }
  .cell {
    height: 13px;
    border-radius: 3px;
    flex: none;
  }
  .cell.empty {
    background: var(--line-soft);
  }
  .blabel {
    font-size: 11px;
    color: var(--faint);
  }
  .insights {
    display: flex;
    gap: 20px;
  }
  .insight {
    flex: 1;
    padding: 18px 20px;
    border: 1px solid oklch(0.9 0.008 70);
    border-radius: var(--radius-card);
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .ititle {
    font-size: 13px;
    font-weight: 600;
  }
  .ibody {
    font-size: 12.5px;
    color: oklch(0.53 0.012 60);
    line-height: 1.5;
  }
</style>
