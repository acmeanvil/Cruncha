// SPDX-License-Identifier: GPL-3.0-or-later
// The variable browser's model, and descriptions kept in a calc's attributes.
import { expect, test } from "vitest";
import type { Cell, CellResult } from "../api";
import { descriptionOf, docFromCells, nodeToContent, withDescription } from "./cells";
import { isTall, matches, variablesOf } from "./variables";

function cell(id: string, kind: string, key: string, source: string, attrs: unknown = {}): Cell {
  return {
    id, kind, order_key: key, parent_id: null, source, attrs, defines: [], reads: [],
    provenance: { state: "human" }, modified_by: "", modified_at_us: 0,
  };
}
const heading = (text: string) => JSON.stringify({ type: "heading", attrs: { level: 2 }, content: [{ type: "text", text }] });

const cells = [
  cell("H1", "prose", "a0", heading("Loads")),
  cell("L", "calc", "a1", "L = 24ft", { display: "?", description: "Span" }),
  cell("W", "calc", "a2", "w = 1.5kip/ft"),
  cell("H2", "prose", "a3", heading("Moment")),
  cell("M", "calc", "a4", "M = w L^2 / 8 = ?? kip·ft", { description: "Midspan moment" }),
  cell("T", "calc", "a5", "@test M < 150kip·ft"),
  cell("PY", "python", "a6", "x = 2"),
];
const results: Record<string, CellResult> = {
  L: { tex: "", error: null, defines: ["L"], reads: [], values: [{ name: "L", tex: "24\\,\\text{ft}", kind: "number" }] },
  W: { tex: "", error: null, defines: ["w"], reads: [], values: [{ name: "w", tex: "1.5\\,\\text{kip/ft}", kind: "number" }] },
  M: { tex: "", error: null, text: "M = 108 kip·ft", defines: ["M"], reads: ["w", "L"], values: [{ name: "M", tex: "108", kind: "number" }] },
  T: { tex: "", error: null, defines: [], reads: ["M"] },
  PY: { tex: "", error: "Python cells can't run yet", defines: ["x"], reads: [] },
};

test("every defined name, in order, with section, value, description and users", () => {
  const vs = variablesOf(docFromCells(cells), (id) => results[id]);
  expect(vs.map((v) => v.name)).toEqual(["L", "w", "M", "x"]);
  const [L, w, M, x] = vs;
  expect(L).toMatchObject({ id: "L", section: "Loads", description: "Span", tex: "24\\,\\text{ft}", usedBy: ["M"] });
  expect(w.description).toBe("");
  expect(M).toMatchObject({ section: "Moment", description: "Midspan moment", usedBy: ["a check"], text: "M = 108 kip·ft" });
  expect(x).toMatchObject({ kind: "python", tex: null, error: "Python cells can't run yet", usedBy: [] });
  expect(vs.filter((v) => matches(v, "span")).map((v) => v.name)).toEqual(["L", "M"]);
  expect(vs.filter((v) => matches(v, "MOMENT")).map((v) => v.name)).toEqual(["M", "x"]);
  expect(vs.filter((v) => matches(v, "")).length).toBe(4);
});

test("descriptions live in the calc's attrs and round-trip through the editor", () => {
  const doc = docFromCells(cells);
  const L = doc.child(1);
  expect(descriptionOf(L.attrs.cellAttrs)).toBe("Span");
  const changed = withDescription(L.attrs.cellAttrs, "  Clear span  ");
  expect(JSON.parse(changed)).toEqual({ display: "?", description: "Clear span" });
  expect(JSON.parse(withDescription(changed, " "))).toEqual({ display: "?" });
  expect(nodeToContent(L).attrs).toEqual({ display: "?", description: "Span" });
  expect(descriptionOf("not json")).toBe("");
});

test("arrays and other multi-line values are collapsible; scalars aren't", () => {
  const base = variablesOf(docFromCells(cells), (id) => results[id])[0];
  expect(isTall(base)).toBe(false);
  expect(isTall({ ...base, shape: "3 × 1 matrix" })).toBe(true);
  expect(isTall({ ...base, tex: "\\begin{pmatrix}1 \\\\ 2\\end{pmatrix}" })).toBe(true);
  expect(isTall({ ...base, tex: null, summary: "ndarray float64 (51,)" })).toBe(false);
});

test("the meta cell (report settings) is not a block, so the editor never shows or deletes it", () => {
  const doc = docFromCells([cell("META", "meta", "0", "", { report: { title: { project: "X" } } }), ...cells]);
  expect(doc.childCount).toBe(cells.length);
  doc.forEach((n) => expect(n.attrs.id).not.toBe("META"));
});
