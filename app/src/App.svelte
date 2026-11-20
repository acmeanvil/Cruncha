<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { open as openDialog, save as saveDialog } from "@tauri-apps/plugin-dialog";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";
  import {
    api,
    DEFAULT_TEXT_SIZES,
    KEEP_EVERYTHING,
    type Cell,
    type CellResult,
    type CommitView,
    type DocInfo,
    type Prefs,
    type PythonStatus,
    type ResultsEvent,
    type Selection,
    type SetupProgress,
    type VersionView,
  } from "./lib/api";
  import DocumentSettings from "./lib/DocumentSettings.svelte";
  import Editor from "./lib/Editor.svelte";
  import Sidebar from "./lib/Sidebar.svelte";
  import Preferences from "./lib/Preferences.svelte";
  import SnapshotDialog from "./lib/SnapshotDialog.svelte";
  import { date, describeRetention } from "./lib/format";
  import type { SaveState } from "./lib/editor/sync";
  import type { OutlineItem } from "./lib/editor/outline";
  import type { Variable } from "./lib/editor/variables";

  let prefs = $state<Prefs>({ defaultRetention: KEEP_EVERYTHING });
  const sizes = $derived(prefs.textSizes ?? DEFAULT_TEXT_SIZES);
  let doc = $state<DocInfo | null>(null);
  let history = $state<VersionView[]>([]);
  let selected = $state<Selection>({ kind: "latest" });
  let cells = $state.raw<Cell[]>([]);
  let results = $state.raw<Record<string, CellResult>>({});
  let loads = $state(0);
  let save = $state<SaveState>({ status: "saved", error: null });
  let modal = $state<"prefs" | "doc" | "snapshot" | null>(null);
  let error = $state<string | null>(null);
  let editor = $state<ReturnType<typeof Editor> | undefined>();
  let outline = $state.raw<OutlineItem[]>([]);
  let variables = $state.raw<Variable[]>([]);
  let sidebar = $state(true);
  let fileMenu = $state(false);
  let barH = $state(0);
  let python = $state<PythonStatus | null>(null);
  let setup = $state<string | null>(null);

  const SETUP_LABEL: Record<SetupProgress["step"], string> = {
    installingUv: "Installing uv…",
    installingPython: "Installing Python 3.13…",
    installingPackages: "Installing NumPy, SciPy, Polars, pint, SymPy, matplotlib…",
    done: "Python is ready",
  };

  async function refreshPython() {
    python = doc ? await api.pythonStatus().catch(() => null) : null;
  }

  const setUpPython = () =>
    attempt(async () => {
      setup = "Starting…";
      try {
        const r = await api.setupPython();
        editor?.applyResults(r);
      } finally {
        setup = null;
        await refreshPython();
      }
    });

  const trustDocument = () =>
    attempt(async () => {
      editor?.applyResults(await api.trustDocument());
      await refreshPython();
    });

  const restartPython = () => attempt(async () => editor?.applyResults(await api.restartPython()));

  const loadKey = $derived(`${doc?.id ?? ""}:${loads}`);
  const editable = $derived(selected.kind === "latest");

  async function attempt(f: () => Promise<void>) {
    error = null;
    try {
      await f();
    } catch (e) {
      error = String(e);
    }
  }

  /** Save pending edits before the editor shows something else. */
  async function flush() {
    await editor?.flush();
  }

  async function show(s: Selection) {
    await attempt(async () => {
      await flush();
      const at = s.kind === "ref" ? { ref: s.name } : { version: s.kind === "version" ? s.version : null };
      const [c, r] = await Promise.all([
        s.kind === "ref" ? api.cellsAtRef(s.name) : api.cells(s.kind === "version" ? s.version : null),
        api.results(at),
      ]);
      cells = c;
      results = r;
      selected = s;
      loads += 1;
    });
  }

  async function load(info: DocInfo | null) {
    doc = info;
    history = info ? await api.history() : [];
    if (info) await show({ kind: "latest" });
    await refreshPython();
    prefs = await api.getPrefs(); // the recent list changed
  }

  const openRecent = (path: string) =>
    attempt(async () => {
      await flush();
      try {
        await load(await api.openDocument(path));
      } catch (e) {
        await api.forgetRecent(path);
        prefs = await api.getPrefs();
        throw `Couldn't open ${path}: ${e}`;
      }
    });

  const fileName = (p: string) => p.split(/[\\/]/).pop() ?? p;
  const folderOf = (p: string) => p.slice(0, Math.max(0, p.length - fileName(p).length - 1));

  onMount(() => {
    void attempt(async () => {
      prefs = await api.getPrefs();
      await load(await api.documentInfo());
    });
    // Save pending edits before the window closes (the backend then releases the document).
    const unlisten = getCurrentWindow().onCloseRequested(async () => {
      await flush();
    });
    // Results computed in the background (Python cells finished). Older versions' are ignored.
    const unResults = listen<ResultsEvent>("results", (e) => {
      const ev = e.payload;
      if (!doc || ev.docId !== doc.id || selected.kind !== "latest" || ev.version < doc.version) return;
      editor?.applyResults(ev.results);
    });
    const unSetup = listen<SetupProgress>("python-setup", (e) => (setup = SETUP_LABEL[e.payload.step]));
    return () => {
      void unlisten.then((f) => f());
      void unResults.then((f) => f());
      void unSetup.then((f) => f());
    };
  });

  const openDocument = () =>
    attempt(async () => {
      await flush();
      const path = await openDialog({ directory: true, title: "Open a Cruncha document (.cruncha)" });
      if (typeof path === "string") await load(await api.openDocument(path));
    });

  const newDocument = () =>
    attempt(async () => {
      await flush();
      const path = await saveDialog({ title: "New Cruncha document", defaultPath: "Untitled.cruncha" });
      if (path) await load(await api.createDocument(path));
    });

  const createSample = () =>
    attempt(async () => {
      await flush();
      const path = await saveDialog({ title: "Create a sample document", defaultPath: "beam-check.cruncha" });
      if (path) await load(await api.createSampleDocument(path));
    });

  async function committed(v: CommitView) {
    doc = v.info;
    history = await api.history();
    if (!python?.hasPython) await refreshPython();
  }

  /** Snapshot the latest version (pending edits are saved first, so they're included). */
  async function takeSnapshot(name: string, note: string) {
    await flush();
    doc = await api.snapshot(name, note);
  }

  /** Make the version on screen the latest again (a new version; it can be undone the same way). */
  const restoreShown = () =>
    attempt(async () => {
      const s = selected;
      if (s.kind === "latest") return;
      const v = await api.restore(s.kind === "ref" ? { ref: s.name } : { version: s.version });
      doc = v.info;
      history = await api.history();
      await show({ kind: "latest" });
    });

  // The open document may follow the default, so its effective policy can change.
  const prefsSaved = (p: Prefs) =>
    attempt(async () => {
      prefs = p;
      doc = await api.documentInfo();
    });

  const heading = $derived.by(() => {
    if (!doc) return "";
    if (selected.kind === "latest") return `Version ${doc.version}`;
    if (selected.kind === "ref") {
      const name = selected.name;
      const r = doc.refs.find((x) => x.name === name);
      return `“${name}” · version ${r?.version ?? "?"}${r?.pinned ? " (kept copy)" : ""}`;
    }
    const v = (selected as { version: number }).version;
    const info = history.find((h) => h.version === v);
    return `Version ${v}${info ? ` · ${date(info.timestampMs)}` : ""}`;
  });

  const saveLabel = $derived(
    { saved: "Saved", pending: "Editing…", saving: "Saving…", error: "Not saved" }[save.status],
  );
