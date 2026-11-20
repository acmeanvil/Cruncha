<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<!-- Every name the document defines: value, description, where it is defined and what uses
     it. Click to jump to the definition; descriptions can be edited here too. -->
<script lang="ts">
  import temml from "temml";
  import { SvelteSet } from "svelte/reactivity";
  import { isTall, matches, type Variable } from "./editor/variables";

  let {
    variables,
    editable,
    onjump,
    ondescribe,
  }: {
    variables: Variable[];
    editable: boolean;
    onjump: (id: string) => void;
    ondescribe: (id: string, description: string) => void;
  } = $props();

  let query = $state("");
  let order = $state<"document" | "name">("document");
  let editing = $state<string | null>(null);
  let draft = $state("");
  /** Expanded multi-line values, by name (collapsed by default). */
  const expanded = new SvelteSet<string>();
  const tall = $derived(variables.filter(isTall));
  const allOpen = $derived(tall.length > 0 && tall.every((v) => expanded.has(v.name)));

  function toggle(name: string) {
    if (expanded.has(name)) expanded.delete(name);
    else expanded.add(name);
  }

  function toggleAll() {
    if (allOpen) expanded.clear();
    else for (const v of tall) expanded.add(v.name);
  }

  const shown = $derived.by(() => {
    const vs = variables.filter((v) => matches(v, query));
    return order === "name" ? [...vs].sort((a, b) => a.name.localeCompare(b.name, undefined, { sensitivity: "base" })) : vs;
  });

  /** Typeset a value (Temml emits MathML; trust is off, so no raw HTML passes through). */
  function math(tex: string): string {
    try {
      return temml.renderToString(tex, { throwOnError: true });
    } catch {
      return "";
    }
  }

  /** `M_allow` as M with a subscript, Hurmet's convention for names. */
  function nameParts(name: string): [string, string] {
    const i = name.indexOf("_");
    return i > 0 ? [name.slice(0, i), name.slice(i + 1)] : [name, ""];
  }

  function startEdit(v: Variable) {
    editing = v.id;
    draft = v.description;
  }

  function finish(v: Variable, save: boolean) {
    if (editing !== v.id) return;
    editing = null;
    if (save && draft.trim() !== v.description) ondescribe(v.id, draft);
  }

  function focus(el: HTMLElement) {
    el.focus();
  }
</script>

