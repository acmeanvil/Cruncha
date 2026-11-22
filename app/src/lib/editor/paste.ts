// SPDX-License-Identifier: GPL-3.0-or-later
// When a paste is Markdown. Plain text that looks like Markdown, or text copied from a code
// editor (VS Code adds HTML but marks it with `vscode-editor-data`), is parsed as Markdown by
// the backend. Rich text (a web page, Word, another Cruncha document) keeps ProseMirror's own
// HTML paste, which carries its structure; so does any paste into a code cell.
import type { EditorView } from "prosemirror-view";

const BLOCK = /^(#{1,6}\s|\s*[-*+]\s|\s*\d{1,9}[.)]\s|>|```|---\s*$)/m;
const INLINE = /\*\*[^*\n]+\*\*|`[^`\n]+`|\[[^\]\n]+\]\([^)\s]+\)|(^|[^\w*])\*[^*\s][^*\n]*\*|(^|\W)_[^_\s][^_\n]*_(\W|$)/;

export function looksLikeMarkdown(text: string): boolean {
  return BLOCK.test(text) || INLINE.test(text);
}

/** The Markdown to parse for this paste, or null for ProseMirror's usual paste. */
export function markdownOf(text: string, html: string, types: readonly string[], inCode: boolean): string | null {
  if (inCode || !text.trim()) return null;
  const fromCodeEditor = types.includes("vscode-editor-data");
  if (html && !fromCodeEditor) return null;
  return looksLikeMarkdown(text) ? text : null;
}

export function markdownPaste(view: EditorView, e: ClipboardEvent): string | null {
  const d = e.clipboardData;
  if (!d) return null;
  const inCode = !!view.state.selection.$from.parent.type.spec.code;
  return markdownOf(d.getData("text/plain"), d.getData("text/html"), Array.from(d.types), inCode);
}
