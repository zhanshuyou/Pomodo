import { hoursMinutes, hoursMinutesCn } from "./format";
import { type Tone, tone } from "./theme";

export type Phase = "focus" | "shortBreak" | "longBreak";

export function tagline(t: Tone): string {
  return tone(
    t,
    "菜单栏与主窗口双入口的番茄计时器，含可自定义的身体提醒与桌面宠物。",
    "一个陪你专注的番茄钟：它记得提醒你站立喝水，也记得在你完成时替你高兴。",
    "它负责计时、催你喝水、盯你站起来，并在你摸鱼时用眼神谴责你。",
  );
}

export function petLine(t: Tone, minutes: number): string {
  return tone(
    t,
    `本轮剩余 ${minutes} 分钟。`,
    `再 ${minutes} 分钟就好，我陪着你。`,
    `还有 ${minutes} 分钟，我盯着你呢`,
  );
}

export interface VerdictInput {
  weekFocusSecs: number;
  /** Percent versus the previous seven days; 0 when there is no prior week. */
  weekDeltaPct: number;
  /** Interruptions this week minus last week. */
  interruptionsDelta: number;
}

/**
 * Pomodo 的评价 — the spec's sample sentences, with the numbers filled in from
 * the real week. The playful line for a good week is the spec's verbatim.
 */
export function petVerdict(t: Tone, s: VerdictInput): string {
  const total = hoursMinutes(s.weekFocusSecs);
  if (s.weekFocusSecs === 0) {
    return tone(
      t,
      "本周尚无专注记录。",
      "这周还没开始，没关系，从下一个番茄开始。",
      "这周你一个番茄都没啃，我可盯着呢。",
    );
  }
  const trend =
    s.interruptionsDelta < 0 ? "中断减少" : s.interruptionsDelta > 0 ? "中断增多" : "中断持平";
  if (s.weekDeltaPct === 0) {
    return tone(
      t,
      `本周专注 ${total}，与上周持平，${trend}。`,
      `这周专注了 ${hoursMinutesCn(s.weekFocusSecs)}，稳稳的。`,
      `这周专注了 ${hoursMinutesCn(s.weekFocusSecs)}，还行，不至于挨骂。`,
    );
  }
  // The absolute change, recovered from this week's total and the percentage.
  const prev = (s.weekFocusSecs * 100) / (100 + s.weekDeltaPct);
  const diff = hoursMinutesCn(Math.abs(s.weekFocusSecs - prev));
  if (s.weekDeltaPct > 0) {
    return tone(
      t,
      `本周专注 ${total}，较上周 +${s.weekDeltaPct}%，${trend}。`,
      `这周你比上周多专注了 ${diff}，很稳。`,
      "这周表现不错，我勉为其难地允许你今晚多睡半小时。",
    );
  }
  return tone(
    t,
    `本周专注 ${total}，较上周 −${Math.abs(s.weekDeltaPct)}%，${trend}。`,
    `这周比上周少专注了 ${diff}，下周慢慢补回来。`,
    `这周比上周少了 ${diff}，我不评价，但我记下了。`,
  );
}

/**
 * The line under the round dots. `left` is how many rounds remain after the
 * current one (roundsPerCycle − round); the artboard's round 2 of 4 gives the
 * spec's "再 2 轮就能哄它去睡长觉（15 分钟）".
 */
export function roundsUntilLongBreak(
  t: Tone,
  phase: Phase,
  left: number,
  longBreakMins: number,
): string {
  const m = `${longBreakMins} 分钟`;
  if (phase === "longBreak") {
    return tone(t, `长休息中（${m}）。`, `好好歇着，${m}都是你的。`, `它在睡长觉，${m}内别吵。`);
  }
  if (phase === "shortBreak") {
    return tone(
      t,
      `短休息中，之后还有 ${left} 轮进入长休息。`,
      `歇一下，之后还有 ${left} 轮就能歇个长的。`,
      `喘口气，还有 ${left} 轮才能哄它去睡长觉`,
    );
  }
  if (left <= 0) {
    return tone(
      t,
      `本轮结束后进入长休息（${m}）。`,
      `这轮结束就能歇个长的（${m}）。`,
      `这轮结束就能哄它去睡长觉（${m}）`,
    );
  }
  return tone(
    t,
    `再 ${left} 轮进入长休息（${m}）。`,
    `再 ${left} 轮就能歇个长的（${m}）。`,
    `再 ${left} 轮就能哄它去睡长觉（${m}）`,
  );
}

/** The 稍后 affordance on every reminder surface. */
export function snoozeLabel(t: Tone, minutes: number): string {
  return tone(
    t,
    `${minutes} 分钟后再提醒`,
    `过 ${minutes} 分钟再叫我`,
    `再赖 ${minutes} 分钟`,
  );
}

export function phaseLabel(phase: Phase): string {
  return phase === "focus" ? "专注中" : "休息中";
}

export function runLabel(running: boolean): string {
  return running ? "让它歇会儿" : "开始专注";
}

export function miniLabel(mini: boolean): string {
  return mini ? "退出迷你模式" : "迷你模式";
}
