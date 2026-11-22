---
name: cruncha-code-cells
description: How to write Python and Rust cells in a Cruncha document so they join its reactive graph. Covers how defined and read names are detected, when cells run (trust, Run for edited cells, Stop), how values convert between calc cells and Python (pint quantities from cruncha.ureg, NumPy arrays, Polars frames, Fractions), plots and output, and Rust cells compiled to WASM with exported functions. Use it when a calc needs NumPy, SciPy, Polars, plotting, iteration, or performance-critical code.
---

# Python and Rust cells

Code cells are part of the same dependency graph as calc cells (ADR-0004):
- A code cell **defines** the names it binds at its top level (assignments, `for`/`with` targets, `def`, `class`).
- It **reads** the free names it uses, also inside its functions.
- When an input changes, the cell re-runs on its own, and so does everything downstream.

There is no hidden kernel state: each Python cell runs in a fresh namespace that holds only what it reads (ADR-0005). A name the scan didn't see is simply not there (`NameError`).

## When to use a code cell rather than a calc

- **Use a calc** for engineering formulas a reviewer must read. The blue echo is the point.
- **Use Python** for data wrangling (Polars), numerical methods (SciPy solvers, ODEs, optimization), loops over many cases, and plots.
- **Use Rust** for heavy inner loops, or for reusable numeric functions that calc cells should call quickly.

When a code cell produces a key design value, also show it in a calc cell, for example `ratio = M_max / M = ?`. That way the report displays it with units and an echo.

## Python

```python
import numpy as np
import matplotlib.pyplot as plt

# Reads L and w (calc quantities). Defines x, moment and M_max.
x = np.linspace(0, L, 51)
moment = (w * x * (L - x) / 2).to("kip*ft")
M_max = moment.max()

plt.plot(x.magnitude, moment.magnitude)
```

**How names are found** (the worker's `ast` scan, when the cell is saved):
- **Imports are local to the cell.** Every cell imports what it uses; `np` is never shared, so two cells can both `import numpy as np`.
- Names starting with `_` are private to the cell. Comprehension variables, `except … as e` names and names you `del` at the top level aren't defines.
- **No `exec`, `eval`, `globals()`, `global` or `from x import *`.** The scan can't see what they do, so the cell gets a warning. If you really need dynamic code, declare the names: `# cruncha: defines a, b` (replaces the scan's defines) and `# cruncha: reads c` (adds reads).
- A syntax error is shown without running anything.

**When cells run**
- A cell runs on its own when its inputs change, but only after it has been run once with its current source. **An edited cell waits for Run** (Shift+Enter or ▶ Run), so half-written code never executes.
- A document runs Python only on a computer that trusts it. Documents opened from elsewhere ask first.
- Cells run one at a time, in dependency order, in one worker process per document. A long cell can be stopped (■ Stop raises `KeyboardInterrupt`); code that ignores that gets the worker restarted, and every Python value is recomputed.
- **Finished runs are stored with the document** (ADR-0018), keyed by a hash of exactly what produced them: the cell's source, the values of everything it reads, and the Python environment (lockfile). A cell whose inputs nobody changed **doesn't run again** when the document is reopened; its stored output and values are used, and calcs read them as usual. Change a source or an input, or upgrade the environment, and the hash changes, so it runs again (when trusted and Python is set up).
- Stored results are data, so they show even where nothing may run (Python not set up, document not trusted), and **past versions and snapshots show the Python output they had**. A cell with no stored result for its inputs shows "no stored result for this version" / NOT CALCULATED.
- A stored result from **another Python environment** (a Cruncha with different package versions) is never reused as current: where the cell can run, it runs again; where it can't (Python not set up, not trusted, a past version, an edited cell), its output is shown labelled "with a different Python environment (‹env›)", and the report notes it.
- A stored result computed on another computer is labelled "computed on ‹host›, ‹date› · not run on this computer", in the app and in the report. Interrupted runs aren't stored; runs whose output is over 64 MB are shown that session only.
- Stored results are a cache: they never add a document version, aren't counted in the history size, and can be dropped in Document settings → History → Clear stored results.

