// SPDX-License-Identifier: GPL-3.0-or-later
// The editor's ProseMirror schema. Every top-level block is one document cell and carries the
// cell's `id` and order `key`; the sync layer (sync.ts) diffs blocks against committed cells.
import { Schema, type NodeSpec, type MarkSpec } from "prosemirror-model";

/** A cell's attributes as clipboard HTML (ADR-0024): what pasting into a Cruncha document
 *  reads back, so calc and code cells survive copy and paste. `from` is the document version
 *  copied from (`document:<id>@v<n>`), set by the clipboard (clipboard.ts), never stored. */
const dataAttrs = (n: { attrs: Record<string, unknown> }, kind: string, extra: Record<string, string> = {}) => {
  const a: Record<string, string> = { "data-cruncha-kind": kind, "data-cruncha-attrs": String(n.attrs.cellAttrs ?? "{}"), ...extra };
  if (n.attrs.from) a["data-cruncha-from"] = String(n.attrs.from);
  return a;
};
const readAttrs = (d: HTMLElement) => ({
  cellAttrs: d.getAttribute("data-cruncha-attrs") ?? "{}",
  from: d.getAttribute("data-cruncha-from"),
});

/** Attributes every top-level block (cell) carries. */
const cellAttrs = {
  id: { default: null as string | null },
  key: { default: null as string | null },
};

const nodes: Record<string, NodeSpec> = {
  doc: { content: "block+" },

  paragraph: {
    group: "block prose",
    content: "inline*",
    attrs: { ...cellAttrs },
    parseDOM: [{ tag: "p" }],
    toDOM: () => ["p", 0],
  },

  heading: {
    group: "block",
    content: "inline*",
    defining: true,
    attrs: { ...cellAttrs, level: { default: 1 } },
    parseDOM: [1, 2, 3, 4, 5, 6].map((level) => ({ tag: `h${level}`, attrs: { level } })),
    toDOM: (n) => [`h${n.attrs.level}`, 0],
  },

  bullet_list: {
    group: "block prose",
    content: "list_item+",
    attrs: { ...cellAttrs },
    parseDOM: [{ tag: "ul" }],
    toDOM: () => ["ul", 0],
  },

  ordered_list: {
    group: "block prose",
    content: "list_item+",
    attrs: { ...cellAttrs, order: { default: 1 } },
    parseDOM: [{ tag: "ol", getAttrs: (d) => ({ order: Number((d as HTMLElement).getAttribute("start") ?? 1) || 1 }) }],
    toDOM: (n) => (n.attrs.order === 1 ? ["ol", 0] : ["ol", { start: n.attrs.order }, 0]),
  },

  list_item: {
    content: "paragraph prose*",
    defining: true,
    parseDOM: [{ tag: "li" }],
    toDOM: () => ["li", 0],
  },

  blockquote: {
    group: "block prose",
    content: "prose+",
    defining: true,
    attrs: { ...cellAttrs },
    parseDOM: [{ tag: "blockquote" }],
    toDOM: () => ["blockquote", 0],
  },

  horizontal_rule: {
    group: "block",
    attrs: { ...cellAttrs },
    parseDOM: [{ tag: "hr" }],
    toDOM: () => ["hr"],
  },

  /** A new page in reports (ADR-0028): drawn as a labelled dashed line. */
  page_break: {
    group: "block",
    atom: true,
    selectable: true,
    attrs: { ...cellAttrs },
    parseDOM: [{ tag: "div.page-break" }],
    toDOM: () => ["div", { class: "page-break", contenteditable: "false" }, ["span", "Page break"]],
  },

  /** A calc statement, edited as text and shown typeset (calcView.ts). */
  calc: {
    group: "block",
    atom: true,
    selectable: true,
    draggable: true,
    attrs: {
      ...cellAttrs,
      source: { default: "" },
      /** The cell's `attrs` JSON, kept verbatim. */
      cellAttrs: { default: "{}" },
      /** Open the source editor as soon as the view exists (a freshly inserted calc). */
      editing: { default: false },
      from: { default: null },
    },
    parseDOM: [
      {
        tag: "div[data-cruncha-kind=calc]",
        priority: 60,
        getAttrs: (d) => ({ source: (d as HTMLElement).getAttribute("data-cruncha-source") ?? "", ...readAttrs(d as HTMLElement) }),
      },
    ],
    toDOM: (n) => ["div", { class: "calc-cell", ...dataAttrs(n, "calc", { "data-cruncha-source": n.attrs.source }) }, n.attrs.source],
  },

  /** Python, Rust or TeX source. */
  code_block: {
    group: "block",
    content: "text*",
    marks: "",
    code: true,
    defining: true,
    attrs: { ...cellAttrs, lang: { default: "python" }, cellAttrs: { default: "{}" }, from: { default: null } },
    parseDOM: [
      {
        tag: "pre[data-cruncha-kind]",
        priority: 60,
        preserveWhitespace: "full",
        contentElement: (d) => (d as HTMLElement).querySelector("code") ?? (d as HTMLElement),
        getAttrs: (d) => ({ lang: (d as HTMLElement).getAttribute("data-lang") ?? "python", ...readAttrs(d as HTMLElement) }),
      },
    ],
    toDOM: (n) => ["pre", { class: "code-cell", "data-lang": n.attrs.lang, ...dataAttrs(n, String(n.attrs.lang)) }, ["code", 0]],
  },

  /** A cell kind the editor can't edit yet (data, plot, drawing, …): kept verbatim. */
  raw: {
    group: "block",
    atom: true,
    selectable: true,
    attrs: { ...cellAttrs, kind: { default: "data" }, source: { default: "" }, cellAttrs: { default: "{}" }, from: { default: null } },
    parseDOM: [
      {
        tag: "div[data-cruncha-raw]",
        priority: 60,
        getAttrs: (d) => ({
          kind: (d as HTMLElement).getAttribute("data-cruncha-raw") ?? "data",
          source: (d as HTMLElement).getAttribute("data-cruncha-source") ?? "",
          ...readAttrs(d as HTMLElement),
        }),
      },
    ],
    toDOM: (n) => [
      "div",
      { class: "raw-cell", ...dataAttrs(n, "raw", { "data-cruncha-raw": n.attrs.kind, "data-cruncha-source": n.attrs.source }) },
      ["span", { class: "kind" }, n.attrs.kind],
      ["pre", n.attrs.source],
    ],
  },

  text: { group: "inline" },
  hard_break: {
    inline: true,
    group: "inline",
    selectable: false,
    parseDOM: [{ tag: "br" }],
    toDOM: () => ["br"],
  },
};

