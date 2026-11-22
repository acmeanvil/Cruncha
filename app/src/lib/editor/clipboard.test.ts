// SPDX-License-Identifier: GPL-3.0-or-later
// Copy and paste of cells between two editors (ADR-0024 §1), through ProseMirror's own
// clipboard serialization and paste, with the editor's hooks.
// @vitest-environment jsdom
import { expect, test, vi } from "vitest";
import { EditorState, NodeSelection, TextSelection } from "prosemirror-state";
import { EditorView } from "prosemirror-view";
import type { Cell } from "../api";

// jsdom has no ClipboardEvent; pasteHTML only needs one to exist.
(globalThis as unknown as { ClipboardEvent: unknown }).ClipboardEvent ??= class extends Event {
  clipboardData = null;
};

vi.mock("../api", () => ({ api: { commit: vi.fn(), previewCalc: async (s: string) => ({ tex: s, error: null }) } }));

const { docFromCells, nodeToContent } = await import("./cells");
const { editorPlugins } = await import("./plugins");
const { adaptPasted, plainText, stampOrigin } = await import("./clipboard");

const cell = (id: string, kind: string, key: string, source: string, attrs: object = {}): Cell => ({
  id, kind, order_key: key, parent_id: null, source, attrs, defines: [], reads: [],
  provenance: { state: "human" }, modified_by: "user:x", modified_at_us: 1,
});
const para = (text: string) => JSON.stringify({ type: "paragraph", content: [{ type: "text", text }] });
const bold = JSON.stringify({
  type: "paragraph",
  content: [{ type: "text", text: "plain " }, { type: "text", marks: [{ type: "strong" }], text: "bold" }],
});

function editor(cells: Cell[], origin: string | null) {
  const state = EditorState.create({ doc: docFromCells(cells), plugins: editorPlugins(() => {}) });
  return new EditorView(document.createElement("div"), {
    state,
    transformCopied: (slice) => stampOrigin(slice, origin),
    transformPasted: (slice, v) => adaptPasted(slice, v),
    clipboardTextSerializer: (slice) => plainText(slice),
  });
}

const A = () =>
  editor(
    [
      cell("P", "prose", "a0", bold),
      cell("C", "calc", "a1", "M = w L^2 / 8 = ?? kip·ft", { description: "Midspan moment" }),
      cell("Y", "python", "a2", "x = np.linspace(0, L, 50)\nmoment = w * x"),
    ],
    "document:DOC_A@v7",
  );
const B = () => editor([cell("Q", "prose", "b0", para("target text")), cell("D", "calc", "b1", "L = 20 ft")], "document:DOC_B@v2");

/** Copy `from`..`to` in `a` and paste it at `at` in `b` (a text position, or a whole block). */
function copyPaste(a: EditorView, from: number, to: number, b: EditorView, at: number) {
  // What a real copy takes: the selection's content, with the blocks around it.
  const { dom } = a.serializeForClipboard(TextSelection.create(a.state.doc, from, to).content());
  b.dispatch(b.state.tr.setSelection(TextSelection.create(b.state.doc, at)));
  b.pasteHTML(dom.innerHTML);
}

const blocks = (v: EditorView) => {
  const out: ReturnType<typeof nodeToContent>[] = [];
  v.state.doc.forEach((n) => out.push(nodeToContent(n)));
  return out;
};

test("whole cells keep their kind, source and description, get new IDs, and record their origin", () => {
  const a = A();
  const b = B();
  const end = b.state.doc.content.size;
  // Select the calc and the Python cell (whole blocks) and paste after B's last block.
  const calcPos = a.state.doc.child(0).nodeSize;
  copyPaste(a, calcPos, a.state.doc.content.size, b, end - b.state.doc.lastChild!.nodeSize - 1);
  const got = blocks(b);
  const calc = got.find((c) => c.kind === "calc" && c.source.startsWith("M ="))!;
  const py = got.find((c) => c.kind === "python")!;
  expect(calc).toBeTruthy();
  expect(calc.attrs).toEqual({ description: "Midspan moment" });
  expect(calc.from).toBe("document:DOC_A@v7");
  expect(py.source).toBe("x = np.linspace(0, L, 50)\nmoment = w * x");
  expect(py.from).toBe("document:DOC_A@v7");
  // New identities, distinct from the source's and B's.
  expect(["C", "Y", "Q", "D"]).not.toContain(calc.id);
  expect(new Set(got.map((c) => c.id)).size).toBe(got.length);
  // B's own cells are untouched.
  expect(got.find((c) => c.id === "D")!.from ?? null).toBeNull();
});

