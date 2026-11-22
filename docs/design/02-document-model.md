# Cruncha Design 02: Document, Execution and Storage Model

*Status: proposal, 2026-11-18. Builds on [01-landscape-survey](../research/01-landscape-survey.md). The Hurmet references come from reading its source at commit `87f30ed` (MIT).*

## 0. Goal

A modern, AI-forward **calculation and report tool**: Mathcad's live, readable, unit-aware math in one document with Jupyter-style **Python and Rust code blocks** (NumPy, SciPy, Polars and so on). Everything lives in a single reactive dependency graph, and storage is **versioned and continuous, with no Save button**. Hurmet's document model is our starting point.

---

## 1. What we take from Hurmet, and what we change

Hurmet is a ProseMirror rich-text editor with inline calculation cells.

| Hurmet feature | Verdict | Notes |
| --- | --- | --- |
| **Rich prose document with calc cells inline and as blocks** | **Keep** | This is the heart of a readable calc package: narrative, figures and math together. |
| **Linear text entry with a live rendered preview** (an inline CodeJar editor plus a Temml MathML preview) | **Keep as the default** | Faster than WYSIWYG for power users and perfect for AI, since the source is just text. MathLive becomes an optional input mode. |
| **Statement form `name = expr = ?? unit`** | **Keep** | The result position and display unit are written in the statement. |
| **Display selectors** `?` (full plus blue echo), `%` (no echo), `@` (result only), `!` (hidden) | **Keep** | The "blue echo" with substituted values is what makes a calc reviewable. |
| **Dual-value quantities** (`{plain, inBaseUnits}`; a doubled selector means unit-aware) | **Change** | This means `2L = ?` and `2L = ??` give different numbers for the same variable, which is a trap. **Cruncha is always unit-aware**, and a selector controls display only. |
| **8-dimension unit vector plus factor and gauge; about 541 units; currency as a dimension** | **Keep the design, port the data** | Fix the known defects found in Hurmet (both fixed in the port): the ℉ gauge is 459 instead of 459.67, and one `Å` entry has the wrong exponent array length. Merge the unit table with Numbat's and qalculate's. |
| **Exact rationals** (BigInt) for `+ − × ÷` and integer powers | **Keep** | Transcendental functions fall back to f64, as in Hurmet. We also normalize after `×` and `÷`. |
| **Data frames with a units row**, row keys, `df.col` and `df["row","col"]` access | **Keep the semantics, replace the storage** | We back them with Polars/Arrow, not raw string columns, and accept CSV, Parquet and Delta as well as TSV. |
| **User functions** (`function … end`, hoisted, pure) | **Keep** | They are pure, so they are safe to call from anywhere. Python and Rust functions join the same namespace. |
| **`@test` assertions and `_` placeholders** | **Keep** | Assertions are a basis for CI. Placeholders mark template inputs. |
| **Results saved in the file** (`〔…〕`) so people and AI can read them without recomputing | **Keep the idea** | This becomes a `results` table plus the text mirror (§5). |
| **Headless CLI** (`hurmet2html`) | **Keep** | `cruncha run` |
| **Markdown file format with `¢`, `〔〕`, `␤`, `¶` delimiters** | **Replace** | Those characters are hard to type and other tools don't understand them. Markdown becomes an *export and mirror* (§5.4); the store is Delta (§5). |
| **Run-time state in ProseMirror node attrs; no cell IDs; position-based lookup** | **Replace** | Cells get stable IDs, and results live in a side table keyed by cell ID. |
| **Document-order evaluation with name shadowing; rename doesn't invalidate readers** | **Replace** | We use a real dependency graph (§3). |
| **String-encoded RPN (`®`, `¿`, NBSP) and regex-driven compile** | **Replace** | We use a typed AST and bytecode in Rust. Hurmet's ~450 golden tests become our **syntax conformance suite**. |
| **Snapshots as full Markdown copies inside the file** | **Replace** | Delta time travel replaces them. |
| **Print pagination by measuring the browser DOM with hard-coded pixel sizes** | **Replace** | Typst renders reports deterministically. |
| **Remote `import(url)` / `fetch(url)` with CORS** | **Keep, generalize** | Imports and data sources are cells with explicit caching and offline fallbacks. Hurmet's fallback path has a bug where the fallback is never used. |

