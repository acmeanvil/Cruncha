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
- Past versions don't keep Python output yet.

**Rules**
- Assign results to top-level names. Those are what other cells can read.
- Don't mutate a value that came from another cell (for example, `shapes` in place). Values are shared by reference, so a mutation would change it for its other readers. Make a new name instead.
- Don't do network or file I/O, except through a `data` cell, which records the source and its version.
- `import` only what the environment has: NumPy, SciPy, Polars, PyArrow, pint, SymPy, matplotlib (pinned in `python/requirements.lock`). Propose adding a package rather than installing it ad hoc.

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

**In the PDF report** a Python cell prints its source (unless Report settings turn that off), its figures, tables (first 50 rows), printed text, and the names it defines. A cell that hasn't run with its current source (Python not set up, not trusted, edited and not run) is printed as NOT CALCULATED, with the reason. Export waits up to two minutes for cells that are running.

**Output:** every matplotlib figure the cell left open (as an image; figures are closed afterwards), then the value of the last expression (Polars frames as a table of the first 50 rows, other values as their `repr`), then what it printed. Output is shown as data, never as HTML, so `_repr_html_` isn't used. Calling Python functions from calc cells is planned, not available yet.

## Rust

Rust cells compile to WASM and run sandboxed (ADR-0006). The first run compiles, which takes seconds; later runs are cached.

```rust
use cruncha::prelude::*;

/// Plastic section modulus of a solid rectangle.
#[cruncha::export]
fn z_rect(b: Length, h: Length) -> Volume {
    b * h * h / 4.0
}
```

- Only `#[cruncha::export]` items become names in the graph. Units are checked by the types (`Length`, `Force`, `Stress`, …), so a dimension error is a compile error.
- Use only crates on the document's allowlist (nalgebra, faer, statrs, …). There is no file system, network or clock.
- Call them from a calc as `Z = z_rect(b, h) = ?? in³`.

## Checklist

- [ ] Defined names are unique in the document and descriptive (`candidates`, not `df2`).
- [ ] Units survive the round trip. Don't hand back bare floats where a quantity belongs.
- [ ] Key results are surfaced in calc cells, so they appear in the report with an echo.
- [ ] No hidden I/O and no dynamic globals.
- [ ] Every cell imports what it uses (imports aren't shared between cells).
