<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<!-- The bottom panel's Log tab (ADR-0029 §3): this session's debug log, live, newest at the
     bottom (following the tail unless scrolled up), or the log file's earlier sessions.
     Filters by level and category; search marks matches; steps through matches and problems;
     copies or saves the rows shown, as the file writes them. -->
<script lang="ts">
  import { keys } from "./keys";
  import { save as saveDialog } from "@tauri-apps/plugin-dialog";
  import { tick } from "svelte";
  import { api } from "./api";
  import { logFeed } from "./logFeed.svelte";
  import { logAction } from "./log";
  import {
    LEVEL_FILTERS,
    categoriesOf,
    countProblems,
    filterRows,
    formatRows,
    matchIndices,
    parseFile,
    problemIndices,
    rowOf,
    segments,
    step,
    timeOf,
    type LevelFilter,
    type LogRow,
  } from "./logView";

  let source = $state<"live" | "file">("live");
  let fileRows = $state.raw<LogRow[]>([]);
  let filePath = $state("");
  let level = $state<LevelFilter>("info");
  let cats = $state.raw(new Set<string>());
  let catMenu = $state(false);
  let query = $state("");
  /** The row stepped to (by match or problem), as an index into `shown`. */
  let current = $state(-1);
  let follow = $state(true);
  let note = $state<string | null>(null);
  let scroller = $state<HTMLElement | undefined>();

  const liveRows = $derived(logFeed.records.map(rowOf));
  const rows = $derived(source === "live" ? liveRows : fileRows);
  const categories = $derived(categoriesOf(rows));
  const shown = $derived(filterRows(rows, { level, categories: cats }));
  const matches = $derived(matchIndices(shown, query));
  const problems = $derived(problemIndices(shown));
  const counts = $derived(countProblems(rows));
  const matchNo = $derived(matches.indexOf(current));

  // Follow the tail: scroll to the newest row when rows arrive, unless the reader scrolled up.
  $effect(() => {
    void shown.length;
    if (!follow || !scroller) return;
    void tick().then(() => scroller && (scroller.scrollTop = scroller.scrollHeight));
  });

  // A new search or filter starts stepping afresh.
  $effect(() => {
    void [query, level, cats, source];
    current = -1;
  });

  function scrolled() {
    if (!scroller) return;
    follow = scroller.scrollTop + scroller.clientHeight >= scroller.scrollHeight - 8;
  }

  async function goTo(i: number) {
    if (i < 0) return;
    current = i;
    follow = false;
    await tick();
    scroller?.querySelector(`[data-i="${i}"]`)?.scrollIntoView({ block: "nearest" });
  }

  function search(e: KeyboardEvent) {
    if (e.key !== "Enter") return;
    e.preventDefault();
    void goTo(step(matches, current, e.shiftKey ? -1 : 1));
  }

  function toggleCat(c: string) {
    const s = new Set(cats);
    if (s.has(c)) s.delete(c);
    else s.add(c);
    cats = s;
  }

  async function earlier() {
    note = null;
    if (source === "file") {
      source = "live";
      follow = true;
      return;
    }
    logAction("Log: earlier sessions");
    try {
      const [path, text] = await api.logFile();
      fileRows = parseFile(text);
      filePath = path;
      source = "file";
      follow = true;
    } catch (e) {
      note = String(e);
    }
  }

  async function copy() {
    try {
      await navigator.clipboard.writeText(formatRows(shown));
      note = `Copied ${shown.length} ${shown.length === 1 ? "line" : "lines"}.`;
    } catch (e) {
      note = `Couldn't copy: ${e}`;
    }
  }

  async function saveAs() {
    note = null;
    try {
      const path = await saveDialog({
        title: "Save the debug log",
        defaultPath: source === "file" ? "cruncha-log.txt" : "cruncha-log-session.txt",
        filters: [{ name: "Text", extensions: ["txt", "log"] }],
      });
      if (!path) return;
      logAction("Log: save as");
      await api.logSave(path, formatRows(shown));
      note = `Saved ${shown.length} ${shown.length === 1 ? "line" : "lines"}.`;
    } catch (e) {
      note = String(e);
    }
  }

  async function reveal() {
    note = null;
    try {
      await api.logReveal();
    } catch (e) {
      note = String(e);
    }
  }

  function clearView() {
    logAction("Log: clear view");
    if (source === "file") source = "live";
    logFeed.clear();
    follow = true;
  }