**Fork or rewrite?** We rewrite. Hurmet's engine is JS and its core would be the wrong layer for us. We reuse its **syntax, unit data and test corpus** in the Rust core, and borrow freely from its ProseMirror schema and UI ideas. MIT code can go into a GPL-3 project as long as we keep Hurmet's copyright notice. As a bonus, **Cruncha should open Hurmet `.md` files**.

---

## 2. Document model

A document is an ordered tree of **blocks**, and every block has a stable ID (a ULID).

```
Document
 ├─ meta: title, page size, number format, unit system preferences, …
 └─ blocks[] (ordered by fractional index, so inserts never renumber)
     ├─ prose      ProseMirror JSON of one block: paragraph, heading, bulleted/numbered list,
     │              quote or rule (now); table, figure, callout (planned). crates/cruncha-prose
     │              reads and writes it, and converts Markdown both ways
     │              └─ may contain inline calc cells (each with its own ID)
     ├─ calc       Hurmet-syntax statement(s)                   → values
     ├─ python     Python source                                → values, figures, tables
     ├─ rust       Rust source (compiled to WASM)               → values, functions
     ├─ data       a named data frame (import / inline table)   → Polars frame
     ├─ plot       declarative plot spec over variables         → figure
     ├─ drawing    sketch / draw() DSL                          → SVG
     ├─ test       @test assertions                             → pass/fail
     ├─ tex        display math (no computation)
     └─ page       page break, header and footer regions, TOC
```

Each cell record:

```text
id, kind, order_key, parent_id (for inline cells), source (text), display (selector,
format, unit), defines[] (names), reads[] (names), provenance, created/modified (by actor)
```

A calc's **description** (`attrs.description`, one line of text) says what the quantity it defines is. It is not part of the statement, so it never affects a result. It is shown beside the typeset calc and in the variable browser (a sidebar list of every defined name with its value, description, section and users, searchable and sortable).

**Results are not part of the document.** Results live in a separate results table keyed by `(cell_id, input_hash)` (§5.2).

**Provenance**, per cell: `human` | `ai:proposed` | `ai:accepted(by, at)` | `imported(source)`. Reports can display it.

---

## 3. Execution model: one reactive graph for math, Python and Rust

### 3.1 Rules

1. **Every name has exactly one defining cell.** Defining a name twice is an error shown on both cells. There is no Hurmet-style shadowing, which removes stale results and evaluation that depends on document order.
2. **Evaluation follows the dependency graph, not document position.** Editing a cell re-runs it and everything downstream, the way marimo does.
3. **Using a name above its definition is allowed but flagged with a lint badge.** Calc packages should read top to bottom for reviewers, and the lint keeps them honest without breaking the computation.
4. **Cycles are errors**, highlighted along the whole cycle.
5. **Invalidation compares what a cell defines before and after an edit.** Renaming `x` to `y` invalidates readers of `x`, which fixes the stale-result bug found in Hurmet.
6. **Results are memoized** by `hash(source, input values, engine version)`. Unchanged inputs mean no re-run, even across reopening the document.
7. **Scheduling is asynchronous, with cancellation and timeouts.** Slow Python or Rust cells don't block math cells that don't depend on them. Running cells show a spinner. Infinite loops can be killed, which Hurmet can't do.

> **Open decision D1:** Mathcad users often reuse names (`M` for several different checks). Rule 1 forbids that. A possible relaxation later: **section scopes**, where a heading can declare `scope` so names defined inside it are local to it. Start strict.

### 3.2 Calc cells (native Rust engine)

- The parser is Hurmet-compatible (implicit multiplication, Unicode identifiers, subscripts, `20 N·m` unit parsing) and produces a typed AST. Echo templates are built from the AST, not by string surgery.
- Values have the type `Value = Number(Rational|f64|Complex) | Quantity(Value, Dim, display unit) | Array(ndarray of Quantity) | Frame(Polars) | Map | Tuple | String | Date | Bool | Function | Module`.
- Built-ins cover Hurmet's set, plus solve, root, integrate, ODE and optimize, via `faer`, `argmin`, `roots` and `diffsol`.