<div class="vars">
  <div class="search">
    <input type="search" placeholder="Find a variable or description" aria-label="Find a variable" bind:value={query} />
    <select aria-label="Order" bind:value={order}>
      <option value="document">In order</option>
      <option value="name">A–Z</option>
    </select>
  </div>
  {#if tall.length}
    <p class="bulk small"><button class="link" onclick={toggleAll}>{allOpen ? "Collapse" : "Expand"} all arrays</button></p>
  {/if}
  {#if variables.length === 0}
    <p class="empty small muted">Variables appear here as calcs define them, e.g. <code>L = 24ft</code>.</p>
  {:else if shown.length === 0}
    <p class="empty small muted">No variable matches “{query}”.</p>
  {:else}
    <ul>
      {#each shown as v, i (v.id + ":" + v.name)}
        {#if order === "document" && !query && v.section && v.section !== shown[i - 1]?.section}
          <li class="section">{v.section}</li>
        {/if}
        {@const multi = isTall(v)}
        {@const open = multi && expanded.has(v.name)}
        <li class="var" class:error={v.error}>
          <div class="row">
          <button class="main" title={v.text || v.error || ""} onclick={() => onjump(v.id)}>
            <span class="name">
              {#if nameParts(v.name)[1]}{nameParts(v.name)[0]}<sub>{nameParts(v.name)[1]}</sub>{:else}{v.name}{/if}
            </span>
            <span class="value">
              {#if multi && v.tex}
                <span class="shape">{v.shape ?? "multi-line value"}</span>
              {:else if v.tex}
                {@html math(v.tex)}
              {:else if v.summary}
                <span class="summary" title="A Python object: Python cells can use it, calcs can't">{v.summary}</span>
              {:else if v.error}
                <span class="err">{v.error}</span>
              {:else}
                <span class="muted">—</span>
              {/if}
            </span>
          </button>
          {#if multi && v.tex}
            <button
              class="toggle"
              aria-expanded={open}
              aria-label="{open ? 'Collapse' : 'Expand'} {v.name}"
              title={open ? "Collapse" : "Show the whole value"}
              onclick={() => toggle(v.name)}>{open ? "▾" : "▸"}</button>
          {/if}
          </div>
          {#if open}
            <div class="full">{@html math(v.tex ?? "")}</div>
          {/if}
          {#if editing === v.id}
            <input
              class="desc-input"
              use:focus
              bind:value={draft}
              placeholder="Describe {v.name}"
              aria-label="Description of {v.name}"
              onkeydown={(e) => {
                if (e.key === "Enter") finish(v, true);
                else if (e.key === "Escape") finish(v, false);
              }}
              onblur={() => finish(v, true)}
            />
          {:else if v.description}
            {#if editable && v.kind === "calc"}
              <button class="desc" title="Edit the description" onclick={() => startEdit(v)}>{v.description}</button>
            {:else}
              <span class="desc">{v.description}</span>
            {/if}
          {:else if editable && v.kind === "calc"}
            <button class="desc add" onclick={() => startEdit(v)}>Add description</button>
          {/if}
          <span class="meta small muted">
            {#if v.kind !== "calc"}<span class="kind">{v.kind}</span>{/if}
            {#if v.usedBy.length}used by {v.usedBy.join(", ")}{:else}not used{/if}
            {#if v.warnings.length}<span class="lint" title={v.warnings.join("\n")}>!</span>{/if}
          </span>
        </li>
      {/each}
    </ul>
    <p class="count small muted">{shown.length} of {variables.length} variables</p>
  {/if}
</div>

<style>
  .vars { padding: 8px 10px; }
  .search { display: flex; gap: 6px; margin-bottom: 6px; position: sticky; top: 0; background: var(--panel); padding-bottom: 4px; z-index: 1; }
  .search input { flex: 1; min-width: 0; }
  .search select { flex: none; }
  ul { list-style: none; margin: 0; padding: 0; }
  .section {
    font-size: 11px; text-transform: uppercase; letter-spacing: 0.06em; color: var(--muted);
    margin: 10px 4px 2px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
  }
  .var { display: grid; padding: 4px 6px; border-radius: 6px; margin: 1px 0; }
  .var:hover { background: var(--sunken); }
  .var.error { box-shadow: inset 2px 0 0 var(--danger); }
  .row { display: flex; align-items: baseline; gap: 4px; min-width: 0; }
  .toggle {
    flex: none; border: none; background: none; padding: 0 4px; color: var(--muted); font-size: 12px; line-height: 1;
    cursor: pointer; border-radius: 4px;
  }
  .toggle:hover { color: var(--text); background: var(--panel); }
  .shape { font: 12px var(--sans); color: var(--muted); }
  .full {
    overflow: auto; max-height: 260px; margin: 4px 0 2px; padding: 4px 6px; font-size: 13.5px;
    background: var(--panel); border: 1px solid var(--border); border-radius: 4px;
  }
  .full :global(math) { display: block; text-align: left; }
  .bulk { margin: 0 4px 4px; text-align: right; }
  .main {
    display: flex; align-items: baseline; gap: 8px; border: none; background: none; padding: 0;
    text-align: left; width: 100%; min-width: 0;
  }
  .name { font: 600 14px var(--doc); font-style: italic; flex: none; }
  .name sub { font-style: normal; font-weight: 500; }
  .value { margin-left: auto; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; min-width: 0; font-size: 13.5px; }
  .err { color: var(--danger); font-size: 12px; }
  .summary { font: 12px var(--mono); color: var(--muted); }
  .desc {
    border: none; background: none; padding: 0; text-align: left; font: italic 13px/1.35 var(--doc);
    color: var(--muted); cursor: text;
  }
  .desc.add { opacity: 0; font-style: normal; font-family: var(--sans); font-size: 12px; }
  .var:hover .desc.add, .desc.add:focus { opacity: 1; }
  .desc-input { font: italic 13px var(--doc); padding: 2px 6px; margin: 2px 0; }
  .meta { display: flex; gap: 6px; align-items: center; }
  .kind { background: var(--sunken); border-radius: 4px; padding: 0 5px; text-transform: capitalize; }
  .lint {
    width: 14px; height: 14px; border-radius: 50%; font: 700 10px/14px var(--sans); text-align: center;
    color: var(--warn); background: var(--warn-bg); cursor: help;
  }
  .empty { padding: 8px 6px; }
  .count { text-align: center; margin: 10px 0 4px; }
  code { font: 12px var(--mono); }
</style>
