<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<!-- Preferences → History: the app-wide default retention for documents (ADR-0011). -->
<script lang="ts">
  import { untrack } from "svelte";
  import { api, type DocInfo, type Prefs, type Preview } from "./api";
  import Modal from "./Modal.svelte";
  import PruneConfirm from "./PruneConfirm.svelte";
  import RetentionControl from "./RetentionControl.svelte";
  import { isKeepEverything } from "./format";

  let {
    prefs,
    doc,
    onclose,
    onsaved,
  }: {
    prefs: Prefs;
    doc: DocInfo | null;
    onclose: () => void;
    onsaved: (p: Prefs) => void;
  } = $props();

  let policy = $state(structuredClone(untrack(() => prefs.defaultRetention)));
  let confirm = $state<Preview | null>(null);
  let busy = $state(false);
  let error = $state<string | null>(null);

  async function save() {
    busy = true;
    error = null;
    try {
      // The open document follows the default, so a stricter default may trim it.
      if (!confirm && doc && doc.retention == null && !isKeepEverything(policy)) {
        const p = await api.retentionPreview(policy);
        if (p.cutoff != null) {
          confirm = p;
          return;
        }
      }
      const next = { ...prefs, defaultRetention: $state.snapshot(policy) };
      await api.setPrefs(next);
      onsaved(next);
      onclose();
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }
</script>

<Modal title="Preferences" {onclose}>
  <h3>History</h3>
  <p class="small muted">
    Every change is saved as a version. This is the default for documents that don't set their
    own policy in Document settings.
  </p>
  <RetentionControl bind:policy disabled={confirm != null} />
  {#if confirm && doc}
    <PruneConfirm preview={confirm} docName={doc.name} />
  {/if}
  {#if error}<p class="error">{error}</p>{/if}
  {#snippet footer()}
    {#if confirm}
      <button onclick={() => (confirm = null)}>Back</button>
      <button class="danger" disabled={busy} onclick={save}>Save</button>
    {:else}
      <button onclick={onclose}>Cancel</button>
      <button class="primary" disabled={busy} onclick={save}>Save</button>
    {/if}
  {/snippet}
</Modal>

<style>
  h3 { margin: 4px 0; font-size: 14px; }
  .error { color: var(--danger); }
</style>
