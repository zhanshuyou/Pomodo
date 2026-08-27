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

export function petVerdict(t: Tone): string {
  return tone(
    t,
    "本周专注 14h20m，较上周 +12%，中断率下降。",
    "这周你比上周多专注了 1 小时 40 分，很稳。",
    "这周表现不错，我勉为其难地允许你今晚多睡半小时。",
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
