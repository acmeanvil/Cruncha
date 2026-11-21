# ADR-0015: Reports are typeset by an embedded Typst, from the evaluated document

**Status:** Proposed
**Date:** 2026-11-20
**Deciders:** acmeanvil

## Context

Cruncha is a calculation *and report* tool: a calc package has to leave the app as a PDF a checker can sign. Design 02 §4 chose Typst for this, and spike S6 tested it ([write-up](../spikes/S6-reports.md)). Three things needed deciding:

- how the engine's math gets into the report;
- where the report's own data (title block, revisions, house style) lives;
- what the report says about results that weren't computed.

Open CalcSheet Studio (LGPL-3.0) was a visual reference for the layout (title block, results with substituted values, a check summary, continuous sheet numbers). No code or format was taken from it.

## Decision

1. **Typst 0.15, embedded as a library** (`crates/cruncha-report`, Apache-2.0). It runs with an in-memory `World`:
   - one built-in template, `templates/calc-package.typ`;
   - the report as `/report.json`, plus its images;
   - Typst's bundled fonts and the system's;
   - no file system, network or packages.

   "Today" is the version's commit date and the PDF timestamp and id come from the document, so **the same version gives byte-identical PDFs**.
2. **Calc math is converted from the engine's TeX** (`cruncha_report::tex::to_typst`), not re-generated from the AST and not run through MiTeX. The converter handles exactly the commands Cruncha's Hurmet port emits; any other command is an error. A test converts and compiles every TeX string from Hurmet's golden tests (371/371).
3. **The report reads results; it never evaluates.** A calc or Python cell without a usable result is printed **NOT CALCULATED** with the reason (Python not set up, not trusted, edited and not run, an input missing, still running). It is never printed as a pass. The cover's summary lists every `@test` (OK / NOT OK / ERROR / NOT CALCULATED) with its sheet number. `cruncha_eval::Outcome::check` carries a test's pass/fail, so nothing parses display text.
4. **Where the data lives:**

   | Data | Where it lives | Versioned |
   | --- | --- | --- |
   | Title block and report options | the document's `meta` cell, `attrs.report` | yes |
   | Revisions | the document's snapshots | yes |
   | House style (company, logo, accent, paper) | Preferences, shared by every document on the computer | no |

   The editor does not show or edit the `meta` cell.
5. **Export waits for Python** (up to two minutes) for cells that are due to run, then typesets whatever is there.

## Options Considered

### Option A: embedded Typst (chosen)
| Dimension | Assessment |
| --- | --- |
| Complexity | Medium: a `World`, a template, a TeX converter |
| Cost | Typst crates add to the build. A first render is about 0.3 s; after an edit, about 30 ms |
| Scalability | Incremental (comemo); deterministic output |
| Team familiarity | New; the template language is small |

**Pros:**
- Real pagination, headers and footers, contents, cross-references ("sheet 3");
- good math layout;
- no TeX install;
- one engine on every platform.

**Cons:**
- The template is Typst code we own.
- Math has to be converted.
- Typst's API changes between minor versions (`FileId` changed in 0.15).

### Option B: HTML and the webview's print to PDF
**Pros:** reuses the screen rendering (Temml MathML). **Cons:**
- Pagination and page numbers are the browser's.
- No cross-references.
- The output differs per platform.
- No headless export (CLI, CI, agents).
- Open CalcSheet Studio documents this limit: "report straight to PDF is Windows only".

### Option C: LaTeX
**Pros:** the engine already emits TeX. **Cons:**
- It needs a TeX distribution (gigabytes) on every machine.
- It's slow.
- Its errors are hard to show to users.

### Option D: Typst math through MiTeX
**Pros:** a general LaTeX converter. **Cons:**
- Its output calls functions from MiTeX's Typst package, so the package would have to be vendored and kept in step with the crate.
- Unknown commands degrade silently instead of failing.

## Trade-off Analysis

The report is a deliverable, so determinism and honesty matter more than reusing screen rendering:
- the same version should give the same PDF;
- the report should never say "OK" for something it didn't compute.

That points to a real typesetter fed with data, not a print of the screen. Converting the engine's TeX keeps one source of truth for what a calc displays: the screen and the report can't diverge without the converter failing loudly.

## Consequences

- **Easier:**
  - PDFs from the CLI and from agents later, with the same code;
  - templates per office or per kind of calc (the subject of ADR-0014);
  - page previews (`render_png`).
- **Harder:**
  - every new TeX construct in the engine needs a converter rule (the corpus test catches it);
  - Typst upgrades need care.
- **Revisit:**
  - fonts: system fonts make output depend on the machine. Bundle an open sans-serif font if reports must match across computers.
  - Python results for past versions (the `results/` table) so any snapshot can be re-printed;
  - per-document house-style overrides.

## Action Items

1. [x] `crates/cruncha-report`: model, `World`, template, TeX converter, corpus test, demo.
2. [x] App: Export report (PDF), Report settings (title block in the meta cell), house style in Preferences.
3. [ ] Bundle a report font (OFL) for machine-independent output.
4. [ ] User templates (ADR-0014) and a report preview in the app.
5. [ ] Print past versions and snapshots once results are stored (`results/` table).
