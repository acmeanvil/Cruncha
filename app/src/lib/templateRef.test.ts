// SPDX-License-Identifier: GPL-3.0-or-later
import { describe, expect, it } from "vitest";
import type { TemplateSummary } from "./api";
import { categoriesOf, filterRows, findTemplates, proseSummary, splitList } from "./templateRef";

const t = (over: Partial<TemplateSummary>): TemplateSummary => ({
  key: "builtin:x",
  library: "builtin",
  path: "x",
  id: "x",
  version: "1.0.0",
  title: "X",
  summary: null,
  category: [],
  tags: [],
  inputs: [],
  outputs: [],
  hidden: false,
  isTable: false,
  error: null,
  ...over,
});

const all = [
  t({ key: "builtin:a", id: "structural.bolts", title: "Bolt shear and tension", category: ["Structural engineering", "Connections"], tags: ["AISC 360"] }),
  t({ key: "builtin:b", id: "fluids.pipes", title: "Pipe friction", summary: "Darcy friction factor for bolted flanges? no", category: ["Fluids"], hidden: true }),
  t({ key: "user:c", library: "user", id: "user.loads", title: "Load combination", category: ["Structural engineering"], inputs: ["D", "L"] }),
];

describe("the Templates tab", () => {
  it("finds every word, title matches first", () => {
    expect(findTemplates(all, "bolt", null, "all", true).map((x) => x.key)).toEqual(["builtin:a", "builtin:b"]);
    expect(findTemplates(all, "bolt", null, "all").map((x) => x.key)).toEqual(["builtin:a"]);
    expect(findTemplates(all, "aisc bolt", null, "all").map((x) => x.key)).toEqual(["builtin:a"]);
    expect(findTemplates(all, "aisc pipe", null, "all", true)).toEqual([]);
  });

  it("filters by category and library, and lists by title", () => {
    expect(findTemplates(all, "", "Structural engineering", "all").map((x) => x.title)).toEqual(["Bolt shear and tension", "Load combination"]);
    expect(findTemplates(all, "", null, "user").map((x) => x.key)).toEqual(["user:c"]);
    expect(categoriesOf(all)).toEqual(["Fluids", "Structural engineering"]);
  });

  it("previews prose as text", () => {
    expect(proseSummary('{"type":"heading","attrs":{"level":3},"content":[{"type":"text","text":"Strength"}]}')).toEqual({ kind: "h3", text: "Strength" });
    const list = '{"type":"bullet_list","content":[{"type":"list_item","content":[{"type":"paragraph","content":[{"type":"text","text":"A307"}]}]},{"type":"list_item","content":[{"type":"paragraph","content":[{"type":"text","text":"A325"}]}]}]}';
    expect(proseSummary(list)).toEqual({ kind: "bullet_list", text: "• A307\n• A325" });
    expect(splitList(" Structural / Connections /", "/")).toEqual(["Structural", "Connections"]);
  });
});

describe("a table template's rows (ADR-0027)", () => {
  const rows = [
    ["1/4", "20", "0.25"],
    ["1/2", "13", "0.5"],
    ["1 1/2", "6", "1.5"],
    ["M8", "1.25", "13"],
  ];
  it("filters by designation, then by any value", () => {
    expect(filterRows(rows, "")).toEqual([0, 1, 2, 3]);
    expect(filterRows(rows, "1/2")).toEqual([1, 2]);
    expect(filterRows(rows, "m8")).toEqual([3]);
    expect(filterRows(rows, "13")).toEqual([1, 3]);
    expect(filterRows(rows, "x")).toEqual([]);
  });
});

describe("the Tables tab (ADR-0027)", () => {
  const nuts = t({ key: "builtin:n", id: "mechanical.tables.iso.hexNut", title: "Hex nuts", isTable: true, category: ["Mechanical engineering", "Standard parts", "Nuts"] });
  const w = t({ key: "builtin:w", id: "structural.tables.aisc.w", title: "W shapes", isTable: true, category: ["Structural engineering", "Shapes", "AISC shapes"] });
  const both = [...all, nuts, w];
  it("lists only tables, grouped by the kind of part; the Templates tab lists the rest", () => {
    expect(findTemplates(both, "", null, "all", false, true)).toEqual([nuts, w]);
    expect(findTemplates(both, "", "Nuts", "all", false, true)).toEqual([nuts]);
    expect(categoriesOf(both, true)).toEqual(["AISC shapes", "Nuts"]);
    expect(findTemplates(both, "", null, "all", true).map((x) => x.key)).not.toContain("builtin:n");
    expect(categoriesOf(both)).not.toContain("Mechanical engineering");
  });
});
