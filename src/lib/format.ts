function pad(n: number): string {
  return String(n).padStart(2, "0");
}

/** `MM:SS`, where MM is not capped at 60. */
/** `14h20m`, the stat-card style. */
export function hoursMinutes(secs: number): string {
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  return `${h}h${String(m).padStart(2, "0")}m`;
}

/** `1 小时 40 分` / `40 分钟`, for running text. */
export function hoursMinutesCn(secs: number): string {
  const h = Math.floor(secs / 3600);
  const m = Math.round((secs % 3600) / 60);
  if (h === 0) return `${m} 分钟`;
  if (m === 0) return `${h} 小时`;
  return `${h} 小时 ${m} 分`;
}

export function mmss(totalSecs: number): string {
  const secs = Math.max(0, Math.floor(totalSecs));
  return `${pad(Math.floor(secs / 60))}:${pad(secs % 60)}`;
}

/** Whole minutes remaining, as interpolated into the pet's line. */
export function minutesLeft(totalSecs: number): number {
  return Math.floor(Math.max(0, totalSecs) / 60);
}

/** The design's `预计 HH:MM 结束` line under the countdown. */
export function endsAt(remainingSecs: number, now: Date = new Date()): string {
  const end = new Date(now.getTime() + Math.max(0, remainingSecs) * 1000);
  return `预计 ${pad(end.getHours())}:${pad(end.getMinutes())} 结束`;
}
