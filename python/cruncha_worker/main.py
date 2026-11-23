# SPDX-License-Identifier: GPL-3.0-or-later
"""The worker loop: read a request, answer it, repeat.

Requests (``op``): ``hello``, ``scan``, ``rename``, ``run``, ``drop``, ``reset``, ``echo`` (protocol
tests and benchmarks), ``shutdown``. Every response carries the request's ``id`` and ``ok``.

A cell runs in a **fresh namespace** holding only what it reads: values from calcs (sent with
the request) and values defined by other Python cells (kept here, in ``STORE``). What it
defines goes back into ``STORE``. So a cell can't depend on anything the graph doesn't know.
Values are shared by reference: a cell that mutates an array it read changes it for others.

SIGINT (sent by the app to cancel) raises ``KeyboardInterrupt`` in the running cell; the
worker reports it as ``interrupted`` and keeps running.
"""

import ast
import builtins
import contextlib
import io
import os
import signal
import sys
import time
import traceback

from . import PROTOCOL_VERSION, figures, outputs, protocol, rename, scan, values

STORE = {}
MAX_CAPTURE = 200_000


def _channels():
    """The protocol's binary stdin/stdout; afterwards fd 0 reads nothing and fd 1 is stderr,
    so prints from C extensions can't corrupt the channel."""
    inp = os.fdopen(os.dup(0), "rb", buffering=0)
    out = os.fdopen(os.dup(1), "wb")
    devnull = os.open(os.devnull, os.O_RDONLY)
    os.dup2(devnull, 0)
    os.dup2(2, 1)
    sys.stdin = open(os.devnull)
    return inp, out


class _Cap(io.StringIO):
    def write(self, s):
        if self.tell() < MAX_CAPTURE:
            return super().write(s)
        return len(s)


def _cell_frames(tb, cell):
    """Traceback lines from the cell's own code only."""
    frames = [f for f in traceback.extract_tb(tb) if f.filename == f"<cell {cell}>"]
    return frames


def _error(e, cell, source):
    lines = source.splitlines()
    if isinstance(e, SyntaxError) and e.filename == f"<cell {cell}>":
        return {"type": "SyntaxError", "message": e.msg, "line": e.lineno, "traceback": ""}
    frames = _cell_frames(e.__traceback__, cell)
    line = frames[-1].lineno if frames else None
    tb = []
    for f in frames:
        code = lines[f.lineno - 1].strip() if f.lineno and 0 < f.lineno <= len(lines) else ""
        where = "" if f.name == "<module>" else f" in {f.name}"
        tb.append(f"  line {f.lineno}{where}: {code}")
    tb.append("".join(traceback.format_exception_only(type(e), e)).strip())
    return {"type": type(e).__name__, "message": str(e), "line": line, "traceback": "\n".join(tb)}


def run(req, blobs_out):
    cell, source = req["cell"], req["source"]
    defines = req.get("defines", [])
    ns = {"__name__": "__cruncha__", "__builtins__": builtins}
    started = time.perf_counter()
    figures.begin(cell)
    for name in req.get("python_inputs", []):
        if name not in STORE:
            return {"status": "error", "error": {"type": "NameError", "message": f"{name} hasn't been computed (its cell hasn't run)", "line": None, "traceback": ""}}
        ns[name] = STORE[name]
    try:
        for name, plain in req.get("inputs", {}).items():
            ns[name] = values.from_plain(plain)
    except Exception as e:
        return {"status": "error", "error": {"type": type(e).__name__, "message": f"couldn't pass a calc value to Python: {e}", "line": None, "traceback": ""}}
    for name in defines:
        STORE.pop(name, None)

    out, err = _Cap(), _Cap()
    last = None
    status, error = "done", None
    try:
        tree = ast.parse(source, f"<cell {cell}>", "exec")
        tail = None
        if tree.body and isinstance(tree.body[-1], ast.Expr):
            tail = ast.Expression(tree.body.pop().value)
        with contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
            exec(compile(tree, f"<cell {cell}>", "exec"), ns)
            if tail is not None:
                last = eval(compile(tail, f"<cell {cell}>", "eval"), ns)
    except KeyboardInterrupt:
        status, error = "interrupted", {"type": "KeyboardInterrupt", "message": "stopped", "line": None, "traceback": ""}
    except BaseException as e:  # SystemExit too: a cell can't end the worker
        status, error = "error", _error(e, cell, source)

    shown = []
    try:
        if status == "done" and last is not None and figures.offer(last):
            last = None
        shown = outputs.figures(blobs_out) + figures.collect(blobs_out)
        shown += outputs.display(last, blobs_out) if status == "done" else []
    except Exception as e:
        shown.append({"kind": "text", "text": f"(couldn't show output: {e})"})

    defined = []
    for name in defines:
        if status == "done" and name in ns:
            v = ns[name]
            STORE[name] = v
            try:
                plain = values.to_plain(v)
            except Exception as e:
                plain = {"type": "unsupported", "kind": type(v).__name__, "summary": f"(couldn't convert: {e})"}
            defined.append({"name": name, "summary": values.summary(v), "plain": plain})
        elif status == "done":
            status, error = "error", {"type": "NameError", "message": f"the cell didn't define {name}", "line": None, "traceback": ""}
    return {
        "status": status,
        "defines": defined,
        "stdout": out.getvalue(),
        "stderr": err.getvalue(),
        "outputs": shown,
        "error": error,
        "ms": (time.perf_counter() - started) * 1000,
    }


def handle(req, blobs_in):
    op = req.get("op")
    blobs_out = []
    if op == "hello":
        return {"protocol": PROTOCOL_VERSION, "python": sys.version.split()[0], "executable": sys.executable, "pid": os.getpid()}, blobs_out
    if op == "scan":
        return scan.scan(req["source"]), blobs_out
    if op == "rename":
        return rename.rename(req["source"], req["names"]), blobs_out
    if op == "run":
        return run(req, blobs_out), blobs_out
    if op == "drop":
        for n in req.get("names", []):
            STORE.pop(n, None)
        return {}, blobs_out
    if op == "reset":
        STORE.clear()
        return {}, blobs_out
    if op == "echo":
        return {"n": len(blobs_in)}, list(blobs_in)
    if op == "arrow_roundtrip":
        from . import bench

        return bench.arrow_roundtrip(blobs_in)
    raise ValueError(f"unknown op {op!r}")


def main():
    outputs.setup()
    figures.install()
    inp, out = _channels()
    signal.signal(signal.SIGINT, signal.default_int_handler)
    while True:
        try:
            req, blobs = protocol.read_message(inp)
        except protocol.Closed:
            return
        except KeyboardInterrupt:
            continue  # an interrupt that arrived between cells
        if req.get("op") == "shutdown":
            protocol.write_message(out, {"id": req.get("id"), "ok": True})
            return
        try:
            res, blobs_out = handle(req, blobs)
            protocol.write_message(out, {"id": req.get("id"), "ok": True, **res}, blobs_out)
        except KeyboardInterrupt:
            protocol.write_message(out, {"id": req.get("id"), "ok": False, "error": "interrupted"})
        except Exception as e:
            protocol.write_message(out, {"id": req.get("id"), "ok": False, "error": f"{type(e).__name__}: {e}"})


if __name__ == "__main__":
    main()