const marks: Record<string, MarkSpec> = {
  strong: { parseDOM: [{ tag: "strong" }, { tag: "b" }], toDOM: () => ["strong", 0] },
  em: { parseDOM: [{ tag: "em" }, { tag: "i" }], toDOM: () => ["em", 0] },
  code: { parseDOM: [{ tag: "code" }], toDOM: () => ["code", 0] },
  sub: { excludes: "sup", parseDOM: [{ tag: "sub" }], toDOM: () => ["sub", 0] },
  sup: { excludes: "sub", parseDOM: [{ tag: "sup" }], toDOM: () => ["sup", 0] },
  /** Only http, https and mailto targets; opened in the system browser (Editor.svelte). */
  link: {
    attrs: { href: {} },
    inclusive: false,
    parseDOM: [
      {
        tag: "a[href]",
        getAttrs: (d) => {
          const href = safeHref((d as HTMLElement).getAttribute("href") ?? "");
          return href ? { href } : false;
        },
      },
    ],
    toDOM: (m) => ["a", { href: m.attrs.href, title: m.attrs.href, rel: "noopener noreferrer" }, 0],
  },
};

/** A link target the editor follows: http, https or mailto (cruncha_prose::safe_href). */
export function safeHref(href: string): string | null {
  const h = href.trim();
  return /^(https?:\/\/|mailto:)/i.test(h) ? h : null;
}

export const schema = new Schema({ nodes, marks });

/** Node types that are a cell's whole content (one top-level block each). */
export const CELL_TYPES = new Set([
  "paragraph", "heading", "bullet_list", "ordered_list", "blockquote", "horizontal_rule", "page_break", "calc", "code_block", "raw",
]);

/** Node types stored as prose cells. */
export const PROSE_TYPES = new Set(["paragraph", "heading", "bullet_list", "ordered_list", "blockquote", "horizontal_rule", "page_break"]);
