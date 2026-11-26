# ADR-0028: Page breaks: a prose block that starts a new page in reports

**Status:** Accepted (2026-11-26)
**Date:** 2026-11-26
**Deciders:** acmeanvil

## Context

A Cruncha document is one continuous column of cells. A PDF report (ADR-0015) flows the cells onto pages by itself. Engineers want to control where a page starts:
- a new calculation on its own page;
- a sketch's page kept separate;
- an appendix.

The owner asked for a way to insert a new page (2026-11-26) and chose a page-break block over a new document or a section heading.

A prose cell is one top-level block (CLAUDE.md), stored as ProseMirror JSON: a paragraph, heading, list, quote or rule. A new block kind is an addition to the stored format. Every reader has to learn it together:
- the editor's schema;
- `cruncha-prose` (JSON and Markdown both ways);
- the report template;
- the canonical fixture.

An app without it couldn't open a document that has one.

## Decision

### 1. A `page_break` prose block

- **Stored form:** `{"type":"page_break"}`, a top-level prose cell with no content and no attributes. It is written by `cruncha_prose::to_json` as the editor writes it.
- **Like a rule:** it is an atom block. It exists only at the top level, never inside a list or a quote.
- **In Markdown:** a line holding only `\pagebreak`, the form pandoc and most Markdown-to-PDF tools accept.
  - Reading such a line, `cruncha-prose` makes a page break.
  - Writing a page break gives that line back.
  - Templates (ADR-0014) and pasted Markdown can therefore carry page breaks.
  - A paragraph whose whole text is `\pagebreak` can't be written as a paragraph. That is acceptable.

### 2. In the editor

- **How it looks:** a full-width dashed line labelled "Page break". It can be selected, moved (⌥↑/⌥↓), deleted, copied and pasted like any block.
- **How you insert one:**
  - a "Page break" button in the editor toolbar;
  - ⌘⇧↵ (Ctrl+Shift+Enter on other platforms; ⌘↵ already saves), which inserts it after the current block;
  - typing `\pagebreak` alone on a line, as a Markdown input rule, as `---` gives a rule today.
- **The outline** doesn't list page breaks. **The variable browser and agents' mirror** show them as `--- page break ---`.

### 3. In reports

- **A page break starts a new page:** Typst's `pagebreak(weak: true)`, so a break at the top of a page, or two in a row, adds no blank page.
- **The table of contents and running heads** are unaffected.

## Options Considered

### Option A: A `page_break` prose block (chosen)

| Dimension | Assessment |
| --- | --- |
| Complexity | Low: one block kind in four places that already handle `horizontal_rule` the same way |
| Cost | Small |
| Scalability | n/a |
| Team familiarity | The rule block is the model |

**Pros:** the break is visible where it sits, versioned with the document, movable like any cell, and carried by templates and Markdown.
**Cons:** a format addition; older builds can't read a document with one.

### Option B: A "new page before" attribute on any cell

**Pros:** no new block.
**Cons:**
- invisible unless the editor draws it;
- lost when the cell is deleted;
- every cell kind gains an attribute.

Rejected.

### Option C: Page breaks only in the report settings (break before each heading of level n)

**Pros:** no format change.
**Cons:** the owner wants a break where they choose, not a rule. It could be added later as a report option.

## Consequences

- **Documents:** engineers control pagination; a report puts each calculation where its author wants it.
- **Readers of the format:** `cruncha-prose`, the editor, the report and the fixture all learn one more block. The canonical fixture gains a page break, so both the Rust and TypeScript tests check it.
- **Compatibility:** documents with a page break need this version or later to open.
- **To revisit:** report options that break before headings or before each template's insert.

## Action Items

1. [x] `cruncha-prose`: `Block::PageBreak` in JSON (`page_break`) and Markdown (`\pagebreak`) both ways; the canonical fixture. *(2026-11-26; `PAGE_BREAK`, tests in `tests/prose.rs`; the fixture's page break is checked by both the Rust and the app's tests.)*
2. [x] Editor: the schema node and its view; the toolbar button, ⌘⇧↵, and the `\pagebreak` input rule; Markdown paste; copy and paste of cells; the agents' mirror line. *(2026-11-26: ⌘↵ already saves, so the shortcut is ⌘⇧↵. Paste and copy go through cruncha-prose and the cell model as for a rule; the agents' mirror shows the block's Markdown, `\pagebreak`, rather than a separate `--- page break ---` line.)*
3. [x] Report: `pagebreak(weak: true)` in the template; a test that a page break gives a new page and two in a row give one. *(2026-11-26: `tests/prose.rs`, which also checks that one at the top of a page adds none.)*
4. [x] `skills/`: the calc-authoring or core skill mentions page breaks, for agents proposing report layout. *(2026-11-26: `cruncha-core`.)*
