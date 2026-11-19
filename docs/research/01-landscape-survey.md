# Cruncha: Open-Source Landscape Survey for an AI-Enabled Mathcad Replacement

*Status: research draft, 2026-11-18. Facts marked (verified) were checked against current sources during this survey. All other facts are from prior knowledge and should be re-checked before we commit to a dependency.*

---

## 1. What we are replacing

Mathcad (Prime and the legacy 15) is the only mainstream tool that does all of these at once:

1. **The math is the document.** Equations are written in 2D textbook notation and evaluated where they appear. No separate code and output panes.
2. **Units are first-class.** `F := 12 kN`, `L := 3 ft`, and `M := F·L` gives a result in kN·m or kip·ft. Dimensional errors are caught.
3. **Live recalculation.** Changing a value at the top updates everything below it, following top-to-bottom, left-to-right evaluation order.
4. **Symbolic and numeric in one sheet**, plus solve blocks, root finding, ODEs, matrices, plots, and light programming.
5. **The worksheet is the deliverable.** It's the calc package an engineer signs, submits, and archives, so it has to be readable and checkable by someone who isn't the author.

Common complaints that open the door for a replacement:

- It's expensive, Windows-only, and has vendor lock-in.
- Files are binary or XML blobs that can't be diffed or reviewed in git.
- Prime is slow and heavy.
- Scripting and automation are weak, and you can't run a calc headless in CI.
- There's no real AI help.

**Goal for Cruncha:** keep capabilities 1–5, fix the complaints, and add an AI layer that writes and checks calculations without ever being the thing that computes the numbers.

---

## 2. The landscape

### 2.1 Direct Mathcad-style worksheet tools

| Project | License | Stack | Symbolic? | Units? | Notes / what to take from it |
| --- | --- | --- | --- | --- | --- |
| **CalcpadCE** (verified) | MIT | C# / .NET | No (fast numeric only) | Yes | Community fork of Calcpad 7.6.2. Upstream Calcpad went closed-source in March 2026 and pulled its public repo. The fork is active (v7.6.4) and adds a VS Code extension, strings and keyword arguments. **Lessons:** a plain-text source that renders to an HTML report is very diffable and fast, and works well with AI. The upstream license change is a cautionary tale about depending on a single vendor. |
| **SMath Studio** (verified) | Proprietary freeware | .NET | Via Maxima plugin | Yes | The closest open-ish clone of the Mathcad UI. Not usable as code. **Useful as a UX reference** for the 2D editor and plugin model. |
| **Hurmet** (verified) | MIT | JS, ProseMirror, Temml | Limited | Yes, with automatic conversion | A browser-based calc sheet that works offline. Mixes prose and calcs in a flow layout and has data frames. **Its document model is very close to what we want.** |
| **EngineeringPaper.xyz** (verified) | MIT (per listings; confirm the LICENSE file) | Svelte, MathLive, SymPy on Pyodide (WASM) | Yes (SymPy) | Yes, automatic checking and conversion | **The closest existing proof of our concept:** MathLive input, SymPy engine, units, systems of equations, plots, all running locally in the browser. Read this code before writing ours. Weak points: Pyodide startup cost, SymPy speed, and no professional report output. |
| **handcalcs** + **forallpeople** | Apache-2.0 | Python, Jupyter, LaTeX | Via Python | forallpeople | Renders Python as "formula → substituted values → result", the style used in hand-calc packages. **We want this rendering mode as a core feature.** |
| **efficalc**, **Blockpad**, Swift Calcs, Miramath | Mixed, mostly commercial or dead | — | — | — | Market signal only. Blockpad (commercial) is a good reference for modern UX. |

### 2.2 Computer algebra systems (CAS)