</script>

<svelte:window onkeydown={(e) => e.key === "Escape" && (fileMenu = false)} />

<div class="app">
  <header class="bar">
    {#if doc}
      <button class="icon" title="Toggle sidebar" aria-label="Toggle sidebar" aria-pressed={sidebar} onclick={() => (sidebar = !sidebar)}>
        <svg viewBox="0 0 16 16" width="16" height="16" aria-hidden="true"><rect x="1.5" y="2.5" width="13" height="11" rx="2" fill="none" stroke="currentColor" /><line x1="6" y1="2.5" x2="6" y2="13.5" stroke="currentColor" /></svg>
      </button>
    {/if}
    <strong class="brand">Cruncha</strong>
    {#if doc}
      <span class="doc" title={doc.path}>{doc.name.replace(/\.cruncha$/, "")}</span>
      {#if selected.kind === "latest"}
        <span class="status {save.status}" title={save.error ?? ""}>{saveLabel}</span>
      {/if}
    {/if}
    <span class="spacer"></span>
    <div class="file">
      <button aria-haspopup="menu" aria-expanded={fileMenu} onclick={() => (fileMenu = !fileMenu)}>File ▾</button>
      {#if fileMenu}
        <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
        <div class="menu" role="menu" tabindex="-1" onclick={() => (fileMenu = false)}>
          <button role="menuitem" onclick={newDocument}>New document…</button>
          <button role="menuitem" onclick={openDocument}>Open…</button>
          <button role="menuitem" onclick={createSample}>New sample document…</button>
          {#if prefs.recent?.length}
            <hr />
            <span class="label small muted">Recent</span>
            {#each prefs.recent.slice(0, 6) as p (p)}
              <button role="menuitem" title={p} onclick={() => openRecent(p)}>{fileName(p).replace(/\.cruncha$/, "")}</button>
            {/each}
          {/if}
        </div>
      {/if}
    </div>
    {#if doc}<button onclick={() => (modal = "doc")}>Document settings</button>{/if}
    <button class="icon" title="Preferences" aria-label="Preferences" onclick={() => (modal = "prefs")}>
      <svg viewBox="0 0 16 16" width="16" height="16" aria-hidden="true"><circle cx="8" cy="8" r="2.2" fill="none" stroke="currentColor" /><path d="M8 1.5v2M8 12.5v2M1.5 8h2M12.5 8h2M3.4 3.4l1.4 1.4M11.2 11.2l1.4 1.4M3.4 12.6l1.4-1.4M11.2 4.8l1.4-1.4" stroke="currentColor" stroke-linecap="round" /></svg>
    </button>
  </header>

  {#if error}
    <div class="error" role="alert">
      {error} <button class="link" onclick={() => (error = null)}>Dismiss</button>
    </div>
  {/if}

  {#if doc}
    <div class="main" class:nosidebar={!sidebar}>
      {#if sidebar}
        <Sidebar
          {outline}
          {variables}
          {editable}
          {history}
          refs={doc.refs}
          latest={doc.version}
          earliest={doc.earliestVersion}
          {selected}
          onselect={show}
          onjump={(pos) => editor?.jumpTo(pos)}
          onjumpcell={(id) => editor?.jumpToCell(id)}
          ondescribe={(id, d) => editor?.setDescription(id, d)}
          onsnapshot={() => (modal = "snapshot")}
        />
      {/if}
      <main style="--bar-h: {barH}px; --text-size: {sizes.text}px; --calc-size: {sizes.calc}px; --code-size: {sizes.code}px">
        <div class="viewing" bind:offsetHeight={barH}>
          <span>{heading}</span>
          {#if selected.kind !== "latest"}
            <span class="muted small">read-only view of history</span>
            <span class="spacer"></span>
            <button onclick={restoreShown} title="Adds a new version with this content; nothing is lost">
              Restore this {selected.kind === "ref" ? "snapshot" : "version"}
            </button>
            <button class="link" onclick={() => show({ kind: "latest" })}>Back to latest</button>
          {:else}
            <span class="muted small">
              Latest · every change is saved · history: {describeRetention(doc.effectiveRetention).toLowerCase()}{doc.retention == null ? " (app default)" : ""}
            </span>
            <span class="spacer"></span>
            {#if python?.hasPython && python.available && python.trusted}
              <button onclick={() => void editor?.runPython()} title="Run every Python cell again">▶ Run Python</button>
              <button onclick={() => void api.stopPython()} title="Stop the running cell">■ Stop</button>
              <button onclick={restartPython} title="Restart the Python worker and recompute every Python value">↻ Restart</button>
            {/if}
            <button class="snap" onclick={() => (modal = "snapshot")} title="Mark this version as a known-good point">Take snapshot…</button>
          {/if}
        </div>
        {#if python?.hasPython && selected.kind === "latest" && (!python.available || !python.trusted || setup)}
          <div class="python-bar" role="status">
            {#if setup}
              <span class="spinner" aria-hidden="true"></span> {setup}
              <span class="muted small">This takes a minute or two the first time (about 500 MB).</span>
            {:else if !python.available}
              <span><strong>Python cells need a Python environment.</strong> Cruncha installs its own Python 3.13 with NumPy, SciPy, Polars, pint, SymPy and matplotlib (about 500 MB, once for all documents).</span>
              <button class="primary" onclick={setUpPython}>Set up Python</button>
            {:else}
              <span><strong>This document has Python code that hasn't run on this computer.</strong> Python cells run with your permissions, like any program. Run them only if you trust where the document came from.</span>
              <button class="primary" onclick={trustDocument}>Trust and run</button>
            {/if}
          </div>
        {/if}
        {#if save.error}
          <div class="error" role="alert">
            Saving failed: {save.error}
            <button class="link" onclick={() => void flush()}>Retry</button>
          </div>
        {/if}
        <div class="page">
          <Editor
            bind:this={editor}
            {cells}
            {results}
            {editable}
            {loadKey}
            oncommitted={committed}
            onsavestate={(s) => (save = s)}
            onoutline={(o) => (outline = o)}
            onvariables={(v) => (variables = v)}
            onerror={(m) => (error = m)}
          />
        </div>
      </main>
    </div>
  {:else}
    <div class="welcome">
      <h1>Cruncha</h1>
      <p class="muted lede">Live, unit-aware engineering calculations with full version history.</p>
      <p class="actions">
        <button class="primary" onclick={newDocument}>New document…</button>
        <button onclick={openDocument}>Open…</button>
        <button onclick={createSample}>Try a sample…</button>
      </p>
      {#if prefs.recent?.length}
        <section class="recent">
          <h2>Recent</h2>
          {#each prefs.recent as p (p)}
            <button class="recent-row" title={p} onclick={() => openRecent(p)}>
              <span class="name">{fileName(p).replace(/\.cruncha$/, "")}</span>
              <span class="small muted path">{folderOf(p)}</span>
            </button>
          {/each}
        </section>
      {/if}
      <p class="small muted">Every change is saved as a new version as you work; there is no Save button.</p>
    </div>
  {/if}
</div>

{#if modal === "prefs"}
  <Preferences {prefs} {doc} onclose={() => (modal = null)} onsaved={prefsSaved} />
{:else if modal === "snapshot" && doc}
  <SnapshotDialog version={doc.version} taken={doc.refs.map((r) => r.name)} onclose={() => (modal = null)} onsave={takeSnapshot} />
{:else if modal === "doc" && doc}
  <DocumentSettings {doc} {prefs} onclose={() => (modal = null)} onchanged={(d) => void load(d)} />
{/if}

<style>
  .app { display: grid; grid-template-rows: auto auto 1fr; height: 100%; }
  .bar {
    display: flex; align-items: center; gap: 8px; padding: 6px 10px;
    background: var(--panel); border-bottom: 1px solid var(--border);
  }
  .brand { color: var(--accent); letter-spacing: 0.01em; }
  .doc { font-weight: 500; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .doc::before { content: "/"; color: var(--border); margin-right: 8px; }
  .spacer { flex: 1; }
  .icon { padding: 4px 6px; line-height: 0; color: var(--muted); }
  .icon:hover { color: var(--text); }
  .status { font-size: 12px; padding: 1px 8px; border-radius: 10px; background: var(--sunken); color: var(--muted); }
  .status.saved::before { content: "✓ "; }
  .status.error { color: var(--danger); background: var(--danger-bg); }
  .file { position: relative; }
  .menu {
    position: absolute; right: 0; top: calc(100% + 4px); z-index: 20; min-width: 220px;
    display: grid; padding: 4px; background: var(--panel); border: 1px solid var(--border);
    border-radius: 8px; box-shadow: 0 8px 24px rgb(0 0 0 / 0.14);
  }
  .menu button {
    border: none; background: none; text-align: left; padding: 6px 10px; border-radius: 5px;
    overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
  }
  .menu button:hover { background: var(--sunken); }
  .menu hr { border: none; border-top: 1px solid var(--border); margin: 4px 0; width: 100%; }
  .menu .label { padding: 2px 10px; }
  .error { background: var(--danger-bg); color: var(--danger); padding: 6px 12px; }
  .main { display: grid; grid-template-columns: 300px 1fr; min-height: 0; }
  .main.nosidebar { grid-template-columns: 1fr; }
  /* Layers: the document is its own stacking context, so its flyouts (block menu, z 5) stay
     under the sticky version bar (z 10); app menus (z 20) and dialogs (Modal, z 30) are above it. */
  main { overflow: auto; min-width: 0; background: var(--bg); isolation: isolate; }
  .viewing {
    position: sticky; top: 0; z-index: 10; display: flex; gap: 10px; align-items: baseline; flex-wrap: wrap;
    padding: 8px 24px; background: color-mix(in srgb, var(--bg) 88%, transparent);
    backdrop-filter: blur(6px); border-bottom: 1px solid var(--border);
  }
  .viewing button { font-size: 12.5px; padding: 2px 10px; }
  .python-bar {
    display: flex; gap: 12px; align-items: center; flex-wrap: wrap; padding: 10px 24px;
    background: var(--warn-bg); color: var(--text); border-bottom: 1px solid var(--border); font-size: 13.5px;
  }
  .python-bar span { flex: 1 1 320px; }
  .spinner {
    flex: none !important; width: 14px; height: 14px; border-radius: 50%; border: 2px solid var(--border);
    border-top-color: var(--accent); animation: spin 0.9s linear infinite;
  }
  @keyframes spin { to { transform: rotate(360deg); } }
  .viewing .snap { color: var(--accent); }
  .page {
    max-width: 880px; margin: 20px auto 48px; background: var(--panel); border: 1px solid var(--border);
    border-radius: 6px; box-shadow: 0 1px 3px rgb(0 0 0 / 0.05);
    /* The editor's sticky toolbar and block-handle gutter read these. */
    --pad-t: 28px; --pad-l: 104px; --pad-r: 64px; --gutter: 100px;
    padding: var(--pad-t) var(--pad-r) 24px var(--pad-l);
  }
  .welcome { max-width: 560px; margin: 10vh auto; padding: 0 16px; text-align: center; }
  .welcome h1 { color: var(--accent); margin-bottom: 4px; font-size: 2.2em; letter-spacing: -0.01em; }
  .lede { font-size: 15px; }
  .actions { display: flex; gap: 8px; justify-content: center; margin: 20px 0 28px; }
  .recent { text-align: left; border: 1px solid var(--border); border-radius: 8px; background: var(--panel); padding: 6px; margin-bottom: 20px; }
  .recent h2 { font-size: 11px; text-transform: uppercase; letter-spacing: 0.06em; color: var(--muted); margin: 6px 8px; }
  .recent-row {
    display: grid; width: 100%; text-align: left; border: none; background: none; padding: 6px 8px;
    border-radius: 5px;
  }
  .recent-row:hover { background: var(--sunken); }
  .recent-row .name { font-weight: 500; }
  .recent-row .path { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  @media (max-width: 900px) {
    .page { --pad-t: 20px; --pad-l: 96px; --pad-r: 24px; --gutter: 94px; padding: var(--pad-t) var(--pad-r) 20px var(--pad-l); margin: 0; border-radius: 0; border-left: none; border-right: none; }
  }
  @media (max-width: 760px) {
    .main { grid-template-columns: 1fr; grid-template-rows: 40% 1fr; }
  }
</style>
