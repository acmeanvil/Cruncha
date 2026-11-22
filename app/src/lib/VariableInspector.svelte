<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<!-- One variable in detail. For a vector, matrix or data frame: its shape, each column's unit,
     type and empty cells, and the first and last rows (the engine sends only those, so a large
     frame is cheap to show). Otherwise the whole value, typeset. -->
<script lang="ts">
  import temml from "temml";
  import { typeOf, unitOf, type Variable } from "./editor/variables";

  let { v, onjump, onclose }: { v: Variable; onjump: () => void; onclose: () => void } = $props();

  const p = $derived(v.preview);
  const cols = $derived(p ? p.columns.slice(0, p.shownColumns) : []);
  const hidden = $derived(p ? p.rows - p.head.length - p.tail.length : 0);

  function math(tex: string): string {
    try {
      return temml.renderToString(tex, { throwOnError: true, displayMode: true });
    } catch {
      return "";
    }
  }
</script>

<aside class="insp" aria-label="Inspector: {v.name}">
  <header>
    <strong class="name">{v.name}</strong>
    <span class="muted small">{v.kind} · {typeOf(v)}{v.shape ? ` · ${v.shape}` : ""}{unitOf(v) ? ` · ${unitOf(v)}` : ""}</span>
    <span class="spacer"></span>
    <button class="link small" onclick={onjump}>Go to definition</button>
    <button class="x" aria-label="Close the inspector" onclick={onclose}>×</button>
  </header>
  <div class="body">
    {#if v.description}<p class="desc">{v.description}</p>{/if}
    {#if v.error}<p class="err small">{v.error}</p>{/if}
    {#if p}
      <details class="schema" open={p.columns.length <= 12}>
        <summary class="small">{p.columns.length} {p.columns.length === 1 ? "column" : "columns"}, {p.rows} {p.rows === 1 ? "row" : "rows"}</summary>
        <table class="cols">
          <thead><tr><th>Column</th><th>Unit</th><th>Type</th><th>Empty</th></tr></thead>
          <tbody>
            {#each p.columns as c, i (i)}
              <tr><td>{c.name}{p.rowNames && i === 0 ? " (row names)" : ""}</td><td>{c.unit ?? ""}</td><td>{c.dtype}</td><td>{c.missing || ""}</td></tr>
            {/each}
          </tbody>
        </table>
      </details>
      <div class="grid">
        <table class="data">
          <thead>
            <tr>
              <th class="idx">#</th>
              {#each cols as c, i (i)}<th>{c.name}{#if c.unit}<span class="u">{c.unit}</span>{/if}</th>{/each}
              {#if p.shownColumns < p.columns.length}<th class="more">+{p.columns.length - p.shownColumns}</th>{/if}
            </tr>
          </thead>
          <tbody>
            {#each p.head as row, r (r)}
              <tr><td class="idx">{r + 1}</td>{#each row as x, i (i)}<td class:num={cols[i]?.dtype === "number"}>{x}</td>{/each}</tr>
            {/each}
            {#if p.tail.length}
              <tr class="gap"><td class="idx">⋮</td><td colspan={cols.length}>{hidden} more {hidden === 1 ? "row" : "rows"}</td></tr>
              {#each p.tail as row, r (r)}
                <tr><td class="idx">{p.rows - p.tail.length + r + 1}</td>{#each row as x, i (i)}<td class:num={cols[i]?.dtype === "number"}>{x}</td>{/each}</tr>
              {/each}
            {/if}
          </tbody>
        </table>
      </div>
    {:else if v.tex}
      <div class="tex">{@html math(v.tex)}</div>
    {:else if v.summary}
      <p class="summary">{v.summary}</p>
      <p class="small muted">A Python object: Python cells can use it, calcs can't.</p>
    {/if}
    {#if v.text && !p}<p class="line small muted">{v.text}</p>{/if}
    <p class="small muted">
      {v.section ? `In “${v.section}” · ` : ""}{v.usedBy.length ? `used by ${v.usedBy.join(", ")}` : "not used"}
    </p>
    {#each v.warnings as w}<p class="warn small">{w}</p>{/each}
  </div>
</aside>

<style>
  .insp { display: grid; grid-template-rows: auto 1fr; min-height: 0; min-width: 0; border-left: 1px solid var(--border); }
  header { display: flex; align-items: baseline; gap: 8px; padding: 6px 10px; border-bottom: 1px solid var(--border); }
  .name { font: italic 600 14px var(--doc); }
  .spacer { flex: 1; }
  .x { border: none; background: none; color: var(--muted); font-size: 16px; line-height: 1; padding: 0 4px; }
  .x:hover { color: var(--text); }
  .body { overflow: auto; padding: 8px 10px; min-height: 0; }
  .desc { font: italic 13.5px var(--doc); margin: 0 0 6px; }
  .err { color: var(--danger); }
  .warn { color: var(--warn); }
  .schema summary { cursor: pointer; color: var(--muted); margin-bottom: 4px; }
  table { border-collapse: collapse; font-size: 12.5px; }
  th, td { padding: 2px 8px; text-align: left; white-space: nowrap; border-bottom: 1px solid color-mix(in srgb, var(--border) 50%, transparent); }
  th { font-weight: 600; color: var(--muted); font-size: 11.5px; }
  .cols { margin-bottom: 8px; }
  .grid { overflow: auto; max-height: 100%; border: 1px solid var(--border); border-radius: 4px; }
  .data { font: 12px var(--mono); width: max-content; }
  .data th { position: sticky; top: 0; background: var(--panel); font-family: var(--sans); }
  .data .u { display: block; font-weight: normal; font-size: 10.5px; }
  .data .num { text-align: right; font-variant-numeric: tabular-nums; }
  .idx { color: var(--muted); text-align: right; background: var(--sunken); }
  .gap td { color: var(--muted); font-family: var(--sans); font-style: italic; }
  .more { color: var(--muted); }
  .tex { overflow: auto; font-size: 14px; }
  .tex :global(math) { text-align: left; }
  .summary { font: 12.5px var(--mono); }
  .line { font-family: var(--mono); white-space: pre-wrap; }
</style>
