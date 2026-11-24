// SPDX-License-Identifier: GPL-3.0-or-later
// The editor's document model without a DOM: cells <-> blocks, cell identity, and what a
// commit sends.
import { beforeEach, describe, expect, test, vi } from "vitest";
import { EditorState, TextSelection } from "prosemirror-state";
import type { Cell } from "../api";

const commit = vi.fn();
vi.mock("../api", () => ({
  api: {
    commit: (...args: unknown[]) => commit(...args),
    previewCalc: async (source: string) => ({ tex: source, error: null }),
  },
}));

const { cellToNode, docFromCells, nodeToContent } = await import("./cells");
const { editorPlugins, insertBlock, moveBlock, newCalc } = await import("./plugins");
const { DocSync, describe: describeChanges } = await import("./sync");
const { schema } = await import("./schema");

function cell(id: string, kind: string, key: string, source: string, extra: Partial<Cell> = {}): Cell {
  return {
    id, kind, order_key: key, parent_id: null, source, attrs: {}, defines: [], reads: [],
    provenance: { state: "human" }, modified_by: "user:x", modified_at_us: 1, ...extra,
  };
}

const para = (text: string) => JSON.stringify({ type: "paragraph", content: [{ type: "text", text }] });

const SAMPLE: Cell[] = [
  cell("P", "prose", "a0", para("Simply supported beam.")),
  cell("H", "prose", "a1", JSON.stringify({ type: "heading", attrs: { level: 2 }, content: [{ type: "text", text: "Loads" }] })),
  cell("C", "calc", "b1", "M = w L^2 / 8 = ?? kip·ft", { attrs: { display: "?" }, reads: ["w", "L"], defines: ["M"] }),
  cell("Y", "python", "c0", "import numpy as np\nx = 1"),
  cell("D", "data", "d0", "a\tb\n1\t2"),
];

function stateFor(cells: Cell[]) {
  return EditorState.create({ doc: docFromCells(cells), plugins: editorPlugins(() => {}) });
}

function blocks(s: EditorState) {
  const out: { id: string; key: string; type: string }[] = [];
  s.doc.forEach((n) => out.push({ id: n.attrs.id, key: n.attrs.key, type: n.type.name }));
  return out;
}

function expectWellFormed(s: EditorState) {
  const bs = blocks(s);
  expect(new Set(bs.map((b) => b.id)).size).toBe(bs.length);
  for (let i = 1; i < bs.length; i++) expect(bs[i - 1].key < bs[i].key).toBe(true);
}

/** Apply a transaction and let appendTransaction plugins run, as the view does. */
function apply(s: EditorState, cmd: (st: EditorState, d: (tr: any) => void) => unknown) {
  let next = s;
  cmd(s, (tr) => (next = s.apply(tr)));
  return next;
}

describe("cells and blocks", () => {
  test("every kind round-trips unchanged", () => {
    for (const c of SAMPLE) {
      const back = nodeToContent(cellToNode(c));
      expect(back).toEqual({ id: c.id, kind: c.kind, order_key: c.order_key, source: c.source, attrs: c.attrs });
    }
  });

  test("unreadable prose becomes text, an empty document one paragraph", () => {
    const n = cellToNode(cell("X", "prose", "a0", "not json"));
    expect(n.type.name).toBe("paragraph");
    expect(n.textContent).toBe("not json");
    expect(docFromCells([]).childCount).toBe(1);
  });
});

describe("cell identity", () => {
  test("new, split and moved blocks get distinct ids and ordered keys", () => {
    let s = stateFor(SAMPLE);
    expectWellFormed(s);
    // Insert a calc after the first paragraph.
    s = s.apply(s.tr.setSelection(TextSelection.create(s.doc, 3)));
    s = apply(s, insertBlock(newCalc()));
    expectWellFormed(s);
    expect(blocks(s)[1].type).toBe("calc");
    expect(blocks(s)[1].id).not.toBe(null);

    // Split the first paragraph: the second half is a new cell.
    s = s.apply(s.tr.split(5));
    expectWellFormed(s);
    expect(blocks(s)[0].id).toBe("P");
    expect(blocks(s)[1].id).not.toBe("P");

    // Move the heading up past both: it keeps its id and gets a key that fits.
    const before = blocks(s).map((b) => b.id);
    const hIndex = before.indexOf("H");
    let pos = 0;
    for (let i = 0; i < hIndex; i++) pos += s.doc.child(i).nodeSize;
    s = s.apply(s.tr.setSelection(TextSelection.create(s.doc, pos + 1)));
    s = apply(s, moveBlock(-1));
    expectWellFormed(s);
    expect(blocks(s).map((b) => b.id).indexOf("H")).toBe(hIndex - 1);
  });
});

