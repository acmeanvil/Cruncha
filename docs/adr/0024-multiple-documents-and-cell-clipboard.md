# ADR-0024: Several open documents in tabs, and cells that survive copy and paste

**Status:** Accepted (2026-11-22)
**Date:** 2026-11-22
**Deciders:** acmeanvil
**Refines:** ADR-0008 (the app shell holds one document), ADR-0009 (agents act on "the open document"). Related: ADR-0014 (snippets, the planned way to reuse code).

## Context

The owner wants several documents open at once, in tabs. One of the main reasons is copying calcs and code between documents to reuse them.

Two things stand in the way:

1. **The app holds one document.** `Session` has one `OpenDoc`, which bundles:
   - its `Store`, the single writer and its lock;
   - its engine and results;
   - two Python workers;
   - its proposals.

   The web UI has one editor. The MCP server and the in-app agent panel act on "the open document".
2. **Cells don't survive the clipboard, even within one document** (checked 2026-11-22). Calc and code nodes have no `parseDOM` rule, so ProseMirror's HTML copy pastes them back as paragraphs:
   - a calc becomes a paragraph of its source text, and its description is lost;
   - a Python cell becomes a code-formatted paragraph.

## Decision

### 1. Cells survive copy and paste (build first: useful with or without tabs)

**Copy.**
- Calc, code and raw nodes serialize to HTML with `data-cruncha-*` attributes:
  - the kind and language;
  - the source;
  - the cell attributes JSON, which carries the description.
- The plain-text copy becomes the document's Markdown mirror form, the format agents already read, so cells pasted into a text editor or chat stay readable.

**Paste into a Cruncha document.**
- Each pasted cell **gets a new ID and order key**. It is a new cell; the source document is not linked.
- Its provenance becomes `Imported { source: "document:<id>@v<version>" }`. The copy carries the source document's ID and version.
- **Name clashes:** if a pasted calc or code cell defines a name the target already defines, the paste opens ADR-0014's clash dialog. For each clash you either rename (`M` → `M_2`, applied to the pasted cells only) or use the document's existing definition (the pasted definition is dropped).
  - Until ADR-0014's insert flow exists, the paste goes through, and the engine's existing "defined twice" error marks the clash.
- A paste is one commit, like any edit.

**Copying part of a block into an existing block.** A selection doesn't have to be whole cells. Pasting part of a block edits the block it lands in; it doesn't make a new cell.

