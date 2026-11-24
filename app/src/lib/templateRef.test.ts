// SPDX-License-Identifier: GPL-3.0-or-later
import { describe, expect, it } from "vitest";
import type { TemplateSummary } from "./api";
import { categoriesOf, findTemplates, proseSummary, splitList } from "./templateRef";

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
