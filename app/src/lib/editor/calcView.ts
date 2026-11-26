// SPDX-License-Identifier: GPL-3.0-or-later
// The calc block: typeset math (Temml) when idle; a source editor with a live preview when
// clicked, as in Hurmet. Enter submits (and commits at once), Escape cancels.
import temml from "temml";
import type { Node } from "prosemirror-model";
import { NodeSelection, TextSelection } from "prosemirror-state";
import type { EditorView, NodeView } from "prosemirror-view";
import type { CellResult } from "../api";
import type { Results } from "./results";
import { descriptionOf, withDescription } from "./cells";
import { schema } from "./schema";

/** The theme's echo and error colours, read once and again when the theme changes: reading
 *  computed style after each calc's MathML is replaced forces a style recalculation of the
 *  whole page, which made inserting many calcs freeze the window (2026-11-26). */
let colours: { echo: string; danger: string } | null = null;
if (typeof window !== "undefined" && window.matchMedia) {
  window.matchMedia("(prefers-color-scheme: dark)").addEventListener?.("change", () => (colours = null));
}
export function resetThemeColours() {
  colours = null;
}
function themeColours() {
  if (!colours) {
    const css = getComputedStyle(document.documentElement);
    colours = {
      echo: css.getPropertyValue("--echo").trim() || "#1d4ed8",
      danger: css.getPropertyValue("--danger").trim() || "#b42318",
    };
  }
  return colours;
}

/** Hurmet colours its echo and errors with fixed colours; use the theme's instead. */
function themed(tex: string): string {
  const { echo, danger } = themeColours();
  return tex.replaceAll("\\textcolor{#0000ff}", `\\textcolor{${echo}}`).replaceAll("\\textcolor{firebrick}", `\\textcolor{${danger}}`);
}

function sourceWithError(el: HTMLElement, source: string, message: string) {
  const code = document.createElement("code");
  code.className = "calc-source";
  code.textContent = source;
  const err = document.createElement("div");
  err.className = "calc-error";
  err.textContent = message;
  el.append(code, err);
}

function appendDescription(el: HTMLElement, description: string) {
  if (!description) return;
  const d = document.createElement("span");
  d.className = "calc-desc";
  d.textContent = description;
  el.append(d);
}

/** Show a calc: its typeset line (or source and error), lints, and its description. */
export function renderMath(el: HTMLElement, r: CellResult | undefined, source: string, description = "") {
  el.replaceChildren();
  el.title = r?.text ?? "";
  if (!r || (!r.tex && !r.error)) {
    const code = document.createElement("code");
    code.className = "calc-source" + (r && !r.pending ? "" : " pending");
    code.textContent = source || "empty calculation";
    el.append(code);
    if (r?.pending) {
      const w = document.createElement("span");
      w.className = "calc-waiting";
      w.textContent = "waiting for Python…";
      el.append(w);
    }
    appendDescription(el, description);
    return;
  }
  if (!r.tex) {
    // Blocked before evaluation (malformed, a duplicate name, a cycle, …).
    sourceWithError(el, source, r.error!);
  } else {
    try {
      // Temml emits MathML markup; it never passes raw HTML through (trust is off).
      const math = document.createElement("div");
      math.className = "calc-math";
      math.innerHTML = temml.renderToString(themed(r.tex), { displayMode: true, throwOnError: true });
      el.append(math);
      if (r.error) el.classList.add("has-error");
      else el.classList.remove("has-error");
    } catch (e) {
      sourceWithError(el, source, `cannot display: ${(e as Error).message}`);
    }
  }
  if (r.warnings?.length) {
    const w = document.createElement("span");
    w.className = "calc-lint";
    w.title = r.warnings.join("\n");
    w.textContent = "!";
    w.setAttribute("aria-label", r.warnings.join("; "));
    el.append(w);
  }
  appendDescription(el, description);
}