### 3.3 Python cells

- **Runtime:** one **out-of-process Python worker per document**. It is an embedded `python-build-standalone` interpreter with NumPy, SciPy, Polars, pint, SymPy and matplotlib preinstalled, plus user-managed `uv` environments. We run it out of process rather than in-process through PyO3 for three reasons:
  - a crash or hang can't take down the app;
  - cells can actually be cancelled;
  - the worker can be sandboxed.

  The worker also serves as the **SymPy symbolic engine**.
- **Finding dependencies:** the worker parses each cell with `ast`. Names assigned at top level are `defines`, and free names that are read are `reads`. For dynamic code (`exec`, `globals()`, `import *`), the cell gets a warning badge, and the user can declare names explicitly with `# cruncha: defines a, b`.
- **Passing values between math and Python:**

  | Cruncha | Python |
  | --- | --- |
  | Quantity | `pint.Quantity` (from the Cruncha unit registry) |
  | Rational | `fractions.Fraction` when exact, `float` otherwise (configurable) |
  | Array of quantities | NumPy `ndarray` wrapped in pint |
  | Frame | **`polars.DataFrame`**, with column units stored in Arrow field metadata |
  | Function | a callable (Python functions can be called from calc cells) |

  Frames and arrays travel as **Arrow IPC through shared memory**, so we don't copy large data.
- **Outputs:** the assigned names, plus the value of the last expression. Rich output is captured as data, never HTML: matplotlib figures as PNG, Polars tables, and interactive figures as JSON specs of allowlisted MIME types (Plotly, Vega-Lite), drawn in sandboxed frames (ADR-0017, ADR-0020).
- **When cells run:** only in documents this computer trusts. A cell re-runs on its own when its inputs change; an edited cell waits for Run (Shift+Enter). Each cell runs in a fresh namespace holding only what it reads, and imports are local to the cell. Results are memoized by source and inputs; a slow cell can be stopped (SIGINT, then a kill).
- **Reproducibility:** the document records its environment (a lockfile of package versions), which is stored in the doc. *Today:* one app-wide environment from `python/requirements.lock` (hashed, installed by uv); the per-document lockfile is still to do.

### 3.4 Rust cells

- **Runtime:** cells compile to **WASM (`wasm32-wasip2`) and run in `wasmtime`**. That gives us sandboxing, determinism, no native ABI problems, and the same behavior on desktop and web. Compiled modules are cached by source hash. The first compile takes seconds and later runs are instant; the UI shows a "compiling…" state.
- **Prelude crate:** `cruncha-prelude` provides `Quantity`, `Array`, the `Frame` view (Arrow), and an `#[cruncha::export]` macro. Exported functions and statics become names in the graph:

  ```rust
  use cruncha::prelude::*;
  #[cruncha::export]
  fn plastic_modulus(b: Length, h: Length) -> Volume { b * h * h / 4.0 }
  ```

- **Dependencies:** an allowlist of crates that build for WASM (nalgebra, faer, statrs, …), configured per document.
- **Escape hatch (later):** native `cdylib` builds for heavy compute, behind an explicit "trust this document" prompt.

### 3.5 Symbolic math

- Calc cells can request symbolic results (`f'(x) = d/dx(...) = ?sym`).
- The calls go through a `SymbolicEngine` trait. The first backend is SymPy in the Python worker, and a Giac backend comes later.
- Results come back as ASTs, so they render and echo like any other calc.

---

## 4. Rendering and reports

- **Screen:** prose in ProseMirror, and math as MathML (Temml-style TeX to MathML, generated in Rust from the AST).
- **Report:** the evaluated document goes to **Typst**, then to PDF. Typst handles deterministic pagination, header and footer, TOC, page numbers, and a provenance appendix. Templates for calc packages include a title block, sign-off and revision table. As built (S6, ADR-0015):
  - the title block lives in the `meta` cell;
  - revisions are snapshots;
  - the house style is in Preferences;
  - results that weren't computed print as NOT CALCULATED.
