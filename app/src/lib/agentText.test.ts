// SPDX-License-Identifier: GPL-3.0-or-later
// @vitest-environment jsdom
import { expect, test } from "vitest";
import { linkCells, renderMarkdown } from "./agentText";

test("raw HTML from an agent is shown as text, never run", () => {
  const html = renderMarkdown('Hi <img src=x onerror="alert(1)"> **bold**');
  expect(html).not.toContain("<img");
  expect(html).toContain("&lt;img");
  expect(html).toContain("<strong>bold</strong>");
  expect(renderMarkdown("[x](javascript:alert(1))")).not.toContain('href="javascript');
});

test("cell IDs become links, also inside code spans", () => {
  const div = document.createElement("div");
  div.innerHTML = renderMarkdown("The moment in cell `M1` and #L2 feed T3; ML2 is not a cell.");
  linkCells(div, new Set(["M1", "L2", "T3"]));
  const refs = [...div.querySelectorAll("button.cellref")].map((b) => (b as HTMLElement).dataset.cell);
  expect(refs).toEqual(["M1", "L2", "T3"]);
});
