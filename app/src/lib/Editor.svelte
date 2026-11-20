<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<!-- The document editor: one ProseMirror block per cell (lib/editor/). Editable for the latest
     version; read-only for history. Saves through DocSync (commit on submit or idle). -->
<script lang="ts">
  import { onDestroy, onMount, untrack } from "svelte";
  import { EditorState, NodeSelection, Plugin, TextSelection } from "prosemirror-state";
  import type { Node as PMNode } from "prosemirror-model";
  import { Decoration, DecorationSet, EditorView } from "prosemirror-view";
  import type { Cell, CellResult, CommitView } from "./api";
  import { CalcView } from "./editor/calcView";
  import { docFromCells, withDescription } from "./editor/cells";
  import {
    editorPlugins,
    insertBlock,
    moveBlock,
    newCalc,
    newCode,
    newHeading,
    newParagraph,
  } from "./editor/plugins";
  import { outlineOf, type OutlineItem } from "./editor/outline";
  import { variablesOf, type Variable } from "./editor/variables";
  import { Results } from "./editor/results";
  import { schema } from "./editor/schema";
  import { DocSync, type SaveState } from "./editor/sync";

  let {
    cells,
    results,
    editable,
    loadKey,
    oncommitted,
    onsavestate,
    onoutline,
    onvariables,
  }: {
    cells: Cell[];
    results: Record<string, CellResult>;
    editable: boolean;
    /** Changing it reloads the editor from `cells` (another document or version). */
    loadKey: string;
    oncommitted: (v: CommitView) => void;
    onsavestate: (s: SaveState) => void;
    onoutline?: (items: OutlineItem[]) => void;
    onvariables?: (vars: Variable[]) => void;
  } = $props();

  let host: HTMLDivElement;
  let view: EditorView | undefined;
  const store = new Results();
  const sync = new DocSync(
    () => view!.state.doc,
    (v, sources) => {
      store.set(v.results, sources);
      oncommitted(v);
      refreshOutline();
    },
    (s) => onsavestate(s),
  );

  /** Commit pending edits now. */
  export function flush(): Promise<void> {
    return sync.flush();
  }

  /** Put the cursor at the block at `pos` and scroll it into view. */
  export function jumpTo(pos: number) {
    if (!view) return;
    const node = view.state.doc.nodeAt(pos);
    if (!node) return;
    const sel = node.isTextblock
      ? TextSelection.create(view.state.doc, pos + 1)
      : NodeSelection.create(view.state.doc, pos);
    view.dispatch(view.state.tr.setSelection(sel).scrollIntoView());
    view.focus();
    (view.nodeDOM(pos) as HTMLElement | null)?.scrollIntoView({ block: "start", behavior: "smooth" });
  }

  /** Position of the block for cell `id`, if it is in the document. */
  function posOf(id: string): number | null {
    let found: number | null = null;
    view?.state.doc.forEach((n, pos) => {
      if (found === null && n.attrs.id === id) found = pos;
    });
    return found;
  }

  /** Jump to the block for cell `id`. */
  export function jumpToCell(id: string) {
    const pos = posOf(id);
    if (pos !== null) jumpTo(pos);
  }

  /** Set the description of calc cell `id`, and save it. */
  export function setDescription(id: string, description: string) {
    const pos = posOf(id);
    if (!view || !editable || pos === null) return;
    const node = view.state.doc.nodeAt(pos)!;
    if (node.type !== schema.nodes.calc) return;
    const cellAttrs = withDescription(node.attrs.cellAttrs, description);
    if (cellAttrs === node.attrs.cellAttrs) return;
    view.dispatch(view.state.tr.setNodeMarkup(pos, undefined, { ...node.attrs, cellAttrs }));
    void sync.flush();
  }

  let outlineTimer: ReturnType<typeof setTimeout> | undefined;
  function refreshOutline(soon = false) {
    clearTimeout(outlineTimer);
    const run = () => {
      if (!view) return;
      onoutline?.(outlineOf(view.state.doc, (id) => store.definesOf(id)));
      onvariables?.(variablesOf(view.state.doc, (id) => store.committed(id)));
    };
    if (soon) outlineTimer = setTimeout(run, 250);
    else run();
  }

  const placeholder = new Plugin({
    props: {
      decorations(state) {
        const d = state.doc;
        if (d.childCount !== 1 || d.firstChild!.type !== schema.nodes.paragraph || d.firstChild!.content.size) return null;
        return DecorationSet.create(d, [
          Decoration.node(0, d.firstChild!.nodeSize, {
            class: "placeholder",
            "data-placeholder": "Start writing. Type = and a space for a calculation, # for a heading.",
          }),
        ]);
      },
    },
  });

  function sourcesOf(cs: Cell[]) {
    return new Map(cs.map((c) => [c.id, c.source]));
  }

  function load() {
    const state = EditorState.create({
      doc: docFromCells(cells),
      plugins: [...editorPlugins(() => void sync.flush()), placeholder],
    });
    sync.load(cells);
    store.set(results, sourcesOf(cells));
    if (!view) {
      view = new EditorView(host, {
        state,
        editable: () => editable,
        nodeViews: {
          calc: (node, v, getPos) => new CalcView(node, v, getPos, store, () => void sync.flush()),
        },
        dispatchTransaction(tr) {
          const v = view!;
          v.updateState(v.state.apply(tr));
          if (tr.docChanged) {
            if (editable) sync.changed();
            refreshOutline(true);
          }
        },
        handleKeyDown(v, e) {
          const sel = v.state.selection;
          if (e.key === "Enter" && sel instanceof NodeSelection && sel.node.type === schema.nodes.calc) {
            (v.nodeDOM(sel.from) as (HTMLElement & { calcView?: CalcView }) | null)?.calcView?.open();
            return true;
          }
          return false;
        },
      });
    } else {
      view.updateState(state);
      view.setProps({ editable: () => editable });
    }
    hovered = null;
    menu = false;
    refreshOutline();
  }

  // The block handle: a label and actions menu beside the block under the pointer.
  let wrap: HTMLDivElement;
  let hovered = $state<{ pos: number; top: number; label: string } | null>(null);
  let menu = $state(false);

  function labelOf(n: PMNode) {
    switch (n.type.name) {
      case "heading": return `H${n.attrs.level}`;
      case "calc": return "Calc";
      case "code_block": return n.attrs.lang === "python" ? "Python" : n.attrs.lang === "rust" ? "Rust" : "TeX";
      case "raw": return n.attrs.kind;
      default: return "Text";
    }
  }

  function hover(e: MouseEvent) {
    if (!view || menu) return;
    let el = e.target as HTMLElement | null;
    while (el && el.parentElement !== view.dom) el = el.parentElement;
    if (!el) return;
    let found: { pos: number; node: PMNode } | null = null;
    view.state.doc.forEach((node, pos) => {
      if (!found && view!.nodeDOM(pos) === el) found = { pos, node };
    });
    if (!found) return;
    const f = found as { pos: number; node: PMNode };
    const top = el.getBoundingClientRect().top - wrap.getBoundingClientRect().top;
    if (hovered?.pos !== f.pos || hovered.top !== top) hovered = { pos: f.pos, top, label: labelOf(f.node) };
  }

  function act(kind: "text" | "calc" | "python" | "up" | "down" | "delete") {
    const h = hovered;
    menu = false;
    if (!view || !h) return;
    const node = view.state.doc.nodeAt(h.pos);
    if (!node) return;
    const sel = node.isTextblock
      ? TextSelection.create(view.state.doc, h.pos + 1)
      : NodeSelection.create(view.state.doc, h.pos);
    view.dispatch(view.state.tr.setSelection(sel));
    if (kind === "up" || kind === "down") return run(moveBlock(kind === "up" ? -1 : 1));
    if (kind === "delete") {
      const tr = view.state.tr.delete(h.pos, h.pos + node.nodeSize);
      if (tr.doc.childCount === 0) tr.insert(0, schema.nodes.paragraph.create());
      tr.setSelection(TextSelection.near(tr.doc.resolve(Math.min(h.pos, tr.doc.content.size))));
      view.dispatch(tr);
      hovered = null;
      view.focus();
      return;
    }
    // Insert below this block (never replacing it, even if it's an empty paragraph).
    const fresh = kind === "calc" ? newCalc() : kind === "python" ? newCode("python") : newParagraph();
    const at = h.pos + node.nodeSize;
    const tr = view.state.tr.insert(at, fresh);
    tr.setSelection(fresh.isAtom ? NodeSelection.create(tr.doc, at) : TextSelection.create(tr.doc, at + 1));
    view.dispatch(tr.scrollIntoView());
    view.focus();
  }

  $effect(() => {
    void loadKey;
    untrack(() => (host ? load() : undefined));
  });

  onMount(() => {
    load();
    if (editable) view?.focus();
  });
  onDestroy(() => view?.destroy());

  function run(cmd: (s: EditorState, d?: EditorView["dispatch"]) => boolean) {
    if (!view) return;
    cmd(view.state, view.dispatch);
    view.focus();
  }