- **Export targets:** PDF, HTML, Markdown mirror (§5.4), DOCX (through Typst or Pandoc, later).

---

## 5. Storage: Delta Lake, always committed, fully versioned

### 5.1 Principles

- **No Save button.** Every meaningful change is an atomic Delta commit. A crash loses at most the current debounce window.
- **Delta versions are the document's history.** Time travel, snapshots and rollback are built on them.
- **Each table has one job:** the document, the data, the results and the history are separate tables in one container.
- **It stays readable outside Cruncha:** Parquet plus a JSON transaction log, readable with Polars, DuckDB or Spark, plus an always-current text mirror.

### 5.2 Container layout

A document is a **directory**, `beam-check.cruncha/`, containing Delta tables:

```
beam-check.cruncha/
├── doc/          Delta table: append-only log of cell revisions (ADR-0010), reduced to the
│                 latest row per id on read. id, kind, order_key, parent_id, source, attrs,
│                 defines, reads, provenance, modified_by, modified_at, rev, deleted,
│                 source_hash
├── blobs/        large cell sources (≥ 1 KiB), zstd, named by sha256 (ADR-0013)
├── pins/         copies of named versions whose history was pruned (ADR-0011)
├── results/      Delta table (ADR-0018): stored code-cell runs. cell_id, input_hash
│                 (SHA-256 of source, inputs, environment, result format), status,
│                 values, outputs (JSON; long strings as blob refs), stdout, stderr,
│                 error, duration_ms, env, computed_at_us, computed_by, doc_version, blobs
├── archive/      Delta table (ADR-0023): the save log, one row per save a prune or archive
│                 removed. version, at_us, actor, message, kind, snapshot, cells, removed_before
├── data/<name>/  one Delta table per data frame (Polars reads and writes these directly)
├── assets/       content-addressed blobs (images, figures): sha256 → file (never rewritten)
├── refs/         Delta table: named versions/tags → {doc_version, data_versions{}, message, author}
├── env/          Python lockfile, Rust crate allowlist, unit-system config
└── README.md     auto-generated text mirror (§5.4)
```

### 5.3 How writes work

- **Commit cadence:** a calc, Python or Rust cell commits when it is **submitted**, as in Hurmet's prompt model. Prose commits after about **1–2 s of idle** and on blur. Each commit appends only the rows that changed (ADR-0010), with large sources written once to `blobs/` (ADR-0013). A typical commit writes one Parquet file of a few KB plus one log entry.
- **Who made each change:** every commit records `actor` (a human user, `ai:<session>`, or `cli`) in Delta's `commitInfo`. The history then answers "who changed what, and when", AI included.
- **Small-file control:** a checkpoint every N commits, and a background `OPTIMIZE` (compaction) when the app is idle. A heavy editing session might produce a few hundred commits an hour. That's well within what Delta can handle after compaction, but **Spike S3 must measure it.**
- **Concurrency:** a single writer per document, namely the engine process holding a lock file. The UI, the AI/MCP agents and the CLI all go **through that process**. If no engine is running, the CLI writes directly using Delta's optimistic concurrency.
- **Data frames:** importing a CSV writes `data/<name>` once. Python cells that return frames write new versions. Results reference data by `(table, version)`, so a cached result is always tied to the exact data that produced it.

### 5.4 Text mirror (for git, review and AI)

On idle, Cruncha regenerates `README.md` inside the container, and `cruncha export --md` writes it anywhere. It is a Hurmet-flavored Markdown that **includes results**, the way Hurmet's `〔…〕` saved results do, but with ASCII-friendly fences:

````markdown
## Beam check

Span ‹calc #01J… L = 3.1 m›, uniform load ‹calc #01J… w = 100 lbf/ft›.

```calc {#01JC3… display=??}
M = w L^2 / 8 = ?? kip·ft
```
> ⇒ **M = 1.293 kip·ft**

