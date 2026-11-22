// SPDX-License-Identifier: GPL-3.0-or-later
// Editor plugins: cell identity (ids and order keys), keymaps and input rules.
import { Plugin, TextSelection, NodeSelection, type EditorState, type Transaction, type Command } from "prosemirror-state";
import { keymap } from "prosemirror-keymap";
import { baseKeymap, chainCommands, exitCode, setBlockType, toggleMark } from "prosemirror-commands";
import { history, redo, undo } from "prosemirror-history";
import { inputRules, textblockTypeInputRule, wrappingInputRule, InputRule } from "prosemirror-inputrules";
import { liftListItem, sinkListItem, splitListItem, wrapInList } from "prosemirror-schema-list";
import { dropCursor } from "prosemirror-dropcursor";
import { gapCursor } from "prosemirror-gapcursor";
import type { Mark, MarkType, Node } from "prosemirror-model";
import { keyBetween, ulid } from "./ids";
import { safeHref, schema } from "./schema";

/**
 * Keeps every top-level block a distinct cell: a fresh id for new blocks and for copies made
 * by splitting or pasting, and order keys that increase down the document (a moved block gets
 * a key between its new neighbours). Runs after every transaction, so other code can create
 * blocks without ids.
 */
export const cellIdentity = new Plugin({
  appendTransaction(_trs, _old, state) {
    const blocks: { node: Node; pos: number }[] = [];
    state.doc.forEach((node, pos) => blocks.push({ node, pos }));
    const seen = new Set<string>();
    const fix = new Map<number, { id: string; key: string }>();
    let prev: string | null = null;
    blocks.forEach(({ node, pos }, i) => {
      let id = node.attrs.id as string | null;
      let key = node.attrs.key as string | null;
      const dupe = id == null || seen.has(id);
      if (dupe) id = ulid();
      seen.add(id!);
      if (key == null || (prev !== null && key <= prev)) {
        // The next key that is still in order bounds the new one.
        let next: string | null = null;
        for (let j = i + 1; j < blocks.length; j++) {
          const k = blocks[j].node.attrs.key as string | null;
          if (k != null && (prev === null || k > prev)) {
            next = k;
            break;
          }
        }
        key = keyBetween(prev, next);
      }
      if (dupe || key !== node.attrs.key) fix.set(pos, { id: id!, key });
      prev = key;
    });
    if (fix.size === 0) return null;
    const tr = state.tr;
    for (const [pos, attrs] of fix) {
      const node = tr.doc.nodeAt(pos)!;
      tr.setNodeMarkup(pos, undefined, { ...node.attrs, ...attrs });
    }
    return tr.setMeta("addToHistory", false);
  },
});

/** The top-level block around the selection: its index and position. */
export function currentBlock(state: EditorState): { index: number; pos: number; node: Node } {
  const sel = state.selection;
  if (sel instanceof NodeSelection && sel.$from.depth === 0) {
    return { index: sel.$from.index(0), pos: sel.from, node: sel.node };
  }
  const index = sel.$from.index(0);
  const pos = sel.$from.before(1);
  return { index, pos, node: state.doc.child(index) };
}

function isEmptyParagraph(n: Node) {
  return n.type === schema.nodes.paragraph && n.content.size === 0;
}

/** Insert `node` after the current block (or in place of an empty paragraph) and focus it. */
export function insertBlock(node: Node): Command {
  return (state, dispatch) => {
    if (!dispatch) return true;
    const { pos, node: cur } = currentBlock(state);
    let tr: Transaction;
    let at: number;
    if (isEmptyParagraph(cur)) {
      tr = state.tr.replaceWith(pos, pos + cur.nodeSize, node);
      at = pos;
    } else {
      at = pos + cur.nodeSize;
      tr = state.tr.insert(at, node);
    }
    if (node.isAtom) tr.setSelection(NodeSelection.create(tr.doc, at));
    else tr.setSelection(TextSelection.create(tr.doc, at + 1));
    dispatch(tr.scrollIntoView());
    return true;
  };
}

