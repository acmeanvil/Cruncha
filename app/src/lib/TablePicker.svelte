<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<!-- A table template's table (ADR-0027 §2): filter the sizes, pick a row, untick the columns
     not wanted, and insert: a line citing the table, a calc per ticked column the row has a
     value for, then the template's own cells. Double-click a row to insert it at once. -->
<script lang="ts">
  import type { TableView } from "./api";
  import { filterRows } from "./templateRef";

  let {
    table,
    editable,
    busy,
    oninsert,
  }: {
    table: TableView;
    editable: boolean;
    busy: boolean;
    oninsert: (row: string, columns: string[] | undefined) => Promise<void> | void;
  } = $props();

  let query = $state("");
  let picked = $state<string | null>(null);
  /** Unticked columns, by var. */
  let off = $state<Set<string>>(new Set());

  const shown = $derived(filterRows(table.rows, query));
  const row = $derived(picked ? table.rows.find((r) => r[0] === picked) ?? null : null);
  const ticked = $derived(table.columns.filter((c) => !off.has(c.var)));
  const count = $derived(row ? table.columns.filter((c, i) => !off.has(c.var) && row[i + 1] !== "").length : 0);

  function toggle(v: string) {
    const next = new Set(off);
    if (next.has(v)) next.delete(v);
    else next.add(v);
    off = next;
  }
  function all(on: boolean) {
    off = on ? new Set() : new Set(table.columns.map((c) => c.var));
  }
  function insert(size = picked) {
    if (!size || !editable || busy) return;
    picked = size;
    void oninsert(size, off.size ? ticked.map((c) => c.var) : undefined);
  }
  const keep = (e: MouseEvent) => e.preventDefault();
  const unitOf = (c: TableView["columns"][number]) => (c.text || !c.unit ? "" : c.unit);
</script>

<div class="picker">
  <div class="bar">
    <input type="search" placeholder="Filter {table.rowLabel.toLowerCase()}s: M8, 1/2, 40…" aria-label="Filter rows" bind:value={query} />
    <span class="small muted">{shown.length} of {table.rows.length}</span>
  </div>
  <div class="scroll" role="grid" aria-label="{table.part} sizes">
    <table>
      <thead>
        <tr>
          <th class="size">{table.rowLabel}</th>
          {#each table.columns as c (c.var)}
            <th class:off={off.has(c.var)} title="{c.description}{unitOf(c) ? ` (${unitOf(c)})` : ''}: inserts as {c.var}">
              <label>
                <input type="checkbox" checked={!off.has(c.var)} onchange={() => toggle(c.var)} />
                <span>{c.heading}</span>
              </label>
              {#if unitOf(c)}<span class="unit">{unitOf(c)}</span>{/if}
            </th>
          {/each}
        </tr>
      </thead>
      <tbody>
        {#each shown as i (i)}
          {@const r = table.rows[i]}
          <tr class:picked={picked === r[0]} onmousedown={keep} onclick={() => (picked = r[0])} ondblclick={() => insert(r[0])}
            aria-selected={picked === r[0]} title={table.rowNotes[r[0]] ?? "Click to pick, double-click to insert"}>
            <td class="size">{r[0]}{#if table.rowNotes[r[0]]}<span class="mark">*</span>{/if}</td>
            {#each table.columns as c, k (c.var)}
              <td class:off={off.has(c.var)} class:text={c.text}>{r[k + 1]}</td>
            {/each}
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
  <div class="go">
    <div class="small muted cols">
      Columns: <button class="link" onmousedown={keep} onclick={() => all(true)}>all</button> ·
      <button class="link" onmousedown={keep} onclick={() => all(false)}>none</button>
    </div>
    <button class="primary" disabled={!editable || busy || !row || count === 0} onmousedown={keep} onclick={() => insert()}
      title={editable ? "Insert the row's values as calcs after the current block (names are checked first)" : "Open the latest version to insert"}>
      {row ? `Insert ${row[0]}` : "Pick a row"}
    </button>
    {#if row}
      <p class="small muted">
        {count} {count === 1 ? "value" : "values"}{#if table.rowNotes[row[0]]}. Note: {table.rowNotes[row[0]]}{/if}
      </p>
    {/if}
  </div>
  <p class="small muted src">{table.source}</p>
  {#if table.notes.length}
    <ul class="notes small">{#each table.notes as n}<li>{n}</li>{/each}</ul>
  {/if}
</div>

<style>
  .picker { display: grid; gap: 6px; margin: 6px 0; }
  .bar { display: flex; gap: 8px; align-items: center; }
  .bar input { flex: 1; min-width: 0; }
  .scroll { overflow: auto; max-height: 340px; border: 1px solid var(--border); border-radius: 4px; }
  table { border-collapse: separate; border-spacing: 0; font: 12px var(--mono); }
  th, td { padding: 2px 8px; white-space: nowrap; text-align: right; border-bottom: 1px solid color-mix(in srgb, var(--border) 50%, transparent); }
  th { position: sticky; top: 0; background: var(--sunken); font-weight: 600; z-index: 1; vertical-align: bottom; }
  th label { display: flex; gap: 3px; align-items: center; justify-content: flex-end; cursor: pointer; }
  th input { margin: 0; }
  .unit { display: block; font-weight: 400; color: var(--muted); font-size: 10.5px; }
  .size { position: sticky; left: 0; text-align: left; background: var(--bg); z-index: 1; font-weight: 600; }
  th.size { z-index: 2; background: var(--sunken); }
  td.text { text-align: left; white-space: normal; min-width: 12em; font-family: inherit; }
  .off { color: var(--muted); opacity: 0.55; }
  tbody tr { cursor: pointer; }
  tbody tr:hover td { background: var(--sunken); }
  tr.picked td { background: color-mix(in srgb, var(--accent) 16%, var(--bg)); }
  .mark { color: var(--accent); margin-left: 2px; }
  .go { display: grid; grid-template-columns: 1fr auto; gap: 4px 8px; align-items: center; }
  .go p { grid-column: 1 / -1; margin: 0; }
  .go button.primary { font-size: 12.5px; padding: 3px 10px; }
  .link { border: none; background: none; padding: 0; color: var(--accent); cursor: pointer; font-size: inherit; }
  .src { margin: 2px 0 0; }
  .notes { margin: 0; padding-left: 18px; }
  .notes li { margin: 2px 0; }
</style>
