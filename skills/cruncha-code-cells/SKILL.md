---
name: cruncha-code-cells
description: How to write Python and Rust cells in a Cruncha document so they join its reactive graph. Covers how defined and read names are detected, how values convert between calc cells and Python (pint quantities, NumPy, Polars data frames with column units, Fractions), plots and rich output, and Rust cells compiled to WASM with exported functions. Use it when a calc needs NumPy, SciPy, Polars, plotting, iteration, or performance-critical code.
---

# Python and Rust cells

Code cells are part of the same dependency graph as calc cells (ADR-0004):
- A code cell **defines** the names it assigns at top level.
- It **reads** the free names it uses.
- Editing an input re-runs everything downstream.

There is no "run all" and no hidden kernel state.

## When to use a code cell rather than a calc

- **Use a calc** for engineering formulas a reviewer must read. The blue echo is the point.
- **Use Python** for data wrangling (Polars), numerical methods (SciPy solvers, ODEs, optimization), loops over many cases, and plots.
- **Use Rust** for heavy inner loops, or for reusable numeric functions that calc cells should call quickly.

When a code cell produces a key design value, also show it in a calc cell, for example `M_max = max_moment(...) = ?`. That way the report displays it with units and an echo.

## Python

```python
import polars as pl
import numpy as np

# Reads: Z_req (a calc-cell quantity) and shapes (a data frame). Defines: candidates, lightest.
candidates = shapes.filter(pl.col("Zx") >= Z_req.to("in^3").magnitude).sort("weight")
lightest = candidates.row(0, named=True)["name"]
```

**Rules**
- Assign results to top-level names. Those are what other cells can read. Names starting with `_` stay private to the cell.
- **No `exec`, `globals()` writes, or `from x import *`.** Static analysis can't see them, and the cell gets a warning. If you really need dynamic code, declare the names: `# cruncha: defines a, b`.
- Don't mutate a value that came from another cell (for example, `shapes` in place). Make a new name instead.
- Don't do network or file I/O, except through a `data` cell, which records the source and its version.
- Pin package choices to the document environment. `import` only what's there; propose adding a package rather than installing it ad hoc.

**Value conversion**

| Calc value | In Python | Back to calc |
| --- | --- | --- |
| Quantity `3.1 m` | `pint.Quantity` (Cruncha's registry), so use `.to("ft")` and `.magnitude` | `pint.Quantity` becomes a quantity |
| Exact number | `fractions.Fraction` | `Fraction`, `int` or `float` |
| Vector or matrix (with unit) | NumPy array wrapped in a pint quantity | same |
| Data frame | `polars.DataFrame`; column units in `cruncha.units(df)` / Arrow field metadata | `polars.DataFrame` (set units with `cruncha.with_units(df, {"A": "in^2"})`) |
| Function | callable | Python functions are callable from calc cells |

**Output:** the value of the last expression is displayed. Matplotlib or Plotly figures, Polars tables, and objects with `_repr_html_` render as rich output in the report.

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
