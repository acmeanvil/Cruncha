// SPDX-License-Identifier: GPL-3.0-or-later
// The editor's ProseMirror schema. Every top-level block is one document cell and carries the
// cell's `id` and order `key`; the sync layer (sync.ts) diffs blocks against committed cells.
import { Schema, type NodeSpec, type MarkSpec } from "prosemirror-model";

/** Attributes every top-level block (cell) carries. */
const cellAttrs = {
  id: { default: null as string | null },
  key: { default: null as string | null },
};

const nodes: Record<string, NodeSpec> = {
  doc: { content: "block+" },

  paragraph: {
    group: "block",
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
    parseDOM: [1, 2, 3].map((level) => ({ tag: `h${level}`, attrs: { level } })),
    toDOM: (n) => [`h${n.attrs.level}`, 0],
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
    },
    toDOM: (n) => ["div", { class: "calc-cell" }, n.attrs.source],
  },

  /** Python, Rust or TeX source. */
  code_block: {
    group: "block",
    content: "text*",
    marks: "",
    code: true,
    defining: true,
    attrs: { ...cellAttrs, lang: { default: "python" }, cellAttrs: { default: "{}" } },
    toDOM: (n) => ["pre", { class: "code-cell", "data-lang": n.attrs.lang }, ["code", 0]],
  },

  /** A cell kind the editor can't edit yet (data, plot, drawing, …): kept verbatim. */
  raw: {
    group: "block",
    atom: true,
    selectable: true,
    attrs: { ...cellAttrs, kind: { default: "data" }, source: { default: "" }, cellAttrs: { default: "{}" } },
    toDOM: (n) => ["div", { class: "raw-cell" }, ["span", { class: "kind" }, n.attrs.kind], ["pre", n.attrs.source]],
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
};

export const schema = new Schema({ nodes, marks });

/** Node types that are a cell's whole content (one top-level block each). */
export const CELL_TYPES = new Set(["paragraph", "heading", "calc", "code_block", "raw"]);
