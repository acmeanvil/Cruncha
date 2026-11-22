// SPDX-License-Identifier: GPL-3.0-or-later
// The variable browser's model: every name the document defines, in document order, with its
// value, description, where it is defined and what uses it.
import type { Node } from "prosemirror-model";
import type { CellResult, Preview } from "../api";
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
  /** Python values: what the object is (shown when calcs can't use it). */
  summary: string | null;
  /** "number", "matrix", … */
  valueKind: string | null;
  /** Vectors, matrices, frames: their size ("3 × 4 matrix"). */
  shape: string | null;
  /** Vectors, matrices, frames: columns and the first and last rows. */
  preview: Preview | null;
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
        summary: v?.summary ?? null,
        valueKind: v?.kind ?? null,
        shape: v?.shape ?? null,
        preview: v?.preview ?? null,
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

/** Is the value too tall for one line (a vector, matrix, frame, or multi-row TeX)? The
 *  browser shows these collapsed to their size, with a toggle. */
export function isTall(v: Variable): boolean {
  return v.shape != null || /\\begin\{|\\\\/.test(v.tex ?? "");
}

/** The unit a value is shown in, from its preview or its plain-text result ("108 kip·ft"). */
export function unitOf(v: Variable): string {
  if (v.preview) {
    const units = new Set(v.preview.columns.map((c) => c.unit).filter(Boolean));
    return units.size === 1 ? [...units][0]! : units.size ? "mixed" : "";
  }
  if (v.valueKind !== "number") return "";
  const m = /[-+]?(?:\d[\d,]*\.?\d*|\.\d+)(?:e[-+]?\d+)?\s+(\S.*)$/i.exec(v.text.split(" = ").pop() ?? "");
  return m ? m[1] : "";
}

/** What kind of value: "number", "matrix", "data frame", "Python object", … */
export function typeOf(v: Variable): string {
  if (v.error && !v.valueKind) return "error";
  const k = v.valueKind ?? "";
  if (k === "python") return "Python object";
  if (k === "dataframe") return "data frame";
  return k || "—";
}

/** Does `v` match the search text (name or description, case-insensitive)? */
export function matches(v: Variable, query: string): boolean {
  const q = query.trim().toLowerCase();
  if (!q) return true;
  return v.name.toLowerCase().includes(q) || v.description.toLowerCase().includes(q) || v.section.toLowerCase().includes(q);
}
