// SPDX-License-Identifier: GPL-3.0-or-later
// @vitest-environment jsdom
// The Python cell's view: state, Run/Stop, and output built from structured data only.
import { expect, test, vi } from "vitest";
import { EditorState } from "prosemirror-state";
import { EditorView } from "prosemirror-view";
import type { Cell, CellResult } from "../api";

vi.mock("../api", () => ({ api: { previewCalc: async () => ({ tex: "", error: null }) } }));

const { docFromCells } = await import("./cells");
const { editorPlugins } = await import("./plugins");
const { CodeView, storedLabel } = await import("./codeView");
const { Results } = await import("./results");

const cell: Cell = {
  id: "PY", kind: "python", order_key: "a0", parent_id: null, source: "x = 1", attrs: {}, defines: ["x"], reads: [],
  provenance: { state: "human" }, modified_by: "", modified_at_us: 0,
};

function mount(result: CellResult, onRun = vi.fn(), onRunKey = vi.fn()) {
  const host = document.createElement("div");
  document.body.replaceChildren(host);
  const results = new Results();
  const stop = vi.fn();
  const view = new EditorView(host, {
    state: EditorState.create({ doc: docFromCells([cell]), plugins: editorPlugins(() => {}, onRunKey) }),
    nodeViews: { code_block: (n, v) => new CodeView(n, v, results, { run: onRun, stop }) },
  });
  results.set({ PY: result }, new Map([["PY", "x = 1"]]));
  return { host, view, stop };
}

const base = { tex: "", error: null, defines: ["x"], reads: [], values: [] };
const py = (p: Partial<NonNullable<CellResult["python"]>>) => ({
  state: "done" as const, note: null, ms: 12, stdout: "", stderr: "", outputs: [], error: null, ...p,
});

test("a finished cell shows its figure, table, printed text and values", () => {
  const { host } = mount({
    ...base,
    values: [{ name: "x", tex: "", kind: "python", summary: "ndarray float64 (50,)" }],
    python: py({
      stdout: "<b>not html</b>\n",
      outputs: [
        { kind: "image", src: "data:image/png;base64,AAAA", width: 864, height: 345 },
        { kind: "table", columns: ["a", "b"], rows: [["1", null]], totalRows: 100 },
      ],
    }),
  });
  expect(host.querySelector(".code-state")?.textContent).toBe("done · 12 ms");
  expect(host.querySelector<HTMLImageElement>(".code-figure")?.src).toBe("data:image/png;base64,AAAA");
  expect(host.querySelectorAll(".code-table td").length).toBe(2);
  expect(host.querySelector(".code-more")?.textContent).toBe("1 of 100 rows");
  // Printed text is text, never markup.
  expect(host.querySelector(".code-stdout")?.textContent).toBe("<b>not html</b>\n");
  expect(host.querySelector(".code-out b:not(.code-def b)")).toBeNull();
  expect(host.querySelector(".code-def")?.textContent).toBe("x = ndarray float64 (50,)");
});

test("errors show the type, message, line and traceback", () => {
  const { host } = mount({
    ...base,
    error: "ZeroDivisionError: division by zero",
    python: py({ state: "error", error: { type: "ZeroDivisionError", message: "division by zero", line: 2, traceback: "  line 2: b = a / 0\nZeroDivisionError: division by zero" } }),
  });
  expect(host.querySelector(".code-error")?.textContent).toContain("ZeroDivisionError: division by zero (line 2)");
  expect(host.querySelector(".code-tb")?.textContent).toContain("line 2: b = a / 0");
  expect(host.querySelector(".code-wrap")?.getAttribute("data-state")).toBe("error");
});

test("Run and Stop", () => {
  const run = vi.fn();
  const { host, stop } = mount({ ...base, python: py({ state: "edited" }) }, run);
  const button = host.querySelector<HTMLButtonElement>(".code-run")!;
  expect(button.textContent).toBe("▶ Run");
  button.click();
  expect(run).toHaveBeenCalledWith("PY");
  const r = mount({ ...base, python: py({ state: "running" }) }, run);
  r.host.querySelector<HTMLButtonElement>(".code-run")!.click();
  expect(r.stop).toHaveBeenCalled();
  expect(stop).not.toHaveBeenCalled();
});

test("Shift+Enter in a Python cell runs it", () => {
  const onKey = vi.fn();
  const { view } = mount({ ...base, python: py({}) }, vi.fn(), onKey);
  view.focus();
  view.dom.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", shiftKey: true, bubbles: true }));
  expect(onKey).toHaveBeenCalledWith("PY");
});

test("the stored results label says when a result wasn't reproduced on this computer", () => {
    const l = storedLabel({ by: "user:pat@office-mac", atUs: Date.UTC(2026, 10, 1) * 1000, env: "env-1", elsewhere: true, otherEnv: false });
    expect(l.text).toMatch(/^computed on office-mac, .* · not run on this computer$/);
    expect(l.title).toContain("pat");
    expect(storedLabel({ by: "user:me@here", atUs: 0, env: "", elsewhere: false, otherEnv: false }).text).toMatch(/^stored /);
  expect(storedLabel({ by: "user:me@here", atUs: 0, env: "env-old", elsewhere: false, otherEnv: true }).text).toMatch(
    /with a different Python environment \(env-old\) · not run here$/,
  );
});
