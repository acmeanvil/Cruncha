<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<!-- The Units tab of the reference panel: every unit a calc can use (cruncha_units::catalog),
     by quantity, with its aliases and its size in SI. Search finds prefixed forms too (kN,
     MPa); the prefix menu applies one to every unit that takes it. Insert types the unit,
     quoted, into the calc being edited (or a new calc). Its buttons don't take focus. -->
<script lang="ts">
  import { onMount } from "svelte";
  import { api, type UnitCatalog } from "./api";
  import { findUnits, sizeText, takesPrefix, unitInsertion, type UnitHit } from "./unitRef";

  let {
    editable,
    oninsert,
  }: {
    editable: boolean;
    oninsert: (text: string, back: number) => void;
  } = $props();

  let catalog = $state<UnitCatalog | null>(null);
  let failed = $state<string | null>(null);
  let query = $state("");
  let quantity = $state<string | null>(null);
  let prefix = $state("");
  let copied = $state<string | null>(null);
  let search: HTMLInputElement | undefined = $state();
  const PAGE = 60;
  let limit = $state(PAGE);

  onMount(() => {
    api.unitCatalog().then(
      (c) => (catalog = c),
      (e) => (failed = String(e)),
    );
    search?.focus();
  });

  const hits = $derived.by((): UnitHit[] => {
    if (!catalog) return [];
    return findUnits(catalog, query, quantity).map((h) =>
      !h.prefix && prefix && takesPrefix(h.unit, prefix) ? { ...h, prefix } : h,
    );
  });
  const groups = $derived.by(() => {
    const shown = hits.slice(0, limit);
    if (query.trim()) return [{ name: "", hits: shown }];
    return (catalog?.quantities ?? [])
      .map((q) => ({ name: q.name, hits: shown.filter((h) => h.unit.quantity === q.name) }))
      .filter((g) => g.hits.length);
  });

  $effect(() => {
    void query;
    void quantity;
    limit = PAGE;
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

<div class="tools">
  <input bind:this={search} type="search" placeholder="Find a unit: ft, kip, MPa, kilonewton, pressure…" aria-label="Find a unit" bind:value={query} />
  <div class="row">
    <select aria-label="Quantity" bind:value={quantity}>
      <option value={null}>All quantities</option>
      {#each catalog?.quantities ?? [] as q (q.name)}<option value={q.name}>{q.name}</option>{/each}
    </select>
    <select aria-label="Prefix" bind:value={prefix} title="Apply an SI prefix to every unit that takes one">
      <option value="">No prefix</option>
      {#each catalog?.prefixes ?? [] as p (p.word)}
        <option value={p.symbol || p.word}>{p.symbol ? `${p.symbol} (${p.word})` : p.word}</option>
      {/each}
    </select>
  </div>
  <p class="small muted hint">
    In a calc, quote a unit: <code>hypot(3 'm', 4 'm')</code>. Combine them: <code>'kip·ft'</code>, <code>'N/m²'</code>, <code>'ft/s^2'</code>.
  </p>
</div>
<div class="list">
  {#if failed}
    <p class="small err">The units didn't load: {failed}</p>
  {:else if !catalog}
    <p class="small muted">Loading…</p>
  {:else if hits.length === 0}
    <p class="small muted">No unit matches “{query}”.</p>
  {/if}
  {#each groups as g (g.name)}
    {#if g.name}<h4>{g.name}</h4>{/if}
    {#each g.hits as h (h.unit.name + h.prefix)}
      {@const text = unitInsertion(h)}
      {@const ok = !h.unit.currency}
      <div class="entry" class:off={!ok}>
        <div class="sig">
          <code class="name">{text.slice(1, -1)}</code>
          {#if h.unit.aliases.length && !h.prefix}<span class="also small muted">also {h.unit.aliases.join(", ")}</span>{/if}
          <span class="spacer"></span>
          {#if ok && editable}
            <button class="small" onmousedown={keep} onclick={() => oninsert(text, 0)} title="Type {text} into the calc you're editing (or a new calc)">Insert</button>
          {/if}
          <button class="small link" onmousedown={keep} onclick={() => copy(text)} title="Copy {text}">{copied === text ? "Copied" : "Copy"}</button>
        </div>
        <div class="meta small">
          <span>{sizeText(catalog!, h)}</span>
          {#if h.unit.prefixes && !h.prefix}<span class="badge" title={h.unit.prefixes === "symbol" ? "e.g. k, M, m, µ before the symbol" : "e.g. kilo, mega before the word"}>SI prefixes</span>{/if}
          {#if h.unit.currency}<span class="badge">needs exchange rates (not available yet)</span>{/if}
        </div>
      </div>
    {/each}
  {/each}
  {#if hits.length > limit}
    <button class="link small more" onmousedown={keep} onclick={() => (limit += PAGE * 3)}>Show more ({hits.length - limit} left)</button>
  {/if}
</div>

<style>
  .tools { padding: 8px 12px 6px; display: grid; gap: 6px; border-bottom: 1px solid var(--border); }
  .row { display: flex; gap: 6px; }
  .row select { flex: 1; min-width: 0; font-size: 12.5px; }
  .hint { margin: 0; }
  .list { overflow: auto; padding: 4px 12px 16px; min-height: 0; }
  h4 { font-size: 11px; text-transform: uppercase; letter-spacing: 0.06em; color: var(--muted); margin: 14px 0 4px; }
  .entry { padding: 6px 0; border-bottom: 1px solid color-mix(in srgb, var(--border) 60%, transparent); }
  .entry.off .name { color: var(--muted); }
  .sig { display: flex; align-items: baseline; gap: 6px; min-width: 0; }
  .sig button { padding: 1px 8px; font-size: 12px; align-self: center; }
  .name { font: 600 13px var(--mono); }
  .also { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; min-width: 0; }
  .spacer { flex: 1; }
  .meta { display: flex; gap: 6px; align-items: center; flex-wrap: wrap; margin-top: 2px; color: var(--muted); }
  .badge { font-size: 11px; background: var(--sunken); border-radius: 4px; padding: 0 6px; }
  code { font: 12px var(--mono); }
  .err { color: var(--danger); }
  .more { margin-top: 8px; }
</style>