```python {#01JC4… defines=[sections]}
import polars as pl
sections = pl.read_parquet("aisc_w_shapes.parquet").filter(pl.col("Zx") > Z_req)
```
> ⇒ `sections`: DataFrame 37 × 12
````

- The mirror is **one-way by default**. Importing it back (`cruncha import README.md`) creates a new document, or a new version of an existing one, by matching cell IDs. That lets you edit in git or an IDE and bring changes back in.
- The inline calc delimiter `‹calc …›` above is tentative (**D2**).

### 5.5 Versions, snapshots and rollback

| User action | Mechanism |
| --- | --- |
| Undo/redo (within a session) | Editor history, which is not tied to Delta versions |
| "History" slider / view any past state | `doc` time travel: `load_version(n)` or `load_as_of(timestamp)`, joined with the `results` and `data` versions from that time |
| Named snapshot ("Issued for review rev A") | A row in `refs` that pins `doc_version` and every `data/*` version. Taken by a person from the History panel ("Take snapshot", with a note), at the latest version; agents may suggest but not take one. The panel shows how many edits came after the newest snapshot. |
| Diff two versions | Delta Change Data Feed / version diff on `doc`, rendered per cell (source diff plus result diff) |
| Roll back | A **new commit** that restores the old rows ("Restore this version/snapshot" when viewing it). History is never rewritten, so a rollback can itself be undone. A snapshot whose version was pruned is restored from its pin. |
| Branch / AI proposal | **D3:** a proposal is a set of pending rows (`doc` rows with `proposal_id`) shown as tracked changes. Accepting commits them; rejecting deletes them. We avoid full Delta branching for now. |

**Retention:** `VACUUM` deletes Parquet files older than the retention window, and that breaks time travel to versions older than it. Policy (details in ADR-0011, implemented in `Store::prune`):
- **Retain all history by default.** Document tables are tiny, so this is cheap.
- A document may opt into limits on saves, size, or days of *activity* (counted back from the last edit). Pruning runs only after new edits, keeps the newest 100 versions, and first copies older refs into `pins/`.
- For large `data/*` tables, offer "compact history". Before vacuuming, it **materializes every pinned `refs` version** as a copy.
- Large raw imports are content-addressed and never vacuumed while any ref uses them.

### 5.6 Sharing