export class CalcView implements NodeView {
  dom: HTMLElement;
  private display: HTMLElement;
  private panel: HTMLElement | null = null;
  private input: HTMLTextAreaElement | null = null;
  private descInput: HTMLInputElement | null = null;
  private previewEl: HTMLElement | null = null;
  private unsubscribe: () => void;
  private timer: number | undefined;
  /** What the display last showed: a results update that changes none of it doesn't redraw. */
  private shown: unknown[] | null = null;

  constructor(
    private node: Node,
    private view: EditorView,
    private getPos: () => number | undefined,
    private results: Results,
    private onSubmit: () => void,
  ) {
    this.dom = document.createElement("div");
    this.dom.className = "calc-cell";
    (this.dom as HTMLElement & { calcView?: CalcView }).calcView = this;
    this.display = document.createElement("div");
    this.display.className = "calc-display";
    this.display.title = "Click to edit (Enter)";
    this.display.addEventListener("mousedown", (e) => {
      if (!this.view.editable) return;
      e.preventDefault();
      this.open((e.target as HTMLElement).closest?.(".calc-desc") ? "description" : "source");
    });
    this.dom.append(this.display);
    this.unsubscribe = results.subscribe(() => this.render());
    this.render();
    if (node.attrs.editing && view.editable) queueMicrotask(() => this.open());
  }

  private render() {
    if (this.panel) return;
    const { id, source, cellAttrs } = this.node.attrs;
    const r = this.results.get(id, source);
    const description = descriptionOf(cellAttrs);
    const key = [source, description, r?.tex, r?.error, r?.text, r?.pending, r?.warnings?.join("\n")];
    if (this.shown && key.every((k, i) => k === this.shown![i])) return;
    this.shown = key;
    renderMath(this.display, r, source, description);
  }

  update(node: Node) {
    if (node.type !== this.node.type) return false;
    this.node = node;
    this.render();
    return true;
  }

  selectNode() {
    this.dom.classList.add("selected");
  }

  deselectNode() {
    this.dom.classList.remove("selected");
  }

  /** Open the editor, with the cursor in the source or the description. */
  open(focus: "source" | "description" = "source") {
    if (this.panel) {
      (focus === "description" ? this.descInput : this.input)?.focus();
      return;
    }
    const pos = this.getPos();
    if (pos !== undefined && this.node.attrs.editing) {
      this.view.dispatch(
        this.view.state.tr.setNodeMarkup(pos, undefined, { ...this.node.attrs, editing: false }).setMeta("addToHistory", false),
      );
    }
    this.dom.classList.add("editing");
    this.panel = document.createElement("div");
    this.panel.className = "calc-editor";
    const input = document.createElement("textarea");
    input.className = "calc-input";
    input.spellcheck = false;
    input.rows = 1;
    input.value = this.node.attrs.source;
    input.placeholder = "M = w L^2 / 8 = ?? kip·ft";
    this.previewEl = document.createElement("div");
    this.previewEl.className = "calc-preview";
    const desc = document.createElement("input");
    desc.className = "calc-desc-input";
    desc.type = "text";
    desc.value = descriptionOf(this.node.attrs.cellAttrs);
    desc.placeholder = "Description (optional): shown beside the result and in Variables";
    desc.setAttribute("aria-label", "Description");
    const hint = document.createElement("div");
    hint.className = "calc-hint";
    hint.textContent = "End with = ? to show the result (= ?? unit to choose its unit) · Enter to save · Shift+Enter new line · Tab to the description · Esc cancels";
    this.panel.append(input, this.previewEl, desc, hint);
    this.display.replaceWith(this.panel);
    this.input = input;
    this.descInput = desc;
    desc.addEventListener("keydown", (e) => {
      if (e.key === "Enter") {
        e.preventDefault();
        this.close(true, 1);
      } else if (e.key === "Escape") {
        e.preventDefault();
        this.close(false, 0);
      }
    });
    // Leaving the editor saves it; moving between its two fields doesn't.
    const leave = (e: FocusEvent) => {
      if (this.panel && e.relatedTarget instanceof globalThis.Node && this.panel.contains(e.relatedTarget)) return;
      this.close(true, 0, false);
    };
    desc.addEventListener("blur", leave);
    const fit = () => {
      input.style.height = "auto";
      input.style.height = `${input.scrollHeight}px`;
    };
    input.addEventListener("input", () => {
      fit();
      window.clearTimeout(this.timer);
      this.timer = window.setTimeout(() => this.refreshPreview(), 60);
    });
    input.addEventListener("keydown", (e) => {
      if (e.key === "Enter" && !e.shiftKey) {
        e.preventDefault();
        this.close(true, 1);
      } else if (e.key === "Escape") {
        e.preventDefault();
        this.close(false, 0);
      } else if (e.key === "ArrowUp" && input.selectionStart === 0 && input.selectionEnd === 0) {
        e.preventDefault();
        this.close(true, -1);
      } else if (e.key === "ArrowDown" && input.selectionStart === input.value.length) {
        e.preventDefault();
        this.close(true, 1);
      }
    });
    input.addEventListener("blur", leave);
    fit();
    this.refreshPreview();
    if (focus === "description") {
      desc.focus();
    } else {
      input.focus();
      input.setSelectionRange(input.value.length, input.value.length);
    }
  }

