<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<!-- The bottom panel's Variables tab: every name the document defines as a table (name, value,
     unit, type, size, description, section, used by), searchable and sortable. Selecting a row
     shows it in the inspector; double-clicking jumps to its definition. -->
<script lang="ts">
  import temml from "temml";
  import { matches, typeOf, unitOf, type Variable } from "./editor/variables";
  import VariableInspector from "./VariableInspector.svelte";

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

  type Col = "document" | "name" | "type" | "size" | "unit" | "section";
  let query = $state("");
  let sort = $state<Col>("document");
  let desc = $state(false);
  let picked = $state<string | null>(null);
  let editing = $state<string | null>(null);
  let draft = $state("");

  const key = (v: Variable) => `${v.id}:${v.name}`;
  const rowsOf = (v: Variable) => v.preview?.rows ?? 1;
  const shown = $derived.by(() => {
    const vs = variables.filter((v) => matches(v, query) || typeOf(v).includes(query.trim().toLowerCase()));
    if (sort === "document") return desc ? [...vs].reverse() : vs;
    const by: Record<Exclude<Col, "document">, (v: Variable) => string | number> = {
      name: (v) => v.name.toLowerCase(),
      type: (v) => typeOf(v),
      size: (v) => rowsOf(v) * (v.preview?.columns.length ?? 1),
      unit: (v) => unitOf(v),
      section: (v) => v.section.toLowerCase(),
    };
    const f = by[sort];
    const out = [...vs].sort((a, b) => {
      const x = f(a), y = f(b);
      return x < y ? -1 : x > y ? 1 : 0;
    });
    return desc ? out.reverse() : out;
  });
  const current = $derived(variables.find((v) => key(v) === picked) ?? null);

  function sortBy(c: Col) {
    if (sort === c) desc = !desc;
    else {
      sort = c;
      desc = false;
    }
  }

  function math(tex: string): string {
    try {
      return temml.renderToString(tex, { throwOnError: true });
    } catch {
      return "";
    }
  }

  function nameParts(name: string): [string, string] {
    const i = name.indexOf("_");
    return i > 0 ? [name.slice(0, i), name.slice(i + 1)] : [name, ""];
  }

  /** A one-line value: scalars typeset; arrays and frames as their size. */
  function valueOf(v: Variable): { tex?: string; text?: string; err?: boolean } {
    if (v.error && !v.tex && !v.summary) return { text: v.error, err: true };
    if (v.preview || v.shape) return { text: v.shape ?? "" };
    if (v.tex && !/\\begin\{|\\\\/.test(v.tex)) return { tex: v.tex };
    if (v.summary) return { text: v.summary };
    return { text: "—" };
  }

  function sizeOf(v: Variable): string {
    const p = v.preview;
    if (!p) return v.valueKind === "python" ? "" : "1";
    const cols = p.columns.length - (p.rowNames ? 1 : 0);
    return cols === 1 ? `${p.rows}` : `${p.rows} × ${cols}`;
  }

  function startEdit(v: Variable) {
    editing = key(v);
    draft = v.description;
  }

  function finish(v: Variable, save: boolean) {
    if (editing !== key(v)) return;
    editing = null;
    if (save && draft.trim() !== v.description) ondescribe(v.id, draft);
  }

  function focus(el: HTMLElement) {
    el.focus();
  }

  function onkey(e: KeyboardEvent) {
    if (!shown.length || (e.key !== "ArrowDown" && e.key !== "ArrowUp" && e.key !== "Enter")) return;
    e.preventDefault();
    const i = shown.findIndex((v) => key(v) === picked);
    if (e.key === "Enter") {
      if (current) onjump(current.id);
      return;
    }
    const j = e.key === "ArrowDown" ? Math.min(i + 1, shown.length - 1) : Math.max(i - 1, 0);
    picked = key(shown[j]);
    document.getElementById(`var-${picked}`)?.scrollIntoView({ block: "nearest" });
  }

  const COLS: { id: Col; label: string; cls: string }[] = [
    { id: "name", label: "Name", cls: "c-name" },
    { id: "document", label: "Value", cls: "c-value" },
    { id: "unit", label: "Unit", cls: "c-unit" },
    { id: "type", label: "Type", cls: "c-type" },
    { id: "size", label: "Size", cls: "c-size" },
  ];
</script>

<div class="wrap" class:split={current}>
  <div class="left">
    <div class="tools">
      <input type="search" placeholder="Find a variable, description, section or type" aria-label="Find a variable" bind:value={query} />
      <span class="small muted">{shown.length === variables.length ? `${variables.length} variables` : `${shown.length} of ${variables.length}`}</span>
    </div>
    {#if variables.length === 0}
      <p class="empty small muted">Variables appear here as cells define them, e.g. <code>L = 24 ft</code>.</p>
    {:else if shown.length === 0}
      <p class="empty small muted">No variable matches “{query}”.</p>
    {:else}
      <div class="scroll">
        <table aria-label="Variables" role="grid" tabindex="0" onkeydown={onkey}>
          <thead>
            <tr>
              {#each COLS as c (c.id)}
                <th class={c.cls} aria-sort={sort === c.id ? (desc ? "descending" : "ascending") : "none"}>
                  <button onclick={() => sortBy(c.id)} title={c.id === "document" ? "Document order" : `Sort by ${c.label.toLowerCase()}`}>
                    {c.id === "document" ? (sort === "document" ? "Value (in order)" : "Value") : c.label}
                    {#if sort === c.id}<span aria-hidden="true">{desc ? "▾" : "▴"}</span>{/if}
                  </button>
                </th>
              {/each}
              <th class="c-desc">Description</th>
              <th class="c-section"><button onclick={() => sortBy("section")}>Section{#if sort === "section"}<span aria-hidden="true">{desc ? "▾" : "▴"}</span>{/if}</button></th>
              <th class="c-used">Used by</th>
            </tr>
          </thead>
          <tbody>
            {#each shown as v (key(v))}
              {@const val = valueOf(v)}
              <tr
                id="var-{key(v)}"
                class:sel={picked === key(v)}
                class:error={v.error}
                onclick={() => (picked = picked === key(v) ? null : key(v))}
                ondblclick={() => onjump(v.id)}
                title="Double-click to go to the definition"
              >
                <td class="c-name">
                  <span class="name">{#if nameParts(v.name)[1]}{nameParts(v.name)[0]}<sub>{nameParts(v.name)[1]}</sub>{:else}{v.name}{/if}</span>
                  {#if v.warnings.length}<span class="lint" title={v.warnings.join("\n")}>!</span>{/if}
                </td>
                <td class="c-value" class:err={val.err}>
                  {#if val.tex}{@html math(val.tex)}{:else}<span class="plain">{val.text}</span>{/if}
                </td>
                <td class="c-unit">{unitOf(v)}</td>
                <td class="c-type"><span class="kind">{v.kind}</span> {typeOf(v)}</td>
                <td class="c-size">{sizeOf(v)}</td>
                <td class="c-desc">
                  {#if editing === key(v)}
                    <input
                      class="desc-input"
                      use:focus
                      bind:value={draft}
                      placeholder="Describe {v.name}"
                      aria-label="Description of {v.name}"
                      onclick={(e) => e.stopPropagation()}
                      onkeydown={(e) => {
                        e.stopPropagation();
                        if (e.key === "Enter") finish(v, true);
                        else if (e.key === "Escape") finish(v, false);
                      }}
                      onblur={() => finish(v, true)}
                    />
                  {:else if editable && v.kind === "calc"}
                    <button class="desc" class:add={!v.description} title="Edit the description"
                      onclick={(e) => { e.stopPropagation(); startEdit(v); }}>{v.description || "Add description"}</button>
                  {:else}
                    <span class="desc">{v.description}</span>
                  {/if}
                </td>
                <td class="c-section">{v.section}</td>
                <td class="c-used">{v.usedBy.length ? v.usedBy.join(", ") : "—"}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  </div>
  {#if current}
    <VariableInspector v={current} onjump={() => onjump(current.id)} onclose={() => (picked = null)} />
  {/if}
</div>

<style>
  .wrap { display: grid; grid-template-columns: 1fr; height: 100%; min-height: 0; }
  .wrap.split { grid-template-columns: minmax(0, 1fr) minmax(320px, 42%); }
  .left { display: grid; grid-template-rows: auto 1fr; min-height: 0; min-width: 0; }
  .tools { display: flex; gap: 10px; align-items: center; padding: 6px 10px; }
  .tools input { flex: 0 1 360px; min-width: 0; }
  .scroll { overflow: auto; min-height: 0; }
  table { border-collapse: separate; border-spacing: 0; width: 100%; font-size: 13px; }
  th {
    position: sticky; top: 0; z-index: 1; background: var(--panel); text-align: left; font-weight: 600;
    font-size: 11.5px; color: var(--muted); border-bottom: 1px solid var(--border); padding: 0;
    white-space: nowrap;
  }
  th button, th:not(:has(button)) { padding: 4px 8px; }
  th button { border: none; background: none; font: inherit; color: inherit; width: 100%; text-align: left; }
  th button:hover { color: var(--text); }
  td {
    padding: 3px 8px; border-bottom: 1px solid color-mix(in srgb, var(--border) 50%, transparent);
    white-space: nowrap; overflow: hidden; text-overflow: ellipsis; max-width: 0;
  }
  table:focus { outline: none; }
  table:focus-visible tr.sel td { outline: 1px solid var(--select-border); outline-offset: -1px; }
  tr { cursor: default; }
  tr:hover td { background: var(--sunken); }
  tr.sel td { background: var(--select); }
  tr.error td.c-name { box-shadow: inset 2px 0 0 var(--danger); }
  .c-name { width: 12%; min-width: 90px; }
  .c-value { width: 18%; }
  .c-unit { width: 8%; color: var(--muted); }
  .c-type { width: 12%; color: var(--muted); }
  .c-size { width: 7%; color: var(--muted); font-variant-numeric: tabular-nums; }
  .c-desc { width: 23%; }
  .c-section { width: 11%; color: var(--muted); }
  .c-used { width: 9%; color: var(--muted); }
  .split .c-section, .split .c-used, .split .c-desc { display: none; }
  .name { font: 600 13.5px var(--doc); font-style: italic; }
  .name sub { font-style: normal; font-weight: 500; }
  .plain { font: 12px var(--mono); color: var(--muted); }
  .err .plain { color: var(--danger); font-family: var(--sans); }
  .kind { background: var(--sunken); border-radius: 4px; padding: 0 5px; text-transform: capitalize; font-size: 11.5px; color: var(--text); }
  .desc {
    border: none; background: none; padding: 0; text-align: left; font: italic 13px var(--doc);
    color: var(--muted); cursor: text; max-width: 100%; overflow: hidden; text-overflow: ellipsis;
  }
  .desc.add { opacity: 0; font-style: normal; font-family: var(--sans); font-size: 12px; }
  tr:hover .desc.add, .desc.add:focus { opacity: 1; }
  .desc-input { font: italic 13px var(--doc); padding: 1px 6px; width: 100%; }
  .lint {
    display: inline-block; width: 14px; height: 14px; border-radius: 50%; font: 700 10px/14px var(--sans);
    text-align: center; color: var(--warn); background: var(--warn-bg); cursor: help; margin-left: 4px;
  }
  .empty { padding: 8px 12px; }
  code { font: 12px var(--mono); }
</style>
