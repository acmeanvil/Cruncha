// SPDX-License-Identifier: GPL-3.0-or-later
// The function reference's model (FunctionReference.svelte): search and what Insert types.
import type { FunctionEntry } from "./api";

const norm = (s: string) => s.toLowerCase().normalize("NFKD");

/** Entries matching `query` (name, signature, summary, category, notes) in `category`;
 *  exact and prefix name matches first. */
export function findEntries(entries: FunctionEntry[], query: string, category: string | null): FunctionEntry[] {
  const q = norm(query.trim());
  const hit = (e: FunctionEntry) =>
    !q || [e.name, e.signature, e.summary, e.category, e.caution ?? "", e.note ?? ""].some((s) => norm(s).includes(q));
  const out = entries.filter((e) => (!category || e.category === category) && hit(e));
  if (q) {
    const rank = (e: FunctionEntry) => (norm(e.name) === q ? 0 : norm(e.name).startsWith(q) ? 1 : 2);
    out.sort((a, b) => rank(a) - rank(b));
  }
  return out;
}

/** What Insert types, and how far back from its end the cursor goes (inside `sqrt()`). */
export function insertion(e: FunctionEntry): [string, number] {
  if (e.insert) return [e.insert, 0];
  if (/^[\p{L}][\p{L}\p{N}]*\(/u.test(e.signature)) return [`${e.name}()`, 1];
  return [e.signature.split(",")[0].replace(/\b[a-z]\b/g, "").trim() || e.name, 0];
}