| Engine | License | Language | Usable in Cruncha (GPL-3.0-or-later)? | Assessment |
| --- | --- | --- | --- | --- |
| **SymPy** | BSD-3 | Python | **Yes** | Mature and broad: solve, integrate, ODEs, matrices, codegen, LaTeX output. LLMs know it extremely well, and several SymPy MCP servers already exist (verified). It's slow on large expressions. **Best v1 symbolic engine.** |
| **SymEngine** | MIT | C++ (Python, Rust bindings) | Yes | A fast core for SymPy-style operations. A good accelerator behind SymPy, but its Rust bindings are thin. |
| **python-flint / FLINT** | MIT / LGPL | C | Yes (LGPL through dynamic linking) | Very fast exact polynomial and number-theory arithmetic. SymPy can already use it. |
| **Symbolica** (verified) | **Source-available, commercial** | Rust (with Python) | **No.** Free only for one core and one instance in non-commercial or hobby use. Professional use needs a paid license. | Technically the best Rust CAS (v2.2 added symbolic integration and JIT evaluators), but its license is incompatible with an open-source engineering tool. **Avoid.** |
| **egg / egglog** | MIT | Rust | Yes | Equality-saturation rewriting engines. Not a CAS, but an excellent base for a **native Rust simplifier and rewriter** that we fully control. |
| **Maxima** | GPL (check whether it is v2-only or v2-or-later) | Common Lisp | Yes if v2-or-later. In practice it runs as a bundled subprocess because of the Lisp runtime. | Old and powerful. SMath and wxMaxima use it. A strong bundled symbolic kernel for things SymPy does badly. |
| **Giac / Xcas** | GPL-3.0-or-later | C++ (has a WASM build) | **Yes, it can link in-process**, including in the WASM web build | Strong and fast, and it powers the HP Prime calculator and GeoGebra's CAS. **Now a serious candidate for a second, fast symbolic engine** next to SymPy. |
| **SageMath** | GPL | Python and everything else | Yes, but too heavy to bundle | Could be an optional remote kernel. |
| **Reduce** | BSD | Lisp | Yes, but awkward | Permissive, but niche. |
| **FriCAS, Cadabra2** | BSD / GPL | — | — | Specialist tools, not relevant for v1. |
| **Julia Symbolics.jl** | MIT | Julia | Awkward (Julia runtime) | Good tech, wrong runtime for us. Watch it, don't adopt it. |

### 2.3 Numeric environments

| Project | License | Role for Cruncha |
| --- | --- | --- |
| **GNU Octave** | GPL-3.0-or-later | MATLAB-compatible numerics. The license allows bundling or embedding (liboctave), but its size argues for an **optional kernel** for MATLAB and Octave users (a "run this region in Octave" region type). Its own symbolic package just wraps SymPy, so it gives us nothing extra there. |
| **Scilab** | GPL-2 | Same idea as Octave, with less demand. Skip. |
| **NumPy / SciPy** | BSD | The Python numeric workhorse, available to user scripts and for fallback algorithms. |
| **Rust numerics**: `faer` and `nalgebra` (linear algebra), `argmin` (optimization), `diffsol` (ODE/DAE), `roots`, `statrs`, `dashu` (arbitrary-precision numbers) | MIT / Apache | **The native numeric core.** Fast, embeddable, and it compiles to WASM. Avoid `rug`, which pulls in LGPL GMP and makes WASM builds harder. |

### 2.4 Units and dimensional analysis

| Project | License | Notes |
| --- | --- | --- |
| **Numbat** (verified) | MIT OR Apache-2.0 | A Rust language where **physical dimensions are types**. Its unit standard library is written in Numbat itself, so it's extensible. It's the best existing Rust design for runtime units, and we could adopt its unit database and dimension checker or depend on it directly. |
| `uom`, `dimensioned` | MIT / Apache | Compile-time units. Wrong fit, because users define quantities at runtime. |
| **pint** | BSD | The standard Python units library. Use it in the Python layer and map it to the Rust units. |
| **forallpeople** | Apache-2.0 | Engineering-friendly units display (for example, automatic prefix selection). |
| **fend** | MIT | A good calculator that shows how unit parsing and edge cases are handled. |
| **libqalculate** | GPL-2.0-or-later | A huge, mature unit and constant database plus a parser for expressions with units. Under GPL-3.0 we **can link it directly or reuse its unit definitions**. Weigh it against the Numbat approach. |

### 2.5 Math input, rendering and typesetting

