<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<!-- Document settings → History: this document's retention, or the app default. -->
<script lang="ts">
  import { untrack } from "svelte";
  import { api, type DocInfo, type Prefs, type Preview } from "./api";
  import Modal from "./Modal.svelte";
  import PruneConfirm from "./PruneConfirm.svelte";
  import RetentionControl from "./RetentionControl.svelte";
  import { bytes, describeRetention } from "./format";

  let {
    doc,
    prefs,
    onclose,
    onchanged,
  }: {
    doc: DocInfo;
    prefs: Prefs;
    onclose: () => void;
    onchanged: (d: DocInfo) => void;
  } = $props();

  let useDefault = $state(untrack(() => doc.retention == null));
  let policy = $state(
    $state.snapshot(untrack(() => doc.retention ?? prefs.defaultRetention)),
  );
  let confirm = $state<{ preview: Preview; action: "save" | "trim" } | null>(null);
  let busy = $state(false);
  let message = $state<string | null>(null);

  const chosen = $derived(useDefault ? prefs.defaultRetention : policy);

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

  const save = () =>
    run(async () => {
      if (!confirm) {
        const p = await api.retentionPreview($state.snapshot(chosen));
        if (p.cutoff != null) {
          confirm = { preview: p, action: "save" };
          return;
        }
      }
      let info = await api.setDocumentRetention(useDefault ? null : $state.snapshot(policy));
      if (confirm && !confirm.preview.waitsForEdit) {
        await api.pruneNow();
        info = (await api.documentInfo()) ?? info;
      }
      onchanged(info);
      onclose();
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

<Modal title="Document settings" {onclose}>
  <p class="small muted">
    {doc.name} · history on disk {bytes(doc.historyBytes)} · versions
    {doc.earliestVersion}–{doc.version} available
  </p>
  <h3>History</h3>
  <label class="choice">
    <input type="radio" bind:group={useDefault} value={true} disabled={confirm != null} />
    Use app default <span class="muted">({describeRetention(prefs.defaultRetention)})</span>
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
  {#if message}<p class="message">{message}</p>{/if}
  {#snippet footer()}
    {#if confirm}
      <button onclick={() => (confirm = null)}>Back</button>
      <button class="danger" disabled={busy} onclick={confirm.action === "save" ? save : trim}>
        {confirm.action === "trim"
          ? "Trim history now"
          : confirm.preview.waitsForEdit
            ? "Save"
            : "Save and trim history"}
      </button>
    {:else}
      <button class="left" disabled={busy} onclick={trim}>Trim history now…</button>
      <button onclick={onclose}>Cancel</button>
      <button class="primary" disabled={busy} onclick={save}>Save</button>
    {/if}
  {/snippet}
</Modal>

<style>
  h3 { margin: 10px 0 4px; font-size: 14px; }
  .choice { display: block; margin: 4px 0; font-weight: 500; }
  .own { margin: 6px 0 0 24px; }
  .message { background: var(--sunken); border-radius: 6px; padding: 6px 10px; }
  .left { margin-right: auto; }
</style>
