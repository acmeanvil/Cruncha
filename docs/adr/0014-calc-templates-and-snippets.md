# ADR-0014: Calculation templates and snippets are one system: Markdown files with a manifest, inserted as cells

**Status:** Accepted (2026-11-22)
**Date:** 2026-11-22
**Deciders:** acmeanvil
**Builds on:** ADR-0002 (Hurmet's `_` placeholders), ADR-0004 (one definition per name), ADR-0009 and ADR-0016 (agents edit through proposals)

## Context

Engineers rarely start a calculation from nothing. They start from their last one, a company standard, or a worked example in a handbook. Cruncha needs reusable pieces at two sizes:

- **Snippets:** a few cells the user saves and reuses, such as a load combination, a unit check, or a standard paragraph of assumptions.
- **Templates:** a complete calculation with inputs, working, checks and references, such as a curved-beam check, bolt shear or psychrometrics.

The design notes (2026-11-21) settled that **these are one system**. A snippet is a small template, and the user manages their own with create, update, delete, rename and duplicate.

The built-in library comes from the owner's Lacewing Lab app (`~/Developer/Tension_Meter`). Its inventory ([research 02](../research/02-lacewing-lab-inventory.md)) lists:

- **286 calculators** in ten sections: Materials, Geometry, Mechanical, Machining, Structural, Thermal, Electrical, Mathematics, Physics and a unit converter.
- **A help file for each** (inputs, results, method, sources) and an engine of about 2,300 formulas.
- **Book-checked tests.** Most help files open on a textbook worked example. For instance, curved beams uses Shigley Example 3-15: rn = 3.641 in, σi = 16.9 kpsi.

The owner also chose three gaps to fill: measurement and uncertainty, atmosphere and moist air, and chemistry beyond the periodic table.

The constraints that matter:

1. **ADR-0004: one definition per name.** Inserting a beam template twice, or into a document that already has `L`, would define names twice.
2. **Reviewability.** A template must be readable as plain text, diffable in git, and shown in full before it's inserted. Its working stays visible in the document; nothing is hidden in a black-box function.
3. **Trust.** A template's numbers must be checked, by its worked examples run in CI, and its sources cited (clause and edition, never copied text).
4. **Code cells.** Many calculators need iteration, tables or charts, so templates must carry Python and Rust cells as well as calcs and prose. Code still runs only in trusted documents.

## Decision

### 1. One file format: a Cruncha template is a Markdown file with TOML front matter

The file is `<id>.cruncha.md`. Its front matter is the manifest. Its body is the cells, in order:

- prose is ordinary Markdown;
- each calc, Python, Rust or TeX cell is a fenced block whose info string names the kind.

````markdown
+++
cruncha-template = 1
id = "mechanical.curvedBeams.rectangle"
version = "1.0.0"
title = "Curved beam, rectangular section"
summary = "Neutral-axis shift and inner/outer fibre stresses of a curved rectangular beam."
category = ["Mechanical engineering", "Beams"]
tags = ["curved beam", "crane hook", "Winkler-Bach"]
from = "lacewing:mechanical.curvedBeams"   # where it was converted from, if anywhere
license = "GPL-3.0-or-later"
inputs = ["h", "b", "r_i", "F"]
outputs = ["r_n", "e", "σ_i", "σ_o"]

[[references]]
work = "Shigley's Mechanical Engineering Design"
edition = "9th"
locator = "Sec. 3-18, Table 3-4, Example 3-15"

[[tests]]
name = "Shigley Example 3-15"
source = "Shigley 9th, Example 3-15, p. 133"
inputs = { h = "4 in", b = "0.75 in", r_i = "2 in", F = "5000 lbf" }
expect = { r_n = "3.641 in", "σ_i" = "16.9 ksi" }
tolerance = "0.5%"
+++

A curved beam's neutral axis moves toward the centre of curvature…

```calc {description="Radial depth of the section"}
h = 4 in _
```

```calc
r_n = h / ln(r_o / r_i) = ?? in
```
````

- **Snippets use the same format.** Only `id`, `version` and `title` are required; `inputs`, `outputs`, `references` and `tests` are for templates.
- **Info-string attributes** carry the cell attributes a template may set: `description` for now. Cell ids and order keys are never in the file; they're created fresh on insert.
- **Markdown for prose** means this ADR depends on the editor's Markdown support (the next planned item). The editor's ProseMirror JSON stays the storage inside documents; Markdown is only the exchange format here. The same converter later gives "export a document as Markdown".
- **A new crate, `cruncha-templates`**, parses and validates the format, renders a preview, and runs a template's tests. Dependencies: `toml` and `pulldown-cmark`, both MIT/Apache-2.0.

### 2. Inputs are placeholders; outputs are what the template promises

- **An input** is a name the template defines with a `_` placeholder value (`h = 4 in _`, Hurmet's convention). It's shown highlighted until the user types a value.
- **An output** is a name the template defines and documents.
- The manifest lists both, so the insert dialog, the agent tools and the tests know the template's interface without evaluating it.

### 3. Inserting resolves name clashes before any cell is created

The insert dialog shows the template's preview, its inputs and outputs, and every name it defines. For each name the document already defines, the user chooses:

- **Rename** (the default for outputs and intermediate names). The dialog suggests a suffix: `M` becomes `M_2`, and `σ_i` becomes `σ_i2`. The rename is applied by the calc lexer to identifiers only, never to text or units, so `L` is renamed but `'L'` (litres) isn't.
- **Use the document's** (offered only for inputs). The template's definition cell for that input is left out, and its calcs read the document's existing value. Example: a beam template's `L` binds to the document's `L_span`.

Code cells:

- Renames in **Rust cells** use the existing `syn` scanner.
- Renames in **Python cells** use the worker's `ast` scan when Python is set up.
- Without a scanner, a Python template whose names clash can only be inserted with "use the document's", or after the clashing names are renamed in the document.

Then:

- **One insert is one commit.** Its message is `Insert template <title> <version>`, and it is undone like any other edit (`Store::restore`).
- **Provenance:** each inserted cell records `Provenance::Imported { source: "template:<id>@<version>" }`. The type already exists, so the container format doesn't change. The history and the review tools can tell what came from which template version.
- **Placement:** cells go after the current block; a template inserted into an empty document becomes its body.
- **Code cells arrive not run.** The normal trust rule applies: an edited, or inserted, Python or Rust cell runs only on an explicit Run in a trusted document.

### 4. Two libraries: built-in (read-only) and the user's own (files)

- **The built-in library** lives in the repository at `templates/<section>/…/<id>.cruncha.md` and is bundled with the app as a resource, like `skills/`. It's versioned with Cruncha.
- **The user's library** is a folder of the same files.
  - The default is `<app data>/library/`. Settings can point it elsewhere, for example to a git repository or a shared drive for a team.
  - Files edited outside Cruncha are picked up the next time the library is opened.
- **No document-local library in this ADR.** A document that needs its own snippets can keep them as ordinary cells.

### 5. The snippet manager

The manager is the Templates tab of the reference panel (beside ƒx Functions and Units; ⌘⇧T) plus a Library page.

- **Browse:** search, a category dropdown like the Units tab, the built-in and user libraries, a preview of the cells, inputs and outputs, references, and test status.
- **Insert:** the dialog in §3.
- **Create:** select blocks in the editor and choose "Save as snippet…". This asks for a title and category and writes a new file. Placeholders (`_`) in the selection become the snippet's inputs.
- **Update:** replace a snippet's cells with the current selection, or edit its title, summary, category and tags in a form. Each update bumps its `version`.
- **Rename:** changes the title, and the file name follows. The `id` stays, so provenance in existing documents still points to it.
- **Duplicate:** copies a snippet to a new id. Duplicating a built-in template into the user's library is how one customises it.
- **Delete:** moves the file to `library/.trash/`, which can be undone until the trash is emptied. Built-in templates can't be deleted, only hidden.
- **Import and export:** copy `.cruncha.md` files in or out, or reveal the folder in Finder.

### 6. Tests and sources: a template is only as good as its checks

- **CI checks every built-in template** (`cargo test -p cruncha-templates`):
  - it parses;
  - its inputs and outputs exist;
  - every calc evaluates without error at its default inputs;
  - every `[[tests]]` case gives its expected values within `tolerance`.
- **Templates with Python cells** are tested where the Python environment exists (as `cruncha-kernel-py`'s tests are) and skipped otherwise.
- **A converted template carries at least one worked example**, taken from the Lacewing help file or engine tests. It cites the source (work, edition, example or clause) in `references` and `tests.source`.
- **Copyright:** formulas, worked-example numbers and citations are facts and go in. Text, figures and tables from copyrighted standards and books never go into the repository (the Engineering Info library is consulted, not copied). Lookup tables (materials, bolt grades, section properties) are data with their own provenance. A table that can't be shown to be free to redistribute stays out, and the template says where to look it up.
- **Licence of the Lacewing material:** the help text and test values are the owner's own work. Moving them into Cruncha licenses them under GPL-3.0-or-later; the owner confirmed this on 2026-11-22: Tension_Meter's licence will be aligned with Cruncha's.

### 7. Agents

- **cruncha-mcp gains `templates`:** search, and read a template's file and manifest.
- **It gains `propose_template(id, bindings, renames)`:** the §3 insert, as a proposal (ADR-0016) the user accepts or rejects.
- **A new skill, `skills/cruncha-templates`,** documents the format so agents can write snippets and templates too. Saving to the user's library is a user action, never an agent's.

### 8. Converting Lacewing Lab: a pilot, then section by section

1. **Pilot: about 12 templates** that between them exercise every shape:
   - a plain algebraic check (bolt shear);
   - a multi-mode calculator (a `ModeCalculatorScreen`, e.g. kinematics) as one template per mode;
   - a section-property template with several shapes;
   - an iterative solution (Colebrook friction factor, in Python);
   - a chart (a beam's shear and moment, Plotly);
   - a table lookup (a material from the library);
   - a code-check sequence (AISC 360 flexure);
   - one template from each gap topic (GUM uncertainty propagation, psychrometrics, combustion stoichiometry).

   The pilot settles the format before 280 more files are written in it.
2. **Then section by section, in the order the owner picks.** Each section is one change, with its tests. Agents can do the mechanical part, from the inventory, the help files and the engine; each converted template gets the owner's review.
3. **The inventory is the checklist:** [research 02](../research/02-lacewing-lab-inventory.md) gains a status column, so no calculator is missed.

## Options Considered

### Option A: Markdown with TOML front matter (chosen)

| Dimension | Assessment |
| --- | --- |
| Complexity | Medium: a parser, the Markdown ↔ ProseMirror converter (shared with Markdown support), a test runner |
| Cost | Two small, permissively licensed crates (`toml`, `pulldown-cmark`) |
| Scalability | Hundreds of files, diffable and reviewable in git; agents write it easily |
| Team familiarity | Markdown and fenced code are familiar to engineers and agents alike |

**Pros:** readable without Cruncha; the same shape as the document; tests and references sit beside the cells they check.
**Cons:** two representations of prose (Markdown in files, ProseMirror in documents), so the converter must round-trip what templates use.

### Option B: JSON files

**Pros:** trivial to parse; carries ProseMirror JSON exactly.
**Cons:** unreadable prose, noisy diffs, and hard to write by hand or review. Rejected.

### Option C: Templates as `.cruncha` documents

**Pros:** reuses everything, including history.
**Cons:** a Delta container per snippet is heavy, binary to git, and its history is irrelevant to a template. Rejected for the library. "Save this document as a template" exports to Option A instead.

### Option D: Templates as user-defined functions or modules (`function beam(w, L) … end`)

**Pros:** no name clashes; compact.
**Cons:** hides the working, which is the opposite of a reviewable calculation; can't carry prose, charts or code cells. It complements templates (a template can define a function) but doesn't replace them.

### Name clashes: rename on insert (chosen) vs. scoped sections

Giving each inserted template its own namespace (`beam1.M`) would avoid renames altogether. But it changes the semantics of ADR-0004 and of every name lookup (calcs, Python, Rust, agents, reports). If renaming proves clumsy in use, that can be a later ADR. The owner chose renaming on insert, and the Markdown + TOML format, on 2026-11-22.

## Trade-off Analysis

- **Text files cost a converter but buy reviewability.** Every template can be read, diffed and checked outside the app, which matters more for a library of 300 engineering calculations than parsing convenience. The converter is needed for Markdown support anyway.
- **Renaming on insert keeps the document model unchanged,** at the cost of an insert dialog that has to be good at suggesting names and showing clashes.
- **Tests from worked examples** make the library trustworthy without a separate verification effort: Lacewing already has the examples.

## Consequences

- Cruncha gets a library of checked calculations, and users get their own, managed as plain files they can share or version.
- Markdown support in the editor becomes a prerequisite, and it's next anyway.
- A new crate (`cruncha-templates`), two new MCP tools, a skill, a Templates tab, a Library page and an insert dialog.
- Every built-in template is tested in CI. A failing worked example blocks the change, as a failing golden test does.
- **To revisit:**
  - scoped namespaces, if renaming is clumsy;
  - "template updated" notices for cells inserted from an older version;
  - a document-local library;
  - a first-class uncertain-value type (the uncertainty gap may want one).

## Action Items

1. [x] Markdown support in the editor, with a Markdown ↔ ProseMirror converter that round-trips headings, paragraphs, emphasis, code, lists, links and block quotes. Done 2026-11-22: `crates/cruncha-prose` (pulldown-cmark) for agents, the mirror, reports, paste and templates; Markdown input rules and list keys in the editor.
2. [ ] `cruncha-templates`: the format (§1), validation, preview, the test runner (§6); `cargo test -p cruncha-templates` in CI.
3. [ ] The insert flow (§3): the dialog, clash detection, renames by the calc lexer and the code-cell scanners, "use the document's", one commit, provenance.
4. [ ] The user library (§4) and the snippet manager (§5): Templates tab (⌘⇧T), Library page, Save as snippet, Update, Rename, Duplicate, Delete to trash, Import/Export, a library folder setting.
5. [ ] MCP `templates` and `propose_template`; `skills/cruncha-templates`.
6. [ ] The pilot (§8), about 12 templates, then a review of the format with the owner.
7. [ ] Convert Lacewing Lab section by section, tracked in research 02. Add the gap templates (uncertainty, atmosphere and moist air, chemistry).
8. [x] The owner confirms the GPL-3.0-or-later licensing of the Lacewing help text and test values (§6). Confirmed 2026-11-22; Tension_Meter's licence will be aligned with Cruncha's.