export const newCalc = () => schema.nodes.calc.create({ editing: true });
export const newCode = (lang = "python") => schema.nodes.code_block.create({ lang });
export const newParagraph = () => schema.nodes.paragraph.create();
export const newHeading = (level: number) => schema.nodes.heading.create({ level });

/** Move the current block up (-1) or down (+1). */
export function moveBlock(dir: -1 | 1): Command {
  return (state, dispatch) => {
    const { index, pos, node } = currentBlock(state);
    const target = index + dir;
    if (target < 0 || target >= state.doc.childCount) return false;
    if (!dispatch) return true;
    const offsetInNode = state.selection.from - pos;
    const tr = state.tr.delete(pos, pos + node.nodeSize);
    let insertAt = 0;
    for (let i = 0; i < target; i++) insertAt += tr.doc.child(i).nodeSize;
    // A moved block keeps its id; its key no longer fits, so cellIdentity re-keys it.
    tr.insert(insertAt, node.type.create({ ...node.attrs, key: null }, node.content, node.marks));
    if (node.isAtom) tr.setSelection(NodeSelection.create(tr.doc, insertAt));
    else tr.setSelection(TextSelection.create(tr.doc, Math.min(insertAt + offsetInNode, insertAt + node.nodeSize - 1)));
    dispatch(tr.scrollIntoView());
    return true;
  };
}

/** In a code block, Tab inserts spaces instead of leaving the editor. */
const codeTab: Command = (state, dispatch) => {
  if (state.selection.$from.parent.type !== schema.nodes.code_block) return false;
  dispatch?.(state.tr.insertText("    "));
  return true;
};

/** `**bold**`-style typing: when the closing delimiter is typed, the delimited text gets the
 *  mark and the delimiters go. `m[1]` is the character before the opening delimiter (kept),
 *  `m[2]` the text. */
function markRule(re: RegExp, type: MarkType, attrs?: (m: RegExpMatchArray) => Record<string, unknown> | null): InputRule {
  return new InputRule(re, (state, m, start, end) => {
    const lead = m[1] ?? "";
    const text = m[2];
    if (!text) return null;
    const a = attrs ? attrs(m) : {};
    if (a === null) return null;
    const from = start + lead.length;
    const $from = state.doc.resolve(from);
    if (!$from.parent.type.allowsMarkType(type)) return null;
    const marks: readonly Mark[] = type.create(a).addToSet($from.marks());
    const tr = state.tr.replaceWith(from, end, schema.text(text, marks));
    return tr.removeStoredMark(type);
  });
}

/** Markdown as you type. Blocks: `# ` heading, `- ` / `* ` list, `1. ` numbered list,
 *  `> ` quote, `---` rule, `` ``` `` code, `= ` calc. Marks: `**bold**`, `*italic*`,
 *  `_italic_` (not inside a word, so `f_c_` stays), `` `code` ``, `[text](https://…)`. */
