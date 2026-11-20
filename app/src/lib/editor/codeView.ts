// SPDX-License-Identifier: GPL-3.0-or-later
// A Python (or Rust, TeX) cell: an editable code block with a header (language, run state,
// Run/Stop) and, below, what it produced: figures, tables, printed text, errors and the
// names it defined. Output is built from structured data with textContent and <img> data
// URLs, never as HTML from the worker.
import type { Node } from "prosemirror-model";
import type { EditorView, NodeView, ViewMutationRecord } from "prosemirror-view";
import type { CellResult, OutputView, PyState } from "../api";
import type { Results } from "./results";

const STATE_LABEL: Record<PyState, string> = {
  no_python: "Python isn't set up",
  past: "not kept for past versions",
  untrusted: "not trusted to run",
  edited: "edited · Run to update",
  blocked: "waiting on an input",
  queued: "queued",
  running: "running…",
  done: "done",
  error: "error",
  interrupted: "stopped",
};

export interface CodeActions {
  run(id: string): void;
  stop(): void;
}

function el<K extends keyof HTMLElementTagNameMap>(tag: K, cls?: string, text?: string): HTMLElementTagNameMap[K] {
  const e = document.createElement(tag);
  if (cls) e.className = cls;
  if (text !== undefined) e.textContent = text;
  return e;
}

function seconds(ms: number) {
  return ms < 1000 ? `${Math.round(ms)} ms` : `${(ms / 1000).toFixed(ms < 10_000 ? 1 : 0)} s`;
}

export function renderOutput(o: OutputView): HTMLElement {
  switch (o.kind) {
    case "image": {
      const img = el("img", "code-figure");
      img.src = o.src;
      img.alt = "Figure";
      // Shown at half its pixel size: figures are rendered at 144 dpi.
      if (o.width) img.style.maxWidth = `min(100%, ${Math.round(o.width / 2)}px)`;
      return img;
    }
    case "table": {
      const wrap = el("div", "code-table");
      const table = el("table");
      const head = el("tr");
      for (const c of o.columns) head.append(el("th", undefined, c));
      table.append(head);
      for (const row of o.rows) {
        const tr = el("tr");
        for (const v of row) tr.append(el("td", v === null ? "null" : undefined, v ?? "null"));
        table.append(tr);
      }
      wrap.append(table);
      if (o.totalRows > o.rows.length) wrap.append(el("div", "code-more", `${o.rows.length} of ${o.totalRows.toLocaleString()} rows`));
      return wrap;
    }
    case "text":
      return el("pre", "code-text", o.text);
  }
}

export class CodeView implements NodeView {
  dom: HTMLElement;
  contentDOM: HTMLElement;
  private head: HTMLElement;
  private out: HTMLElement;
  private unsubscribe: () => void;

  constructor(
    private node: Node,
    private view: EditorView,
    private results: Results,
    private actions: CodeActions,
  ) {
    this.dom = el("div", "code-wrap");
    this.head = el("div", "code-head");
    this.head.contentEditable = "false";
    const pre = el("pre", "code-cell");
    pre.dataset.lang = node.attrs.lang;
    this.contentDOM = el("code");
    pre.append(this.contentDOM);
    this.out = el("div", "code-out");
    this.out.contentEditable = "false";
    this.dom.append(this.head, pre, this.out);
    this.unsubscribe = results.subscribe(() => this.render());
    this.render();
  }

  private result(): CellResult | undefined {
    const id = this.node.attrs.id as string | null;
    return id ? this.results.committed(id) : undefined;
  }

  private render() {
    const lang = this.node.attrs.lang as string;
    const r = this.result();
    const py = r?.python ?? null;
    this.head.replaceChildren();
    this.head.append(el("span", "code-lang", lang === "python" ? "Python" : lang === "rust" ? "Rust" : "TeX"));
    this.dom.dataset.state = py?.state ?? "";
    if (lang !== "python") {
      if (r?.error) this.head.append(el("span", "code-state", r.error));
      this.out.replaceChildren();
      return;
    }
    if (py) {
      const label = STATE_LABEL[py.state] + (py.state === "done" && py.ms != null ? ` · ${seconds(py.ms)}` : "");
      this.head.append(el("span", `code-state ${py.state}`, label));
    }
    const id = this.node.attrs.id as string | null;
    if (this.view.editable && id) {
      const busy = py?.state === "running" || py?.state === "queued";
      const b = el("button", "code-run", busy ? "■ Stop" : "▶ Run");
      b.title = busy ? "Stop the running cell" : "Run this cell (Shift+Enter)";
      b.addEventListener("mousedown", (e) => e.preventDefault());
      b.addEventListener("click", () => (busy ? this.actions.stop() : this.actions.run(id)));
      this.head.append(b);
    }
    this.renderOutput(r);
  }

  private renderOutput(r: CellResult | undefined) {
    const py = r?.python;
    const parts: HTMLElement[] = [];
    if (py?.note) parts.push(el("div", "code-note", py.note));
    for (const o of py?.outputs ?? []) parts.push(renderOutput(o));
    if (py?.stdout) parts.push(el("pre", "code-stdout", py.stdout));
    if (py?.stderr) parts.push(el("pre", "code-stderr", py.stderr));
    if (py?.error) {
      const e = el("div", "code-error");
      e.append(el("strong", undefined, `${py.error.type}: `), document.createTextNode(py.error.message));
      if (py.error.line) e.append(el("span", "code-line", ` (line ${py.error.line})`));
      if (py.error.traceback && py.error.traceback.split("\n").length > 1) e.append(el("pre", "code-tb", py.error.traceback));
      parts.push(e);
    } else if (r?.error && !py?.error && py?.state !== "edited") {
      parts.push(el("div", "code-error", r.error));
    }
    if (r?.values?.length && py?.state === "done") {
      const defs = el("div", "code-defs");
      for (const v of r.values) {
        const d = el("span", "code-def");
        d.append(el("b", undefined, v.name), document.createTextNode(" = " + (v.summary ?? v.kind)));
        if (v.kind === "python") d.title = "A Python object: other Python cells can use it, calcs can't";
        defs.append(d);
      }
      parts.push(defs);
    }
    this.out.replaceChildren(...parts);
    this.dom.classList.toggle("has-output", parts.length > 0);
    this.dom.classList.toggle("stale", py?.state === "edited");
  }

  update(node: Node) {
    if (node.type !== this.node.type || node.attrs.lang !== this.node.attrs.lang) return false;
    this.node = node;
    this.render();
    return true;
  }

  stopEvent(e: Event) {
    const t = e.target as globalThis.Node | null;
    return !!t && (this.head.contains(t) || this.out.contains(t));
  }

  ignoreMutation(m: ViewMutationRecord) {
    if (m.type === "selection") return false;
    return !this.contentDOM.contains(m.target);
  }

  destroy() {
    this.unsubscribe();
  }
}
