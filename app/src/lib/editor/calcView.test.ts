// SPDX-License-Identifier: GPL-3.0-or-later
// @vitest-environment jsdom
// The calc block in a real (simulated) DOM: open, type, submit, and what gets committed.
import { beforeEach, expect, test, vi } from "vitest";
import { EditorState, NodeSelection } from "prosemirror-state";
import { EditorView } from "prosemirror-view";
import type { Cell } from "../api";

const commit = vi.fn();
vi.mock("../api", () => ({
  api: {
    commit: (...a: unknown[]) => commit(...a),
    // A stand-in for the backend: the source as TeX, `)` alone as an error.
    previewCalc: async (source: string) =>
      source.includes("bad") ? { tex: "", error: "malformed" } : { tex: source.replace(/\^(\d)/g, "^{$1}"), error: null },
  },
}));

const { docFromCells } = await import("./cells");
const { editorPlugins } = await import("./plugins");
const { CalcView } = await import("./calcView");
const { Results } = await import("./results");
const { DocSync } = await import("./sync");
const { schema } = await import("./schema");

function cell(id: string, kind: string, key: string, source: string): Cell {
  return {
    id, kind, order_key: key, parent_id: null, source, attrs: {}, defines: [], reads: [],
    provenance: { state: "human" }, modified_by: "", modified_at_us: 0,
  };
}

const flushPromises = () => new Promise((r) => setTimeout(r, 0));

function mount(cells: Cell[]) {
  const host = document.createElement("div");
  document.body.append(host);
  const results = new Results();
  let view!: EditorView;
  const sync = new DocSync(() => view.state.doc, (v, s) => results.set(v.results, s), () => {});
  const submit = () => void sync.flush();
  view = new EditorView(host, {
    state: EditorState.create({ doc: docFromCells(cells), plugins: editorPlugins(submit) }),
    nodeViews: { calc: (n, v, g) => new CalcView(n, v, g, results, submit) },
    dispatchTransaction(tr) {
      view.updateState(view.state.apply(tr));
      if (tr.docChanged) sync.changed();
    },
  });
  sync.load(cells);
  results.set({ C: { tex: "x = 1", error: null } }, new Map([["C", "x = 1"]]));
  return { view, host, sync };
}

beforeEach(() => {
  commit.mockReset();
  commit.mockResolvedValue({ info: {}, results: { C: { tex: "x = 2^{2}", error: null } } });
  document.body.replaceChildren();
});

test("a committed calc shows typeset math", () => {
  const { host } = mount([cell("C", "calc", "a0", "x = 1")]);
  expect(host.querySelector(".calc-display math")).not.toBeNull();
});

test("click, type, Enter: the new source is committed at once and rendered", async () => {
  const { host, view } = mount([cell("C", "calc", "a0", "x = 1")]);
  host.querySelector(".calc-display")!.dispatchEvent(new MouseEvent("mousedown", { bubbles: true }));
  const input = host.querySelector<HTMLTextAreaElement>("textarea.calc-input")!;
  expect(input).not.toBeNull();
  expect(input.value).toBe("x = 1");
  input.value = "x = 2^2";
  input.dispatchEvent(new Event("input"));
  input.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true }));
  await flushPromises();
  expect(commit).toHaveBeenCalledTimes(1);
  const [changes, message] = commit.mock.calls[0];
  expect(changes).toEqual([expect.objectContaining({ op: "upsert", cell: expect.objectContaining({ id: "C", source: "x = 2^2" }) })]);
  expect(message).toBe("edit calc x = 2^2");
  // The editor closed, the cursor moved into a new paragraph below, and math is shown.
  expect(host.querySelector("textarea")).toBeNull();
  expect(view.state.doc.childCount).toBe(2);
  expect(view.state.selection.$from.parent.type).toBe(schema.nodes.paragraph);
  await flushPromises();
  expect(host.querySelector(".calc-display math")).not.toBeNull();
});

