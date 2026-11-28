// SPDX-License-Identifier: GPL-3.0-or-later
// The owner's RangeError (2026-11-27): inserting a template while a gap cursor sat after the
// last cell (a table row's calcs, which are atoms) asked for a block past the end.
import { expect, test } from "vitest";
import { EditorState } from "prosemirror-state";
import { GapCursor } from "prosemirror-gapcursor";
import { schema } from "./schema";
import { currentBlock, insertBlock, newParagraph } from "./plugins";

const calc = (s: string) => schema.nodes.calc.create({}, s ? schema.text(s) : undefined);

function at(doc: ReturnType<typeof schema.node>, pos: number) {
  return EditorState.create({ doc, selection: new GapCursor(doc.resolve(pos)) });
}

test("a gap cursor after the last block or between blocks names the block before it", () => {
  const doc = schema.node("doc", null, [calc("a = 1"), calc("b = 2"), calc("c = 3")]);
  const sizes = [0, 1, 2].map((i) => doc.child(i).nodeSize);
  const end = at(doc, doc.content.size);
  expect(currentBlock(end).index).toBe(2);
  expect(currentBlock(end).pos).toBe(sizes[0] + sizes[1]);
  const between = at(doc, sizes[0]);
  expect(currentBlock(between).index).toBe(0);
  expect(currentBlock(between).pos).toBe(0);

  // Inserting there puts the new block at the cursor.
  let state = end;
  insertBlock(newParagraph())(state, (tr) => (state = state.apply(tr)));
  expect(state.doc.childCount).toBe(4);
  expect(state.doc.lastChild!.type).toBe(schema.nodes.paragraph);
});
