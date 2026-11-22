// SPDX-License-Identifier: GPL-3.0-or-later
// Copy and paste of cells (ADR-0024 §1). The HTML on the clipboard carries calc, code and raw
// cells as `data-cruncha-*` attributes (schema.ts), so they paste back as cells, in this
// document, another tab or another copy of Cruncha. Cell IDs are never on the clipboard: a
// pasted cell gets a new ID (plugins.ts `cellIdentity`), and the document version it came
// from (`from`), so it is committed as Imported.
//
// Copying part of a block pastes into the block at the cursor (ProseMirror's open slices). Two
// rules are added here. Part of a code cell is copied as inline code: ProseMirror would put only
// bare text on the clipboard, losing that it was code, so the conversion happens at copy time.
// And anything inline pasted into a code cell is plain text, with line breaks as newlines.
import { Fragment, Slice, type Node } from "prosemirror-model";
import type { EditorView } from "prosemirror-view";
import { proseText } from "./cells";
import { PROSE_TYPES, schema } from "./schema";

const CELL_NODES = new Set(["calc", "code_block", "raw"]);

/** On copy: stamp whole cells with the document version they come from. */
export function stampOrigin(slice: Slice, origin: string | null): Slice {
  const only = slice.content.childCount === 1 ? slice.content.firstChild! : null;
  if (only?.type === schema.nodes.code_block && slice.openStart > 0) {
    return new Slice(Fragment.from(codeParagraph(only.textContent)), 1, 1);
  }
  if (!origin) return slice;
  const stamp = (n: Node) =>
    CELL_NODES.has(n.type.name) && !n.attrs.from ? n.type.create({ ...n.attrs, from: origin }, n.content, n.marks) : n;
  const nodes: Node[] = [];
  slice.content.forEach((n, _, i) => {
    // An open first or last node is part of a block, not a whole cell.
    const open = (i === 0 && slice.openStart > 0) || (i === slice.content.childCount - 1 && slice.openEnd > 0);
    nodes.push(open ? n : stamp(n));
  });
  return new Slice(Fragment.from(nodes), slice.openStart, slice.openEnd);
}

/** On paste or drop. A move within this document isn't an import, so its cells lose `from`;
 *  part of a code cell pasted outside code becomes inline code. */
export function adaptPasted(slice: Slice, view: EditorView): Slice {
  let out = slice;
  if (view.dragging?.move) {
    const clear = (n: Node) => (CELL_NODES.has(n.type.name) && n.attrs.from ? n.type.create({ ...n.attrs, from: null }, n.content, n.marks) : n);
    const nodes: Node[] = [];
    out.content.forEach((n) => nodes.push(clear(n)));
    out = new Slice(Fragment.from(nodes), out.openStart, out.openEnd);
  }
  const target = view.state.selection.$from.parent;
  if (target.type.spec.code && !hasCells(out)) {
    const text = textOf(out);
    return new Slice(text ? Fragment.from(schema.text(text)) : Fragment.empty, 0, 0);
  }
  return out;
}

function hasCells(slice: Slice): boolean {
  let found = false;
  slice.content.forEach((n) => (found ||= CELL_NODES.has(n.type.name)));
  return found;
}

/** A slice's text, a hard break or a new block as a newline. */
function textOf(slice: Slice): string {
  const blocks: string[] = [];
  slice.content.forEach((n) => {
    if (n.isText) {
      blocks.push(n.text ?? "");
      return;
    }
    let t = "";
    n.descendants((d) => {
      if (d.isText) t += d.text;
      else if (d.type === schema.nodes.hard_break) t += "\n";
      else if (d.isBlock && t && !t.endsWith("\n")) t += "\n";
      return true;
    });
    blocks.push(t);
  });
  return blocks.join("\n");
}

/** Code as a paragraph of inline code, one line per hard break. */
function codeParagraph(text: string): Node {
  const parts: Node[] = [];
  text.split("\n").forEach((line, i) => {
    if (i > 0) parts.push(schema.nodes.hard_break.create());
    if (line) parts.push(schema.text(line, [schema.marks.code.create()]));
  });
  return schema.nodes.paragraph.create(null, parts);
}

/** The plain-text copy (other apps, the calc editor): one calc or code cell is its source;
 *  anything else is Markdown, cells fenced as the agents' mirror writes them. */
export function plainText(slice: Slice): string {
  const nodes: Node[] = [];
  slice.content.forEach((n) => nodes.push(n));
  if (nodes.length === 1 && (nodes[0].type.name === "calc" || nodes[0].type.name === "code_block")) {
    return nodes[0].type.name === "calc" ? nodes[0].attrs.source : nodes[0].textContent;
  }
  return nodes
    .map((n) => {
      switch (n.type.name) {
        case "calc":
          return "```calc\n" + n.attrs.source + "\n```";
        case "code_block":
          return "```" + n.attrs.lang + "\n" + n.textContent + "\n```";
        case "raw":
          return n.attrs.source;
        default:
          return PROSE_TYPES.has(n.type.name) ? proseText(JSON.stringify(n.toJSON())) : n.textContent;
      }
    })
    .join("\n\n");
}
