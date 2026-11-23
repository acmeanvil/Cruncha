<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<!-- Names that pasted cells define and the document already defines (ADR-0024 §1, the clash
     dialog ADR-0014's insert will use too). For each: rename it in the pasted cells (the
     suggestion reads the same in calcs, Python and Rust), or use the document's own value and
     leave the pasted definition out. A cell that also defines names that don't clash can only
     be renamed. -->
<script lang="ts" module>
  export interface Clash {
    name: string;
    cells: number[];
    suggestion: string;
    others: string[];
  }
  export interface Choice {
    name: string;
    /** The new name, or null: use the document's. */
    rename: string | null;
  }
</script>

<script lang="ts">
  import { untrack } from "svelte";
  import Modal from "./Modal.svelte";

  let {
    clashes,
    error = null,
    onpaste,
    oncancel,
  }: {
    clashes: Clash[];
    error?: string | null;
    onpaste: (choices: Choice[]) => void;
    oncancel: () => void;
  } = $props();

  // The choices start from the clashes the dialog opened with.
  let choices = $state(untrack(() => clashes.map((c) => ({ name: c.name, use: false, to: c.suggestion }))));
  const valid = $derived(choices.every((c) => c.use || /^[\p{L}][\p{L}\p{N}_′]*$/u.test(c.to.trim())));
</script>

<Modal title="These names are already defined" onclose={oncancel} width={560}>
  <p class="small muted">
    A name is defined once per document. Rename it in the pasted cells (uses included), or keep the document's own value
    and leave the pasted definition out.
  </p>
  <table>
    <thead><tr><th>Name</th><th>Rename to</th><th>Or</th></tr></thead>
    <tbody>
      {#each clashes as c, i (c.name)}
        <tr>
          <td class="mono">{c.name}</td>
          <td><input class="mono" bind:value={choices[i].to} disabled={choices[i].use} aria-label="New name for {c.name}" /></td>
          <td>
            <label title={c.others.length ? `That cell also defines ${c.others.join(", ")}, so it can't be left out` : ""}>
              <input type="checkbox" bind:checked={choices[i].use} disabled={c.others.length > 0} />
              use the document's
            </label>
          </td>
        </tr>
      {/each}
    </tbody>
  </table>
  {#if error}<p class="small err">{error}</p>{/if}
  {#snippet footer()}
    <button onclick={oncancel}>Cancel</button>
    <button
      class="primary"
      disabled={!valid}
      onclick={() => onpaste(choices.map((c) => ({ name: c.name, rename: c.use ? null : c.to.trim() })))}
    >Paste</button>
  {/snippet}
</Modal>

<style>
  table { width: 100%; border-collapse: collapse; margin: 8px 0; }
  th { text-align: left; font-weight: 500; color: var(--muted); font-size: 12px; padding: 4px 6px; }
  td { padding: 4px 6px; border-top: 1px solid var(--border); }
  input:not([type]) { width: 100%; }
  .mono { font-family: var(--mono); }
  .err { color: var(--danger); }
</style>
