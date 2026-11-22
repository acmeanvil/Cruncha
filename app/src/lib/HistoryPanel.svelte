<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<!-- Version history grouped by snapshot (named, known-good versions; store tags that survive
     history trimming): a snapshot is a major version, the edits after it are numbered
     "Rev A · 1", "Rev A · 2", … (history.ts). Selecting one shows the document as it was.
     Archive… (ADR-0023) removes the oldest groups: their snapshots are kept, their saves go to
     the save log, and they move from this tab to the archive (ArchiveDialog). -->
<script lang="ts">
  import { api, type ArchivePreview, type DocInfo, type RefView, type Selection, type VersionView } from "./api";
  import ArchiveDialog from "./ArchiveDialog.svelte";
  import { SvelteMap, SvelteSet } from "svelte/reactivity";
  import { bytes, date } from "./format";
  import { archiveCutoff, entryMatches, groupHistory, splitArchived, type Entry, type Group } from "./history";

  let {
    history,
    refs,
    latest,
    earliest,
    selected,
    onselect,
    onsnapshot,
    onarchived,
  }: {
    history: VersionView[];
    refs: RefView[];
    latest: number;
    earliest: number;
    selected: Selection;
    onselect: (s: Selection) => void;
    onsnapshot?: () => void;
    /** After archiving: the document's new info. Omitted, the tab can't archive. */
    onarchived?: (info: DocInfo) => void;
  } = $props();

  let showMaintenance = $state(false);
  let query = $state("");
  /** Groups the user opened or closed (by name); otherwise only the newest is open. */
  const toggled = new SvelteMap<string, boolean>();
  /** Groups showing all their versions, not just the newest PAGE. */
  const full = new SvelteSet<string>();
  const PAGE = 25;

  const all = $derived(splitArchived(groupHistory(history, refs, earliest), earliest));
  const groups = $derived(all.shown);

  // Archiving (ADR-0023): pick the newest group to archive; every older one goes with it.
  let archiving = $state(false);
  let pick = $state<number | null>(null);
  let preview = $state<ArchivePreview | null>(null);
  let busy = $state(false);
  let failed = $state<string | null>(null);
  let showArchive = $state(false);
  let saves = $state(0);
  const picked = (i: number) => pick != null && i >= pick;

  $effect(() => {
    void earliest;
    api.archivedSaves().then((l) => (saves = l.length), () => (saves = 0));
  });

  function startArchiving() {
    archiving = !archiving;
    pick = null;
    preview = null;
    failed = null;
  }

  async function choose(i: number) {
    pick = pick === i ? null : i;
    preview = null;
    failed = null;
    const cutoff = pick == null ? null : archiveCutoff(groups, pick);
    if (cutoff == null) return;
    try {
      preview = await api.archivePreview(cutoff);
    } catch (e) {
      failed = String(e);
    }
  }

  async function archive() {
    if (!preview || !onarchived) return;
    busy = true;
    try {
      const info = await api.archiveHistory(preview.cutoff);
      archiving = false;
      pick = null;
      preview = null;
      onarchived(info);
    } catch (e) {
      failed = String(e);
    } finally {
      busy = false;
    }
  }

  const archivedNames = $derived(all.archived.map((g) => g.name).reverse());
  const isSelected = (v: VersionView) =>
    (selected.kind === "latest" && v.version === latest) ||
    (selected.kind === "version" && selected.version === v.version);
  const visible = (e: Entry) =>
    (showMaintenance || e.version.kind !== "maintenance" || e.version.prunedBefore != null) && entryMatches(e, query);
  const searching = $derived(query.trim() !== "");

  function isOpen(g: Group, i: number): boolean {
    if (searching) return true;
    const t = toggled.get(g.name);
    if (t !== undefined) return t;
    return i === 0 || g.entries.some((e) => isSelected(e.version)) || (selected.kind === "ref" && selected.name === g.name);
  }

  function span(g: Group): string {
    if (g.newest == null || g.oldest == null) return "";
    const a = date(g.oldest), b = date(g.newest);
    return a === b ? b : `${a} – ${b}`;
  }

  const edits = (n: number) => `${n} edit${n === 1 ? "" : "s"}`;
</script>

