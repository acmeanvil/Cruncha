// SPDX-License-Identifier: GPL-3.0-or-later
// Conversion between store cells and editor blocks.
//
// A prose cell's `source` is the ProseMirror JSON of its block without the cell attributes
// (`{"type":"paragraph","content":[…]}`), as the store and sample documents already use.
import { Node } from "prosemirror-model";
import type { Cell } from "../api";
import { PROSE_TYPES, schema } from "./schema";

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
      if (!PROSE_TYPES.has(json?.type ?? "")) json = textParagraph(c.source);
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
    case "heading":
    case "bullet_list":
    case "ordered_list":
    case "blockquote":
    case "horizontal_rule":
      return { ...base, kind: "prose", source: JSON.stringify(withoutCellAttrs(n.toJSON())), attrs: {} };
    case "calc":
      return { ...base, kind: "calc", source: n.attrs.source, attrs: parseAttrs(n.attrs.cellAttrs) };
    case "code_block":
      return { ...base, kind: n.attrs.lang, source: n.textContent, attrs: parseAttrs(n.attrs.cellAttrs) };
    default:
      return { ...base, kind: n.attrs.kind, source: n.attrs.source, attrs: parseAttrs(n.attrs.cellAttrs) };
  }
}

interface PMJson {
  type: string;
  attrs?: Record<string, unknown>;
  content?: PMJson[];
  [k: string]: unknown;
}

/** ProseMirror JSON without the cell attributes (`id`, `key`), at every depth: a paragraph
 *  inside a list carries them too, as nulls. */
function withoutCellAttrs(json: PMJson): PMJson {
  if (json.attrs) {
    const { id: _i, key: _k, ...rest } = json.attrs;
    if (Object.keys(rest).length) json.attrs = rest;
    else delete json.attrs;
  }
  if (json.content) json.content = json.content.map(withoutCellAttrs);
  return json;
}

/** A prose cell's source as short text, Markdown-like (`## Title`, `- item`), for lists of
 *  changes such as proposals. Other text is returned as it is. */
export function proseText(source: string): string {
  let json: PMJson;
  try {
    json = JSON.parse(source);
  } catch {
    return source;
  }
  const inline = (n: PMJson): string =>
    (n.content ?? []).map((c) => (c.type === "hard_break" ? "\n" : typeof c.text === "string" ? c.text : inline(c))).join("");
  const block = (n: PMJson): string => {
    switch (n.type) {
      case "heading":
        return `${"#".repeat(Number(n.attrs?.level ?? 1))} ${inline(n)}`;
      case "bullet_list":
      case "ordered_list": {
        const start = Number(n.attrs?.order ?? 1);
        return (n.content ?? [])
          .map((item, i) => {
            const marker = n.type === "bullet_list" ? "- " : `${start + i}. `;
            const body = (item.content ?? []).map(block).join("\n");
            return marker + body.replace(/\n/g, "\n" + " ".repeat(marker.length));
          })
          .join("\n");
      }
      case "blockquote":
        return (n.content ?? []).map(block).join("\n").replace(/^/gm, "> ");
      case "horizontal_rule":
        return "---";
      default:
        return inline(n);
    }
  };
  return block(json);
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