- A `.cruncha` directory is awkward to email. **Export to `.crunchaz`**, a zip that by default contains the latest version (option: full history). Mathcad's `.mcdx` and Office files are zips too.
- Opening a `.crunchaz` unpacks it into a working directory.
- For git, either commit the container directory (binary Parquet, which doesn't diff well), or commit only the **text mirror plus data files**. Recommended: the mirror, with a CI job running `cruncha run` to recompute and check `@test`s.

### 5.7 Why not SQLite or a CRDT alone

- **SQLite** is a single file with good atomicity. But Polars can't scan it natively, so data frames would need copying in and out. It also has no time travel unless we build it ourselves.
- **A CRDT (Loro)** would give real-time collaboration and fine-grained history. It is not an analytics store, though, and its history is opaque to other tools.
- **Delta** gives us analytics-native data, time travel and interoperability in one format. Its weak spot is high-frequency tiny writes, which the commit cadence in §5.3 addresses.
- **Later:** live multi-user editing can use Loro in memory, persisting its updates as Delta commits. That would need no format change.

---

## 6. AI-forward by construction

- **Everything is addressable:** cell IDs, names, versions and data tables. Agents work through an **MCP server built into the engine** with a small set of coarse tools: `read_doc` (the text mirror with results), `propose_edit` (cells), `run`, `explain(cell)`, `check(doc)`, `history(cell)` and `query_data(sql or polars)`.
- **The AI never writes numbers.** It writes cells; the engine computes them. AI edits arrive as proposals with `ai:proposed` provenance until a human accepts them.
- **Checker passes:**
  - unit sanity;
  - lint for names used before they are defined;
  - re-deriving key results with a second method or engine (SymPy against the numeric result);
  - flagging cells marked as placeholders that were never filled in;
  - "explain this calc to a reviewer".
- **History-aware:** "what changed since rev A and which results moved" is a Delta diff plus a results diff, which an LLM can summarize.

---

## 7. Code layout (proposed)

```
crates/
  cruncha-units      dimension vectors, unit DB (Hurmet + Numbat/qalculate merge), parsing
  cruncha-lang       lexer, parser (Hurmet syntax), AST, echo/TeX/MathML generation
  cruncha-eval       values, rationals, operators, built-ins, numerics
  cruncha-graph      dependency graph, scheduler, memoization, cancellation
  cruncha-store      Delta container (delta-rs), refs, mirror export/import, Hurmet .md import
  cruncha-kernel-py  Python worker protocol (Arrow IPC), AST dependency scan
  cruncha-kernel-rs  Rust→WASM compile cache, wasmtime host, prelude crate
  cruncha-report     Typst World implementation, templates
  cruncha-mcp        MCP server
  cruncha-cli        `cruncha run | export | import | check`
python/cruncha/      worker runtime + pip package (headless API, pint registry bridge)
app/                 Tauri shell + ProseMirror UI
tests/conformance/   ported Hurmet golden tests + our own
```

---

## 8. Spikes and decisions before writing real code

| ID | Spike | Answers |
| --- | --- | --- |
| S1 | Port Hurmet's lexer and parser to Rust against its 217 calc golden tests | Can we match its syntax? How big is the parser really? **Done 2026-11-19** ([write-up](../spikes/S1-calc-parser.md)): 282/283 golden cases match; about 4,500 lines of Rust in `crates/cruncha-lang`. |
| S2 | `cruncha-graph`: a dependency graph over mixed calc and Python cells, with rename invalidation and cancellation | Is the execution model sound? **Core done 2026-11-19** (`crates/cruncha-graph`): order, rules 1, 3, 4 and 5, and memoized evaluation of calc cells in the app (`engine.rs`, rule 6). Cancellation and async scheduling (rule 7) wait for the Python worker (S4). |
| S3 | **Delta store benchmark:** 2,000 small MERGE commits on local disk; time-travel latency; OPTIMIZE and checkpoint behavior; a Polars round trip of `data/*`; crash halfway through a commit | Does always-committed Delta hold up? This is the riskiest assumption. |
| S4 | Python worker: an Arrow IPC round trip of a 1M-row Polars frame with units in metadata; AST dependency scan | Data-passing cost, and how reliable dependency detection is. **Done 2026-11-20** ([write-up](../spikes/S4-python-worker.md)): the frame round-trips in 0.13–0.22 s over pipes (shared memory deferred); the scan matches bytecode on 98.6 % of the stdlib. Python cells run in the app (`python.rs`, `crates/cruncha-kernel-py`). |
| S5 | A Rust cell compiled to WASM with the prelude, measuring compile time and call overhead | Is the Rust-block UX acceptable? |
| S6 | A Typst report from an evaluated doc with blue-echo rendering | Report quality. **Done 2026-11-20** ([write-up](../spikes/S6-reports.md), ADR-0015): embedded Typst 0.15 with an in-memory `World`. The engine's TeX converts to Typst math (371/371 golden strings compile). A 4-page calc package renders in 0.3 s, and in 30 ms after an edit, byte-identical each time. Export report (PDF) is in the app (`crates/cruncha-report`). |

| ID | Decision | Recommendation |
| --- | --- | --- |
| D1 | Name uniqueness vs. shadowing | Strict uniqueness now; section scopes later |
| D2 | Inline calc syntax in the text mirror | Pick after S1. It must be ASCII-typeable and not clash with Markdown or Hurmet. |
| D3 | AI proposals: pending rows vs. Delta branches | ~~Pending rows~~ Superseded by ADR-0016: a separate append-only `proposals/` table; accepting is a human's commit |
| D4 | Python: out-of-process vs. embedded PyO3 | Out of process (worker). S4: the overhead doesn't hurt (0.1–0.2 ms per cell, 0.2 s for a 1M-row frame) |
| D5 | Always unit-aware (break from Hurmet's `?`/`??` duality) | Yes; `??` is accepted on import and treated as `?` |