| Project | License | Role |
| --- | --- | --- |
| **MathLive** (verified) | MIT | The `<math-field>` web component: a WYSIWYG 2D math editor that outputs LaTeX, MathML, and **MathJSON (an AST)**. **This gives us a Mathcad-like editor.** |
| **Cortex Compute Engine** | MIT | MathLive's companion. Parses and canonicalizes MathJSON in JS. Useful for front-end validation and instant feedback. |
| **KaTeX / Temml / MathJax** | MIT / MIT / Apache | Display rendering. Temml produces MathML Core, which now renders natively in modern browsers. |
| **Typst** (verified) | Apache-2.0 | A Rust typesetting engine that can be **embedded as a library** by implementing its `World` trait, or through the `typst-as-lib` wrapper. Fast, with good math layout. **This is our PDF and calc-package report engine**, with no LaTeX install required. |
| **Tectonic** | MIT | A self-contained LaTeX engine in Rust. Fallback for users who need LaTeX output. |

### 2.6 Notebook and reactive-execution models

| Project | License | Lesson |
| --- | --- | --- |
| **marimo** (verified) | Apache-2.0 | A reactive Python notebook: cells form a dependency graph, editing one re-runs its dependents, and files are stored as plain `.py`. **This is exactly Mathcad's recalculation model, already solved and stored as text.** Study its DAG and file format. |
| **Pluto.jl** | MIT | Same reactive idea in Julia. |
| **Jupyter** | BSD | The kernel protocol is a useful idea for out-of-process kernels such as Octave and Maxima. Its notebook model, where cells run in whatever order you click, is the opposite of what we want. |
| **Quarto** | MIT | Prose plus computation published to PDF. Reference for report publishing. |
| **TeXmacs** | GPL | A structured WYSIWYG math editor with CAS sessions. A reference for 2D editing. |

### 2.7 Application shell

| Option | Verdict |
| --- | --- |
| **Tauri** (Rust backend with an OS webview, MIT/Apache) | **Recommended.** The Rust engine runs natively and the UI reuses web math tooling (MathLive, MathML). The same front end can ship as a pure-web build with the engine compiled to WASM. |
| egui / iced / Slint (native Rust UI) | Rich 2D math text editing in these is a multi-year project. Not worth it. |
| Electron | Works, but it's heavy and has no advantage over Tauri for a Rust core. |

### 2.8 AI and LLM integration prior art

- **SymPy MCP servers** (verified): `sdiehl/sympy-mcp` (Apache, unmaintained but forkable), `SymKit`, `mcp-sympy`. The common pattern is that the **LLM orchestrates and the CAS computes**, which is the right pattern for us. A known pitfall: MCP servers that expose too many fine-grained tools (one review covered a 171-tool server) perform badly. Expose a few coarse tools instead.
- I didn't find any open-source worksheet tool with a serious AI layer. That gap is our differentiator.

---

## 3. What we can reuse, and how

**Licensing rule:** Cruncha is **GPL-3.0-or-later**. We can use permissive code (MIT, BSD, Apache-2.0), LGPL code, and GPL code licensed "v2-or-later" or "v3". That makes Giac, Octave, libqalculate, and FLINT all available to link in-process.

We still can't use:
- **GPL-2.0-only code**, which is incompatible with GPLv3. Check each GPL dependency for "or later" wording.
- **Proprietary or source-available code** such as Symbolica, or non-commercial licenses.

Trade-off: anyone distributing Cruncha or a derivative has to provide its source under the GPL. If closed-source plugins or a dual-licensing business model might ever matter, we need a contributor agreement (a CLA, or a DCO plus a copyright-assignment policy) **before accepting outside contributions**.

