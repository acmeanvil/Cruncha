<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<!-- Preferences: editor text sizes, and the app-wide default retention for documents (ADR-0011). -->
<script lang="ts">
  import { untrack } from "svelte";
  import { api, DEFAULT_TEXT_SIZES, type DocInfo, type Prefs, type Preview, type TextSizes } from "./api";
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

  let policy = $state($state.snapshot(untrack(() => prefs.defaultRetention)));
  let sizes = $state<TextSizes>({ ...DEFAULT_TEXT_SIZES, ...untrack(() => prefs.textSizes) });
  const SIZE_ROWS: { key: keyof TextSizes; label: string; hint: string }[] = [
    { key: "text", label: "Text", hint: "prose; headings scale with it (H1 2×, H2 1.5×, H3 1.17×)" },
    { key: "calc", label: "Calcs", hint: "typeset math, sources and descriptions" },
    { key: "code", label: "Code", hint: "Python cells and their output" },
  ];
  const clamp = (n: number, d: number) => (Number.isFinite(n) ? Math.min(32, Math.max(9, n)) : d);
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
      const textSizes = {
        text: clamp(sizes.text, DEFAULT_TEXT_SIZES.text),
        calc: clamp(sizes.calc, DEFAULT_TEXT_SIZES.calc),
        code: clamp(sizes.code, DEFAULT_TEXT_SIZES.code),
      };
      const next = { ...$state.snapshot(prefs), defaultRetention: $state.snapshot(policy), textSizes };
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
  <h3>Text sizes</h3>
  <div class="sizes">
    {#each SIZE_ROWS as row (row.key)}
      <label for="size-{row.key}">{row.label}</label>
      <span><input id="size-{row.key}" type="number" min="9" max="32" step="0.5" bind:value={sizes[row.key]} /> px</span>
      <span class="small muted">{row.hint}</span>
    {/each}
  </div>
  <p class="small"><button class="link" onclick={() => (sizes = { ...DEFAULT_TEXT_SIZES })}>Reset to defaults</button></p>
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
  h3 + .sizes { margin-top: 8px; }
  .sizes { display: grid; grid-template-columns: auto auto 1fr; gap: 6px 12px; align-items: center; }
  .sizes input { width: 5em; }
  p + h3 { margin-top: 14px; }
  .error { color: var(--danger); }
</style>
