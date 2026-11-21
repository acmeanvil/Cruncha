<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<!-- Document settings: what belongs to this document, by topic. Document (its type sizes) and
     Report (title block and contents) live in its meta cell, so saving them is a version like
     any edit; History is its retention policy (ADR-0011). App-wide defaults are in Settings. -->
<script lang="ts" module>
  export type DocTab = "document" | "report" | "history";
</script>

<script lang="ts">
  import { onMount, untrack } from "svelte";
  import {
    api,
    DEFAULT_TEXT_SIZES,
    type CommitView,
    type DocInfo,
    type DocumentSettings,
    type Prefs,
    type Preview,
    type TitleBlock,
  } from "./api";
  import Modal from "./Modal.svelte";
  import PruneConfirm from "./PruneConfirm.svelte";
  import RetentionControl from "./RetentionControl.svelte";
  import TextSizeFields from "./TextSizeFields.svelte";
  import { bytes, describeRetention } from "./format";

  let {
    doc,
    prefs,
    tab: initialTab = "document",
    onclose,
    onchanged,
    oncommitted,
    onsettings,
  }: {
    doc: DocInfo;
    prefs: Prefs;
    tab?: DocTab;
    onclose: () => void;
    /** The retention policy changed (or history was trimmed). */
    onchanged: (d: DocInfo) => void;
    /** The meta cell was committed (type sizes or report settings). */
    oncommitted: (v: CommitView, settings: DocumentSettings) => void;
    /** Open the app's Settings. */
    onsettings: () => void;
  } = $props();

  const TABS: { id: DocTab; label: string }[] = [
    { id: "document", label: "Document" },
    { id: "report", label: "Report" },
    { id: "history", label: "History" },
  ];
  let tab = $state<DocTab>(untrack(() => initialTab));

  let settings = $state<DocumentSettings | null>(null);
  let loaded: string = "";
  const appSizes = $derived({ ...DEFAULT_TEXT_SIZES, ...prefs.textSizes });

  let useDefault = $state(untrack(() => doc.retention == null));
  let policy = $state($state.snapshot(untrack(() => doc.retention ?? prefs.defaultRetention)));
  const chosen = $derived(useDefault ? prefs.defaultRetention : policy);
  const retentionChanged = $derived(
    useDefault !== (doc.retention == null) || (!useDefault && JSON.stringify(policy) !== JSON.stringify(doc.retention)),
  );

  let confirm = $state<{ preview: Preview; action: "save" | "trim" } | null>(null);
  let busy = $state(false);
  let message = $state<string | null>(null);

  const FIELDS: { key: keyof TitleBlock; label: string; placeholder?: string }[] = [
    { key: "title", label: "Title" },
    { key: "subject", label: "Subject", placeholder: "e.g. Flexure and shear, level 3" },
    { key: "project", label: "Project" },
    { key: "projectNumber", label: "Project No." },
    { key: "client", label: "Client" },
    { key: "preparedBy", label: "Prepared by" },
    { key: "checkedBy", label: "Checked by" },
    { key: "approvedBy", label: "Approved by" },
  ];

  onMount(async () => {
    try {
      settings = await api.documentSettings();
      loaded = JSON.stringify(settings);
    } catch (e) {
      message = String(e);
    }
  });

  async function run(f: () => Promise<void>) {
    busy = true;
    message = null;
    try {
      await f();
    } catch (e) {
      message = String(e);
    } finally {
      busy = false;
    }
  }

  /** Save every tab: the meta cell (one commit, if it changed) and the retention policy. */
  const save = () =>
    run(async () => {
      if (retentionChanged && !confirm) {
        const p = await api.retentionPreview($state.snapshot(chosen));
        if (p.cutoff != null) {
          confirm = { preview: p, action: "save" };
          tab = "history";
          return;
        }
      }
      if (settings && JSON.stringify(settings) !== loaded) {
        const snap = $state.snapshot(settings);
        oncommitted(await api.setDocumentSettings(snap), snap);
      }
      if (retentionChanged) {
        let info = await api.setDocumentRetention(useDefault ? null : $state.snapshot(policy));
        if (confirm && !confirm.preview.waitsForEdit) {
          await api.pruneNow();
          info = (await api.documentInfo()) ?? info;
        }
        onchanged(info);
      }
      onclose();
    });

  let clearing = $state(false);
  const clearResults = () =>
    run(async () => {
      onchanged(await api.clearResults());
      clearing = false;
      message = "Stored results were cleared.";
    });

  const trim = () =>
    run(async () => {
      if (!confirm) {
        const p = await api.retentionPreview(null);
        if (p.cutoff == null) {
          message = "Nothing to trim: the current policy keeps all of this document's history.";
        } else if (p.waitsForEdit) {
          message = "Nothing to trim until the document is edited again.";
        } else {
          confirm = { preview: p, action: "trim" };
        }
        return;
      }
      const r = await api.pruneNow();
      confirm = null;
      const info = await api.documentInfo();
      if (info) onchanged(info);
      message = r
        ? `History before version ${r.prunedBefore} was removed.` +
          (r.pinned.length ? ` Kept copies of ${r.pinned.join(", ")}.` : "")
        : "Nothing was removed.";
    });
</script>

