function pad(n: number): string {
  return String(n).padStart(2, "0");
}

/** `MM:SS`, where MM is not capped at 60. */
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
