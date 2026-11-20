# SPDX-License-Identifier: GPL-3.0-or-later
"""Rich output of a cell: matplotlib figures (PNG), tables (Polars frames) and text reprs.

Outputs are structured data, never HTML: the app renders them itself, so nothing a cell
prints can inject markup into the editor."""

import sys

TABLE_ROWS = 50


def setup():
    """Headless matplotlib, if it gets imported."""
    import os

    os.environ.setdefault("MPLBACKEND", "Agg")


def figures(blobs):
    """PNG images of the figures the cell left open (then closes them)."""
    plt = sys.modules.get("matplotlib.pyplot")
    if plt is None:
        return []
    out = []
    import io

    for num in plt.get_fignums():
        fig = plt.figure(num)
        buf = io.BytesIO()
        fig.savefig(buf, format="png", dpi=144, bbox_inches="tight")
        w, h = fig.get_size_inches() * 144
        blobs.append(buf.getvalue())
        out.append({"kind": "image", "mime": "image/png", "blob": len(blobs) - 1, "width": int(w), "height": int(h)})
    plt.close("all")
    return out


def table(df):
    """The first rows of a Polars frame as columns and strings."""
    head = df.head(TABLE_ROWS)
    return {
        "kind": "table",
        "columns": [{"name": n, "dtype": str(t)} for n, t in zip(df.columns, df.dtypes)],
        "rows": [[None if x is None else str(x) for x in row] for row in head.iter_rows()],
        "total_rows": df.height,
    }


def display(value, blobs):
    """Output for the value of a cell's last expression."""
    if value is None:
        return []
    mod = type(value).__module__.split(".")[0]
    if mod == "polars" and hasattr(value, "head") and hasattr(value, "height"):
        return [table(value)]
    if mod == "matplotlib":
        return []  # figures are collected anyway
    r = repr(value)
    if len(r) > 20_000:
        r = r[:20_000] + "…"
    return [{"kind": "text", "text": r}]