test("Escape cancels; emptying a calc deletes it", async () => {
  const { host, view } = mount([cell("P", "prose", "a0", '{"type":"paragraph","content":[{"type":"text","text":"hi"}]}'), cell("C", "calc", "a1", "x = 1")]);
  const open = () => host.querySelector(".calc-display")!.dispatchEvent(new MouseEvent("mousedown", { bubbles: true }));
  open();
  let input = host.querySelector<HTMLTextAreaElement>("textarea")!;
  input.value = "y = 3";
  input.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true }));
  expect(view.state.doc.child(1).attrs.source).toBe("x = 1");
  open();
  input = host.querySelector<HTMLTextAreaElement>("textarea")!;
  input.value = "  ";
  input.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true }));
  await flushPromises();
  expect(view.state.doc.childCount).toBe(1);
  expect(commit.mock.calls[0][0]).toEqual([{ op: "delete", id: "C" }]);
});

test("a parse error shows the source and the message", async () => {
  const { host } = mount([cell("C", "calc", "a0", "x = 1")]);
  host.querySelector(".calc-display")!.dispatchEvent(new MouseEvent("mousedown", { bubbles: true }));
  const input = host.querySelector<HTMLTextAreaElement>("textarea")!;
  input.value = "bad )";
  input.dispatchEvent(new Event("input"));
  await new Promise((r) => setTimeout(r, 100));
  expect(host.querySelector(".calc-preview .calc-error")?.textContent).toBe("malformed");
});

test("Enter on a selected calc opens it", () => {
  const { host, view } = mount([cell("C", "calc", "a0", "x = 1")]);
  view.dispatch(view.state.tr.setSelection(NodeSelection.create(view.state.doc, 0)));
  // EditorView's handleKeyDown lives in Editor.svelte; call the view's open() the same way.
  (view.nodeDOM(0) as HTMLElement & { calcView?: { open(): void } }).calcView!.open();
  expect(host.querySelector("textarea")).not.toBeNull();
});

test("Temml typesets the evaluator's TeX, with theme colours and lints", async () => {
  const { renderMath } = await import("./calcView");
  const el = document.createElement("div");
  // cruncha-eval's output for `M = w L^2 / 8 = ?? kip·ft` (spike S2b report).
  const tex =
    "M = w \\dfrac{L^{2}}{8} = \\textcolor{#0000ff}{(1.5\\, {\\text{kip/ft}}) \\dfrac{(24\\, {\\text{ft}})^{2}}{8}} = 108 \; {\\text{kip}\\mkern1mu{\\cdot}\\mkern1mu\\text{ft}}";
  renderMath(el, { tex, error: null, text: "M = 108 kip·ft", warnings: ["L is used here but defined further down"] }, "src");
  expect(el.querySelector("math")).not.toBeNull();
  expect(el.querySelector(".calc-error")).toBeNull();
  expect(el.innerHTML).not.toContain("#0000ff");
  expect(el.querySelector(".calc-lint")?.getAttribute("title")).toContain("further down");
  expect(el.title).toBe("M = 108 kip·ft");
  // An evaluation error keeps the typeset line (Hurmet puts the message in it).
  renderMath(el, { tex: "y = \\textcolor{firebrick}{\\text{Error. Variable q not found.}}", error: "Error. Variable q not found.", warnings: [] }, "y = q");
  expect(el.querySelector("math")).not.toBeNull();
  expect(el.classList.contains("has-error")).toBe(true);
});

test("a description is edited with the calc, committed in its attrs and shown beside it", async () => {
  const { host } = mount([cell("C", "calc", "a0", "x = 1")]);
  host.querySelector(".calc-display")!.dispatchEvent(new MouseEvent("mousedown", { bubbles: true }));
  const desc = host.querySelector<HTMLInputElement>("input.calc-desc-input")!;
  expect(desc.value).toBe("");
  // Moving from the source to the description keeps the editor open.
  const input = host.querySelector<HTMLTextAreaElement>("textarea")!;
  input.dispatchEvent(new FocusEvent("blur", { relatedTarget: desc }));
  expect(host.querySelector("textarea")).not.toBeNull();
  desc.value = "Unit test value";
  desc.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true }));
  await flushPromises();
  expect(commit).toHaveBeenCalledTimes(1);
  expect(commit.mock.calls[0][0][0].cell.attrs).toEqual({ description: "Unit test value" });
  expect(host.querySelector(".calc-display .calc-desc")?.textContent).toBe("Unit test value");
  // Clicking the description opens the editor there.
  host.querySelector(".calc-desc")!.dispatchEvent(new MouseEvent("mousedown", { bubbles: true }));
  expect(document.activeElement?.className).toBe("calc-desc-input");
});