**Rules**
- Assign results to top-level names. Those are what other cells can read.
- Don't mutate a value that came from another cell (for example, `shapes` in place). Values are shared by reference, so a mutation would change it for its other readers. Make a new name instead.
- Don't do network or file I/O, except through a `data` cell, which records the source and its version. On macOS the worker is sandboxed, so this is enforced: opening the user's files, sockets or subprocesses fails (typically with `PermissionError`). Python's temp files (`tempfile`) work; they go to the worker's scratch directory.
- `import` only what the environment has: NumPy, SciPy, Polars, PyArrow, pandas, pint, SymPy, matplotlib, Plotly (pinned in `python/requirements.lock`). Propose adding a package rather than installing it ad hoc.

**Value conversion**

| Calc value | In Python | Back to calc |
| --- | --- | --- |
| Quantity `3.1 m` | `pint.Quantity` from `cruncha.ureg`, in the calc's unit; use `.to("ft")` and `.magnitude` | a quantity, in pint's unit when Hurmet knows its name (`kip·ft`), otherwise in SI |
| Exact number | `fractions.Fraction` (`int` when whole) | `Fraction` and `int` stay exact; `float` |
| Vector or matrix (with unit) | NumPy array, wrapped in a quantity when it has a unit | 1-D and 2-D numeric arrays up to 100,000 values |
| Data frame | `polars.DataFrame` (column units aren't carried yet) | `polars.DataFrame` up to 10,000 rows |
| Bool, text, complex | `bool`, `str`, `complex` | same |
| Anything else | not passed (the cell reports it) | stays in Python: other Python cells can read it; a calc reading it gets an error saying what it is |

Build quantities with Cruncha's registry so they combine with calc values: `from cruncha import ureg, Q_`, then `Q_(12, "kip")`. A quantity from another pint registry can't come back to a calc.

**In the PDF report** a Python cell prints its source (unless Report settings turn that off), its figures, tables (first 50 rows), printed text, and the names it defines. A cell that hasn't run with its current source and has no stored result for its inputs (Python not set up, not trusted, edited and not run) is printed as NOT CALCULATED, with the reason. Output from a stored result computed on another computer prints with that note, and the summary counts such cells. Export waits up to two minutes for cells that are running.

**Output**, in this order:
1. every matplotlib figure the cell left open, as an image (the figures are closed afterwards);
2. interactive figures (ADR-0017);
3. the value of the last expression: Polars frames as a table of the first 50 rows, other values as their `repr`;
4. what the cell printed.

Output is shown as data, never as HTML, so `_repr_html_` isn't used. Calling Python functions from calc cells is planned, not available yet.

**Interactive figures (Plotly):** use these for plots the reader should zoom, pan or hover, such as a moment diagram where hovering shows the value at a station.

```python
import plotly.graph_objects as go

# Reads x and moment (quantities). Units go into the axis titles.
fig = go.Figure(go.Scatter(x=x, y=moment, name="M"))
fig.update_layout(title="Bending moment")
fig          # or fig.show(), or cruncha.show(fig, other_fig)
```

- A figure is shown when it is the cell's last expression, when you call `fig.show()` (it never opens a browser), or through `cruncha.show(*figs)` for several in one cell. Each figure shows once, however many of these you use.
- **Units:** pint quantities are unwrapped to magnitudes. An axis without a title gets the unit (`ft`; `M (ft·kip)` for a named trace). With Plotly Express, `px.line(x=x, y=moment)` labels the axes `x (ft)` and `y (ft·kip)`. Set your own titles to override.
- Figures use Cruncha's look (the `plotly_white+cruncha` template) unless you set a template.
- **Limits:** a figure's spec may be up to 8 MB. Over that, it isn't shown, and the cell says why: downsample, or use `go.Scattergl` with fewer points. Use `Scattergl` for more than about 10,000 points; it is ten times faster to draw.
- Only Plotly figures and objects whose `_repr_mimebundle_` offers Plotly or Vega-Lite JSON are figures. Vega-Lite isn't drawn yet; it shows its summary. HTML, JavaScript and Bokeh output are refused: `cruncha.show` raises `TypeError`, and a last expression shows as its `repr`.
- **Agents** see a figure as its summary in `read_doc`: `figure (plotly): Bending moment: M (ft·kip) vs ft; 1 trace: M (scatter, 41 points)`. `explain` adds each trace's data range (`M (scatter, 41 points): x 0 … 20 ft; y 0 … 100 ft·kip`). Quote ranges from there; don't read values off a picture.
- **In the report**, a figure prints as vector graphics in the report's fonts, drawn by the app when the report is exported. A figure the app couldn't draw prints as FIGURE NOT RENDERED with the reason; it is never left out. `cruncha.show(fig, static=png_or_svg_bytes)` gives the report your own image instead.
- Zoom and hidden traces aren't saved; a re-run draws the figure from its spec again.

## Rust

Rust cells (ADR-0006) work like Python cells. They are compiled to WebAssembly and run sandboxed, with no files, network or clock.

```rust
// Reads L and w (calc quantities). Defines M and x.
let M = (w * L * L / 8.0).to("kip*ft");
let x: Vec<f64> = (0..=50).map(|i| L.si() * i as f64 / 50.0).collect();
println!("M = {M}");
```

**Names**
- Top-level `let` bindings, `const` and `static` are defines.
- Free names the cell uses are reads, including inside `println!`/`format!` (and `{name}` captures).
- Functions written in the cell are local helpers. They can't use the cell's variables; pass them in.

**Inputs arrive typed:**
- a calc quantity with a unit → `Quantity`;
- a plain number → `f64`;
- a vector → `Vec<f64>`, or `Vec<Quantity>` with a unit;
- a matrix → `Vec<Vec<f64>>`;
- true/false → `bool`, text → `String`.

Frames and complex numbers can't be passed yet. A Python cell's value reaches a Rust cell (and the other way round) only when it converts like a calc value.

**`Quantity`** holds an SI value and its dimensions, and is `Copy`.
- `*`, `/` and powers combine dimensions; `+`, `-` and comparisons require equal dimensions, and panic otherwise (the error points at your line).
- `q.to("kip*ft")` sets the display unit, from Cruncha's unit table; `q.value()` is the number in that unit, and `q.si()` the SI value.
- `Quantity::new(12.0, "kip")` makes one.

**Outputs:** each define goes back:
- `Quantity` with its unit;
- numbers, `bool`, `String`;
- `Vec<f64>`, `Vec<Quantity>`, `Vec<Vec<f64>>`.

Any other type is kept by name only: calcs and other cells can't use it.

**Running:**
- The same trust rule as Python applies: a document runs Rust cells only where it is trusted, and an edited cell waits for Run.
- The first build after setting up Rust (Settings, or the banner) already happened during setup. A changed cell compiles in about a second; an unchanged one loads in under a millisecond.
- Compile errors and panics show at the cell's line.
- A cell that runs too long can be stopped.
- Runs are stored with the document like Python's, keyed by the toolchain version.
- Crates beyond the prelude (`std`, `Quantity`) aren't available yet.

## Checklist

- [ ] Defined names are unique in the document and descriptive (`candidates`, not `df2`).
- [ ] Units survive the round trip. Don't hand back bare floats where a quantity belongs.
- [ ] Key results are surfaced in calc cells, so they appear in the report with an echo.
- [ ] No hidden I/O and no dynamic globals.
- [ ] Every cell imports what it uses (imports aren't shared between cells).