| Copied | Pasted into | Result |
| --- | --- | --- |
| Part of a paragraph, heading, list or quote (text with its bold, italic, links, …) | Prose | Inserted at the cursor, marks kept (ProseMirror's own slice paste) |
| The same | A code cell, or a calc being edited | The plain text, inserted at the cursor |
| Part of a code cell's source (selected in its editor) | Another code cell | The text, inserted at the cursor |
| The same | Prose | Inline code (`code` mark), split into lines with hard breaks |
| Part of a calc's source (selected in the calc editor) | A calc being edited | The text, inserted at the cursor |
| The same | Prose | Plain text |
| A selection that starts or ends inside a block and spans others | Prose | The partial first and last blocks merge into the blocks around the cursor (ProseMirror's open slices); the whole cells between them are pasted as cells (with new IDs, as above) |
| Whole calc or code cells | Inside a prose block | The block splits at the cursor and the cells go between the halves (calc and code cells are always top-level) |

- An edited block keeps its ID and history. The paste is an ordinary edit, so the block's provenance doesn't change.
- Names are rechecked at commit as for any edit. A calc edited into a clash shows the "defined twice" error. The clash dialog is only for whole pasted cells.
- When the target is a code cell or the calc editor, only the plain text is pasted. Marks and cell structure never enter source code.

### 2. Several documents in tabs, in one window

**Backend.**
- `Session` holds several `OpenDoc`s, keyed by document ID, each with its own `Store`.
- The app process is still the single writer of each document it has open, and each container's `.lock` still applies.
- Every document command takes a document ID; the UI passes the active tab's.
- **A document can be open in only one tab.** Opening it again switches to its tab.

**UI.**
- A tab bar shows the document name, an unsaved-sync dot while a commit is pending, and a close button.
- Keys: ⌘T opens a document, ⌘W closes the tab, ⌘⇧[ and ⌘⇧] switch tabs.
- Each tab keeps its editor state, scroll position and selection.
- The sidebar, the bottom panel and the History tab show the active tab's document.
- Closing a tab runs the end-of-session pruning (ADR-0011) for that document.
- The tabs reopen at the next launch.

**Python and Rust.**
- Each document has its own workers, as now, so a document's code can't see another's variables.
- Workers start when a tab first needs them.
- A background tab's workers stop after 10 minutes without a run, to cap memory. Its results stay, from the stored results (ADR-0018), and the workers restart on the next run.
- The trust rule is per document, as now.

**Agents see every open document and work between them.**
- **`documents`** lists the open documents: ID, name, path, latest version, which tab is active, and whether its code may run (trust).
- **A `document` argument** (ID or name, as `documents` shows it) is accepted by every tool that reads or changes one: `read_doc`, `run`, `explain`, `check`, `history`, `propose_edit`, `proposals` and `withdraw`.
  - Without the argument, a panel conversation acts on the tab it started in, and the external server acts on the active tab.
  - A name matching more than one open document is refused with the matches listed, so the agent names the one it means.
- **Copying between documents: `propose_copy`.** It takes `from: {document, cell_ids}`, a target `document` and `after`, the cell to insert after.
  - It reads the cells in the source and records **one proposal in the target**.
  - The proposed cells have new IDs and `Imported { source: "document:<id>@v<version>" }` provenance, as a human paste would.
  - Its "if accepted" results come from the target's engine, so clashes and errors are reported before anyone accepts.
  - A name clash is refused with the clashing names, as `propose_edit` refuses one. The agent can retry with `rename: {"M": "M_2"}`, which renames those names in the copied cells only (by the calc lexer and the code-cell scanners, as ADR-0014's insert does).
  - Requests like "copy the Python cell from the beam check to the column check" become: `documents`, then `read_doc` on the source to find the cell, then `propose_copy`.
- **Every change stays a proposal** in the document it changes, accepted or rejected by a human in that document's tab (ADR-0016). An agent never writes directly, in any document.
  - A proposal for a background tab marks that tab with a badge.
  - The agent's reply says which document each proposal is in.
- **Agents work only on documents the user has opened.** They can't open, create or close documents. That may come later, with a permission prompt.
- **A panel conversation belongs to the tab it started in.** That is its default document, and the panel shows the active tab's conversations, though a conversation can read and propose to any open document. Its MCP server (one per agent, ADR-0009) serves every open document.
- **The external server stays one server**, and the connection file that looking_glass reads doesn't change. I'll tell the looking_glass session about `documents`, the `document` argument and `propose_copy`.

### 3. Not now

- **Tear-off windows** (one document per window) are left out (owner, 2026-11-22: tabs only). A second document view can be had by running **a second copy of Cruncha**:
  - Each copy is its own process and writer.
  - A document open in one copy can't be opened in the other: the container's `.lock` refuses it ("container is open for writing by another process").
  - Copying cells between the two works through the system clipboard: the `data-cruncha-*` HTML from §1.
  - Each copy has its own in-app agents and their tools. Only one copy's external agent server should run at a time: the connection file names the server started last, and a copy removes the file on stop only if the file is still its own.
- **Linked cells** (a pasted cell that follows its source) are not planned: copies are independent. ADR-0014 templates are the way to reuse with a recorded origin.

## Owner's decisions (2026-11-22)

1. Tabs in one window only; a second copy of Cruncha covers side-by-side work (§3).
2. A background tab's workers stop after 10 minutes idle.
3. Agents see every open document and work between them, always through proposals (§2, "Agents").
4. A pasted cell is an independent copy; nothing links it to its source.

## Options Considered

### A. Tabs in one window with per-document state (chosen)

| Dimension | Assessment |
| --- | --- |
| Complexity | Medium: `Session` becomes a map, and commands take a document ID; the editor gets per-tab state |
| Memory | Each open document holds its engine; workers are on demand and stop when idle |
| Copy and paste | The same webview, so the clipboard, including the HTML cells, works directly |

### B. One window per document

Native multi-window is a familiar macOS pattern, and each window stays simple. But copying between windows relies on the system clipboard's HTML only, agents and the sidebar need a notion of the focused window, and tabs were asked for. Kept as a later addition.

### C. One app process per document

Isolation is total, but every process loads the whole app and its own agent server, and there would be several connection files. Rejected.

## Consequences

- **Copying cells works within a document too**, which it doesn't today.
- **Every Tauri command and session method gains a document ID.** `api.ts` and `lib.rs` change together, with backend tests per document.
- **Agents see every open document.** They read any of them and propose to any of them, including copies between them (`propose_copy`), but only as proposals, and only in documents the user opened. Tools default to the conversation's tab (panel) or the active tab (external server). `skills/cruncha-core` says how to name a document and copy between them.
- **Copying part of a block is an ordinary edit** of the block it lands in. Only whole pasted cells get new IDs, `Imported` provenance and the clash dialog.
- **Memory grows with open documents.** Idle background workers are stopped.

## Action Items

1. [x] Copy and paste of cells: `data-cruncha-*` serialization and parsing, new IDs and order keys, `Imported` provenance, Markdown plain text; partial blocks per the table in §1 (into prose, into code cells and the calc editor, selections spanning blocks, cells pasted inside a block); tests for each row, within a document and between two editors. Done 2026-11-22: `app/src/lib/editor/clipboard.ts`, schema `parseDOM` rules, `clipboard.test.ts` (seven cases through ProseMirror's own copy and paste).
2. [x] `Session` with several documents, and document IDs on commands and events; backend tests with two documents open (separate stores, engines, workers, proposals). Done 2026-11-22: `Session` holds an `OpenDoc` per tab with `select`/`activate`; every document command takes `doc`; background results flush to their own document; `several_documents_stay_separate`.
3. [x] Tab bar, per-tab editor state, keys, reopen at launch; the sidebar and panels follow the active tab. Done 2026-11-22: tab bar, per-tab editor state (kept `EditorState` with undo history, the version shown, scroll), ⌘T, ⌘⇧[ ], ⌘W via a macOS menu (Close Window moves to ⇧⌘W), tabs reopened at launch (`Prefs.open_tabs`).
4. [ ] Workers per document, started on demand and stopped when idle.
5. [ ] Agents: `documents`; the `document` argument on every tool, with ambiguous names refused; `propose_copy` (with `rename`), "if accepted" from the target's engine; proposals badge background tabs; panel conversations per tab, acting on any open document; skills updated; tell the looking_glass session.
6. [ ] Name clashes on paste through ADR-0014's clash dialog (with ADR-0014 item 3).
