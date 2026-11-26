// SPDX-License-Identifier: GPL-3.0-or-later
// @vitest-environment jsdom
// The owner's freeze (2026-11-26): inserting a 25-cell table row into a document locked the
// window. The real editor component, a fake backend: the insert must finish promptly.
import { expect, test, vi } from "vitest";
import { flushSync, mount, tick } from "svelte";
import type { Cell } from "../api";

let version = 1;
const committed: unknown[] = [];
vi.mock("../api", () => ({
  api: {
    pasteClashes: async () => [],
    renameCells: async (cells: { source: string }[]) => cells.map((c) => c.source),
    suffixNames: async () => ({}),
    markdownCells: async () => [],
    openUrl: async () => {},
    runPython: async () => ({}),
    stopPython: async () => {},
    previewCalc: async (s: string) => ({ tex: s, error: null }),
    commit: async (changes: { id: string; source?: string; kind?: string }[]) => {
      committed.push(changes);
      version += 1;
      const results: Record<string, unknown> = {};
      for (const c of changes) if (c.kind === "calc") results[c.id] = { tex: c.source ?? "", error: null, text: c.source ?? "" };
      return { info: { id: "D", version, name: "t.cruncha" }, results, cells: [] };
    },
  },
}));
vi.mock("../log", () => ({ logAction: () => {}, logWarn: () => {}, logError: () => {} }));

// jsdom has no layout: what the editor calls for scrolling does nothing here.
Element.prototype.scrollIntoView = () => {};
const { default: Editor } = await import("../Editor.svelte");

const cell = (id: string, kind: string, key: string, source: string): Cell => ({
  id, kind, order_key: key, parent_id: null, source, attrs: {}, defines: [], reads: [],
  provenance: { state: "human" }, modified_by: "", modified_at_us: 0,
});

test("a 25-cell template row inserts without locking up", async () => {
  const cells = [cell("P0", "prose", "a0", JSON.stringify({ type: "paragraph" }))];
  for (let i = 0; i < 25; i++) cells.push(cell(`K${i}`, "calc", `b${String(i).padStart(2, "0")}`, `k${i} = ${i} mm`));
  const host = document.createElement("div");
  document.body.append(host);
  let outlines = 0;
  const ed = mount(Editor, {
    target: host,
    props: {
      cells, results: {}, editable: true, loadKey: "1", docId: "D",
      oncommitted: () => {}, onsavestate: () => {}, onoutline: () => outlines++, onvariables: () => {},
    },
  }) as unknown as { insertTemplate: (t: unknown, s?: string) => Promise<void> };
  flushSync();
  const t = {
    id: "mechanical.tables.x", version: "1.0.0", title: "Row", inputs: [], row: "M6",
    cells: [
      { kind: "prose", source: JSON.stringify({ type: "paragraph", content: [{ type: "text", text: "Screw M6" }] }), attrs: {} },
      ...Array.from({ length: 24 }, (_, i) => ({ kind: "calc", source: `d_${i} = ${i} mm`, attrs: { description: `col ${i}` } })),
    ],
  };
  const start = performance.now();
  await ed.insertTemplate(t);
  for (let i = 0; i < 20; i++) {
    await tick();
    await new Promise((r) => setTimeout(r, 10));
  }
  const ms = performance.now() - start;
  expect(committed.length).toBeGreaterThan(0);
  expect(ms).toBeLessThan(3000);
  console.log(`insert + settle: ${ms.toFixed(0)} ms, commits ${committed.length}, outline updates ${outlines}`);
}, 20000);
