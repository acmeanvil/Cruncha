<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<!-- The Templates tab of the reference panel (ADR-0014 §5): the built-in library and the
     user's own, searchable and by category. A template's page previews its cells, inputs and
     outputs, references and worked examples (run here with the calc engine), and inserts it
     (the editor checks its names first, §3). Snippets are made from the editor's selection
     and managed here: details, update from the selection, duplicate, export, delete (to the
     trash). Built-in templates are duplicated to be changed, and hidden instead of deleted. -->
<script lang="ts">
  import { onMount } from "svelte";
  import { save as saveDialog } from "@tauri-apps/plugin-dialog";
  import { api, type SnippetDetails, type TemplateCell, type TemplateDetail, type TemplateSummary } from "./api";
  import { categoriesOf, findTemplates, proseSummary, type Which } from "./templateRef";
  import SnippetDialog from "./SnippetDialog.svelte";
  import LibraryDialog from "./LibraryDialog.svelte";

  let {
    editable,
    oninsert,
    selection,
  }: {
    editable: boolean;
    oninsert: (t: TemplateDetail) => Promise<void> | void;
    /** The editor's selected blocks, as cells. */
    selection: () => TemplateCell[];
  } = $props();

  let all = $state<TemplateSummary[] | null>(null);
  let failed = $state<string | null>(null);
  let query = $state("");
  let category = $state<string | null>(null);
  let which = $state<Which>("all");
  let showHidden = $state(false);
  let detail = $state<TemplateDetail | null>(null);
  let error = $state<string | null>(null);
  let note = $state<string | null>(null);
  let busy = $state(false);
  let library = $state(false);
  let openTest = $state<string | null>(null);
  /** The details form: a new snippet from the selection, or a snippet's details. */
  let form = $state<{ mode: "new" | "edit"; cells: TemplateCell[]; initial: SnippetDetails; error: string | null } | null>(null);
  let search: HTMLInputElement | undefined = $state();

  async function load(select?: string) {
    try {
      all = await api.templates();
      failed = null;
      if (select) await show(select);
      else if (detail && !all.some((t) => t.key === detail!.key)) detail = null;
    } catch (e) {
      failed = String(e);
    }
  }
  onMount(() => {
    void load();
    search?.focus();
  });

  const listed = $derived(all ? findTemplates(all, query, category, which, showHidden) : []);
  const categories = $derived(all ? categoriesOf(all) : []);
  const hiddenCount = $derived(all?.filter((t) => t.hidden).length ?? 0);

  async function show(key: string) {
    error = null;
    note = null;
    openTest = null;
    try {
      detail = await api.template(key);
    } catch (e) {
      error = String(e);
    }
  }

  async function act(f: () => Promise<string | void>, done?: string) {
    error = null;
    note = null;
    busy = true;
    try {
      const key = await f();
      await load(typeof key === "string" ? key : undefined);
      if (done) note = done;
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  async function insert() {
    if (!detail) return;
    busy = true;
    error = null;
    try {
      await oninsert(detail);
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  function describeCells(cells: TemplateCell[]): string {
    const n = cells.length;
    const kinds = new Set(cells.map((c) => c.kind));
    return `${n} ${n === 1 ? "cell" : "cells"} from the editor${kinds.has("python") || kinds.has("rust") ? ", code included" : ""}. Calcs whose value ends with _ become its inputs.`;
  }

  function newSnippet() {
    const cells = selection();
    if (cells.length === 0) {
      error = "Select the cells to save in the document first.";
      return;
    }
    error = null;
    form = { mode: "new", cells, initial: { title: "", summary: null, category: [], tags: [] }, error: null };
  }

  function editDetails() {
    if (!detail) return;
    form = { mode: "edit", cells: [], initial: { title: detail.title, summary: detail.summary, category: detail.category, tags: detail.tags }, error: null };
  }

  async function saveForm(d: SnippetDetails) {
    if (!form) return;
    const f = form;
    try {
      const key = f.mode === "new" ? await api.templateSave(d, f.cells) : await api.templateUpdateDetails(detail!.key, d);
      const made = f.mode === "new";
      form = null;
      which = which === "builtin" ? "all" : which;
      await load(key);
      note = made ? "Saved to your library." : "Details saved.";
    } catch (e) {
      form = { ...f, error: String(e) };
    }
  }

  function updateFromSelection() {
    if (!detail) return;
    const cells = selection();
    if (cells.length === 0) {
      error = "Select the cells to save in the document first.";
      return;
    }
    if (!confirm(`Replace the cells of “${detail.title}” with the ${cells.length} selected ${cells.length === 1 ? "cell" : "cells"}?`)) return;
    const key = detail.key;
    void act(() => api.templateUpdateCells(key, cells), "Cells replaced: a new version.");
  }

  async function exportFile() {
    if (!detail) return;
    const dest = await saveDialog({ defaultPath: `${detail.id}.cruncha.md`, filters: [{ name: "Cruncha template", extensions: ["md"] }] });
    if (!dest) return;
    const key = detail.key;
    void act(() => api.templateExport(key, dest), "Exported.");
  }

  function remove() {
    if (!detail) return;
    const { key, title } = detail;
    void act(async () => {
      await api.templateDelete(key);
      detail = null;
    }, `“${title}” is in the trash (Library… to restore it).`);
  }

  const keep = (e: MouseEvent) => e.preventDefault();
  const libLabel = (t: TemplateSummary) => (t.library === "user" ? "Mine" : "Built-in");
</script>

{#if detail}
  {@const d = detail}
  <div class="page">
    <div class="nav">
      <button class="link" onmousedown={keep} onclick={() => (detail = null)}>‹ All templates</button>
    </div>
    <div class="body">
      <h3>{d.title}</h3>
      <p class="meta small muted">
        <span class="badge" class:mine={d.library === "user"}>{libLabel(d)}</span>
        v{d.version} · <code>{d.id}</code>{#if d.category.length} · {d.category.join(" › ")}{/if}
      </p>
      {#if d.summary}<p class="sum">{d.summary}</p>{/if}

      <div class="actions">
        <button class="primary" disabled={!editable || busy || d.problems.length > 0} onmousedown={keep} onclick={insert}
          title={editable ? "Insert its cells after the current block (names are checked first)" : "Open the latest version to insert"}>Insert</button>
        <button onmousedown={keep} disabled={busy} onclick={() => act(() => api.templateDuplicate(d.key), "Duplicated into your library.")}
          title="Copy it into your library under a new id, to change it">Duplicate</button>
        {#if d.library === "user"}
          <button onmousedown={keep} disabled={busy} onclick={editDetails}>Edit details…</button>
          <button onmousedown={keep} disabled={busy} onclick={updateFromSelection} title="Replace its cells with the cells selected in the document">Update from selection</button>
        {/if}
        <button onmousedown={keep} disabled={busy} onclick={exportFile}>Export…</button>
        {#if d.library === "user"}
          <button class="danger" onmousedown={keep} disabled={busy} onclick={remove}>Delete</button>
        {:else}
          <button onmousedown={keep} disabled={busy} onclick={() => act(() => api.templateHide(d.id, !d.hidden).then(() => d.key), d.hidden ? "Shown in the list again." : "Hidden from the list (Library… to show it).")}>{d.hidden ? "Show" : "Hide"}</button>
        {/if}
      </div>
      {#if note}<p class="small ok">{note}</p>{/if}
      {#if error}<p class="small err">{error}</p>{/if}
      {#if d.problems.length}
        <div class="problems small">
          <strong>This template has problems, so it can't be inserted:</strong>
          <ul>{#each d.problems as p}<li>{p}</li>{/each}</ul>
        </div>
      {/if}

      {#if d.inputs.length || d.outputs.length}
        <dl class="names">
          {#if d.inputs.length}<dt>Inputs</dt><dd>{#each d.inputs as n}<code>{n}</code>{/each}</dd>{/if}
          {#if d.outputs.length}<dt>Outputs</dt><dd>{#each d.outputs as n}<code>{n}</code>{/each}</dd>{/if}
        </dl>
      {/if}

      {#if d.tests.length}
        <h4>Worked examples</h4>
        <ul class="tests">
          {#each d.tests as t (t.name)}
            <li>
              <button class="test" onmousedown={keep} onclick={() => (openTest = openTest === t.name ? null : t.name)} aria-expanded={openTest === t.name}>
                <span class="mark" class:pass={t.passed} class:skip={!!t.skipped}>{t.skipped ? "–" : t.passed ? "✓" : "✗"}</span>
                <span>{t.name}</span>
              </button>
              {#if openTest === t.name}
                <div class="test-body small">
                  {#if t.source}<p class="muted">{t.source}</p>{/if}
                  {#if t.skipped}<p class="muted">Not run here: {t.skipped}.</p>{/if}
                  {#if t.lines.length}<pre>{t.lines.join("\n")}</pre>{/if}
                </div>
              {/if}
            </li>
          {/each}
        </ul>
      {/if}

      {#if d.references.length}
        <h4>References</h4>
        <ul class="refs small">
          {#each d.references as r}
            <li>{r.work}{#if r.edition}, {r.edition}{/if}{#if r.locator}: {r.locator}{/if}</li>
          {/each}
        </ul>
      {/if}

      <h4>Cells <span class="muted small">({d.cells.length})</span></h4>
      <div class="cells">
        {#each d.cells as c, i (i)}
          {#if c.kind === "prose"}
            {@const p = proseSummary(c.source)}
            <div class="prose {p.kind}">{p.text}</div>
          {:else}
            <div class="cell">
              {#if c.attrs?.description}<div class="desc small muted">{c.attrs.description}</div>{/if}
              <pre class="src" class:code={c.kind !== "calc"}>{#if c.kind !== "calc"}<span class="lang">{c.kind}</span>{/if}{c.source}</pre>
            </div>
          {/if}
        {/each}
      </div>
      {#if d.from || d.license}
        <p class="small muted foot">{#if d.from}From {d.from}. {/if}{#if d.license}Licence {d.license}.{/if}</p>
      {/if}
    </div>
  </div>
{:else}
  <div class="tools">
    <input bind:this={search} type="search" placeholder="Find a template: bolt, psychrometrics, beam…" aria-label="Find a template" bind:value={query} />
    <div class="row">
      <select aria-label="Category" bind:value={category}>
        <option value={null}>All categories</option>
        {#each categories as c (c)}<option value={c}>{c}</option>{/each}
      </select>
      <select aria-label="Library" bind:value={which}>
        <option value="all">All libraries</option>
        <option value="builtin">Built-in</option>
        <option value="user">Mine</option>
      </select>
    </div>
  </div>
  <div class="list">
    {#if failed}
      <p class="small err">The library didn't load: {failed}</p>
    {:else if !all}
      <p class="small muted">Loading…</p>
    {:else if listed.length === 0}
      <p class="small muted">{which === "user" && !query ? "Your library is empty. Select cells in the document, then Save selection as snippet." : `Nothing matches “${query}”.`}</p>
    {/if}
    {#each listed as t (t.key)}
      <button class="entry" class:off={t.hidden || !!t.error} onmousedown={keep} onclick={() => show(t.key)}>
        <span class="title">{t.title}</span>
        {#if t.library === "user"}<span class="badge mine">Mine</span>{/if}
        {#if t.hidden}<span class="badge">hidden</span>{/if}
        {#if t.error}<span class="badge err">doesn't read</span>{/if}
        <span class="sum small">{t.error ?? t.summary ?? ""}</span>
        {#if !category && t.category.length}<span class="cat small muted">{t.category.join(" › ")}</span>{/if}
      </button>
    {/each}
  </div>
  <div class="foot-bar">
    {#if error}<p class="small err">{error}</p>{/if}
    {#if note}<p class="small ok">{note}</p>{/if}
    <div class="row">
      <button onmousedown={keep} disabled={!editable} onclick={newSnippet} title="Save the selected cells as a snippet in your library">Save selection as snippet…</button>
      <span class="spacer"></span>
      {#if hiddenCount}<label class="small muted"><input type="checkbox" bind:checked={showHidden} /> hidden ({hiddenCount})</label>{/if}
      <button onmousedown={keep} onclick={() => (library = true)}>Library…</button>
    </div>
  </div>
{/if}

{#if form}
  <SnippetDialog
    heading={form.mode === "new" ? "Save as snippet" : "Snippet details"}
    action={form.mode === "new" ? "Save" : "Save details"}
    initial={form.initial}
    note={form.mode === "new" ? describeCells(form.cells) : "A new title renames the file; the id stays, so documents still know where their cells came from."}
    error={form.error}
    onsave={saveForm}
    oncancel={() => (form = null)}
  />
{/if}
{#if library && all}
  <LibraryDialog templates={all} onchanged={(key) => void load(key)} onclose={() => (library = false)} />
{/if}

<style>
  .tools { padding: 8px 12px 6px; display: grid; gap: 6px; border-bottom: 1px solid var(--border); }
  .row { display: flex; gap: 6px; align-items: center; }
  .row select { flex: 1; min-width: 0; font-size: 12.5px; }
  .spacer { flex: 1; }
  .list { overflow: auto; padding: 4px 12px 8px; min-height: 0; }
  .entry {
    display: flex; flex-wrap: wrap; align-items: baseline; gap: 2px 6px; width: 100%; text-align: left;
    border: none; border-bottom: 1px solid color-mix(in srgb, var(--border) 60%, transparent); border-radius: 0;
    background: none; padding: 8px 2px; color: var(--text);
  }
  .entry:hover { background: var(--sunken); }
  .entry.off .title { color: var(--muted); }
  .title { font-weight: 600; font-size: 13px; }
  .entry .sum { flex-basis: 100%; color: var(--text); opacity: 0.85; }
  .cat { flex-basis: 100%; }
  .badge { font-size: 10.5px; background: var(--sunken); border-radius: 4px; padding: 0 6px; color: var(--muted); }
  .badge.mine { background: color-mix(in srgb, var(--accent) 16%, transparent); color: var(--accent); }
  .badge.err { color: var(--danger); }
  .foot-bar { border-top: 1px solid var(--border); padding: 6px 12px 8px; }
  .foot-bar p { margin: 0 0 4px; }
  .foot-bar button { font-size: 12.5px; }
  .page { display: grid; grid-template-rows: auto 1fr; min-height: 0; }
  .nav { padding: 6px 12px; border-bottom: 1px solid var(--border); }
  .body { overflow: auto; padding: 8px 12px 16px; min-height: 0; }
  h3 { margin: 4px 0 2px; font-size: 15px; }
  h4 { margin: 14px 0 4px; font-size: 12.5px; text-transform: uppercase; letter-spacing: 0.04em; color: var(--muted); }
  .meta { margin: 0 0 6px; display: flex; flex-wrap: wrap; gap: 4px; align-items: center; }
  .meta code { font: 11.5px var(--mono); }
  .sum { margin: 4px 0 8px; font-size: 13px; }
  .actions { display: flex; flex-wrap: wrap; gap: 6px; margin: 8px 0; }
  .actions button { font-size: 12.5px; padding: 3px 10px; }
  .problems { background: var(--warn-bg); color: var(--warn); padding: 6px 8px; border-radius: 4px; }
  .problems ul { margin: 4px 0 0; padding-left: 18px; }
  .names { display: grid; grid-template-columns: auto 1fr; gap: 4px 10px; margin: 8px 0; font-size: 12.5px; }
  .names dt { color: var(--muted); }
  .names dd { margin: 0; display: flex; flex-wrap: wrap; gap: 4px; }
  .names code { font: 12px var(--mono); background: var(--sunken); padding: 0 5px; border-radius: 3px; }
  .tests, .refs { list-style: none; margin: 0; padding: 0; }
  .refs li { padding: 2px 0; }
  .test { display: flex; gap: 6px; border: none; background: none; padding: 3px 0; text-align: left; font-size: 13px; color: var(--text); }
  .mark { width: 1em; color: var(--danger); font-weight: 700; }
  .mark.pass { color: var(--ok, #2a8a3e); }
  .mark.skip { color: var(--muted); }
  .test-body { margin: 0 0 6px 1.6em; }
  .test-body p { margin: 2px 0; }
  .test-body pre { margin: 2px 0; font: 11.5px var(--mono); white-space: pre-wrap; background: var(--sunken); padding: 4px 6px; border-radius: 4px; }
  .cells { display: grid; gap: 6px; }
  .prose { font-size: 12.5px; white-space: pre-wrap; }
  .prose.h1, .prose.h2, .prose.h3, .prose.h4 { font-weight: 600; font-size: 13px; margin-top: 4px; }
  .cell { border-left: 2px solid var(--accent); padding-left: 6px; }
  .desc { margin-bottom: 1px; }
  .src { margin: 0; font: 12px var(--mono); white-space: pre-wrap; overflow-wrap: anywhere; }
  .src.code { background: var(--sunken); padding: 4px 6px; border-radius: 4px; }
  .lang { display: block; font-size: 10.5px; color: var(--muted); }
  .foot { margin-top: 12px; }
  .err { color: var(--danger); }
  .ok { color: var(--ok, #2a8a3e); }
</style>
