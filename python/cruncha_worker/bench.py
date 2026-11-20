# SPDX-License-Identifier: GPL-3.0-or-later
"""Spike S4: receive a frame as an Arrow IPC stream with column units in field metadata
(``cruncha.unit``), turn it into a Polars frame, compute a column, and send it back the
same way. Timings in the response."""

import io
import time


def arrow_roundtrip(blobs):
    import polars as pl
    import pyarrow as pa
    import pyarrow.ipc as ipc

    t0 = time.perf_counter()
    table = ipc.open_stream(pa.py_buffer(blobs[0])).read_all()
    t1 = time.perf_counter()
    units = {f.name: (f.metadata or {}).get(b"cruncha.unit", b"").decode() for f in table.schema}
    df = pl.from_arrow(table)
    t2 = time.perf_counter()
    df = df.with_columns((pl.col("load") * pl.col("span") ** 2 / 8).alias("moment"))
    units["moment"] = "kip·ft" if units.get("load") == "kip/ft" and units.get("span") == "ft" else ""
    t3 = time.perf_counter()
    out_table = df.to_arrow()
    fields = [f.with_metadata({"cruncha.unit": units.get(f.name, "")}) for f in out_table.schema]
    out_table = out_table.cast(pa.schema(fields))
    sink = io.BytesIO()
    with ipc.new_stream(sink, out_table.schema) as w:
        w.write_table(out_table)
    t4 = time.perf_counter()
    ms = lambda a, b: (b - a) * 1000
    return {
        "rows": df.height,
        "units_in": units,
        "ms": {"read_ipc": ms(t0, t1), "to_polars": ms(t1, t2), "compute": ms(t2, t3), "write_ipc": ms(t3, t4)},
    }, [sink.getvalue()]