| Need | Build on | Build ourselves |
| --- | --- | --- |
| 2D math editing | MathLive | MathJSON ↔ Cruncha AST mapping; region layout |
| Expression language and units | Numbat's design and unit database (fork or depend on it) | Mathcad-style operators (`:=`, `=`, range variables, ranges), unit display preferences |
| Recalculation | marimo/salsa-style incremental DAG (`salsa` crate, MIT/Apache) | Mathcad evaluation-order semantics, region dependencies |
| Numerics | faer, nalgebra, argmin, diffsol, roots | Solve blocks (`given…find`), unit-aware wrappers |
| Symbolic | SymPy (embedded through PyO3), with SymEngine or FLINT acceleration | Native `egg`-based simplifier for common fast paths (v2+) |
| Reports | Typst (embedded) | Calc-package templates, handcalcs-style substitution, sign-off blocks |
| Plots | Plotly.js or Vega-Lite in the UI; `plotters` for headless or Typst output | Unit-aware axes |
| Python scripting | PyO3 + maturin, pint, NumPy/SciPy | `cruncha` Python package: run worksheets headless, define custom functions |
| Second symbolic engine | Giac (linked in-process, GPL-3.0) | Symbolic bridge trait so SymPy and Giac are interchangeable and can cross-check each other |
| Optional kernels | Octave, Maxima, Sage (bundled or user-installed subprocesses) | Kernel protocol (Jupyter-like but minimal) |
| AI | MCP; an LLM API | Worksheet-aware tools, provenance tracking, review workflows |
| Mathcad import | — | `.xmcd` (Mathcad 11–15, XML) and `.mcdx` (Prime, an OPC/zip package of XML) converters. **A major lever for adoption.** |

---

## 4. Proposed architecture (strawman)

```
┌──────────────────────────────────────────────────────────────────┐
│  UI  (Tauri webview, or a pure web build)                        │
│  MathLive regions · prose (ProseMirror) · plots · AI side panel  │
└───────────────▲──────────────────────────────────────────────────┘
                │ JSON-RPC (in-process via Tauri, or WASM)
┌───────────────┴──────────────────────────────────────────────────┐
│  cruncha-core  (Rust)                                            │
│  • Document model: plain-text .cru format, diffable, git-friendly│
│  • Parser → AST (also from MathJSON)                             │
│  • Incremental dependency graph (salsa) → reactive recompute     │
│  • Units & dimensions (Numbat-derived)                           │
│  • Numerics (faer / argmin / diffsol), solve blocks              │
│  • Symbolic bridge trait ──► SymPy (PyO3) │ Giac (FFI) │ egg     │
│  • Kernel bridge ──► Octave / Maxima / Sage (subprocess)         │
│  • Report renderer ──► Typst (PDF), HTML                         │
└───────▲───────────────────────────────▲──────────────────────────┘
        │ PyO3 / maturin                │ MCP server + CLI
┌───────┴────────────────┐   ┌──────────┴──────────────────────────┐
│ cruncha (Python pkg)   │   │ AI layer                            │
│ headless run, CI,      │   │ tools: read_sheet, edit_region,     │
│ custom funcs, pint,    │   │ evaluate, check_units, explain,     │
│ SciPy interop          │   │ symbolic(op), render_report         │
└────────────────────────┘   └─────────────────────────────────────┘
```

### Key design positions

1. **Text-first file format.** The `.cru` worksheet is human-readable text, in the spirit of Calcpad source or marimo's `.py` files, holding regions, math, units, and prose. Git diffs, code review, and AI editing all become trivial. The 2D editor is a view of this text, not the source of truth.
2. **The engine computes and the AI never does.** Every number in a sheet comes from the deterministic engine. The AI writes and edits regions, explains them, and audits them, and its suggestions are checked by evaluating them.
3. **Provenance on every region.** Each region records whether a human wrote it, the AI wrote it and nobody has reviewed it, or the AI wrote it and a human accepted it. Reports can show these flags. This matters for regulated, PE-stamped work.
4. **Swappable symbolic engine.** We start with SymPy for breadth and add Giac for speed. Having two independent CAS engines also lets the checker re-derive a result in the second engine and flag disagreements. Hot paths move to native Rust over time.
5. **Headless by design.** `cruncha run sheet.cru --set L=4m --pdf out.pdf` works in CI. Calcs become testable artifacts, which Mathcad can't offer.

### AI features, roughly in order of value

1. **Natural language to worksheet:** "Size a simply supported W-beam for 30 ft span, 2 kip/ft" produces a draft sheet with assumptions stated and flagged as unreviewed.
2. **Calc checker:** unit consistency, sanity bounds, missing load cases, "this value differs from the typical range", and independent re-derivation through a different method.
3. **Explain and annotate** a region or a whole sheet for a reviewer.
4. **Importing legacy files:** Mathcad `.xmcd`/`.mcdx`, Excel calc sheets, and photos or PDFs of hand calcs, with the AI helping wherever a deterministic converter fails.
5. **Parametric exploration:** "What span makes deflection govern?" The AI drives the engine through sweeps and solves.
6. **Design-code lookup** with citations. Most engineering design standards are copyrighted, so we would rely on user-supplied references rather than bundling the text.

