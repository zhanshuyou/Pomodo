<script module lang="ts">
  // Shared by the sidebar's own dashed row and the ⌘N shortcut in App.svelte.
  // The editor is inline rather than window.prompt(): wry's WKWebView has no
  // UI delegate for JS prompts, so prompt() silently returns null on macOS.
  let adding = $state(false);

  export function beginAdd() {
    adding = true;
  }
</script>

<script lang="ts">
  import StatBar from "../../lib/components/StatBar.svelte";
  import { emptyTasks } from "../../lib/copy";
  import {
    addTask,
    deleteTask,
    renameTask,
    reorderTasks,
    setActiveTask,
    setTaskEstimate,
    toggleTask,
  } from "../../lib/ipc";
  import { app } from "../../lib/state.svelte";
  import { REMINDER_COLORS } from "../../lib/theme";

  /** The artboard draws at most three pips, so that is the ceiling here too. */
  const MAX_PIPS = 3;

  let draft = $state("");
  let draftEstimate = $state(1);
  let input = $state<HTMLInputElement | null>(null);

  /** The task whose name is being edited in place, if any. */
  let renamingId = $state<number | null>(null);
  let renameDraft = $state("");
  let renameInput = $state<HTMLInputElement | null>(null);

  $effect(() => {
    if (renamingId !== null) renameInput?.focus();
  });

  function beginRename(task: { id: number; name: string }) {
    renamingId = task.id;
    renameDraft = task.name;
  }

  function cancelRename() {
    renamingId = null;
    renameDraft = "";
  }

  function submitRename() {
    if (renamingId === null) return;
    const id = renamingId;
    const name = renameDraft.trim();
    renamingId = null;
    renameDraft = "";
    if (name) void renameTask(id, name);
  }

  function onRenameKey(event: KeyboardEvent) {
    // The row underneath also listens for Enter (select) — keep it out of this.
    event.stopPropagation();
    if (event.key === "Enter") {
      event.preventDefault();
      submitRename();
    } else if (event.key === "Escape") {
      event.preventDefault();
      cancelRename();
    }
  }

  function move(id: number, delta: -1 | 1) {
    const ids = app.tasks.map((t) => t.id);
    const from = ids.indexOf(id);
    const to = from + delta;
    if (from < 0 || to < 0 || to >= ids.length) return;
    ids.splice(from, 1);
    ids.splice(to, 0, id);
    void reorderTasks(ids);
  }

  $effect(() => {
    if (adding) input?.focus();
  });

  function cancelAdd() {
    adding = false;
    draft = "";
    draftEstimate = 1;
  }

  async function submitAdd() {
    const name = draft.trim();
    const estimate = draftEstimate;
    if (!name) {
      cancelAdd();
      return;
    }
    cancelAdd();
    await addTask(name, estimate);
  }

  function onDraftKey(event: KeyboardEvent) {
    if (event.key === "Enter") {
      event.preventDefault();
      void submitAdd();
    } else if (event.key === "Escape") {
      event.preventDefault();
      cancelAdd();
    }
  }

  /** Picking the task already in focus lets it go — there is no other way to
   *  focus on nothing in particular. */
  function pick(id: number) {
    return setActiveTask(app.timer.activeTask === id ? null : id);
  }

  // Driven by the reminder engine: acknowledging 喝水 / 站立 moves these.
  const bodyStats = $derived([
    {
      name: "喝水",
      value: `${app.body.waterCups} / ${app.body.waterGoal} 杯`,
      pct: (app.body.waterCups / Math.max(1, app.body.waterGoal)) * 100,
      color: REMINDER_COLORS.water,
    },
    {
      name: "站立",
      value: `${app.body.stands} / ${app.body.standGoal} 次`,
      pct: (app.body.stands / Math.max(1, app.body.standGoal)) * 100,
      color: REMINDER_COLORS.stand,
    },
    {
      name: "久坐最长",
      value: `${app.body.longestSitMins} 分钟`,
      pct: Math.min(
        100,
        (app.body.longestSitMins / Math.max(1, app.body.sitGoalMins)) * 100,
      ),
      color: REMINDER_COLORS.stretch,
    },
  ]);

  const doneCount = $derived(app.tasks.filter((t) => t.done).length);

  function meta(task: { estimate: number; spent: number; done: boolean }): string {
    if (task.done) return `已完成 · ${task.spent} 个番茄`;
    if (task.spent > 0) return `进行中 · 已投入 ${task.spent} 个番茄`;
    return `预计 ${task.estimate} 个番茄`;
  }
</script>