</script>

<div class="log">
  <div class="tools">
    <select bind:value={level} aria-label="Level">
      {#each LEVEL_FILTERS as f (f.value)}<option value={f.value}>{f.label}</option>{/each}
    </select>
    <div class="cats">
      <button aria-haspopup="true" aria-expanded={catMenu} onclick={() => (catMenu = !catMenu)} title="Show only some categories">
        {cats.size ? `${cats.size} ${cats.size === 1 ? "category" : "categories"}` : "All categories"} ▾
      </button>
      {#if catMenu}
        <div class="menu">
          <label><input type="checkbox" checked={cats.size === 0} onchange={() => (cats = new Set())} /> All</label>
          {#each categories as c (c)}
            <label><input type="checkbox" checked={cats.has(c)} onchange={() => toggleCat(c)} /> {c}</label>
          {/each}
          {#if !categories.length}<span class="muted small">None yet</span>{/if}
        </div>
      {/if}
    </div>
    <input type="search" placeholder={keys("Search (↵ next, ⇧↵ previous)")} bind:value={query} onkeydown={search} aria-label="Search the log" />
    {#if query}
      <span class="muted small">{matches.length ? `${matchNo >= 0 ? matchNo + 1 : "–"} of ${matches.length}` : "no matches"}</span>
      <button class="sm" disabled={!matches.length} onclick={() => goTo(step(matches, current, -1))} title={keys("Previous match (⇧↵)")} aria-label="Previous match">↑</button>
      <button class="sm" disabled={!matches.length} onclick={() => goTo(step(matches, current, 1))} title="Next match (↵)" aria-label="Next match">↓</button>
    {/if}
    <span class="sep"></span>
    <button class="sm" disabled={!problems.length} onclick={() => goTo(step(problems, current, -1))} title="Previous error or warning">‹ Problem</button>
    <button class="sm" disabled={!problems.length} onclick={() => goTo(step(problems, current, 1))} title="Next error or warning">Problem ›</button>
    <span class="counts small" title="Errors and warnings in {source === 'live' ? 'this session' : 'the file'}">
      <span class:err={counts.errors > 0}>{counts.errors} {counts.errors === 1 ? "error" : "errors"}</span>,
      <span class:warn={counts.warnings > 0}>{counts.warnings} {counts.warnings === 1 ? "warning" : "warnings"}</span>
    </span>
    <span class="spacer"></span>
    <button class="sm" onclick={copy} disabled={!shown.length} title="Copy the lines shown">Copy</button>
    <button class="sm" onclick={saveAs} disabled={!shown.length} title="Save the lines shown to a file">Save as…</button>
    <button class="sm" onclick={reveal} title="Show the log file in Finder">Open folder</button>
    <button class="sm" onclick={clearView} title="Clear this list (the log file is kept)">Clear view</button>
    <button class="sm" class:on={source === "file"} aria-pressed={source === "file"} onclick={earlier}
      title={source === "file" ? "Back to this session's live log" : "Read the log file: earlier sessions, including a crashed one's last lines"}>
      {source === "file" ? "This session" : "Earlier sessions"}
    </button>
  </div>
  {#if note || source === "file"}
    <div class="note small muted">
      {#if source === "file"}<span title={filePath}>The log file (read-only): {filePath}</span>{/if}
      {#if note}<span>{note}</span>{/if}
    </div>
  {/if}
  <div class="rows" bind:this={scroller} onscroll={scrolled} role="log" aria-live="off">
    {#each shown as r, i (r.key)}
      <div class="row {r.level ?? 'marker'}" class:cur={i === current} data-i={i}>
        {#if r.level === null}
          <span class="msg">{#each segments(r.message, query) as s, k (k)}{#if s.hit}<mark>{s.text}</mark>{:else}{s.text}{/if}{/each}</span>
        {:else}
          <span class="at" title={r.at}>{timeOf(r.at)}</span>
          <span class="lvl">{r.level.toUpperCase()}</span>
          <span class="cat">{#each segments(r.category, query) as s, k (k)}{#if s.hit}<mark>{s.text}</mark>{:else}{s.text}{/if}{/each}</span>
          <span class="msg">{#each segments(r.message, query) as s, k (k)}{#if s.hit}<mark>{s.text}</mark>{:else}{s.text}{/if}{/each}</span>
        {/if}
      </div>
    {:else}
      <p class="muted small empty">{rows.length ? "No records at this level or in these categories." : "No records yet."}</p>
    {/each}
  </div>
  {#if !follow && source === "live"}
    <button class="tail" onclick={() => { follow = true; if (scroller) scroller.scrollTop = scroller.scrollHeight; }}>↓ Follow the newest</button>
  {/if}
</div>

<style>
  .log { position: relative; display: grid; grid-template-rows: auto auto 1fr; height: 100%; min-height: 0; }
  .tools { display: flex; gap: 6px; align-items: center; padding: 5px 10px; flex-wrap: wrap; }
  .tools input[type="search"] { flex: 0 1 260px; min-width: 120px; }
  .tools select { font-size: 12.5px; }
  .sm { font-size: 12px; padding: 2px 8px; }
  .sm.on { border-color: var(--accent); color: var(--accent); }
  .sep { width: 1px; align-self: stretch; background: var(--border); margin: 2px 4px; }
  .spacer { flex: 1; }
  .counts { color: var(--muted); white-space: nowrap; }
  .counts .err { color: var(--danger); font-weight: 600; }
  .counts .warn { color: var(--warn); font-weight: 600; }
  .cats { position: relative; }
  .cats > button { font-size: 12.5px; padding: 2px 8px; }
  .menu {
    position: absolute; left: 0; bottom: calc(100% + 4px); z-index: 20; min-width: 180px; max-height: 260px; overflow: auto;
    display: grid; gap: 2px; padding: 6px 8px; background: var(--panel); border: 1px solid var(--border);
    border-radius: 8px; box-shadow: 0 8px 24px rgb(0 0 0 / 0.14); font: 12px var(--mono);
  }
  .menu label { display: flex; gap: 6px; align-items: center; white-space: nowrap; }
  .note { display: flex; gap: 12px; padding: 0 10px 4px; overflow: hidden; white-space: nowrap; text-overflow: ellipsis; }
  .rows { overflow: auto; min-height: 0; font: 11.5px/1.45 var(--mono); border-top: 1px solid var(--border); padding: 2px 0; }
  .row {
    display: grid; grid-template-columns: 9.5em 4.5em minmax(6em, max-content) 1fr; gap: 0 10px;
    padding: 0 10px; border-bottom: 1px solid color-mix(in srgb, var(--border) 35%, transparent);
  }
  .row:hover { background: var(--sunken); }
  .row.cur { background: var(--select); box-shadow: inset 2px 0 0 var(--select-border); }
  .row.marker { grid-template-columns: 1fr; color: var(--muted); font-style: italic; }
  .at { color: var(--muted); }
  .cat { color: var(--muted); }
  .msg { white-space: pre-wrap; word-break: break-word; min-width: 0; }
  .lvl { font-weight: 600; color: var(--muted); }
  .row.debug { color: var(--muted); }
  .row.error .lvl, .row.error .msg { color: var(--danger); }
  .row.error { background: color-mix(in srgb, var(--danger-bg) 60%, transparent); }
  .row.warn .lvl, .row.warn .msg { color: var(--warn); }
  mark { background: color-mix(in srgb, var(--accent) 30%, transparent); color: inherit; border-radius: 2px; }
  .row.cur mark { background: color-mix(in srgb, var(--accent) 55%, transparent); }
  .empty { padding: 8px 12px; margin: 0; }
  .tail { position: absolute; right: 18px; bottom: 10px; font-size: 12px; padding: 2px 10px; box-shadow: 0 2px 8px rgb(0 0 0 / 0.15); }
</style>
