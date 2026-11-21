# Spike S6: Typst reports

*Date: 2026-11-20. Code: `crates/cruncha-report` (model, `World`, template `templates/calc-package.typ`, TeX converter `src/tex.rs`, `tests/tex_corpus.rs`, `examples/report_demo.rs`), `app/src-tauri/src/report.rs`. Machine: Intel i7-1068NG7 MacBook, macOS 26, rustc 1.99.1, Typst 0.15.1.*

Design 02 asked: can an evaluated document become a calc package of report quality (blue echo, title block, sign-off, revisions) through Typst? Open CalcSheet Studio (open-aec.com, LGPL-3.0) was a visual reference. It shows each result as name = formula = substituted values = result with units, and runs a cover, contents and sheet numbers through everything. Its summary of checks prints anything without an answer as "NOT CHECKED", never as "OK". No code was taken from it.

## Method

- **Report model** (`model.rs`). The app builds it from the latest version and its results:
  - title block, revisions, house style and options;
  - one block per cell: heading, paragraph, calc (Typst math, description, status) or code (source, outputs, defines);
  - provenance.

  A status is one of OK, Pass, Fail, Error or NotCalculated (with its reason).
- **Template** (`calc-package.typ`) reads `/report.json`. It lays out:
  - a header band: company or logo, title and project, project number, "Sheet n of m";
  - a footer: prepared/checked/revision on one side; document, version, content hash and Cruncha version on the other;
  - a cover with the title block, the revision table (snapshots), a summary of every check and problem with its sheet number, and the contents;
  - numbered headings;
  - calcs with the description in a right-hand column, and a coloured edge and badge for errors, failures and NOT CALCULATED;
  - Python cells: source, figures, tables, text and defined names, kept on one page when short;
  - a provenance appendix.
- **`World`**: everything in memory, with no files, network or packages.
  - Fonts: Typst's bundled ones plus the system's, scanned once per process.
  - "Today": the commit date of the version.
- **Math:** the engine's TeX is converted to Typst math by `tex::to_typst`.
- **Corpus test** (`tests/tex_corpus.rs`): every TeX string the engine produces for Hurmet's golden tests is converted and compiled, in one document and then one by one if that fails. That is:
  - Hurmet's 66 parser TeX goldens;
  - the full display line of every evaluated assignment, function and calc golden, with echoes, errors and failed tests.
- **`report_demo`** (release build): a beam check evaluated by `cruncha-eval` (always unit-aware), with prose, echoes, a passing and a failing `@test`, an error, a matrix, a Python cell with a figure and a table, and a NOT CALCULATED Python cell. It renders the report repeatedly and writes the PDF and PNG pages.

```sh
cargo test -p cruncha-report
cargo run -p cruncha-report --release --example report_demo -- <out dir> 5
```

## Results

| Metric | Value |
| --- | --- |
| TeX → Typst corpus | **371 / 371** convert and compile (66 parser goldens + 305 evaluated statements); the whole batch compiles in 0.2 s |
| First pass of the converter | 352 / 371. Fixes: named arguments inside math calls are math (`augment: #(…)`), unicode delimiters (`⟨⟩`), and Hurmet's `aqua`/`Salmon` colors |
| Fonts | 404 families scanned in 125–139 ms (warm OS cache). The first run after a cold start took about 4 s, so the app scans in the background at start-up |
| First render of the demo (4 pages) | **290–340 ms** compile and layout + 8–10 ms PDF |
| Re-render, unchanged | 0.3 ms (memoized) + 5 ms PDF |
| Re-render after one edit | **26–27 ms** + 6 ms PDF |
| Determinism | byte-identical PDFs across renders (date, id and timestamp come from the version) |
| PDF size | 113 KB with a vector figure, fonts subset |

## Findings

1. **Typst is fast enough to re-export on every change.** After an edit a re-render takes about 30 ms, so a live preview pane is feasible later (`render_png` exists for that).
2. **Converting the engine's TeX works, and keeps the screen and the report in step.**
   - The subset is small (about 180 commands, 21 environments) and generated, so the converter is exact.
   - Unknown commands are errors, not silent drops. The corpus test catches any new construct.
   - Three Typst details mattered:
     - letters must be separated (`ab` is one identifier in Typst math);
     - `,` and `;` are argument separators inside calls, so they are escaped;
     - named arguments inside math calls are parsed as math, so values need `#`.
3. **MiTeX doesn't fit.** Its output calls functions from its Typst package (`mitexcolor`, `mitexarray`, …), so the package would have to be vendored and kept in step with the crate. It also accepts LaTeX the engine never emits, and unknown commands degrade instead of failing. The converter here is about 1,000 lines.
4. **Pass/fail has to come from the evaluator, not the display.** `cruncha_eval::Outcome::check` (new) carries whether a `@test` held. The report and its summary use it. An error or missing input is NOT CALCULATED or ERROR, never OK.
5. **Fonts are the remaining portability issue.** System fonts make the output depend on the machine. The bundled fonts are serif (Libertinus, New Computer Modern) and mono. Calc packages usually use a sans-serif face, so the template asks for Helvetica Neue, Helvetica or Arial and falls back to Libertinus. Bundling an OFL sans-serif font would make reports identical everywhere.
6. **The report's data lives in three places:**
   - the title block and options in the document's existing `meta` cell (versioned; no format change);
   - revisions are the snapshots;
   - the house style is the office's (Preferences).

   ADR-0015 records this.
7. **Python output for past versions** isn't stored yet, so only the latest version can be exported with its figures (planned `results/` table).

## Not done yet

- A report preview in the app; templates chosen per document or office (ADR-0014).
- Printing past versions and snapshots (needs stored results).
- A bundled report font.
- Units on Polars table columns in the report (as in the app, they aren't carried yet).
- Sign-off with a date per role, and a check-copy watermark ("NOT FOR CONSTRUCTION" until a snapshot).
