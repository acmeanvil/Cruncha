// SPDX-License-Identifier: GPL-3.0-or-later
// Conversion between store cells and editor blocks.
//
// A prose cell's `source` is the ProseMirror JSON of its block without the cell attributes
// (`{"type":"paragraph","content":[…]}`), as the store and sample documents already use.
import { Node } from "prosemirror-model";
import type { Cell } from "../api";
import { schema } from "./schema";

const CODE_KINDS = new Set(["python", "rust", "tex"]);

function textParagraph(text: string) {
  return { type: "paragraph", content: text ? [{ type: "text", text }] : [] };
}

/** The editor block for a cell. Never throws: unreadable prose becomes plain text. */
export function cellToNode(c: Cell): Node {
  const cell = { id: c.id, key: c.order_key };
  const attrsJson = JSON.stringify(c.attrs ?? {});
  switch (c.kind) {
    case "prose": {
      let json: { type?: string; attrs?: Record<string, unknown> };
      try {
        json = JSON.parse(c.source);
      } catch {
        json = textParagraph(c.source);
      }
      if (json?.type !== "paragraph" && json?.type !== "heading") json = textParagraph(c.source);
      try {
        return Node.fromJSON(schema, { ...json, attrs: { ...(json.attrs ?? {}), ...cell } });
      } catch {
        return Node.fromJSON(schema, { ...textParagraph(c.source), attrs: cell });
      }
    }
    case "calc":
      return schema.nodes.calc.create({ ...cell, source: c.source, cellAttrs: attrsJson });
    default:
      if (CODE_KINDS.has(c.kind)) {
        return schema.nodes.code_block.create(
          { ...cell, lang: c.kind, cellAttrs: attrsJson },
          c.source ? schema.text(c.source) : null,
        );
      }
      return schema.nodes.raw.create({ ...cell, kind: c.kind, source: c.source, cellAttrs: attrsJson });
  }
}

/** The cell content of a block: everything the store keeps, except bookkeeping. */
export interface CellContent {
  id: string;
  kind: string;
  order_key: string;
  source: string;
  attrs: unknown;
}

function parseAttrs(json: string): unknown {
  try {
    return JSON.parse(json);
  } catch {
    return {};
  }
}

export function nodeToContent(n: Node): CellContent {
  const base = { id: n.attrs.id as string, order_key: n.attrs.key as string };
  switch (n.type.name) {
    case "paragraph":
    case "heading": {
      const json = n.toJSON() as { attrs?: Record<string, unknown> };
      const { id: _i, key: _k, ...rest } = json.attrs ?? {};
      if (Object.keys(rest).length) json.attrs = rest;
      else delete json.attrs;
      return { ...base, kind: "prose", source: JSON.stringify(json), attrs: {} };
    }
    case "calc":
      return { ...base, kind: "calc", source: n.attrs.source, attrs: parseAttrs(n.attrs.cellAttrs) };
    case "code_block":
      return { ...base, kind: n.attrs.lang, source: n.textContent, attrs: parseAttrs(n.attrs.cellAttrs) };
    default:
      return { ...base, kind: n.attrs.kind, source: n.attrs.source, attrs: parseAttrs(n.attrs.cellAttrs) };
  }
}

/** A stable string for comparing a block with the committed cell. */
export function signature(c: CellContent): string {
  return JSON.stringify([c.kind, c.order_key, c.source, c.attrs]);
}

/** Store cells (document order) as an editor document; an empty document gets one paragraph. */
/** Is `c` a block in the editor? The `meta` cell (document settings: report title block,
 *  options) is edited in dialogs, not shown in the document. */
export const isBlock = (c: Cell) => c.parent_id == null && c.kind !== "meta";

export function docFromCells(cells: Cell[]): Node {
  const blocks = cells.filter(isBlock).map(cellToNode);
  if (blocks.length === 0) blocks.push(schema.nodes.paragraph.create());
  return schema.nodes.doc.create(null, blocks);
}

/** A calc's description (`attrs.description`): shown beside it and in the variable browser. */
export function descriptionOf(cellAttrs: string): string {
  const a = parseAttrs(cellAttrs) as { description?: unknown } | null;
  return typeof a?.description === "string" ? a.description : "";
}

/** `cellAttrs` JSON with the description set (an empty one is removed). */
export function withDescription(cellAttrs: string, description: string): string {
  const a = parseAttrs(cellAttrs);
  const obj: Record<string, unknown> = a && typeof a === "object" && !Array.isArray(a) ? { ...(a as object) } : {};
  const d = description.trim();
  if (d) obj.description = d;
  else delete obj.description;
  return JSON.stringify(obj);
}
