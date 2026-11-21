<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<!-- Report settings: the cover's title block and what the report includes. Kept in the
     document (its meta cell), so changes are versions like any other edit. -->
<script lang="ts">
  import { onMount } from "svelte";
  import { api, type CommitView, type ReportSettings, type TitleBlock } from "./api";
  import Modal from "./Modal.svelte";

  let {
    docName,
    onclose,
    onsaved,
  }: {
    docName: string;
    onclose: () => void;
    onsaved: (v: CommitView) => void;
  } = $props();

  let settings = $state<ReportSettings | null>(null);
  let busy = $state(false);
  let error = $state<string | null>(null);

  const FIELDS: { key: keyof TitleBlock; label: string; placeholder?: string }[] = [
    { key: "title", label: "Title", placeholder: "" },
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
      settings = await api.reportSettings();
    } catch (e) {
      error = String(e);
    }
  });

  async function save() {
    if (!settings) return;
    busy = true;
    error = null;
    try {
      onsaved(await api.setReportSettings($state.snapshot(settings)));
      onclose();
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }
</script>

<Modal title="Report settings" {onclose}>
  {#if settings}
    <h3>Title block</h3>
    <div class="fields">
      {#each FIELDS as f (f.key)}
        <label for="tb-{f.key}">{f.label}</label>
        <input
          id="tb-{f.key}"
          bind:value={settings.title[f.key]}
          placeholder={f.key === "title" ? docName.replace(/\.cruncha$/, "") : (f.placeholder ?? "")}
        />
      {/each}
    </div>
    <p class="small muted">Revisions on the cover are this document's snapshots. The company, logo and paper size are in Preferences.</p>
    <h3>Contents</h3>
    <label class="check"><input type="checkbox" bind:checked={settings.options.checkSummary} /> Summary of checks and problems on the cover</label>
    <label class="check"><input type="checkbox" bind:checked={settings.options.tableOfContents} /> Table of contents</label>
    <label class="check"><input type="checkbox" bind:checked={settings.options.showCode} /> Python source (output is always printed)</label>
  {:else if !error}
    <p class="muted">Loading…</p>
  {/if}
  {#if error}<p class="error">{error}</p>{/if}
  {#snippet footer()}
    <button onclick={onclose}>Cancel</button>
    <button class="primary" disabled={busy || !settings} onclick={save}>Save</button>
  {/snippet}
</Modal>

<style>
  h3 { margin: 4px 0 8px; font-size: 14px; }
  p + h3 { margin-top: 14px; }
  .fields { display: grid; grid-template-columns: auto 1fr; gap: 6px 12px; align-items: center; }
  .fields input { width: 100%; min-width: 0; }
  .check { display: flex; gap: 8px; align-items: center; margin: 4px 0; }
  .error { color: var(--danger); }
</style>