test("a copied calc pasted within the same document is a second cell, not the same one", () => {
  const a = A();
  const calcPos = a.state.doc.child(0).nodeSize;
  a.dispatch(a.state.tr.setSelection(NodeSelection.create(a.state.doc, calcPos)));
  const { dom } = a.serializeForClipboard(a.state.selection.content());
  a.dispatch(a.state.tr.setSelection(TextSelection.create(a.state.doc, 1)));
  a.pasteHTML(dom.innerHTML);
  const calcs = blocks(a).filter((c) => c.kind === "calc");
  expect(calcs).toHaveLength(2);
  expect(calcs.map((c) => c.id)).toContain("C"); // the original keeps its ID
  expect(new Set(calcs.map((c) => c.id)).size).toBe(2);
});

test("part of a paragraph pastes into the paragraph at the cursor, with its marks", () => {
  const a = A();
  const b = B();
  // "bold" (the last 4 characters of A's first paragraph), into "target text" after "target".
  const p = a.state.doc.child(0);
  copyPaste(a, p.nodeSize - 5, p.nodeSize - 1, b, 1 + "target".length);
  expect(b.state.doc.child(0).textContent).toBe("targetbold text");
  const marked = b.state.doc.child(0).content.toJSON().find((n: { text: string }) => n.text === "bold");
  expect(marked.marks).toEqual([{ type: "strong" }]);
  expect(b.state.doc.childCount).toBe(2); // no new cell
  expect(blocks(b)[0].id).toBe("Q");
});

test("part of a code cell pastes into prose as inline code, and into a code cell as text", () => {
  const a = A();
  const b = B();
  const py = a.state.doc.child(0).nodeSize + a.state.doc.child(1).nodeSize;
  // "moment = w * x", the code cell's second line.
  const start = py + 1 + "x = np.linspace(0, L, 50)\n".length;
  copyPaste(a, start, start + "moment = w * x".length, b, 1);
  const first = b.state.doc.child(0);
  expect(first.type.name).toBe("paragraph");
  expect(first.textContent).toBe("moment = w * xtarget text");
  expect(first.content.firstChild!.marks.map((m) => m.type.name)).toEqual(["code"]);

  // Into another code cell: plain text at the cursor.
  const a2 = A();
  const pyPos = a2.state.doc.child(0).nodeSize + a2.state.doc.child(1).nodeSize;
  copyPaste(a2, start, start + "moment".length, a2, pyPos + 1);
  expect(a2.state.doc.child(2).textContent.startsWith("momentx = np.linspace")).toBe(true);
  expect(a2.state.doc.childCount).toBe(3);
});

test("the plain-text copy: one cell is its source; more is Markdown with fenced cells", () => {
  const a = A();
  const calcPos = a.state.doc.child(0).nodeSize;
  const calcOnly = a.state.doc.slice(calcPos, calcPos + a.state.doc.child(1).nodeSize);
  expect(a.serializeForClipboard(calcOnly).text).toBe("M = w L^2 / 8 = ?? kip·ft");
  const all = a.serializeForClipboard(a.state.doc.slice(0, a.state.doc.content.size)).text;
  expect(all).toBe("plain bold\n\n```calc\nM = w L^2 / 8 = ?? kip·ft\n```\n\n```python\nx = np.linspace(0, L, 50)\nmoment = w * x\n```");
});

test("moving a cell by dragging within a document isn't an import", () => {
  const a = A();
  const calcPos = a.state.doc.child(0).nodeSize;
  const slice = stampOrigin(a.state.doc.slice(calcPos, calcPos + a.state.doc.child(1).nodeSize), "document:DOC_A@v7");
  (a as unknown as { dragging: unknown }).dragging = { slice, move: true };
  const moved = adaptPasted(slice, a);
  expect(moved.content.firstChild!.attrs.from).toBeNull();
});

test("a selection from mid-paragraph through whole cells: text merges, cells split the target block", () => {
  const a = A();
  const b = B();
  // From "bold" in A's paragraph to the end of A: part of a paragraph, then the calc and code cells.
  const p = a.state.doc.child(0);
  copyPaste(a, p.nodeSize - 5, a.state.doc.content.size, b, 1 + "target".length);
  const got = blocks(b);
  // "target" + "bold" stays one paragraph; the calc and Python cell follow; " text" is split off after them.
  expect(got.map((c) => c.kind)).toEqual(["prose", "calc", "python", "prose", "calc"]);
  expect(b.state.doc.child(0).textContent).toBe("targetbold");
  expect(b.state.doc.child(3).textContent).toBe(" text");
  expect(got[0].id).toBe("Q");
  expect(got[1].from).toBe("document:DOC_A@v7");
  expect(new Set(got.map((c) => c.id)).size).toBe(got.length);
});