<aside class="sidebar">
  <header>
    <span class="title">今天要啃的</span>
    <span class="count">{doneCount} / {app.tasks.length} 完成</span>
  </header>

  <div class="list">
    {#if app.tasks.length === 0 && !adding}
      <button class="empty" type="button" onclick={beginAdd}>{emptyTasks(app.tone)}</button>
    {/if}
    {#each app.tasks as task, index (task.id)}
      <div
        class="task"
        class:selected={app.timer.activeTask === task.id && !task.done}
        role="button"
        tabindex="0"
        onclick={() => void pick(task.id)}
        onkeydown={(e) => e.key === "Enter" && void pick(task.id)}
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
          {#if renamingId === task.id}
            <input
              class="rename"
              bind:this={renameInput}
              bind:value={renameDraft}
              type="text"
              aria-label="任务名称"
              onclick={(e) => e.stopPropagation()}
              onkeydown={onRenameKey}
              onblur={submitRename}
            />
          {:else}
            <!-- The row is already the button; the name only adds a double-click. -->
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <span
              class="name"
              class:done={task.done}
              title="双击改名"
              ondblclick={(e) => {
                e.stopPropagation();
                beginRename(task);
              }}
            >
              {task.name}
            </span>
          {/if}
          <span class="meta">{meta(task)}</span>
        </div>

        <!-- One pip per estimated pomodoro; click a pip to re-estimate. -->
        <div class="pips" role="group" aria-label="预计番茄数">
          {#each [0, 1, 2] as i (i)}
            {#if i < Math.min(task.estimate, MAX_PIPS) || i === Math.min(task.estimate, MAX_PIPS)}
              <button
                class="pip"
                class:on={i < Math.min(task.spent, task.estimate)}
                class:ghost={i >= task.estimate}
                type="button"
                aria-label="预计 {i + 1} 个番茄"
                title="预计 {i + 1} 个番茄"
                onclick={(e) => {
                  e.stopPropagation();
                  void setTaskEstimate(task.id, i + 1);
                }}
              ></button>
            {/if}
          {/each}
        </div>

        <div class="rowacts">
          <button
            class="act"
            type="button"
            aria-label="上移"
            disabled={index === 0}
            onclick={(e) => {
              e.stopPropagation();
              move(task.id, -1);
            }}>↑</button
          >
          <button
            class="act"
            type="button"
            aria-label="下移"
            disabled={index === app.tasks.length - 1}
            onclick={(e) => {
              e.stopPropagation();
              move(task.id, 1);
            }}>↓</button
          >
          <button
            class="act del"
            type="button"
            aria-label="删除{task.name}"
            title="删除"
            onclick={(e) => {
              e.stopPropagation();
              void deleteTask(task.id);
            }}>×</button
          >
        </div>
      </div>
    {/each}
  </div>

  {#if adding}
    <div class="add-row">
      <input
        class="add-input"
        bind:this={input}
        bind:value={draft}
        type="text"
        placeholder="要啃什么？回车添加，⎋ 取消"
        aria-label="新任务名称"
        onkeydown={onDraftKey}
        onblur={() => void submitAdd()}
      />
      <div class="pips" role="group" aria-label="预计番茄数">
        {#each [1, 2, 3] as n (n)}
          <button
            class="pip"
            class:on={n <= draftEstimate}
            type="button"
            aria-label="预计 {n} 个番茄"
            onpointerdown={(e) => e.preventDefault()}
            onclick={() => (draftEstimate = n)}
          ></button>
        {/each}
      </div>
    </div>
  {:else}
    <button class="add" type="button" onclick={beginAdd}>＋ 加一件事（⌘N）</button>
  {/if}

  <div class="body-stats">
    <span class="label">身体这边的账</span>
    {#each bodyStats as stat (stat.name)}
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
    border: none;
    padding: 0;
    cursor: pointer;
  }
  .pip.on {
    background: var(--accent);
  }
  /* The next pip up, faint, so there is something to click to raise the estimate. */
  .pip.ghost {
    opacity: 0;
  }
  .task:hover .pip.ghost {
    opacity: 0.35;
  }
  .rename {
    padding: 2px 6px;
    border: 1px solid var(--accent);
    border-radius: 5px;
    background: var(--card);
    font: inherit;
    font-size: 13.5px;
    color: var(--ink);
    outline: none;
    min-width: 0;
  }
  .rowacts {
    display: flex;
    gap: 2px;
    opacity: 0;
    transition: opacity 0.15s ease;
  }
  .task:hover .rowacts,
  .task:focus-within .rowacts {
    opacity: 1;
  }
  .act {
    width: 20px;
    height: 20px;
    border: none;
    border-radius: 5px;
    background: transparent;
    color: var(--faint);
    font-size: 12px;
    line-height: 1;
    cursor: pointer;
    padding: 0;
  }
  .act:hover:not(:disabled) {
    background: oklch(0.94 0.008 70);
    color: var(--ink);
  }
  .act:disabled {
    opacity: 0.3;
    cursor: default;
  }
  .act.del:hover {
    color: oklch(0.55 0.15 25);
  }
  .add-row {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .add-row .add-input {
    flex: 1;
    min-width: 0;
  }
  .empty {
    padding: 18px 13px;
    border: none;
    border-radius: var(--radius-control);
    background: oklch(0.97 0.006 70);
    color: var(--dim);
    font-size: 13px;
    text-align: center;
    cursor: pointer;
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
  .add-input {
    padding: 10px 13px;
    border: 1px solid var(--accent);
    border-radius: var(--radius-control);
    background: var(--card);
    font: inherit;
    font-size: 13px;
    color: var(--ink);
    outline: none;
  }
  .add-input::placeholder {
    color: var(--faint);
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
