// SPDX-License-Identifier: GPL-3.0-or-later
// Markdown in the editor: the prose node types store the JSON cruncha-prose writes, typing
// Markdown applies it, and which pastes count as Markdown.
// @vitest-environment jsdom
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { expect, test, vi } from "vitest";
import { EditorState, TextSelection } from "prosemirror-state";
import { EditorView } from "prosemirror-view";
import type { Cell } from "../api";

vi.mock("../api", () => ({ api: { commit: vi.fn(), previewCalc: async (s: string) => ({ tex: s, error: null }) } }));

const { cellToNode, docFromCells, nodeToContent, proseText } = await import("./cells");
const { editorPlugins } = await import("./plugins");
const { markdownOf } = await import("./paste");

const prose = (id: string, source: string): Cell => ({
  id, kind: "prose", order_key: "a0", parent_id: null, source, attrs: {}, defines: [], reads: [],
  provenance: { state: "human" }, modified_by: "user:x", modified_at_us: 1,
});

test("prose cells round-trip exactly as cruncha-prose writes them", () => {
  // vitest runs in app/.
  const sources: string[] = JSON.parse(readFileSync(resolve("../crates/cruncha-prose/tests/fixtures/canonical.json"), "utf8"));
  expect(sources.length).toBeGreaterThan(5);
  for (const s of sources) expect(nodeToContent(cellToNode(prose("P", s))).source).toBe(s);
});

function editor() {
  const state = EditorState.create({ doc: docFromCells([prose("P", '{"type":"paragraph"}')]), plugins: editorPlugins(() => {}) });
  const view = new EditorView(document.createElement("div"), { state });
  return view;
}

/** Types `text` a character at a time, as the input rules see it. */
function type(view: EditorView, text: string) {
  for (const ch of text) {
    const { from, to } = view.state.selection;
    const handled = view.someProp("handleTextInput", (f) => f(view, from, to, ch, () => view.state.tr.insertText(ch, from, to)));
    if (!handled) view.dispatch(view.state.tr.insertText(ch, from, to));
  }
}

const json = (view: EditorView) => view.state.doc.toJSON().content.map((n: { attrs?: object }) => ({ ...n, attrs: undefined }));

test("typing Markdown marks applies them and drops the delimiters", () => {
  const v = editor();
  type(v, "a **bold** and *it* and _it_ and `code` and f_c_ and [AISC](https://aisc.org) x");
  const content = v.state.doc.firstChild!.content.toJSON() as { text: string; marks?: { type: string; attrs?: { href: string } }[] }[];
  const marked = content.filter((n) => n.marks).map((n) => [n.text, n.marks!.map((m) => m.attrs?.href ?? m.type).join("+")]);
  expect(marked).toEqual([["bold", "strong"], ["it", "em"], ["it", "em"], ["code", "code"], ["AISC", "https://aisc.org"]]);
  expect(v.state.doc.textContent).toBe("a bold and it and it and code and f_c_ and AISC x");
});

test("a link to another scheme stays text", () => {
  const v = editor();
  type(v, "[x](javascript:alert(1))");
  expect(v.state.doc.textContent).toBe("[x](javascript:alert(1))");
});

test("block rules: lists, numbered lists from any start, quotes and rules", () => {
  let v = editor();
  type(v, "- one");
  expect(json(v)[0].type).toBe("bullet_list");
  v = editor();
  type(v, "3. three");
  expect(v.state.doc.firstChild!.type.name).toBe("ordered_list");
  expect(v.state.doc.firstChild!.attrs.order).toBe(3);
  v = editor();
  type(v, "> quoted");
  expect(v.state.doc.firstChild!.type.name).toBe("blockquote");
  v = editor();
  type(v, "---");
  expect(v.state.doc.toJSON().content.map((n: { type: string }) => n.type)).toEqual(["horizontal_rule", "paragraph"]);
  expect(v.state.selection instanceof TextSelection && v.state.selection.$from.parent.type.name).toBe("paragraph");
});

test("a list is one cell, and a calc can't go inside one", () => {
  const v = editor();
  type(v, "- one");
  const cells = [] as string[];
  v.state.doc.forEach((n) => cells.push(nodeToContent(n).kind));
  expect(cells).toEqual(["prose"]);
  type(v, " = ");
  // "= " starts a calc only at the start of a top-level paragraph.
  expect(v.state.doc.firstChild!.type.name).toBe("bullet_list");
});

test("which pastes are Markdown", () => {
  expect(markdownOf("# Title\n\ntext", "", ["text/plain"], false)).toBe("# Title\n\ntext");
  expect(markdownOf("- a\n- b", "", ["text/plain"], false)).not.toBeNull();
  expect(markdownOf("some **bold** text", "", ["text/plain"], false)).not.toBeNull();
  expect(markdownOf("plain words only", "", ["text/plain"], false)).toBeNull();
  expect(markdownOf("f_c and M_u", "", ["text/plain"], false)).toBeNull();
  // Rich text keeps its HTML; code editors' HTML doesn't count; code cells paste as text.
  expect(markdownOf("# Title", "<h1>Title</h1>", ["text/plain", "text/html"], false)).toBeNull();
  expect(markdownOf("# Title", "<div>…</div>", ["text/plain", "text/html", "vscode-editor-data"], false)).toBe("# Title");
  expect(markdownOf("# Title", "", ["text/plain"], true)).toBeNull();
});

test("proposal text for lists and quotes", () => {
  const list = '{"type":"ordered_list","attrs":{"order":2},"content":[{"type":"list_item","content":[{"type":"paragraph","content":[{"type":"text","text":"a"}]}]},{"type":"list_item","content":[{"type":"paragraph","content":[{"type":"text","text":"b"}]}]}]}';
  expect(proseText(list)).toBe("2. a\n3. b");
  expect(proseText('{"type":"blockquote","content":[{"type":"paragraph","content":[{"type":"text","text":"q"}]}]}')).toBe("> q");
  expect(proseText('{"type":"heading","attrs":{"level":2},"content":[{"type":"text","text":"H"}]}')).toBe("## H");
});
