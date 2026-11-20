// SPDX-License-Identifier: GPL-3.0-or-later
// The variable browser's model: every name the document defines, in document order, with its
// value, description, where it is defined and what uses it.
import type { Node } from "prosemirror-model";
import type { CellResult } from "../api";
import { descriptionOf } from "./cells";

export interface Variable {
  name: string;
  /** Defining cell. */
  id: string;
  /** Position of the defining block in the editor. */
  pos: number;
  /** "calc", "python", "rust", … */
  kind: string;
  /** The defining calc's description (shared by every name it defines). */
  description: string;
  /** The value as TeX, if it was evaluated. */
  tex: string | null;
  /** "number", "matrix", … */
  valueKind: string | null;
  /** The defining cell's plain-text result line. */
  text: string;
  error: string | null;
  warnings: string[];
  /** The nearest heading above the definition. */
  section: string;
  /** Names defined by the cells that read this one (or "a check" for cells that define none). */
  usedBy: string[];
}

/** Every variable `doc` defines. `resultOf` gives a cell's last committed result. */
export function variablesOf(doc: Node, resultOf: (id: string) => CellResult | undefined): Variable[] {
  const out: Variable[] = [];
  const readers: { reads: string[]; label: string[] }[] = [];
  let section = "";
  doc.forEach((node, pos) => {
    if (node.type.name === "heading") {
      section = node.textContent;
      return;
    }
    const id = node.attrs.id as string | null;
    const r = id ? resultOf(id) : undefined;
    if (!id || !r) return;
    const defines = r.defines ?? [];
    readers.push({ reads: r.reads ?? [], label: defines.length ? defines : [node.type.name === "calc" ? "a check" : "a cell"] });
    const kind = node.type.name === "calc" ? "calc" : node.type.name === "code_block" ? node.attrs.lang : node.type.name;
    const description = node.type.name === "calc" ? descriptionOf(node.attrs.cellAttrs) : "";
    for (const name of defines) {
      const v = r.values?.find((x) => x.name === name);
      out.push({
        name, id, pos, kind, description, section,
        tex: v?.tex || null,
        valueKind: v?.kind ?? null,
        text: r.text ?? "",
        error: r.error,
        warnings: r.warnings ?? [],
        usedBy: [],
      });
    }
  });
  for (const v of out) {
    const users = new Set<string>();
    for (const r of readers) if (r.reads.includes(v.name)) for (const l of r.label) users.add(l);
    v.usedBy = [...users];
  }
  return out;
}

/** Does `v` match the search text (name or description, case-insensitive)? */
export function matches(v: Variable, query: string): boolean {
  const q = query.trim().toLowerCase();
  if (!q) return true;
  return v.name.toLowerCase().includes(q) || v.description.toLowerCase().includes(q) || v.section.toLowerCase().includes(q);
}
