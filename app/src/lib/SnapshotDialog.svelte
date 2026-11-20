<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<!-- Take a snapshot: name the latest version as a known-good point. -->
<script lang="ts">
  import Modal from "./Modal.svelte";

  let {
    version,
    taken,
    onclose,
    onsave,
  }: {
    version: number;
    /** Names already used (they must be unique). */
    taken: string[];
    onclose: () => void;
    onsave: (name: string, note: string) => Promise<void>;
  } = $props();

  let name = $state("");
  let note = $state("");
  let busy = $state(false);
  let error = $state<string | null>(null);
  const clash = $derived(taken.includes(name.trim()));

  async function save(e?: Event) {
    e?.preventDefault();
    if (!name.trim() || clash || busy) return;
    busy = true;
    error = null;
    try {
      await onsave(name.trim(), note.trim());
      onclose();
    } catch (err) {
      error = String(err);
    } finally {
      busy = false;
    }
  }

  function focus(el: HTMLElement) {
    el.focus();
  }
</script>

<Modal title="Take a snapshot" {onclose}>
  <form onsubmit={save}>
    <p class="small muted">
      Marks the document as it is now (version {version}, plus any edits still being saved) as a
      known-good point. Snapshots are kept when older
      history is trimmed, and any of them can be restored later.
    </p>
    <label>
      Name
      <input use:focus bind:value={name} placeholder="e.g. Checked, Issued for review, Rev B" maxlength="80" />
    </label>
    {#if clash}<p class="small err">There is already a snapshot named “{name.trim()}”.</p>{/if}
    <label>
      Note <span class="muted small">(optional)</span>
      <textarea bind:value={note} rows="3" placeholder="What makes this version good: what was checked, by whom, what it was issued for"></textarea>
    </label>
    {#if error}<p class="small err" role="alert">{error}</p>{/if}
    <div class="buttons">
      <button type="button" onclick={onclose}>Cancel</button>
      <button type="submit" class="primary" disabled={!name.trim() || clash || busy}>Take snapshot</button>
    </div>
  </form>
</Modal>

<style>
  form { display: grid; gap: 10px; }
  label { display: grid; gap: 4px; font-weight: 500; }
  textarea { resize: vertical; font: inherit; }
  .err { color: var(--danger); margin: 0; }
  .buttons { display: flex; justify-content: flex-end; gap: 8px; margin-top: 4px; }
  p { margin: 0; }
</style>