describe("saving", () => {
  beforeEach(() => commit.mockReset());

  function setup(cells: Cell[]) {
    let s = stateFor(cells);
    const sync = new DocSync(() => s.doc, () => {}, () => {});
    sync.load(cells);
    return {
      sync,
      get: () => s,
      set: (n: EditorState) => {
        s = n;
        sync.changed();
      },
    };
  }

  test("a template inserts as one version of its own, its cells imported from it", async () => {
    commit.mockResolvedValue({ info: {}, results: {} });
    const t = setup(SAMPLE);
    // A pending edit is committed first, on its own.
    let pos = 0;
    t.get().doc.forEach((n, p) => {
      if (n.attrs.id === "C") pos = p;
    });
    let s = t.get();
    t.set(s.apply(s.tr.setNodeMarkup(pos, undefined, { ...s.doc.nodeAt(pos)!.attrs, source: "M = 3 = ?" })));
    await t.sync.insertAs("template:test.beam@1.0.0", "Insert template Beam 1.0.0", () => {
      s = t.get();
      const nodes = [cellToNode(cell(null as unknown as string, "prose", null as unknown as string, para("Beam"))), cellToNode(cell(null as unknown as string, "calc", null as unknown as string, "w = 1 kN/m_"))];
      t.set(s.apply(s.tr.insert(s.doc.content.size, nodes)));
    });
    expect(commit).toHaveBeenCalledTimes(2);
    expect(commit.mock.calls[0][1]).toBe("edit calc M = 3 = ?");
    const [changes, message] = commit.mock.calls[1];
    expect(message).toBe("Insert template Beam 1.0.0");
    expect(changes.map((c: { cell: Cell }) => [c.cell.kind, c.cell.provenance])).toEqual([
      ["prose", { state: "imported", source: "template:test.beam@1.0.0" }],
      ["calc", { state: "imported", source: "template:test.beam@1.0.0" }],
    ]);
    // Later edits are the user's own again.
    s = t.get();
    t.set(s.apply(s.tr.insertText("x", 1)));
    await t.sync.flush();
    expect(commit.mock.calls[2][0][0].cell.provenance).toEqual({ state: "human" });
  });

  test("only changed cells are sent, keeping their bookkeeping", async () => {
    commit.mockResolvedValue({ info: {}, results: {} });
    const t = setup(SAMPLE);
    // Edit the calc's source.
    let pos = 0;
    t.get().doc.forEach((n, p) => {
      if (n.attrs.id === "C") pos = p;
    });
    const s = t.get();
    t.set(s.apply(s.tr.setNodeMarkup(pos, undefined, { ...s.doc.nodeAt(pos)!.attrs, source: "M = 2 = ?" })));
    await t.sync.flush();
    expect(commit).toHaveBeenCalledTimes(1);
    const [changes, message] = commit.mock.calls[0];
    expect(changes).toHaveLength(1);
    expect(changes[0].op).toBe("upsert");
    expect(changes[0].cell.source).toBe("M = 2 = ?");
    expect(changes[0].cell.reads).toEqual(["w", "L"]); // kept until the evaluator recomputes
    expect(message).toBe("edit calc M = 2 = ?");

    // Nothing changed since: no commit.
    await t.sync.flush();
    expect(commit).toHaveBeenCalledTimes(1);
  });

  test("deletes are sent; a new empty block is not", async () => {
    commit.mockResolvedValue({ info: {}, results: {} });
    const t = setup(SAMPLE.slice(0, 2));
    let s = t.get();
    // Delete the heading, add an empty paragraph at the end.
    const first = s.doc.child(0).nodeSize;
    s = s.apply(s.tr.delete(first, first + s.doc.child(1).nodeSize));
    s = s.apply(s.tr.insert(s.doc.content.size, schema.nodes.paragraph.create()));
    t.set(s);
    await t.sync.flush();
    const [changes, message] = commit.mock.calls[0];
    expect(changes).toEqual([{ op: "delete", id: "H" }]);
    expect(message).toBe("delete text");
  });

  test("a failed commit is retried on the next flush", async () => {
    commit.mockRejectedValueOnce("disk full").mockResolvedValue({ info: {}, results: {} });
    const states: string[] = [];
    let s = stateFor(SAMPLE);
    const sync = new DocSync(() => s.doc, () => {}, (st) => states.push(st.status));
    sync.load(SAMPLE);
    s = s.apply(s.tr.insertText("!", 2));
    sync.changed();
    await sync.flush();
    expect(sync.state.status).toBe("error");
    await sync.flush();
    expect(commit).toHaveBeenCalledTimes(2);
    expect(sync.state.status).toBe("saved");
  });

  test("messages", () => {
    const c = { id: "a", kind: "calc", order_key: "a", source: "x = 1", attrs: {} };
    const p = { ...c, kind: "prose" };
    expect(describeChanges([c], [], [])).toBe("add calc x = 1");
    expect(describeChanges([], [p], [])).toBe("edit text");
    expect(describeChanges([c], [p, p], [])).toBe("add 1, edit 2 cells");
  });
});