  private refreshPreview() {
    const el = this.previewEl;
    const text = this.input?.value ?? "";
    if (!el) return;
    if (!text.trim()) {
      el.replaceChildren();
      return;
    }
    void this.results.preview(text).then((r) => {
      if (this.previewEl === el && this.input?.value === text) renderMath(el, r, text);
    });
  }

  /**
   * Close the editor. `save`: keep the new text (and commit now). `move`: where the cursor goes
   * afterwards (-1 before the block, 1 after it, 0 select the block).
   */
  private close(save: boolean, move: -1 | 0 | 1, refocus = true) {
    if (!this.panel || !this.input) return;
    const text = this.input.value.trim();
    const description = this.descInput?.value ?? "";
    const panel = this.panel;
    // Clear first: removing the focused textarea fires `blur`, which calls close again.
    this.panel = this.input = this.descInput = this.previewEl = null;
    window.clearTimeout(this.timer);
    panel.replaceWith(this.display);
    this.dom.classList.remove("editing");
    const pos = this.getPos();
    if (pos === undefined) return;
    const view = this.view;
    let tr = view.state.tr;
    if (save && text === "") {
      // An emptied calc is removed.
      tr = tr.delete(pos, pos + this.node.nodeSize);
      if (tr.doc.childCount === 0) tr.insert(0, schema.nodes.paragraph.create());
      const at = Math.min(pos, tr.doc.content.size);
      tr.setSelection(TextSelection.near(tr.doc.resolve(at)));
    } else {
      const cellAttrs = save ? withDescription(this.node.attrs.cellAttrs, description) : this.node.attrs.cellAttrs;
      if (save && (text !== this.node.attrs.source || cellAttrs !== this.node.attrs.cellAttrs)) {
        tr = tr.setNodeMarkup(pos, undefined, { ...this.node.attrs, source: text, cellAttrs });
      }
      const after = pos + this.node.nodeSize;
      if (move === 1) {
        const next = tr.doc.nodeAt(after);
        if (!next || !next.isTextblock) {
          tr.insert(after, schema.nodes.paragraph.create());
        }
        tr.setSelection(TextSelection.create(tr.doc, after + 1));
      } else if (move === -1) {
        tr.setSelection(TextSelection.near(tr.doc.resolve(pos), -1));
      } else {
        tr.setSelection(NodeSelection.create(tr.doc, pos));
      }
    }
    view.dispatch(tr.scrollIntoView());
    this.render();
    if (refocus) view.focus();
    if (save) this.onSubmit();
  }

  stopEvent(e: Event) {
    return !!this.panel && this.panel.contains(e.target as globalThis.Node);
  }

  ignoreMutation() {
    return true;
  }

  destroy() {
    window.clearTimeout(this.timer);
    this.unsubscribe();
  }
}