</script>

{#if editable}
  <div class="tools" role="toolbar" aria-label="Insert">
    <span class="small muted">Insert</span>
    <button onclick={() => run(insertBlock(newParagraph()))} title="Text paragraph">Text</button>
    <button onclick={() => run(insertBlock(newHeading(2)))} title="Heading (type ## then space)">Heading</button>
    <button onclick={() => run(insertBlock(newCalc()))} title="Calculation (⌘E, or type = then space)">Calc</button>
    <button onclick={() => run(insertBlock(newCode("python")))} title="Python cell (⌘⌥P); not run yet">Python</button>
    <span class="sep"></span>
    <button onclick={() => run(moveBlock(-1))} title="Move block up (⌥↑)" aria-label="Move block up">↑</button>
    <button onclick={() => run(moveBlock(1))} title="Move block down (⌥↓)" aria-label="Move block down">↓</button>
  </div>
{/if}
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="wrap" bind:this={wrap} onmousemove={hover} onmouseleave={() => !menu && (hovered = null)}>
  {#if editable && hovered}
    <div class="handle" style="top: {hovered.top}px">
      <button class="grip" title="Block actions" aria-haspopup="menu" aria-expanded={menu} onclick={() => (menu = !menu)}>
        {hovered.label}
      </button>
      {#if menu}
        <div class="menu" role="menu">
          <button role="menuitem" onclick={() => act("text")}>Add text below</button>
          <button role="menuitem" onclick={() => act("calc")}>Add calc below</button>
          <button role="menuitem" onclick={() => act("python")}>Add Python below</button>
          <hr />
          <button role="menuitem" onclick={() => act("up")}>Move up <kbd>⌥↑</kbd></button>
          <button role="menuitem" onclick={() => act("down")}>Move down <kbd>⌥↓</kbd></button>
          <hr />
          <button role="menuitem" class="danger-text" onclick={() => act("delete")}>Delete</button>
        </div>
      {/if}
    </div>
  {/if}
  <div class="editor" class:readonly={!editable} bind:this={host}></div>
</div>

<svelte:window onkeydown={(e) => e.key === "Escape" && (menu = false)} />

<style>
  .wrap { position: relative; }
  .handle { position: absolute; left: -92px; width: 84px; display: flex; justify-content: flex-end; z-index: 3; }
  .grip {
    font-size: 11px; padding: 1px 6px; color: var(--muted); background: none; border: 1px solid transparent;
    margin-top: 4px; white-space: nowrap; max-width: 84px; overflow: hidden; text-overflow: ellipsis;
  }
  .grip:hover, .grip[aria-expanded="true"] { border-color: var(--border); background: var(--panel); color: var(--text); }
  .menu {
    position: absolute; top: 26px; right: 0; min-width: 190px; display: grid; padding: 4px;
    background: var(--panel); border: 1px solid var(--border); border-radius: 8px;
    box-shadow: 0 8px 24px rgb(0 0 0 / 0.14);
  }
  .menu button {
    border: none; background: none; text-align: left; padding: 5px 10px; border-radius: 5px;
    display: flex; justify-content: space-between; gap: 12px;
  }
  .menu button:hover { background: var(--sunken); }
  .menu hr { border: none; border-top: 1px solid var(--border); margin: 4px 0; width: 100%; }
  .menu kbd { font: 11px var(--mono); color: var(--muted); }
  .danger-text { color: var(--danger); }
  .tools {
    display: flex; gap: 6px; align-items: center; padding: 0 0 12px; margin-bottom: 8px; flex-wrap: wrap;
    border-bottom: 1px solid var(--border);
  }
  .tools button { padding: 3px 10px; font-size: 13px; }
  .sep { width: 1px; height: 18px; background: var(--border); margin: 0 4px; }
  .editor :global(.ProseMirror) {
    outline: none; min-height: 50vh; font: 16.5px/1.6 var(--doc); padding: 4px 0 40vh;
    white-space: pre-wrap; word-wrap: break-word; color: var(--text);
  }
  .editor :global(:is(h1, h2, h3)) { font-family: var(--sans); font-weight: 650; letter-spacing: -0.01em; line-height: 1.25; }
  .editor :global(math) { font-size: 1.08em; }
  .editor :global(.ProseMirror > *) { position: relative; }
  .editor :global(p) { margin: 0.35em 0; }
  .editor :global(h1) { font-size: 1.75em; margin: 0.6em 0 0.35em; }
  .editor :global(h2) { font-size: 1.3em; margin: 1.1em 0 0.3em; }
  .editor :global(h3) { font-size: 1.08em; margin: 0.9em 0 0.25em; }
  .editor :global(.placeholder::before) {
    content: attr(data-placeholder); color: var(--muted); position: absolute; pointer-events: none;
  }
  .editor :global(.calc-cell) {
    margin: 0.25em -9px; padding: 2px 8px; border-radius: 6px; border: 1px solid transparent;
  }
  .editor :global(.calc-cell:hover) { background: var(--sunken); }
  .editor :global(.calc-cell.selected) { border-color: var(--select-border); background: var(--select); }
  .editor :global(.calc-cell.editing) { border-color: var(--accent); background: var(--panel); }
  .editor :global(.calc-display) { cursor: text; min-height: 1.6em; position: relative; display: flex; align-items: center; gap: 8px; }
  .editor :global(.calc-math) { overflow-x: auto; overflow-y: hidden; }
  .editor :global(.calc-display.has-error) { box-shadow: inset 3px 0 0 var(--danger); padding-left: 8px; margin-left: -8px; }
  .editor :global(.calc-lint) {
    flex: none; width: 18px; height: 18px; border-radius: 50%; font: 700 11px/18px var(--sans); text-align: center;
    color: var(--warn); background: var(--warn-bg); cursor: help;
  }
  .editor.readonly :global(.calc-display) { cursor: default; }
  .editor :global(.calc-display math) { margin: 0.2em 0; text-align: left; display: block; }
  .editor :global(.calc-input) {
    width: 100%; resize: none; font: 14px/1.5 var(--mono); color: var(--text);
    background: var(--sunken); border: 1px solid var(--border); border-radius: 4px; padding: 6px 8px;
  }
  .editor :global(.calc-preview) { min-height: 1.6em; padding: 2px 0; }
  .editor :global(.calc-hint) { font-size: 11px; color: var(--muted); }
  .editor :global(.calc-desc) {
    margin-left: auto; padding-left: 16px; flex: 0 1 auto; max-width: 45%; text-align: right;
    font: italic 14px/1.35 var(--doc); color: var(--muted);
  }
  .editor :global(.calc-desc-input) {
    width: 100%; margin: 2px 0 4px; font: italic 14px var(--doc); color: var(--text);
    background: var(--sunken); border: 1px solid var(--border); border-radius: 4px; padding: 4px 8px;
  }
  .editor :global(.calc-source) { font: 14px var(--mono); color: var(--code); }
  .editor :global(.calc-source.pending) { color: var(--muted); }
  .editor :global(.calc-error) { color: var(--danger); font: 12.5px var(--sans); }
  .editor :global(pre.code-cell) {
    font: 13px/1.5 var(--mono); background: var(--sunken); border-radius: 6px;
    padding: 22px 12px 10px; margin: 0.4em 0; white-space: pre-wrap; position: relative;
  }
  .editor :global(pre.code-cell::before) {
    content: attr(data-lang); position: absolute; top: 4px; left: 12px; font: 11px var(--sans);
    color: var(--muted); text-transform: capitalize;
  }
  .editor :global(.raw-cell) {
    background: var(--sunken); border-radius: 6px; padding: 6px 12px; margin: 0.4em 0;
  }
  .editor :global(.raw-cell .kind) { font-size: 11px; color: var(--muted); }
  .editor :global(.raw-cell pre) { margin: 0; font: 13px var(--mono); white-space: pre-wrap; }
  .editor :global(.ProseMirror-selectednode) { outline: 2px solid var(--select-border); }
  .editor :global(.ProseMirror-gapcursor) { display: none; position: absolute; }
  .editor :global(.ProseMirror-focused .ProseMirror-gapcursor) { display: block; }
  .editor :global(.ProseMirror-gapcursor::after) {
    content: ""; display: block; position: absolute; top: -2px; width: 20px;
    border-top: 1px solid var(--text); animation: gapblink 1.1s steps(2, start) infinite;
  }
  @keyframes gapblink { to { visibility: hidden; } }
</style>