---

## 5. Risks and open questions

| Risk / question | Notes |
| --- | --- |
| **Flow layout vs. free-form canvas** | Mathcad allows regions anywhere on the page. Hurmet and EngineeringPaper use a top-to-bottom flow and are much simpler, diffable, and friendlier to AI. **Recommendation: flow layout with optional side-by-side columns.** We need a decision on this. |
| SymPy speed and Python packaging | Embedding CPython in a Tauri app (python-build-standalone) adds roughly 30–60 MB. Pyodide for the web build has a slow cold start. Mitigation: a native Rust fast path for arithmetic, units, and numerics, so Python loads only for symbolic work. |
| Mathcad semantic fidelity | Range variables, vectorization, and solve-block semantics are subtle, so importers need a corpus of real sheets to test against. |
| Upstream license drift | Calcpad showed that an open-source upstream can close. Prefer projects with several maintainers or foundations behind them, and pin and vendor critical pieces. |
| GPL dependency audit | Confirm the exact license wording ("or later" or not) for Maxima, Giac, Octave, and libqalculate before bundling any of them. Add `cargo-deny` license checks to CI. |
| Symbolica temptation | It's the best Rust CAS, but its license blocks us. Revisit only if it relicenses. |
| AI liability | Provenance, mandatory human acceptance, and reports that show what is unreviewed make the AI an assistant rather than the engineer of record. |

---

## 6. Suggested next steps

1. **Spike A (1–2 weeks):** fork-read EngineeringPaper.xyz and Hurmet. Write down their document models, unit handling, and where they break.
2. **Spike B:** Rust prototype of `.cru` parsing, a units core (try depending on `numbat` first), and incremental recompute with `salsa`. Evaluate 50 lines of a typical beam calc and benchmark it.
3. **Spike C:** a Tauri shell with MathLive regions that round-trip MathJSON ↔ AST ↔ `.cru` text.
4. **Spike D:** a Typst report from an evaluated sheet in handcalcs style (formula → substitution → result).
5. **Spike E:** an MCP server over the engine with about six coarse tools. Test whether an LLM can author a correct sheet from a prompt, using a small eval set of standard engineering problems.
6. Then write an ADR fixing the file format, layout model, and symbolic-engine boundary.

---

## Sources

- CalcpadCE: https://github.com/imartincei/CalcpadCE · docs https://imartincei.github.io/CalcpadCE/ · on upstream Calcpad's status: https://eximentor.de/en/calcpad/
- SMath Studio vs Mathcad: https://wiki.smath.com/en-US/SMath_Studio_vs_Mathcad · https://alternativeto.net/software/mathcad/
- Hurmet: https://github.com/ronkok/Hurmet · https://hurmet.pages.dev/guides/Why-Hurmet.html
- EngineeringPaper.xyz: https://github.com/mgreminger/EngineeringPaper.xyz · https://alternativeto.net/software/engineeringpaper-xyz/about
- handcalcs: https://github.com/connorferster/handcalcs (mirror listing: https://awesome.ecosyste.ms/projects/github.com%2Fconnorferster%2Fhandcalcs)
- Symbolica: https://symbolica.io/posts/symbolica_2_0_release · https://symbolica.io/license/ · https://github.com/symbolica-dev/symbolica
- Numbat: https://awesome.ecosyste.ms/projects/github.com%2Fsharkdp%2Fnumbat · https://docs.rs/crate/numbat/latest/source/Cargo.toml
- MathLive: https://github.com/arnog/mathlive
- Typst as a library: https://docs.rs/crate/typst-as-lib/latest · https://docs.rs/typst-library/0.14.2/typst_library/trait.World.html
- marimo: https://pypi.org/pypi/marimo
- SymPy MCP: https://www.stephendiehl.com/posts/computer_algebra_mcp/ · https://mcpservers.org/servers/sdiehl/sympy-mcp
