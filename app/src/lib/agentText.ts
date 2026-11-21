// SPDX-License-Identifier: GPL-3.0-or-later
// Agent replies as Markdown, safely: no raw HTML (markdown-it escapes it), and cell IDs the
// agent mentions become links that jump to the cell.
import MarkdownIt from "markdown-it";

const md = new MarkdownIt({ html: false, linkify: true, breaks: false });

/** Render Markdown to HTML. Raw HTML in the text is shown as text, never interpreted. */
export function renderMarkdown(text: string): string {
  return md.render(text);
}

/** Wrap every occurrence of a known cell ID in `root` (also inside code spans) in a
 *  `<button class="cellref" data-cell="…">`. */
export function linkCells(root: HTMLElement, ids: Set<string>) {
  if (ids.size === 0) return;
  // Longest first, so an ID that is a prefix of another doesn't win.
  const pattern = new RegExp(
    `#?\\b(${[...ids].sort((a, b) => b.length - a.length).map((s) => s.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")).join("|")})\\b`,
    "g",
  );
  const walker = root.ownerDocument.createTreeWalker(root, NodeFilter.SHOW_TEXT);
  const nodes: Text[] = [];
  for (let n = walker.nextNode(); n; n = walker.nextNode()) {
    if (!(n.parentElement?.closest("a, button"))) nodes.push(n as Text);
  }
  for (const node of nodes) {
    const text = node.data;
    pattern.lastIndex = 0;
    if (!pattern.test(text)) continue;
    pattern.lastIndex = 0;
    const frag = root.ownerDocument.createDocumentFragment();
    let last = 0;
    for (const m of text.matchAll(pattern)) {
      frag.append(text.slice(last, m.index));
      const b = root.ownerDocument.createElement("button");
      b.className = "cellref";
      b.dataset.cell = m[1];
      b.title = "Show this cell in the document";
      b.textContent = m[0];
      frag.append(b);
      last = (m.index ?? 0) + m[0].length;
    }
    frag.append(text.slice(last));
    node.replaceWith(frag);
  }
}
