<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<!-- The document editor: one ProseMirror block per cell (lib/editor/). Editable for the latest
     version; read-only for history. Saves through DocSync (commit on submit or idle). -->
<script lang="ts">
  import { onDestroy, onMount, untrack } from "svelte";
  import { EditorState, NodeSelection, Plugin, TextSelection } from "prosemirror-state";
  import { Fragment, Slice, type Node as PMNode } from "prosemirror-model";
  import { Decoration, DecorationSet, EditorView } from "prosemirror-view";
  import type { Cell, CellResult, CommitView } from "./api";
  import { CalcView } from "./editor/calcView";
  import { CodeView } from "./editor/codeView";
  import { api } from "./api";
  import { cellToNode, docFromCells, withDescription } from "./editor/cells";
  import { markdownPaste } from "./editor/paste";
  import { adaptPasted, plainText, stampOrigin } from "./editor/clipboard";
  import {
    currentBlock,
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
    onerror,
    onproposal,
    onfunctions,
    onunits,
    origin = null,
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
    onerror?: (message: string) => void;
    /** A proposal marker was clicked. */
    onproposal?: () => void;
    /** The toolbar's function reference button. */
    onfunctions?: () => void;
    /** The toolbar's units button. */
    onunits?: () => void;
    /** The shown document version (`document:<id>@v<n>`): copied cells carry it (ADR-0024). */
    origin?: string | null;
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

  /** Results computed in the background (Python cells finished), for the latest version. */
  export function applyResults(r: Record<string, CellResult>) {
    store.set(r, sync.sources());
    refreshOutline();
  }

  /** Save, then run Python cells (`ids`, or all). */
  export async function runPython(ids?: string[]) {
    await sync.flush();
    try {
      applyResults(await api.runPython(ids));
    } catch (e) {
      onerror?.(String(e));
    }
  }

  const codeActions = {
    run: (id: string) => void runPython([id]),
    stop: () => void api.stopPython().catch((e) => onerror?.(String(e))),
  };

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

  /**
   * Type `text` into the calc being edited, at its cursor (then the cursor goes `back`
   * characters before the end of `text`, e.g. inside `sqrt()`). With no calc open, add a calc
   * below the current block with `text` and open it.
   */
  export function insertInCalc(text: string, back = 0) {
    if (!view || !editable) return;
    const active = document.activeElement;
    if (active instanceof HTMLTextAreaElement && active.classList.contains("calc-input") && host.contains(active)) {
      const { selectionStart: a, selectionEnd: b } = active;
      active.setRangeText(text, a, b, "end");
      const at = a + text.length - back;
      active.setSelectionRange(at, at);
      active.dispatchEvent(new Event("input", { bubbles: true }));
      active.focus();
      return;
    }
    const sel = view.state.selection;
    const at = sel instanceof NodeSelection ? sel.to : sel.$from.depth >= 1 ? sel.$from.after(1) : sel.$from.pos;
    const fresh = schema.nodes.calc.create({ editing: true, source: text });
    const tr = view.state.tr.insert(at, fresh);
    tr.setSelection(NodeSelection.create(tr.doc, at));
    view.dispatch(tr.scrollIntoView());
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
            "data-placeholder": "Start writing. Type = and a space for a calculation; Markdown works: # heading, - list, **bold**.",
          }),
        ]);
      },
    },
  });

  /** Open proposals in the document (ADR-0016): cells they change (id → label) and cells they
   *  add after a cell (`null`: at the start). Decorations only, never document content. */
  type Marks = { touched: Map<string, string>; inserts: { id: string; after: string | null; label: string }[] };
  let marks: Marks = { touched: new Map(), inserts: [] };

  export function setProposalMarks(m: Marks) {
    marks = m;
    view?.dispatch(view.state.tr.setMeta("proposalMarks", true));
  }

  function insertWidget(label: string) {
    return () => {
      const el = document.createElement("button");
      el.className = "proposed-insert";
      el.contentEditable = "false";
      el.textContent = `＋ ${label}`;
      el.title = "Open the Proposals tab to review it";
      el.onclick = () => onproposal?.();
      return el;
    };
  }

  const proposalMarks = new Plugin({
    props: {
      decorations(state) {
        if (!marks.touched.size && !marks.inserts.length) return null;
        const d = state.doc;
        // An insert after another proposed cell shows with that one's anchor.
        const byId = new Map(marks.inserts.map((i) => [i.id, i]));
        const anchor = (after: string | null): string | null => {
          const seen = new Set<string>();
          while (after && byId.has(after) && !seen.has(after)) {
            seen.add(after);
            after = byId.get(after)!.after;
          }
          return after;
        };
        const decos: Decoration[] = [];
        const placed = new Set<string>();
        d.forEach((n, pos) => {
          const label = marks.touched.get(n.attrs.id);
          if (label) decos.push(Decoration.node(pos, pos + n.nodeSize, { class: "proposed", "data-proposal": label }));
          marks.inserts.forEach((ins, i) => {
            if (anchor(ins.after) === n.attrs.id) {
              placed.add(ins.id);
              decos.push(Decoration.widget(pos + n.nodeSize, insertWidget(ins.label), { side: i - 1000, key: ins.id }));
            }
          });
        });
        marks.inserts.forEach((ins, i) => {
          if (placed.has(ins.id)) return;
          // At the start, or (anchor gone) at the end.
          const top = anchor(ins.after) === null;
          decos.push(Decoration.widget(top ? 0 : d.content.size, insertWidget(ins.label), { side: i - 1000, key: ins.id }));
        });
        return DecorationSet.create(d, decos);
      },
    },
  });

  function sourcesOf(cs: Cell[]) {
    return new Map(cs.map((c) => [c.id, c.source]));
  }

  function load() {
    const state = EditorState.create({
      doc: docFromCells(cells),
      plugins: [...editorPlugins(() => void sync.flush(), (id) => void runPython([id])), placeholder, proposalMarks],
    });
    sync.load(cells);
    store.set(results, sourcesOf(cells));
    if (!view) {
      view = new EditorView(host, {
        state,
        editable: () => editable,
        nodeViews: {
          calc: (node, v, getPos) => new CalcView(node, v, getPos, store, () => void sync.flush()),
          code_block: (node, v) => new CodeView(node, v, store, codeActions),
        },
        dispatchTransaction(tr) {
          const v = view!;
          v.updateState(v.state.apply(tr));
          if (tr.docChanged) {
            if (editable) sync.changed();
            refreshOutline(true);
          }
        },
        // Pasted Markdown (plain text, or text from a code editor, that looks like Markdown)
        // becomes cells: prose blocks, and calc/python/rust fences as cells. The backend
        // parses it (cruncha-prose), as it does for agents and templates.
        handlePaste(v, e) {
          const md = markdownPaste(v, e);
          if (md == null) return false;
          api.markdownCells(md).then(
            (pieces) => {
              const nodes = pieces.map((p) => cellToNode({ id: null, order_key: null, kind: p.kind, source: p.source, attrs: {} } as unknown as Cell));
              if (nodes.length === 0) return;
              const inline = nodes.length === 1 && nodes[0].type === schema.nodes.paragraph;
              const slice = new Slice(Fragment.from(nodes), inline ? 1 : 0, inline ? 1 : 0);
              v.dispatch(v.state.tr.replaceSelection(slice).scrollIntoView());
            },
            () => v.dispatch(v.state.tr.insertText(md).scrollIntoView()),
          );
          return true;
        },
        // Copy and paste of cells (ADR-0024; clipboard.ts).
        transformCopied: (slice) => stampOrigin(slice, origin),
        transformPasted: (slice, v) => adaptPasted(slice, v),
        clipboardTextSerializer: (slice) => plainText(slice),
        handleDOMEvents: {
          // Links never navigate the app's page; ⌘-click opens one in the browser.
          click(v, e) {
            const a = (e.target as HTMLElement | null)?.closest?.("a[href]");
            if (!a || !v.dom.contains(a)) return false;
            e.preventDefault();
            if (e.metaKey || e.ctrlKey) void api.openUrl(a.getAttribute("href") ?? "");
            return false;
          },
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
      case "bullet_list": case "ordered_list": return "List";
      case "blockquote": return "Quote";
      case "horizontal_rule": return "Rule";
      case "calc": return "Calc";
      case "code_block": return n.attrs.lang === "python" ? "Python" : n.attrs.lang === "rust" ? "Rust" : "TeX";
      case "raw": return n.attrs.kind;
      default: return "Text";
    }
  }

  /** The top-level block at height `y` (viewport px), including the margins around it. */
  function blockAtY(y: number): { pos: number; node: PMNode; el: HTMLElement } | null {
    if (!view) return null;
    let found: { pos: number; node: PMNode; el: HTMLElement } | null = null;
    let nearest = Infinity;
    view.state.doc.forEach((node, pos) => {
      const el = view!.nodeDOM(pos) as HTMLElement | null;
      if (!el?.getBoundingClientRect) return;
      const r = el.getBoundingClientRect();
      // Distance from the block (0 inside it): the margin between two blocks goes to the nearer.
      const d = y < r.top ? r.top - y : y > r.bottom ? y - r.bottom : 0;
      if (d < nearest) {
        nearest = d;
        found = { pos, node, el };
      }
    });
    return nearest <= 16 ? found : null;
  }

  // The handle is chosen by the pointer's height anywhere over the page, including the left
  // margin where it sits, so moving to it never loses it; leaving hides it after a pause.
  let hideTimer: ReturnType<typeof setTimeout> | undefined;
  let frame = 0;
  function hover(e: MouseEvent) {
    clearTimeout(hideTimer);
    if (!view || menu || (e.target as HTMLElement).closest?.(".handle")) return;
    const y = e.clientY;
    cancelAnimationFrame(frame);
    frame = requestAnimationFrame(() => {
      const f = blockAtY(y);
      if (!f) return;
      const top = f.el.getBoundingClientRect().top - wrap.getBoundingClientRect().top;
      if (hovered?.pos !== f.pos || hovered.top !== top) hovered = { pos: f.pos, top, label: labelOf(f.node) };
    });
  }

  function unhover() {
    clearTimeout(hideTimer);
    hideTimer = setTimeout(() => {
      if (!menu) hovered = null;
    }, 400);
  }

  /** Before inserting from the toolbar: if the cursor's block is scrolled out of sight, move
   *  the cursor to the last block in view, so new blocks appear where the user is looking. */
  function cursorIntoView() {
    if (!view) return;
    const scroller = host.closest("main") ?? document.documentElement;
    const box = scroller.getBoundingClientRect();
    const bar = parseFloat(getComputedStyle(scroller).getPropertyValue("--bar-h")) || 0;
    const tools = toolsEl?.getBoundingClientRect().height ?? 0;
    const top = box.top + bar + tools;
    const curEl = view.nodeDOM(currentBlock(view.state).pos) as HTMLElement | null;
    const r = curEl?.getBoundingClientRect?.();
    if (r && r.bottom > top && r.top < box.bottom) return; // visible
    let last: { pos: number; node: PMNode } | null = null;
    view.state.doc.forEach((node, pos) => {
      const er = (view!.nodeDOM(pos) as HTMLElement | null)?.getBoundingClientRect?.();
      if (er && er.top >= top && er.bottom <= box.bottom) last = { pos, node };
    });
    if (!last) return;
    const l = last as { pos: number; node: PMNode };
    const s = l.node.isTextblock
      ? TextSelection.create(view.state.doc, l.pos + l.node.nodeSize - 1)
      : NodeSelection.create(view.state.doc, l.pos);
    view.dispatch(view.state.tr.setSelection(s));
  }

  function insert(node: PMNode) {
    cursorIntoView();
    run(insertBlock(node));
  }

  let toolsEl: HTMLDivElement | undefined = $state();

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
  <div class="tools" role="toolbar" aria-label="Insert" bind:this={toolsEl}>
    <span class="small muted">Insert</span>
    <button onclick={() => insert(newParagraph())} title="Text paragraph">Text</button>
    <button onclick={() => insert(newHeading(2))} title="Heading (type ## then space)">Heading</button>
    <button onclick={() => insert(newCalc())} title="Calculation (⌘E, or type = then space)">Calc</button>
    <button onclick={() => insert(newCode("python"))} title="Python cell (⌘⌥P); not run yet">Python</button>
    {#if onfunctions}
      <span class="sep"></span>
      <button onclick={onfunctions} title="Functions, operators and constants (⌘⇧F)"><i>ƒ</i>x Functions</button>
    {/if}
    {#if onunits}
      <button onclick={onunits} title="Units: lengths, forces, pressures… (⌘⇧U)">Units</button>
    {/if}
    <span class="sep"></span>
    <button onclick={() => run(moveBlock(-1))} title="Move block up (⌥↑)" aria-label="Move block up">↑</button>
    <button onclick={() => run(moveBlock(1))} title="Move block down (⌥↓)" aria-label="Move block down">↓</button>
  </div>
{/if}
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="wrap" bind:this={wrap} onmousemove={hover} onmouseleave={unhover}>
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

<svelte:window
  onkeydown={(e) => e.key === "Escape" && (menu = false)}
  onmousedown={(e) => {
    // A click anywhere outside the block handle closes its menu.
    if (menu && !(e.target as HTMLElement).closest?.(".handle")) {
      menu = false;
      hovered = null;
    }
  }}
/>

<style>
  /* The wrap reaches into the page's left margin, where the handle sits, so the hover area
     is continuous from the text to the handle. */
  .wrap { position: relative; margin-left: calc(-1 * var(--gutter, 100px)); padding-left: var(--gutter, 100px); }
  .handle { position: absolute; left: calc(var(--gutter, 100px) - 92px); width: 84px; display: flex; justify-content: flex-end; z-index: 5; }
  .grip {
    font-size: 11px; padding: 1px 6px; color: var(--muted); background: none; border: 1px solid transparent;
    margin-top: 4px; white-space: nowrap; max-width: 84px; overflow: hidden; text-overflow: ellipsis;
  }
  .grip:hover, .grip[aria-expanded="true"] { border-color: var(--border); background: var(--panel); color: var(--text); }
  .menu {
    /* Opens rightward, over the page: leftward it would be clipped by the scrolling main area. */
    position: absolute; top: 26px; left: 0; min-width: 190px; display: grid; padding: 4px;
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
  /* Sticky under the version bar (its height is --bar-h, set by App), spanning the page's
     padding so text never shows beside it. */
  .tools {
    display: flex; gap: 6px; align-items: center; flex-wrap: wrap;
    position: sticky; top: var(--bar-h, 0px); z-index: 8;
    margin: calc(-1 * var(--pad-t, 28px)) calc(-1 * var(--pad-r, 64px)) 8px calc(-1 * var(--pad-l, 104px));
    padding: 10px var(--pad-r, 64px) 10px var(--pad-l, 104px);
    background: var(--panel); border-bottom: 1px solid var(--border); border-radius: 6px 6px 0 0;
  }
  .tools button { padding: 3px 10px; font-size: 13px; }
  .sep { width: 1px; height: 18px; background: var(--border); margin: 0 4px; }
  .editor :global(.ProseMirror) {
    outline: none; min-height: 50vh; font: var(--text-size, 15px)/1.6 var(--doc); padding: 4px 0 40vh;
    white-space: pre-wrap; word-wrap: break-word; color: var(--text);
  }
  .editor :global(:is(h1, h2, h3, h4, h5, h6)) { font-family: var(--sans); font-weight: 650; letter-spacing: -0.01em; line-height: 1.25; }
  .editor :global(math) { font-size: 1.08em; }
  .editor :global(.ProseMirror > *) { position: relative; }
  /* Open proposals (ADR-0016): a bar beside changed cells, a button where cells would be added. */
  .editor :global(.proposed) { box-shadow: -10px 0 0 -7px var(--accent); }
  .editor :global(.proposed::after) {
    content: attr(data-proposal); position: absolute; right: 0; top: -0.6em;
    font: 11px var(--sans); color: var(--accent); background: var(--accent-bg);
    border-radius: 8px; padding: 0 6px; pointer-events: none;
  }
  .editor :global(.proposed-insert) {
    display: block; width: 100%; text-align: left; margin: 4px 0; padding: 4px 8px;
    border: 1px dashed var(--accent); border-radius: 6px; background: var(--accent-bg);
    color: var(--accent); font: 12.5px var(--sans); cursor: pointer;
  }
  .editor :global(p) { margin: 0.35em 0; }
  .editor :global(:is(ul, ol)) { margin: 0.35em 0; padding-left: 1.6em; }
  .editor :global(li > p) { margin: 0.1em 0; }
  .editor :global(blockquote) { margin: 0.5em 0; padding: 0 0 0 0.9em; border-left: 3px solid var(--border); color: var(--muted); }
  .editor :global(hr) { border: none; border-top: 1px solid var(--border); margin: 1em 0; }
  .editor :global(hr.ProseMirror-selectednode) { border-top-color: var(--accent); }
  .editor :global(a[href]) { color: var(--accent); text-decoration: underline; text-underline-offset: 2px; cursor: text; }
  /* The standard HTML heading scale, relative to the text size. */
  .editor :global(h1) { font-size: 2em; margin: 0.5em 0 0.3em; }
  .editor :global(h2) { font-size: 1.5em; margin: 0.8em 0 0.3em; }
  .editor :global(h3) { font-size: 1.17em; margin: 0.9em 0 0.25em; }
  .editor :global(h4) { font-size: 1em; margin: 0.9em 0 0.2em; }
  .editor :global(h5) { font-size: 0.83em; margin: 0.9em 0 0.2em; }
  .editor :global(h6) { font-size: 0.67em; margin: 0.9em 0 0.2em; text-transform: uppercase; letter-spacing: 0.04em; }
  .editor :global(.placeholder::before) {
    content: attr(data-placeholder); color: var(--muted); position: absolute; pointer-events: none;
  }
  .editor :global(.calc-cell) {
    margin: 0.25em -9px; padding: 2px 8px; border-radius: 6px; border: 1px solid transparent;
    font-size: var(--calc-size, 15px);
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
    width: 100%; resize: none; font: 0.93em/1.5 var(--mono); color: var(--text);
    background: var(--sunken); border: 1px solid var(--border); border-radius: 4px; padding: 6px 8px;
  }
  .editor :global(.calc-preview) { min-height: 1.6em; padding: 2px 0; }
  .editor :global(.calc-hint) { font-size: 11px; color: var(--muted); }
  .editor :global(.calc-desc) {
    margin-left: auto; padding-left: 16px; flex: 0 1 auto; max-width: 45%; text-align: right;
    font: italic 0.93em/1.35 var(--doc); color: var(--muted);
  }
  .editor :global(.calc-desc-input) {
    width: 100%; margin: 2px 0 4px; font: italic 0.93em var(--doc); color: var(--text);
    background: var(--sunken); border: 1px solid var(--border); border-radius: 4px; padding: 4px 8px;
  }
  .editor :global(.calc-source) { font: 0.93em var(--mono); color: var(--code); }
  .editor :global(.calc-source.pending) { color: var(--muted); }
  .editor :global(.calc-error) { color: var(--danger); font: 0.83em var(--sans); }
  .editor :global(.code-wrap) {
    margin: 0.5em 0; border: 1px solid var(--border); border-radius: 6px; background: var(--sunken);
    font-size: var(--code-size, 13px);
  }
  .editor :global(.code-wrap[data-state="running"]) { border-color: var(--accent); }
  .editor :global(.code-wrap[data-state="error"]) { border-color: var(--danger); }
  .editor :global(.code-head) {
    display: flex; align-items: center; gap: 8px; padding: 3px 6px 0 12px; font: 11.5px var(--sans);
    color: var(--muted); user-select: none;
  }
  .editor :global(.code-state.running), .editor :global(.code-state.queued) { color: var(--accent); }
  .editor :global(.code-state.error) { color: var(--danger); }
  .editor :global(.code-state.edited), .editor :global(.code-state.untrusted), .editor :global(.code-state.no_python) { color: var(--warn); }
  .editor :global(.code-stored) {
    font-size: 11px; padding: 0 6px; border-radius: 8px; background: var(--sunken); white-space: nowrap;
    overflow: hidden; text-overflow: ellipsis; min-width: 0;
  }
  .editor :global(.code-stored.elsewhere) { color: var(--warn); }
  .editor :global(.code-run) {
    margin-left: auto; font-size: 11.5px; padding: 1px 8px; border-radius: 4px; background: var(--panel);
  }
  .editor :global(pre.code-cell) {
    font: 1em/1.5 var(--mono); margin: 0; padding: 4px 12px 10px; white-space: pre-wrap; background: none;
  }
  .editor :global(.code-out) {
    display: none; padding: 8px 12px 10px; border-top: 1px solid var(--border); background: var(--panel);
    border-radius: 0 0 6px 6px; font: 1em var(--sans); overflow-x: auto;
  }
  .editor :global(.code-wrap.has-output .code-out) { display: grid; gap: 8px; }
  .editor :global(.code-wrap.stale .code-out) { opacity: 0.55; }
  .editor :global(.code-figure) { display: block; background: white; border-radius: 4px; }
  /* The fixed output containers lay out as if their children were the grid's own. */
  .editor :global(.code-out-note), .editor :global(.code-outputs), .editor :global(.code-out-rest) { display: contents; }
  .editor :global(.code-plot) { display: grid; gap: 2px; min-width: 0; }
  .editor :global(.code-plot iframe.figure-frame) { width: 100%; height: 384px; border: 0; background: transparent; display: block; }
  .editor :global(.code-plot-summary) { color: var(--muted); font-size: 0.85em; }
  .editor :global(.code-out pre) { margin: 0; font: 0.96em/1.45 var(--mono); white-space: pre-wrap; }
  .editor :global(.code-stderr) { color: var(--warn); }
  .editor :global(.code-error) { color: var(--danger); }
  .editor :global(.code-tb) { color: var(--danger); opacity: 0.85; margin-top: 4px !important; }
  .editor :global(.code-note) { color: var(--muted); font-style: italic; }
  .editor :global(.code-defs) { display: flex; flex-wrap: wrap; gap: 4px 14px; color: var(--muted); font: 0.92em var(--mono); }
  .editor :global(.code-def b) { color: var(--text); font-weight: 600; }
  .editor :global(.code-table) { overflow-x: auto; }
  .editor :global(.code-table table) { border-collapse: collapse; font: 0.96em var(--mono); }
  .editor :global(.code-table th), .editor :global(.code-table td) {
    border-bottom: 1px solid var(--border); padding: 2px 10px; text-align: right; white-space: nowrap;
  }
  .editor :global(.code-table th) { color: var(--muted); font-weight: 600; }
  .editor :global(.code-table td.null) { color: var(--muted); }
  .editor :global(.code-more) { color: var(--muted); font-size: 0.92em; margin-top: 4px; }
  .editor :global(.calc-waiting) { color: var(--muted); font: italic 0.83em var(--sans); }
  .editor :global(.raw-cell) {
    background: var(--sunken); border-radius: 6px; padding: 6px 12px; margin: 0.4em 0;
  }
  .editor :global(.raw-cell .kind) { font-size: 11px; color: var(--muted); }
  .editor :global(.raw-cell pre) { margin: 0; font: var(--code-size, 13px) var(--mono); white-space: pre-wrap; }
  .editor :global(.ProseMirror-selectednode) { outline: 2px solid var(--select-border); }
  .editor :global(.ProseMirror-gapcursor) { display: none; position: absolute; }
  .editor :global(.ProseMirror-focused .ProseMirror-gapcursor) { display: block; }
  .editor :global(.ProseMirror-gapcursor::after) {
    content: ""; display: block; position: absolute; top: -2px; width: 20px;
    border-top: 1px solid var(--text); animation: gapblink 1.1s steps(2, start) infinite;
  }
  @keyframes gapblink { to { visibility: hidden; } }
</style>
