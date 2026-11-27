<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<!-- The reference panel: functions, units (UnitReference), templates (TemplateLibrary) and
     tables (TemplateLibrary of table templates only, ADR-0027).
     The function reference: every built-in function, operator and constant of the calc language,
     searchable and by category, with a signature, what it does and a worked example (each
     example's result is checked by cruncha-eval's tests/reference.rs). Insert types it into the
     calc being edited, or into a new calc. Its buttons don't take focus, so the calc stays open. -->
<script lang="ts">
  import { keys } from "./keys";
  import { onMount } from "svelte";
  import { api, type FunctionReference } from "./api";
  import { findEntries, insertion } from "./functionRef";
  import { logAction } from "./log";
  import UnitReference from "./UnitReference.svelte";
  import TemplateLibrary from "./TemplateLibrary.svelte";
  import type { TemplateCell, TemplateDetail } from "./api";

  let {
    editable,
    oninsert,
    onclose,
    oninserttemplate,
    selection,
    tab = $bindable("functions"),
  }: {
    editable: boolean;
    oninsert: (text: string, back: number) => void;
    onclose: () => void;
    /** Insert a template into the document (ADR-0014 §3). */
    oninserttemplate: (t: TemplateDetail, suffix?: string) => Promise<void> | void;
    /** The editor's selected blocks, for Save as snippet. */
    selection: () => TemplateCell[];
    tab?: "functions" | "units" | "templates" | "tables";
  } = $props();

  let reference = $state<FunctionReference | null>(null);
  let failed = $state<string | null>(null);
  let query = $state("");
  let category = $state<string | null>(null);
  let copied = $state<string | null>(null);
  let search: HTMLInputElement | undefined = $state();

  onMount(() => {
    api.functionReference().then(
      (r) => (reference = r),
      (e) => (failed = String(e)),
    );
    search?.focus();
  });

  const shown = $derived(reference ? findEntries(reference.entries, query, category) : []);
  // One alphabetical list (search keeps its best-match-first order), as the Units tab.
  const byName = (a: { name: string }, b: { name: string }) =>
    a.name.localeCompare(b.name, undefined, { sensitivity: "base" });
  const listed = $derived(query.trim() ? shown : [...shown].sort(byName));
  const categories = $derived([...(reference?.categories ?? [])].sort((a, b) => a.localeCompare(b)));

  async function copy(text: string) {
    try {
      await navigator.clipboard.writeText(text);
      copied = text;
      setTimeout(() => (copied = copied === text ? null : copied), 1200);
    } catch {
      /* no clipboard */
    }
  }

  const keep = (e: MouseEvent) => e.preventDefault();
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<aside class="ref" aria-label="Functions, units, templates and tables" onkeydown={(e) => e.key === "Escape" && onclose()}>
  <header>
    <div class="tabs" role="tablist">
      <button role="tab" aria-selected={tab === "functions"} class:on={tab === "functions"} onmousedown={keep} onclick={() => (logAction("Reference tab: functions"), (tab = "functions"))}><i>ƒ</i>x Functions</button>
      <button role="tab" aria-selected={tab === "units"} class:on={tab === "units"} onmousedown={keep} onclick={() => (logAction("Reference tab: units"), (tab = "units"))}>Units</button>
      <button role="tab" aria-selected={tab === "templates"} class:on={tab === "templates"} onmousedown={keep} onclick={() => (logAction("Reference tab: templates"), (tab = "templates"))} title={keys("Templates and snippets (⌘⇧T)")}>Templates</button>
      <button role="tab" aria-selected={tab === "tables"} class:on={tab === "tables"} onmousedown={keep} onclick={() => (logAction("Reference tab: tables"), (tab = "tables"))} title={keys("Tables of standard parts (⌘⇧L)")}>Tables</button>
    </div>
    <span class="spacer"></span>
    <button class="x" aria-label="Close the function reference" onclick={onclose}>×</button>
  </header>
  {#if tab === "units"}
    <UnitReference {editable} {oninsert} />
  {:else if tab === "templates"}
    <TemplateLibrary {editable} oninsert={oninserttemplate} {selection} />
  {:else if tab === "tables"}
    <TemplateLibrary {editable} oninsert={oninserttemplate} {selection} tables />
  {:else}
  <div class="tools">
    <input bind:this={search} type="search" placeholder="Find a function: sqrt, degrees, interpolate…" aria-label="Find a function" bind:value={query} />
    <div class="row">
      <select aria-label="Category" bind:value={category}>
        <option value={null}>All categories</option>
        {#each categories as c (c)}<option value={c}>{c}</option>{/each}
      </select>
    </div>
  </div>
  <div class="list">
    {#if failed}
      <p class="small err">The reference didn't load: {failed}</p>
    {:else if !reference}
      <p class="small muted">Loading…</p>
    {:else if shown.length === 0}
      <p class="small muted">Nothing matches “{query}”.</p>
    {/if}
    {#each listed as e (e.name)}
        {@const ok = e.available !== false}
        {@const [text, back] = insertion(e)}
        <div class="entry" class:off={!ok}>
          <div class="sig">
            <code>{e.signature}</code>
            {#if !ok}<span class="badge">not available yet</span>{/if}
            <span class="spacer"></span>
            {#if ok && editable}
              <button class="small" onmousedown={keep} onclick={() => oninsert(text, back)}
                title="Type {text} into the calc you're editing (or a new calc)">Insert</button>
            {/if}
            <button class="small link" onmousedown={keep} onclick={() => copy(text)} title="Copy {text}">{copied === text ? "Copied" : "Copy"}</button>
          </div>
          <p class="sum">{e.summary}{#if !category}<span class="cat small muted"> · {e.category}</span>{/if}</p>
          {#if e.caution}<p class="caution small">{e.caution}</p>{/if}
          {#if e.note}<p class="note small">{e.note}</p>{/if}
          {#if e.example}
            <pre class="ex">{e.example.join("\n")}<span class="res">  →  {e.result}</span></pre>
          {/if}
        </div>
    {/each}
  </div>
  {/if}
</aside>

<style>
  .ref {
    display: grid; grid-template-rows: auto auto 1fr; min-height: 0; height: 100%;
    background: var(--panel); border-left: 1px solid var(--border);
  }
  header { display: flex; align-items: center; padding: 4px 12px 0; border-bottom: 1px solid var(--border); }
  .tabs { display: flex; gap: 2px; }
  .tabs button {
    border: none; background: none; border-radius: 0; padding: 6px 10px 5px; font-size: 13px; color: var(--muted);
    border-bottom: 2px solid transparent; margin-bottom: -1px;
  }
  .tabs button:hover { color: var(--text); }
  .tabs button.on { color: var(--text); border-bottom-color: var(--accent); font-weight: 600; }
  .spacer { flex: 1; }
  .x { border: none; background: none; color: var(--muted); font-size: 18px; line-height: 1; padding: 0 4px; }
  .x:hover { color: var(--text); }
  .tools { padding: 8px 12px 6px; display: grid; gap: 6px; border-bottom: 1px solid var(--border); }
  .row { display: flex; gap: 6px; }
  .row select { flex: 1; min-width: 0; font-size: 12.5px; }
  .list { overflow: auto; padding: 4px 12px 16px; min-height: 0; }
  .entry { padding: 8px 0; border-bottom: 1px solid color-mix(in srgb, var(--border) 60%, transparent); }
  .entry.off code { color: var(--muted); }
  .sig { display: flex; align-items: center; gap: 6px; }
  .sig button { padding: 1px 8px; font-size: 12px; }
  code { font: 13px var(--mono); }
  .badge { font-size: 11px; background: var(--sunken); border-radius: 4px; padding: 0 6px; color: var(--muted); }
  .sum { margin: 3px 0 0; font-size: 13px; }
  .caution { margin: 3px 0 0; color: var(--warn); background: var(--warn-bg); padding: 2px 6px; border-radius: 4px; }
  .note { margin: 3px 0 0; color: var(--muted); }
  .ex {
    margin: 5px 0 0; padding: 4px 8px; font: 12px var(--mono); background: var(--sunken); border-radius: 4px;
    white-space: pre-wrap; overflow-wrap: anywhere;
  }
  .res { color: var(--accent); }
  .err { color: var(--danger); }
</style>
