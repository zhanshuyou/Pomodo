import type { Rules } from "./ipc";

export const WEEKDAY_SHORT = ["一", "二", "三", "四", "五", "六", "日"] as const;

/** Minutes past midnight → `HH:MM`, the value an `<input type="time">` wants. */
export function minutesToTime(min: number): string {
  const m = Math.max(0, Math.min(24 * 60 - 1, Math.floor(min)));
  return `${String(Math.floor(m / 60)).padStart(2, "0")}:${String(m % 60).padStart(2, "0")}`;
}

/** `HH:MM` → minutes past midnight; anything unparsable gives `null`. */
export function timeToMinutes(value: string): number | null {
  const m = /^(\d{1,2}):(\d{2})$/.exec(value.trim());
  if (!m) return null;
  const h = Number(m[1]);
  const mm = Number(m[2]);
  if (h > 23 || mm > 59) return null;
  return h * 60 + mm;
}

/** 周一 – 周五 / 每天 / 从不 / a list like 周一、三、五. */
export function weekdaysLabel(days: readonly boolean[]): string {
  const on = days.map((d, i) => (d ? i : -1)).filter((i) => i >= 0);
  if (on.length === 7) return "每天";
  if (on.length === 0) return "从不";
  const weekdaysOnly = on.length === 5 && on.every((i) => i < 5);
  if (weekdaysOnly) return "周一 – 周五";
  const weekendOnly = on.length === 2 && on[0] === 5 && on[1] === 6;
  if (weekendOnly) return "周末";
  return "周" + on.map((i) => WEEKDAY_SHORT[i]).join("、");
}

/** `15:00–16:00`, for a QuietWindow row. */
export function quietLabel(fromMin: number, toMin: number): string {
  return `${minutesToTime(fromMin)}–${minutesToTime(toMin)}`;
}

export function escalationLabel(after: number): string {
  return after > 0 ? `忽略 ${after} 次后升级为全屏` : "不升级";
}

export function withWeekday(rules: Rules, index: number, on: boolean): Rules {
  const weekdays = rules.weekdays.slice();
  weekdays[index] = on;
  return { ...rules, weekdays };
}