<div class="history">
  <div class="top">
    <input type="search" placeholder="Find a version, message or snapshot" aria-label="Find a version" bind:value={query} />
    {#if onsnapshot}
      <button class="take" onclick={onsnapshot} title="Mark the latest version as a known-good point">+ Snapshot</button>
    {/if}
    {#if onarchived && groups.length > 1}
      <button class="take" class:on={archiving} onclick={startArchiving} title="Move the oldest snapshots and their saves out of this list to free space">Archive…</button>
    {/if}
  </div>
  {#if archiving}
    <div class="archiving small">
      {#if !preview}
        <p>Tick the newest group to archive; every older one goes with it. The group you're working in stays.</p>
      {:else}
        <p>
          Archive <strong>{groups.slice(pick ?? 0).map((g) => g.name).reverse().join(", ")}</strong>:
          {preview.saves} save{preview.saves === 1 ? "" : "s"} (v{preview.fromVersion}–v{preview.cutoff - 1}), about {bytes(preview.bytesFreed)} freed.
        </p>
        <p class="muted">
          {#if preview.snapshots.length}The snapshots {preview.snapshots.join(", ")} are kept and can still be opened, printed and restored.{/if}
          Who saved what, and when, stays in the save log. The individual saves can't be opened again; this can't be undone.
        </p>
      {/if}
      {#if failed}<p class="err">{failed}</p>{/if}
      <div class="abtns">
        <button onclick={startArchiving} disabled={busy}>Cancel</button>
        <button class="primary" onclick={archive} disabled={!preview || busy}>{busy ? "Archiving…" : "Archive"}</button>
      </div>
    </div>
  {/if}
  <p class="opts small muted">
    <label><input type="checkbox" bind:checked={showMaintenance} /> show maintenance</label>
  </p>
  {#if refs.length === 0}
    <p class="small muted hint">
      A snapshot marks a known-good point, such as “Rev A” or “Issued for review”. Edits after it
      are numbered “Rev A · 1”, “Rev A · 2”, … until the next one. Snapshots are kept even when
      older history is trimmed, and you can restore one at any time.
    </p>
  {/if}

  {#each groups as g, i (g.name + ":" + (g.snapshot?.version ?? -1))}
    {@const shown = g.entries.filter(visible)}
    {@const open = isOpen(g, i)}
    {#if !searching || shown.length || (g.snapshot && g.name.toLowerCase().includes(query.trim().toLowerCase()))}
      <section class="group" class:draft={!g.snapshot}>
        <div class="ghead">
          {#if archiving && i > 0}
            <input
              type="checkbox"
              class="pick"
              checked={picked(i)}
              aria-label="Archive {g.name} and everything older"
              onchange={() => choose(i)}
            />
          {/if}
          <button
            class="chev"
            aria-expanded={open}
            aria-label="{open ? 'Collapse' : 'Expand'} {g.name}"
            onclick={() => toggled.set(g.name, !open)}>{open ? "▾" : "▸"}</button>
          {#if g.snapshot}
            {@const r = g.snapshot}
            <button
              class="row snap"
              class:sel={selected.kind === "ref" && selected.name === r.name}
              title={r.message || `The snapshot “${r.name}”`}
              onclick={() => onselect({ kind: "ref", name: r.name })}
            >
              <span class="head">
                <span class="tag">{r.name}</span>
                <span class="muted small">v{r.version}{r.pinned ? " · kept copy" : ""}</span>
                <span class="muted small when">{date(r.createdAtMs)}</span>
              </span>
              <span class="small muted">
                {g.edits === 0 ? (i === 0 ? "no edits since" : "no edits after") : `${edits(g.edits)} after`}{g.complete ? "" : " (some trimmed)"}
              </span>
            </button>
          {:else}
            <button class="row title" onclick={() => toggled.set(g.name, !open)}>
              <span class="head">
                <span class="gname">Draft</span>
                <span class="muted small">{g.edits ? edits(g.edits) : ""}{refs.length ? " before the first snapshot" : ""}</span>
              </span>
              {#if span(g)}<span class="small muted">{span(g)}</span>{/if}
            </button>
          {/if}
        </div>
        {#if open}
          <div class="entries">
            {#each full.has(g.name) || searching ? shown : shown.slice(0, PAGE) as e (e.version.version)}
              {@const v = e.version}
              {#if v.prunedBefore != null}
                <div class="marker small">History before version {v.prunedBefore} was trimmed</div>
              {:else}
                <button
                  class="row"
                  class:sel={isSelected(v)}
                  class:maint={v.kind === "maintenance"}
                  onclick={() =>
                    onselect(v.version === latest ? { kind: "latest" } : { kind: "version", version: v.version })}
                >
                  <span class="head">
                    {#if e.label}<span class="lbl">{e.label}</span>{/if}
                    <span class="ver">v{v.version}</span>
                    {#if v.version === latest}<span class="badge">latest</span>{/if}
                    {#if v.kind === "restore"}<span class="badge">restore{v.restoredFrom != null ? ` of v${v.restoredFrom}` : ""}</span>{/if}
                    {#each e.tags as t}<span class="tag">{t}</span>{/each}
                    <span class="muted small when">{date(v.timestampMs)}</span>
                  </span>
                  <span class="msg">{v.message || v.kind}</span>
                  <span class="muted small">{v.actor}</span>
                </button>
              {/if}
            {/each}
            {#if !searching && !full.has(g.name) && shown.length > PAGE}
              <button class="link more small" onclick={() => full.add(g.name)}>Show {shown.length - PAGE} older</button>
            {/if}
          </div>
        {/if}
      </section>
    {/if}
  {/each}
  {#if searching && !groups.some((g) => g.entries.some(visible))}
    <p class="small muted end">No version matches “{query}”.</p>
  {/if}
  {#if all.archived.length || saves}
    <p class="small muted end">
      Archived: {archivedNames.length > 1 ? `${archivedNames[0]} – ${archivedNames[archivedNames.length - 1]}` : (archivedNames[0] ?? "older history")}
      · {saves} save{saves === 1 ? "" : "s"} ·
      <button class="link small" onclick={() => (showArchive = true)}>Show</button>
    </p>
  {:else if earliest > 0}
    <p class="small muted end">Versions before {earliest} are no longer available.</p>
  {/if}
</div>

{#if showArchive}
  <ArchiveDialog
    groups={all.archived}
    onclose={() => (showArchive = false)}
    onopen={(name) => {
      showArchive = false;
      onselect({ kind: "ref", name });
    }}
  />
{/if}

<style>
  .history { padding: 6px 10px 12px; }
  .top { display: flex; gap: 6px; position: sticky; top: 0; background: var(--panel); padding: 2px 0 4px; z-index: 1; }
  .top input { flex: 1; min-width: 0; }
  .take { flex: none; font-size: 12px; padding: 2px 8px; color: var(--accent); }
  .take.on { background: var(--accent-bg); }
  .archiving { border: 1px solid var(--border); border-radius: 8px; padding: 6px 10px; margin: 2px 0 8px; background: var(--sunken); }
  .archiving p { margin: 4px 0; }
  .abtns { display: flex; gap: 6px; justify-content: flex-end; margin-top: 6px; }
  .err { color: var(--danger); }
  .pick { margin: 9px 2px 0 0; flex: none; }
  .opts { margin: 0 2px 6px; }
  .hint { margin: 0 4px 8px; }
  .group { margin: 2px 0 6px; }
  .ghead { display: flex; align-items: flex-start; gap: 2px; }
  .chev {
    flex: none; border: none; background: none; padding: 7px 3px; color: var(--muted); font-size: 11px; line-height: 1;
    border-radius: 4px; cursor: pointer;
  }
  .chev:hover { color: var(--text); background: var(--sunken); }
  .entries { margin-left: 14px; border-left: 1px solid var(--border); padding-left: 4px; }
  .row.snap { border-left: 3px solid var(--accent); }
  .row.title .gname { font-weight: 600; }
  .row {
    display: grid; gap: 1px; width: 100%; text-align: left; border: 1px solid transparent;
    background: none; border-radius: 6px; padding: 6px 8px; margin: 1px 0; min-width: 0;
  }
  .row:hover { background: var(--sunken); }
  .row.sel { background: var(--select); border-color: var(--select-border); }
  .row.maint { opacity: 0.6; }
  .head { display: flex; gap: 6px; align-items: baseline; flex-wrap: wrap; }
  .lbl { font-weight: 600; font-size: 13px; }
  .ver { font-family: var(--mono); font-size: 11.5px; color: var(--muted); }
  .when { margin-left: auto; }
  .msg { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .tag {
    background: var(--accent-bg); color: var(--accent); border-radius: 4px;
    padding: 0 6px; font-size: 12px; font-weight: 600;
  }
  .badge { background: var(--sunken); border-radius: 4px; padding: 0 6px; font-size: 11.5px; }
  .marker {
    text-align: center; color: var(--muted); border-top: 1px dashed var(--border);
    border-bottom: 1px dashed var(--border); margin: 6px 0; padding: 4px;
  }
  .more { display: block; margin: 4px 8px; }
  .end { text-align: center; }
</style>
