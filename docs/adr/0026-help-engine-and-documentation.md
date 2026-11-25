# ADR-0026: One help system: bundled Markdown pages, a help engine, inline help in the app, and a static site

**Status:** Proposed
**Date:** 2026-11-24
**Deciders:** acmeanvil

## Context

Cruncha has reference material but no help system.

**What exists today:**
- **The ƒx Functions palette** (⌘⇧F) reads `crates/cruncha-eval/reference/functions.json`: 113 built-ins, operators and constants, each with a summary and an example. `tests/reference.rs` evaluates every example.
- **The Units palette** (⌘⇧U) reads `cruncha_units::catalog`.
- **The Templates tab** (⌘⇧T) shows each template's own prose, references and worked examples.
- **`skills/`** is the agent-facing specification of the calc syntax and the MCP tools. It is written for agents and is too terse and tool-oriented for an engineer.
- **The rest is scattered:** tooltips and `title` attributes in the UI, keyboard shortcuts mentioned only in tooltips, and error texts (94 in `cruncha-eval/src/error.rs`) with no explanation or way forward. Concepts are explained nowhere a user can find them: snapshots, retention, trust, proposals, stored results, reports.

**What a user needs, by moment:**
- **Learning:** a getting-started tutorial and a tour of the document model (cells, the graph, units, history).
- **Doing:** how-to guides, such as "fill a template's inputs from my own values", "issue Rev A", "trim history", "connect an agent".
- **Looking up:** the calc language, every function, unit, setting, shortcut and error.
- **Stuck in the moment:** help on the thing under the cursor or the error just shown, without leaving the document.

**Lacewing Lab (the owner's earlier app) already solved part of this:**
- Each calculator has a Markdown help file (`help/calculators/<id>.md`), and there are eight guides (`help/guides/`).
- A generator turns them into app data, and a test fails while the two disagree.

Cruncha should keep those ideas, one source checked by tests, and add search, context and the generated references.

**Constraints:**
- **Offline:** the app works without a network, so help is bundled and versioned with the app.
- **No drift:** reference material that exists as data (functions, units, errors, settings, shortcuts, templates, MCP tools) is generated from that data, never retyped.
- **Live examples:** an example calc in the help must give the result the help shows, as `reference.rs` already checks for functions.
- **Safety:** the app's page shows no HTML from content (CLAUDE.md: worker output and links), so help pages render through the same safe path as prose.
- **Agents:** agents answering "how do I…" should read the same pages, not a second copy.

## Decision

### 1. One source: Markdown pages in `help/`, with front matter

- **The files:** `help/<section>/<id>.md`.
  - TOML front matter, as templates have (ADR-0014): `id`, `title`, `summary`, `keywords`, `related`, and `context` (the keys that open the page, §3).
  - Then the body in Markdown.
- **Four sections, by what the reader is doing** (the Diátaxis split):
  - `tutorials/`: learning;
  - `how-to/`: doing a task;
  - `reference/`: looking up;
  - `concepts/`: understanding.
- **Calc examples** are fenced `calc` blocks. A `⇒` line after one gives the expected result. The help tests evaluate every example with `cruncha-eval`.
- **Links:**
  - `help:<id>` between pages;
  - `help:fn/<name>`, `help:unit/<name>`, `help:error/<code>` and `help:setting/<key>` into the generated reference;
  - http, https and mailto only for the outside world, opened in the browser as prose links are.
- **Lacewing's guides** (`help/guides/`, the owner's own work) are a starting point where a topic carries over: entering values, files, reports, variables and clipboard. They are reworded for Cruncha.

### 2. Generated reference pages, never retyped

A build step writes these pages from the data that already defines them.

