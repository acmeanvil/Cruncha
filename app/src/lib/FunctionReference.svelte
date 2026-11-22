<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<!-- The function reference: every built-in function, operator and constant of the calc language,
     searchable and by category, with a signature, what it does and a worked example (each
     example's result is checked by cruncha-eval's tests/reference.rs). Insert types it into the
     calc being edited, or into a new calc. Its buttons don't take focus, so the calc stays open. -->
<script lang="ts">
  import { onMount } from "svelte";
  import { api, type FunctionReference } from "./api";
  import { findEntries, insertion } from "./functionRef";

  let {
    editable,
    oninsert,
    onclose,
  }: {
    editable: boolean;
    oninsert: (text: string, back: number) => void;
    onclose: () => void;
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
  const groups = $derived.by(() => {
    if (query.trim()) return [{ name: "", entries: shown }];
    return (reference?.categories ?? [])
      .map((c) => ({ name: c, entries: shown.filter((e) => e.category === c) }))
      .filter((g) => g.entries.length);
  });

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
<aside class="ref" aria-label="Function reference" onkeydown={(e) => e.key === "Escape" && onclose()}>
  <header>
    <strong><i>ƒ</i>x Functions</strong>
    <span class="spacer"></span>
    <button class="x" aria-label="Close the function reference" onclick={onclose}>×</button>
  </header>
  <div class="tools">
    <input bind:this={search} type="search" placeholder="Find a function: sqrt, degrees, interpolate…" aria-label="Find a function" bind:value={query} />
    <div class="cats" role="group" aria-label="Category">
      <button class:on={category === null} onmousedown={keep} onclick={() => (category = null)}>All</button>
      {#each reference?.categories ?? [] as c (c)}
        <button class:on={category === c} onmousedown={keep} onclick={() => (category = category === c ? null : c)}>{c}</button>
      {/each}
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
    {#each groups as g (g.name)}
      {#if g.name}<h4>{g.name}</h4>{/if}
      {#each g.entries as e (e.name)}
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
          <p class="sum">{e.summary}</p>
          {#if e.caution}<p class="caution small">{e.caution}</p>{/if}
          {#if e.note}<p class="note small">{e.note}</p>{/if}
          {#if e.example}
            <pre class="ex">{e.example.join("\n")}<span class="res">  →  {e.result}</span></pre>
          {/if}
        </div>
      {/each}
    {/each}
  </div>
</aside>

<style>
  .ref {
    display: grid; grid-template-rows: auto auto 1fr; min-height: 0; height: 100%;
    background: var(--panel); border-left: 1px solid var(--border);
  }
  header { display: flex; align-items: center; padding: 8px 12px; border-bottom: 1px solid var(--border); }
  .spacer { flex: 1; }
  .x { border: none; background: none; color: var(--muted); font-size: 18px; line-height: 1; padding: 0 4px; }
  .x:hover { color: var(--text); }
  .tools { padding: 8px 12px 6px; display: grid; gap: 6px; border-bottom: 1px solid var(--border); }
  .cats { display: flex; flex-wrap: wrap; gap: 4px; }
  .cats button { font-size: 11.5px; padding: 1px 8px; border-radius: 10px; }
  .cats button.on { background: var(--accent-bg); color: var(--accent); border-color: var(--accent); }
  .list { overflow: auto; padding: 4px 12px 16px; min-height: 0; }
  h4 { font-size: 11px; text-transform: uppercase; letter-spacing: 0.06em; color: var(--muted); margin: 14px 0 4px; }
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
