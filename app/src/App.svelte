<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { open as openDialog, save as saveDialog } from "@tauri-apps/plugin-dialog";
  import { onMount } from "svelte";
  import {
    api,
    KEEP_EVERYTHING,
    type Cell,
    type DocInfo,
    type Prefs,
    type Selection,
    type VersionView,
  } from "./lib/api";
  import CellList from "./lib/CellList.svelte";
  import DocumentSettings from "./lib/DocumentSettings.svelte";
  import HistoryPanel from "./lib/HistoryPanel.svelte";
  import Preferences from "./lib/Preferences.svelte";
  import { date, describeRetention } from "./lib/format";

  let prefs = $state<Prefs>({ defaultRetention: KEEP_EVERYTHING });
  let doc = $state<DocInfo | null>(null);
  let history = $state<VersionView[]>([]);
  let selected = $state<Selection>({ kind: "latest" });
  let cells = $state<Cell[]>([]);
  let modal = $state<"prefs" | "doc" | null>(null);
  let error = $state<string | null>(null);

  async function attempt(f: () => Promise<void>) {
    error = null;
    try {
      await f();
    } catch (e) {
      error = String(e);
    }
  }

  async function show(s: Selection) {
    await attempt(async () => {
      cells =
        s.kind === "ref"
          ? await api.cellsAtRef(s.name)
          : await api.cells(s.kind === "version" ? s.version : null);
      selected = s;
    });
  }

  async function load(info: DocInfo | null) {
    doc = info;
    history = info ? await api.history() : [];
    if (info) await show({ kind: "latest" });
  }

  onMount(() =>
    void attempt(async () => {
      prefs = await api.getPrefs();
      await load(await api.documentInfo());
    }),
  );

  const openDocument = () =>
    attempt(async () => {
      const path = await openDialog({ directory: true, title: "Open a Cruncha document (.cruncha)" });
      if (typeof path === "string") await load(await api.openDocument(path));
    });

  const createSample = () =>
    attempt(async () => {
      const path = await saveDialog({
        title: "Create a sample document",
        defaultPath: "beam-check.cruncha",
      });
      if (path) await load(await api.createSampleDocument(path));
    });

  // The open document may follow the default, so its effective policy can change.
  const prefsSaved = (p: Prefs) =>
    attempt(async () => {
      prefs = p;
      doc = await api.documentInfo();
    });

  const heading = $derived.by(() => {
    if (!doc) return "";
    if (selected.kind === "latest") return `Latest · version ${doc.version}`;
    if (selected.kind === "ref") {
      const name = selected.name;
      const r = doc.refs.find((x) => x.name === name);
      return `“${name}” · version ${r?.version ?? "?"}${r?.pinned ? " (kept copy)" : ""}`;
    }
    const v = (selected as { version: number }).version;
    const info = history.find((h) => h.version === v);
    return `Version ${v}${info ? ` · ${date(info.timestampMs)}` : ""}`;
  });
</script>

<div class="app">
  <header class="bar">
    <strong class="brand">Cruncha</strong>
    {#if doc}<span class="doc" title={doc.path}>{doc.name}</span>{/if}
    <span class="spacer"></span>
    <button onclick={openDocument}>Open…</button>
    <button onclick={createSample}>New sample…</button>
    {#if doc}<button onclick={() => (modal = "doc")}>Document settings</button>{/if}
    <button onclick={() => (modal = "prefs")}>Preferences</button>
  </header>

  {#if error}
    <div class="error" role="alert">
      {error} <button class="link" onclick={() => (error = null)}>Dismiss</button>
    </div>
  {/if}

  {#if doc}
    <div class="main">
      <HistoryPanel
        {history}
        refs={doc.refs}
        latest={doc.version}
        earliest={doc.earliestVersion}
        {selected}
        onselect={show}
      />
      <main>
        <div class="viewing">
          <span>{heading}</span>
          {#if selected.kind !== "latest"}
            <span class="muted small">read-only view of history</span>
            <button class="link" onclick={() => show({ kind: "latest" })}>Back to latest</button>
          {:else}
            <span class="muted small">
              read-only for now · history: {describeRetention(doc.effectiveRetention)}{doc.retention ==
              null
                ? " (app default)"
                : ""}
            </span>
          {/if}
        </div>
        <div class="page"><CellList {cells} /></div>
      </main>
    </div>
  {:else}
    <div class="welcome">
      <h1>Cruncha</h1>
      <p class="muted">Live, unit-aware engineering calculations with full version history.</p>
      <p>
        <button class="primary" onclick={openDocument}>Open a document…</button>
        <button onclick={createSample}>Create a sample document…</button>
      </p>
      <p class="small muted">This early build shows documents read-only: cells at any version, history and named versions.</p>
    </div>
  {/if}
</div>

{#if modal === "prefs"}
  <Preferences {prefs} {doc} onclose={() => (modal = null)} onsaved={prefsSaved} />
{:else if modal === "doc" && doc}
  <DocumentSettings
    {doc}
    {prefs}
    onclose={() => (modal = null)}
    onchanged={(d) => void load(d)}
  />
{/if}

<style>
  .app { display: grid; grid-template-rows: auto auto 1fr; height: 100%; }
  .bar {
    display: flex; align-items: center; gap: 8px; padding: 8px 12px;
    background: var(--panel); border-bottom: 1px solid var(--border);
  }
  .brand { color: var(--accent); }
  .doc { font-weight: 500; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .spacer { flex: 1; }
  .error { background: var(--danger-bg); color: var(--danger); padding: 6px 12px; }
  .main { display: grid; grid-template-columns: 320px 1fr; min-height: 0; }
  main { overflow: auto; min-width: 0; }
  .viewing {
    position: sticky; top: 0; display: flex; gap: 10px; align-items: baseline; flex-wrap: wrap;
    padding: 10px 24px; background: var(--bg); border-bottom: 1px solid var(--border);
  }
  .page {
    max-width: 860px; margin: 16px auto; background: var(--panel); border: 1px solid var(--border);
    border-radius: 8px; padding: 16px 12px;
  }
  .welcome { max-width: 520px; margin: 12vh auto; text-align: center; padding: 0 16px; }
  .welcome h1 { color: var(--accent); margin-bottom: 4px; }
  @media (max-width: 760px) {
    .main { grid-template-columns: 1fr; grid-template-rows: 40% 1fr; }
  }
</style>