<Modal title="Document settings" {onclose} width={640}>
  <p class="small muted head">{doc.name}</p>
  <div class="tabs" role="tablist">
    {#each TABS as t (t.id)}
      <button role="tab" aria-selected={tab === t.id} class:on={tab === t.id} disabled={confirm != null} onclick={() => (tab = t.id)}>
        {t.label}
      </button>
    {/each}
  </div>

  {#if !settings && !message}
    <p class="muted">Loading…</p>
  {:else if settings}
    {#if tab === "document"}
      <h3>Text sizes</h3>
      <label class="choice">
        <input type="radio" checked={settings.textSizes == null} onchange={() => settings && (settings.textSizes = null)} />
        Use the app's <span class="muted">({appSizes.text} / {appSizes.calc} / {appSizes.code} px)</span>
      </label>
      <label class="choice">
        <input type="radio" checked={settings.textSizes != null} onchange={() => settings && (settings.textSizes = { ...appSizes })} />
        Set for this document
      </label>
      {#if settings.textSizes}
        <div class="own">
          <TextSizeFields sizes={settings.textSizes} onchange={(t) => settings && (settings.textSizes = t)} />
        </div>
      {/if}
      <p class="small muted note">Saved in the document, so everyone who opens it sees the same sizes.</p>
    {:else if tab === "report"}
      <h3>Title block</h3>
      <div class="fields">
        {#each FIELDS as f (f.key)}
          <label for="tb-{f.key}">{f.label}</label>
          <input
            id="tb-{f.key}"
            bind:value={settings.report.title[f.key]}
            placeholder={f.key === "title" ? doc.name.replace(/\.cruncha$/, "") : (f.placeholder ?? "")}
          />
        {/each}
      </div>
      <p class="small muted note">
        Revisions on the cover are this document's snapshots. The company, logo and paper size are the office's, in
        <button class="link" onclick={onsettings}>Settings</button>.
      </p>
      <h3>Contents</h3>
      <label class="check"><input type="checkbox" bind:checked={settings.report.options.checkSummary} /> Summary of checks and problems on the cover</label>
      <label class="check"><input type="checkbox" bind:checked={settings.report.options.tableOfContents} /> Table of contents</label>
      <label class="check"><input type="checkbox" bind:checked={settings.report.options.showCode} /> Python source (output is always printed)</label>
    {:else}
      <p class="small muted">
        History on disk {bytes(doc.historyBytes)} · versions {doc.earliestVersion}–{doc.version} available
      </p>
      <label class="choice">
        <input type="radio" bind:group={useDefault} value={true} disabled={confirm != null} />
        Use the app default <span class="muted">({describeRetention(prefs.defaultRetention)})</span>
      </label>
      <label class="choice">
        <input type="radio" bind:group={useDefault} value={false} disabled={confirm != null} />
        Set for this document
      </label>
      {#if !useDefault}
        <div class="own"><RetentionControl bind:policy disabled={confirm != null} /></div>
      {/if}
      {#if confirm}
        <PruneConfirm preview={confirm.preview} docName={doc.name} />
      {/if}
      {#if !confirm}
        <p><button disabled={busy} onclick={trim}>Trim history now…</button></p>
      {/if}
      <h3>Stored results</h3>
      <p class="small muted">
        Python output kept with the document ({bytes(doc.resultsBytes)}), so cells whose inputs haven't changed
        don't run again and past versions show what they computed. It isn't part of the history size.
      </p>
      {#if clearing}
        <p class="small">
          Drop every stored result? Past versions will show their Python cells as not calculated, and cells run
          again when the document is next opened with Python.
          <button disabled={busy} onclick={() => (clearing = false)}>Cancel</button>
          <button class="danger" disabled={busy} onclick={clearResults}>Clear stored results</button>
        </p>
      {:else}
        <p><button disabled={busy || confirm != null || doc.resultsBytes === 0} onclick={() => (clearing = true)}>Clear stored results…</button></p>
      {/if}
    {/if}
  {/if}
  {#if message}<p class="message">{message}</p>{/if}

  {#snippet footer()}
    {#if confirm}
      <button onclick={() => (confirm = null)}>Back</button>
      <button class="danger" disabled={busy} onclick={confirm.action === "save" ? save : trim}>
        {confirm.action === "trim" ? "Trim history now" : confirm.preview.waitsForEdit ? "Save" : "Save and trim history"}
      </button>
    {:else}
      <button onclick={onclose}>Cancel</button>
      <button class="primary" disabled={busy || !settings} onclick={save}>Save</button>
    {/if}
  {/snippet}
</Modal>

<style>
  .head { margin: 0 0 8px; }
  .tabs { display: flex; gap: 2px; border-bottom: 1px solid var(--border); margin: 0 -18px 12px; padding: 0 18px; }
  .tabs button {
    border: none; background: none; border-radius: 6px 6px 0 0; padding: 6px 12px; color: var(--muted);
    border-bottom: 2px solid transparent; margin-bottom: -1px;
  }
  .tabs button.on { color: var(--text); border-bottom-color: var(--accent); font-weight: 600; }
  h3 { margin: 6px 0 8px; font-size: 14px; }
  * + h3 { margin-top: 14px; }
  .choice { display: block; margin: 4px 0; font-weight: 500; }
  .own { margin: 8px 0 0 24px; }
  .note { margin-top: 10px; }
  .fields { display: grid; grid-template-columns: auto 1fr; gap: 6px 12px; align-items: center; }
  .fields input { width: 100%; min-width: 0; }
  .check { display: flex; gap: 8px; align-items: center; margin: 4px 0; }
  .message { background: var(--sunken); border-radius: 6px; padding: 6px 10px; }
</style>