export function markdownRules(): InputRule[] {
  const { nodes, marks } = schema;
  return [
    textblockTypeInputRule(/^(#{1,6})\s$/, nodes.heading, (m) => ({ level: m[1].length })),
    wrappingInputRule(/^\s*([-+*])\s$/, nodes.bullet_list),
    wrappingInputRule(
      /^(\d+)\.\s$/,
      nodes.ordered_list,
      (m) => ({ order: Number(m[1]) }),
      (m, n) => n.childCount + n.attrs.order === Number(m[1]),
    ),
    wrappingInputRule(/^\s*>\s$/, nodes.blockquote),
    // "---" in an empty top-level paragraph: a rule, then a new paragraph.
    new InputRule(/^--(-)$/, (state, _m, start, end) => {
      const $s = state.doc.resolve(start);
      if ($s.depth !== 1 || $s.parent.type !== nodes.paragraph || $s.parent.content.size !== end - start) return null;
      const pos = $s.before(1);
      const tr = state.tr.replaceWith(pos, pos + $s.parent.nodeSize, [nodes.horizontal_rule.create(), nodes.paragraph.create()]);
      return tr.setSelection(TextSelection.create(tr.doc, pos + 2));
    }),
    markRule(/(^|[^*])\*\*([^*\s](?:[^*]*[^*\s])?)\*\*$/, marks.strong),
    markRule(/(^|[^*\w])\*([^*\s](?:[^*]*[^*\s])?)\*$/, marks.em),
    markRule(/(^|[^_\w])_([^_\s](?:[^_]*[^_\s])?)_$/, marks.em),
    markRule(/(^|[^`])`([^`]+)`$/, marks.code),
    markRule(/(^|[^!])\[([^\]]+)\]\(([^)\s]+)\)$/, marks.link, (m) => {
      const href = safeHref(m[3]);
      return href ? { href } : null;
    }),
  ];
}

export function editorPlugins(onSubmit: () => void, onRunCode?: (id: string) => void) {
  const submit: Command = () => {
    onSubmit();
    return true;
  };
  /** Shift+Enter in a Python cell: save it and run it. */
  const runCode: Command = (state) => {
    const parent = state.selection.$from.parent;
    if (parent.type !== schema.nodes.code_block || parent.attrs.lang !== "python" || !onRunCode) return false;
    const id = parent.attrs.id as string | null;
    if (id) onRunCode(id);
    return true;
  };
  return [
    cellIdentity,
    inputRules({
      rules: [
        ...markdownRules(),
        textblockTypeInputRule(/^```(python|rust|tex)?\s$/, schema.nodes.code_block, (m) => ({ lang: m[1] ?? "python" })),
        // "= " at the start of an empty paragraph starts a calc.
        new InputRule(/^=\s$/, (state, _m, start, end) => {
          const $s = state.doc.resolve(start);
          if ($s.depth !== 1 || $s.parent.type !== schema.nodes.paragraph || $s.parent.content.size !== end - start - 1) return null;
          const pos = $s.before(1);
          const tr = state.tr.replaceWith(pos, pos + $s.parent.nodeSize, newCalc());
          return tr.setSelection(NodeSelection.create(tr.doc, pos));
        }),
      ],
    }),
    history(),
    keymap({
      "Mod-z": undo,
      "Mod-Shift-z": redo,
      "Mod-y": redo,
      "Mod-b": toggleMark(schema.marks.strong),
      "Mod-i": toggleMark(schema.marks.em),
      "Mod-`": toggleMark(schema.marks.code),
      "Mod-e": insertBlock(newCalc()),
      "Mod-Alt-p": insertBlock(newCode("python")),
      "Mod-Alt-0": setBlockType(schema.nodes.paragraph),
      "Mod-Alt-1": setBlockType(schema.nodes.heading, { level: 1 }),
      "Mod-Alt-2": setBlockType(schema.nodes.heading, { level: 2 }),
      "Mod-Alt-3": setBlockType(schema.nodes.heading, { level: 3 }),
      "Mod-Alt-4": setBlockType(schema.nodes.heading, { level: 4 }),
      "Mod-Alt-5": setBlockType(schema.nodes.heading, { level: 5 }),
      "Mod-Alt-6": setBlockType(schema.nodes.heading, { level: 6 }),
      "Alt-ArrowUp": moveBlock(-1),
      "Alt-ArrowDown": moveBlock(1),
      "Mod-s": submit,
      // Save now; in a code block also leave it for a new paragraph below.
      "Mod-Enter": (state, dispatch) => {
        exitCode(state, dispatch);
        onSubmit();
        return true;
      },
      "Shift-Enter": runCode,
      Enter: splitListItem(schema.nodes.list_item),
      Tab: chainCommands(codeTab, sinkListItem(schema.nodes.list_item)),
      "Shift-Tab": liftListItem(schema.nodes.list_item),
      "Mod-Shift-8": wrapInList(schema.nodes.bullet_list),
      "Mod-Shift-7": wrapInList(schema.nodes.ordered_list),
    }),
    keymap(baseKeymap),
    dropCursor(),
    gapCursor(),
  ];
}
