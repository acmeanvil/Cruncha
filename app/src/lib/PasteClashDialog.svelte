<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<!-- Names that pasted cells, or an inserted template, define and the document already defines
     (ADR-0024 §1, ADR-0014 §3). For each: rename it in the pasted cells (the
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
  import { logAction } from "./log";

  let {
    clashes,
    error = null,
    onpaste,
    oncancel,
    title = "These names are already defined",
    what = "pasted cells",
    action = "Paste",
    canUse = () => true,
    suffixer,
  }: {
    clashes: Clash[];
    error?: string | null;
    onpaste: (choices: Choice[]) => void;
    oncancel: () => void;
    title?: string;
    /** What the cells are, in the explanation ("pasted cells", "template"). */
    what?: string;
    action?: string;
    /** Whether "use the document's" is offered for a name (a template offers it for inputs only, ADR-0014 §3). */
    canUse?: (name: string) => boolean;
    /** The renames that add one suffix to every name (old → new), for "Add to every name". */
    suffixer?: (suffix: string) => Promise<Record<string, string>>;
  } = $props();

  // The choices start from the clashes the dialog opened with.
  let choices = $state(untrack(() => clashes.map((c) => ({ name: c.name, use: false, to: c.suggestion }))));
  // A calc name: letters, digits, marks (x̄), primes, subscripts including `_(k,max)`.
  const valid = $derived(choices.every((c) => c.use || /^[\p{L}][\p{L}\p{N}\p{M}_′″‴(),]*$/u.test(c.to.trim())));
  let suffix = $state("");
  let suffixError = $state<string | null>(null);
  function cancel() {
    logAction(`Name clashes (${what}): Cancel`);
    oncancel();
  }

  function decide() {
    const out = choices.map((c) => ({ name: c.name, rename: c.use ? null : c.to.trim() }));
    const renames = out.filter((c) => c.rename !== null && c.rename !== c.name).length;
    logAction(`Name clashes (${what}): ${action} (${renames} renamed, ${out.length - renames} kept of ${out.length})`);
    onpaste(out);
  }

  async function addSuffix() {
    if (!suffixer || !suffix.trim()) return;
    try {
      const map = await suffixer(suffix);
      suffixError = null;
      choices = choices.map((c) => (map[c.name] ? { ...c, use: false, to: map[c.name] } : c));
    } catch (e) {
      suffixError = String(e);
    }
  }
</script>

<Modal {title} onclose={cancel} width={560}>
  <p class="small muted">
    A name is defined once per document. Rename it in the {what} (uses included), or keep the document's own value
    and leave the {what}'s definition out.
  </p>
  {#if suffixer}
    <div class="suffix">
      <label for="suffix-all">Add to every name</label>
      <input id="suffix-all" class="mono" placeholder="bolt1" bind:value={suffix} onkeydown={(e) => e.key === "Enter" && addSuffix()} />
      <button onclick={addSuffix} disabled={!suffix.trim()} title="S_y becomes S_(y,bolt1), Sy becomes Sy_bolt1">Apply</button>
    </div>
    {#if suffixError}<p class="small err">{suffixError}</p>{/if}
  {/if}
  <table>
    <thead><tr><th>Name</th><th>Rename to</th><th>Or</th></tr></thead>
    <tbody>
      {#each clashes as c, i (c.name)}
        <tr>
          <td class="mono">{c.name}</td>
          <td><input class="mono" bind:value={choices[i].to} disabled={choices[i].use} aria-label="New name for {c.name}" /></td>
          <td>
            {#if canUse(c.name)}
              <label title={c.others.length ? `That cell also defines ${c.others.join(", ")}, so it can't be left out` : ""}>
                <input type="checkbox" bind:checked={choices[i].use} disabled={c.others.length > 0} />
                use the document's
              </label>
            {:else}
              <span class="small muted" title="Only an input can use the document's value">rename only</span>
            {/if}
          </td>
        </tr>
      {/each}
    </tbody>
  </table>
  {#if error}<p class="small err">{error}</p>{/if}
  {#snippet footer()}
    <button onclick={cancel}>Cancel</button>
    <button
      class="primary"
      disabled={!valid}
      onclick={decide}
    >{action}</button>
  {/snippet}
</Modal>

<style>
  table { width: 100%; border-collapse: collapse; margin: 8px 0; }
  th { text-align: left; font-weight: 500; color: var(--muted); font-size: 12px; padding: 4px 6px; }
  td { padding: 4px 6px; border-top: 1px solid var(--border); }
  input:not([type]) { width: 100%; }
  .mono { font-family: var(--mono); }
  .suffix { display: flex; gap: 6px; align-items: center; margin: 8px 0 0; font-size: 12.5px; }
  .suffix input { width: 10em; }
  .err { color: var(--danger); }
</style>
