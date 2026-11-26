// SPDX-License-Identifier: GPL-3.0-or-later
// The Templates tab's model (ADR-0014 §5): search over the two libraries, categories, and
// a prose cell's text for the preview.
import type { TemplateSummary } from "./api";

export type Which = "all" | "builtin" | "user";

function words(s: string): string[] {
  return s.toLowerCase().normalize("NFKD").split(/[^\p{L}\p{N}]+/u).filter(Boolean);
}

/** The category a list groups by: the first level, or for tables (ADR-0027) the last, the
 *  kind of part ("Nuts", "Pipe"). */
export const groupOf = (t: TemplateSummary, tables = false): string | undefined => (tables ? t.category.at(-1) : t.category[0]);

/** Templates matching `query` (every word found in the title, summary, tags, category, id or
 *  names), in `category` (see `groupOf`) and library; hidden ones only when asked; table
 *  templates only, or none of them. The title matches first, then the rest; otherwise by title. */
export function findTemplates(
  all: TemplateSummary[],
  query: string,
  category: string | null,
  which: Which,
  showHidden = false,
  tables = false,
): TemplateSummary[] {
  const q = words(query);
  const scored: [number, TemplateSummary][] = [];
  for (const t of all) {
    if (which !== "all" && t.library !== which) continue;
    if (t.hidden && !showHidden) continue;
    if (t.isTable !== tables) continue;
    if (category && groupOf(t, tables) !== category) continue;
    if (q.length) {
      const title = words(t.title);
      const rest = words([t.summary ?? "", ...t.tags, ...t.category, t.id, ...t.inputs, ...t.outputs].join(" "));
      let score = 0;
      let all = true;
      for (const w of q) {
        if (title.some((x) => x.startsWith(w))) score += 2;
        else if (rest.some((x) => x.startsWith(w))) score += 1;
        else all = false;
      }
      if (!all) continue;
      scored.push([score, t]);
    } else scored.push([0, t]);
  }
  return scored
    .sort((a, b) => b[0] - a[0] || a[1].title.localeCompare(b[1].title, undefined, { sensitivity: "base" }))
    .map(([, t]) => t);
}

/** The categories to group by (see `groupOf`), sorted, of table templates or of the others. */
export function categoriesOf(all: TemplateSummary[], tables = false): string[] {
  const of = all.filter((t) => t.isTable === tables).map((t) => groupOf(t, tables));
  return [...new Set(of.filter((c): c is string => !!c))].sort((a, b) => a.localeCompare(b));
}

interface PMNode {
  type?: string;
  text?: string;
  attrs?: Record<string, unknown>;
  content?: PMNode[];
}

/** A prose cell's ProseMirror JSON as plain text for the preview: its kind (heading level,
 *  list) and its text, list items on their own lines. */
export function proseSummary(json: string): { kind: string; text: string } {
  let n: PMNode;
  try {
    n = JSON.parse(json);
  } catch {
    return { kind: "paragraph", text: json };
  }
  const text = (x: PMNode): string => {
    if (x.text) return x.text;
    if (x.type === "hard_break") return "\n";
    const inner = (x.content ?? []).map(text);
    return x.type === "list_item" ? "• " + inner.join(" ").trim() + "\n" : inner.join("");
  };
  const kind = n.type === "heading" ? `h${n.attrs?.level ?? 2}` : (n.type ?? "paragraph");
  return { kind, text: text(n).trim() };
}

/** "Structural / Connections" ↔ ["Structural", "Connections"]; "a, b" ↔ ["a", "b"]. */
export const splitList = (s: string, sep: string) =>
  s
    .split(sep)
    .map((x) => x.trim())
    .filter(Boolean);

/** A table's rows whose designation (or, failing that, any value) has every word of `query`,
 *  as their indices. `"1/2"` finds `1/2` and `1 1/2`; `m8` finds `M8`. */
export function filterRows(rows: string[][], query: string): number[] {
  const q = query.toLowerCase().split(/\s+/).filter(Boolean);
  const out: number[] = [];
  rows.forEach((r, i) => {
    if (!q.length) return out.push(i);
    const size = r[0].toLowerCase();
    const all = r.join(" ").toLowerCase();
    if (q.every((w) => size.includes(w)) || q.every((w) => all.includes(w))) out.push(i);
  });
  return out;
}
