// SPDX-License-Identifier: GPL-3.0-or-later
// The document outline: headings, with the names calcs define under each.
import type { Node } from "prosemirror-model";

export interface OutlineItem {
  /** Position of the heading block in the editor. */
  pos: number;
  level: number;
  text: string;
  /** Names defined by calc cells under this heading (until the next heading). */
  defines: string[];
}

/** Outline of `doc`. `definesOf` gives the names a calc cell defines, by cell id. */
export function outlineOf(doc: Node, definesOf: (id: string) => string[] | undefined): OutlineItem[] {
  const out: OutlineItem[] = [];
  doc.forEach((node, pos) => {
    if (node.type.name === "heading") {
      out.push({ pos, level: node.attrs.level, text: node.textContent, defines: [] });
    } else if (node.type.name === "calc" && out.length) {
      const names = definesOf(node.attrs.id) ?? [];
      out[out.length - 1].defines.push(...names);
    }
  });
  return out;
}