| Source | Pages |
| --- | --- |
| `cruncha-eval/reference/functions.json` | one page per function category, each function with its example |
| `cruncha_units::catalog` | units by quantity, prefixes, aliases |
| `cruncha-eval/src/error.rs` plus `help/errors.toml` | every error: what it means, the usual causes, what to do |
| The preferences and document settings (`Prefs`, the meta cell's attributes) | every setting, its default and where it lives (this computer, or the document) |
| A shortcut table (`app/src/lib/shortcuts.ts`, also used by the menus) | every keyboard shortcut |
| The built-in template library | the template catalogue, by category |
| `cruncha-mcp`'s tool list | the agent tools, for users connecting agents outside Cruncha |

- **The error text gets a code the UI can link from.** Each error's message stays as it is and gains a stable code (`BAD_ROOT`, `UNIT_COMP`, …).
- **`help/errors.toml` holds the explanations** keyed by code, and a test fails when an error has none.
- **The settings and shortcut tables** get the same guard: a test fails when one has no help.

### 3. The help engine: `crates/cruncha-help`

- **What it holds:** the pages (written and generated), a search index, and a context map.
- **Search:** a small in-memory index over titles, summaries, keywords and body text, built at startup from the bundled pages. There are a few hundred pages, so no search crate is needed, and this keeps it free of lock-in.
- **Context:** a key from the app returns the best page and a short answer:
  - `fn:hypot`, `unit:ksi`, `error:UNIT_COMP`;
  - `setting:retention`, `ui:history.archive`, `template:<id>`, `mcp:propose_template`.
- **Rendering:** a page becomes the same safe structure prose cells use (`cruncha-prose`'s tree), never HTML.
  - Live calc examples render with the engine, typeset like calcs in a document.
  - A template's help is the template itself: its prose, references and worked examples.
- **Agents:**
  - `cruncha-mcp` gains `help(query?, id?)`: search, or read a page as Markdown;
  - the `cruncha-core` skill points agents there for anything about using the app.

  Skills stay the agents' spec, and help stays the humans' guide. Where a skill and a help page describe the same syntax, a test checks that their examples agree.

### 4. Inline help in the app

- **A Help panel:**
  - **Where:** a fourth tab of the reference panel, beside ƒx Functions, Units and Templates.
  - **Shortcut:** ⌘? (F1 also works).
  - **What it has:** search, the four sections, back and forward, and "Open in browser" for the static site.
- **Help on what's under the cursor:**
  - **In a calc:** F1, or ⌘-click on a function or unit name, opens its reference in the panel.
  - **On hover:** a function name shows its signature and one-line summary.
- **Errors explain themselves:** a calc's error line gets a "Why?" link to its error page. The page has the usual causes and an example of the fix.
- **Settings and dialogs:** a "?" beside a setting or dialog heading shows the page's summary in a popover, with a link to the full page. Examples are the retention policy, trust, archive, the library folder and the agents settings.
- **Shortcuts:** ⌘/ shows the shortcut sheet, from the same table as the menus.
- **First run:** the start screen links to the getting-started tutorial. A guided tour is deferred.

### 5. The static documentation site

- **The same pages build into a static site:** HTML, search included, no server.
- **The generator:** an example in `cruncha-help` (`cargo run -p cruncha-help --example site -- <out dir>`), using pulldown-cmark as the rest of Cruncha does.
  - Calc examples are typeset to MathML or SVG with their checked results.
  - It doesn't use mdBook or Zola. Both are fine licences (MPL-2.0, MIT), but each would be a second Markdown dialect and a second renderer to keep in step with the app.
- **Publishing:** a GitHub Pages workflow for each release, so the site matches a released version. The app's "Open in browser" goes to that version's site.
- **Report and manual:** a PDF manual is possible later through the report engine (Typst, ADR-0015). It isn't part of this ADR.

### 6. Licence of the documentation

Help pages are licensed GPL-3.0-or-later, like the code and templates, so pages, examples and generated references can move freely between them. (CC BY-SA 4.0 is the usual choice for documentation and is one-way compatible with GPLv3. It is the alternative if the owner prefers it; see the Options.)

## Options Considered

### Option A: Bundled Markdown + a help engine crate + generated references + own static site (chosen)

| Dimension | Assessment |
| --- | --- |
| Complexity | Medium: one crate, a generator, a panel and context hooks in the editor |
| Cost | Writing the pages; the references come free from data |
| Scalability | A few hundred pages in memory; the templates add their own help as the library grows |
| Team familiarity | The same Markdown, front matter and test pattern as templates and Lacewing's help |

**Pros:**
- one source for app, site and agents;
- references can't drift;
- examples are checked;
- offline;
- no new renderer or dependency with lock-in.

**Cons:**
- a static site generator to maintain;
- search quality is only as good as a simple index (enough at this size).

### Option B: mdBook (or Zola) for the docs, a web view in the app

**Pros:** a mature site generator, themes, search.
**Cons:**
- the app would show the site's HTML inside its page, which breaks the rule that content never renders as HTML;
- context links, live calc examples and generated references would need custom preprocessors anyway;
- two renderers (mdBook's and the app's) disagree on the details.

Rejected for the app. Still possible for the site alone, if the own generator proves costly.

### Option C: Help only on the web (no bundled pages)

**Pros:** least work in the app.
**Cons:**
- no offline help;
- versions drift from the installed app;
- no inline help on errors and settings, which is where users need it most.

Rejected.

### Option D: Help written into the UI components (tooltips and dialog text only)

**Pros:** help sits exactly where it's needed.
**Cons:**
- not searchable;
- not available to agents or the site;
- every explanation lives in a Svelte file, untested.

It's how the app works today, and it doesn't scale. Rejected, though short tooltips stay for what fits in a line.

### Documentation licence: GPL-3.0-or-later (chosen) or CC BY-SA 4.0

- **GPL-3.0-or-later:** keeps one licence across code, templates and docs, and examples move freely.
- **CC BY-SA 4.0:** more familiar for prose, and others can reuse it more easily outside software. It can flow into GPLv3 code, but not back.

The owner decides.

## Trade-off Analysis

The decision is mostly about where the truth lives. Cruncha already has its references as data: functions, units, templates, MCP tools. Generating help from that data costs little and can't drift. Hand-written pages are needed only for concepts, tasks and tutorials, the parts that need a writer.

Rendering help through the prose renderer rather than HTML keeps the app's content-security rule intact. It also makes calc examples in help behave exactly like calcs in a document.

A separate site generator would be quicker to start. But it would split the rendering and the context features, the part that makes help useful inside the app.

## Consequences

- **The gain:** every function, unit, error, setting and shortcut has a help page, and tests keep them complete. A new error code or setting can't ship without an explanation.
- **Inline help:**
  - errors link to what to do;
  - settings explain themselves;
  - F1 works in calcs.
- **Agents** answer usage questions from the same pages the user reads.
- **The cost:**
  - error texts gain stable codes that the UI and help refer to, so changing a message no longer breaks a link;
  - the reference panel gains a fourth tab;
  - the editor gains context lookups for the name under the cursor.
- **Writing work:** the hand-written pages are real work. The first set is the tutorial, the document model, units, history and snapshots, trust and code cells, templates, reports and agents.
- **To revisit:** a guided first-run tour; a PDF manual through Typst; translations (the front matter leaves room for a `lang` key).

## Action Items

1. [ ] `crates/cruncha-help`: the page format, loading, the search index, the context map; tests that every page parses, every `help:` link resolves and every calc example gives its `⇒` result.
2. [ ] Generated references: functions, units, errors (with `help/errors.toml` and a completeness test), settings, shortcuts (one table shared with the menus), templates, MCP tools.
3. [ ] The Help panel (⌘?, F1): search, sections, back/forward; context help on the name under the cursor in a calc; "Why?" on errors; "?" popovers on settings and dialogs; the shortcut sheet (⌘/).
4. [ ] The first hand-written pages: getting started, the document model, units, history and snapshots, trust and code cells, templates, reports, agents. Draw on Lacewing's guides where a topic carries over.
5. [ ] MCP `help(query?, id?)` and a pointer in `skills/cruncha-core`. Add a test that the skills' and the help's examples of the same syntax agree.
6. [ ] The static site: `cargo run -p cruncha-help --example site`, a release workflow to GitHub Pages, and the app's "Open in browser" linking to the matching version.
7. [ ] The owner confirms the documentation licence (§6).
