<script lang="ts">
  import StatBar from "../../lib/components/StatBar.svelte";
  import { addTask, setActiveTask, toggleTask } from "../../lib/ipc";
  import { app } from "../../lib/state.svelte";

  // Placeholder until plan 05 wires the reminder engine's counters.
  const BODY_STATS = [
    { name: "喝水", value: "6 / 8 杯", pct: 75, color: "oklch(0.66 0.09 195)" },
    { name: "站立", value: "4 / 6 次", pct: 66, color: "oklch(0.63 0.13 40)" },
    { name: "久坐最长", value: "68 分钟", pct: 76, color: "oklch(0.7 0.12 60)" },
  ];

  const doneCount = $derived(app.tasks.filter((t) => t.done).length);

  function meta(task: { estimate: number; spent: number; done: boolean }): string {
    if (task.done) return `已完成 · ${task.spent} 个番茄`;
    if (task.spent > 0) return `进行中 · 已投入 ${task.spent} 个番茄`;
    return `预计 ${task.estimate} 个番茄`;
  }

  async function onAdd() {
    const name = window.prompt("要啃什么？");
    if (!name?.trim()) return;
    await addTask(name.trim(), 1);
  }
</script>

<aside class="sidebar">
  <header>
    <span class="title">今天要啃的</span>
    <span class="count">{doneCount} / {app.tasks.length} 完成</span>
  </header>

  <div class="list">
    {#each app.tasks as task (task.id)}
      <div
        class="task"
        class:selected={app.timer.activeTask === task.id && !task.done}
        role="button"
        tabindex="0"
        onclick={() => void setActiveTask(task.id)}
        onkeydown={(e) => e.key === "Enter" && void setActiveTask(task.id)}
      >
        <button
          class="box"
          class:checked={task.done}
          type="button"
          aria-label={task.done ? "标记为未完成" : "标记为完成"}
          onclick={(e) => {
            e.stopPropagation();
            void toggleTask(task.id);
          }}
        >
          <span class="tick"></span>
        </button>

        <div class="text">
          <span class="name" class:done={task.done}>{task.name}</span>
          <span class="meta">{meta(task)}</span>
        </div>

        <div class="pips">
          {#each [0, 1, 2] as i (i)}
            <span class="pip" class:on={i < Math.min(task.spent, 3)}></span>
          {/each}
        </div>
      </div>
    {/each}
  </div>

  <button class="add" type="button" onclick={onAdd}>＋ 加一件事（⌘N）</button>

  <div class="body-stats">
    <span class="label">身体这边的账</span>
    {#each BODY_STATS as stat (stat.name)}
      <StatBar name={stat.name} value={stat.value} pct={stat.pct} color={stat.color} />
    {/each}
  </div>
</aside>

<style>
  .sidebar {
    width: 372px;
    flex: none;
    border-left: 1px solid oklch(0.91 0.008 70);
    padding: 26px 24px;
    display: flex;
    flex-direction: column;
    gap: 16px;
    overflow-y: auto;
  }
  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .title {
    font-size: 14px;
    font-weight: 600;
  }
  .count {
    font-size: 12.5px;
    color: var(--dim);
  }
  .list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .task {
    display: flex;
    align-items: center;
    gap: 11px;
    padding: 11px 13px;
    border: 1px solid var(--line);
    border-radius: var(--radius-control);
    background: var(--card);
    cursor: pointer;
  }
  .task.selected {
    border-color: var(--accent);
    background: oklch(0.975 0.008 70);
  }
  .box {
    width: 17px;
    height: 17px;
    flex: none;
    border: 1.5px solid oklch(0.82 0.008 70);
    border-radius: 5px;
    background: var(--card);
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    padding: 0;
  }
  .box.checked {
    border-color: var(--accent);
    background: var(--accent);
  }
  .tick {
    width: 7px;
    height: 7px;
    border-radius: 1px;
    background: transparent;
  }
  .box.checked .tick {
    background: var(--card);
  }
  .text {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }
  .name {
    font-size: 13.5px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .name.done {
    text-decoration: line-through;
    color: oklch(0.62 0.012 60);
  }
  .meta {
    font-size: 11.5px;
    color: var(--faint);
  }
  .pips {
    display: flex;
    gap: 3px;
  }
  .pip {
    width: 7px;
    height: 7px;
    background: var(--track);
  }
  .pip.on {
    background: var(--accent);
  }
  .add {
    padding: 10px 13px;
    border: 1px dashed oklch(0.85 0.008 70);
    border-radius: var(--radius-control);
    background: transparent;
    font-size: 13px;
    color: var(--dim);
    cursor: pointer;
    text-align: left;
  }
  .add:hover {
    background: oklch(0.97 0.006 70);
  }
  .body-stats {
    margin-top: auto;
    padding-top: 16px;
    border-top: 1px solid var(--line-soft);
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .label {
    font-size: 11px;
    letter-spacing: 0.14em;
    text-transform: uppercase;
    color: var(--faint);
  }
</style>
