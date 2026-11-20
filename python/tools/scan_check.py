# SPDX-License-Identifier: GPL-3.0-or-later
"""Spike S4: check the dependency scan against an independent ground truth, the bytecode.

For each file (treated as one cell): the names the compiled module code stores at top level
(``STORE_NAME``/``STORE_GLOBAL`` in the module's own code object; inlined comprehension
variables use fast locals, so they don't count) are its defines, and the names loaded as
globals anywhere (``LOAD_NAME``/``LOAD_GLOBAL`` in the module and every nested code object)
that aren't defines, imports or builtins are its reads. Imports and private names are
excluded on both sides, as the scan does. Usage::

    <python> python/tools/scan_check.py <file-or-dir>...
"""

import dis
import os
import sys
import time
import types

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
from cruncha_worker.scan import BUILTINS, scan  # noqa: E402

IMPORTS = {"IMPORT_NAME", "IMPORT_FROM"}


def truth(source):
    code = compile(source, "<cell>", "exec")
    stores, imported, deleted, loads = set(), set(), set(), set()
    prev = None
    for ins in dis.get_instructions(code):
        if ins.opname in ("STORE_NAME", "STORE_GLOBAL"):
            (imported if prev in IMPORTS else stores).add(ins.argval)
        elif ins.opname == "DELETE_NAME":
            deleted.add(ins.argval)  # e.g. `except E as e` deletes `e` when the handler ends
        prev = ins.opname if ins.opname not in ("SWAP", "COPY", "POP_TOP") else prev
    # A name exists after the cell when its last top-level store isn't followed by a delete.
    last = {}
    for ins in dis.get_instructions(code):
        if ins.opname in ("STORE_NAME", "STORE_GLOBAL", "DELETE_NAME"):
            last[ins.argval] = ins.opname
    every_store = set(stores)
    stores = {n for n in stores if last.get(n) != "DELETE_NAME"}

    def walk(co, is_class):
        own = {i.argval for i in dis.get_instructions(co) if i.opname == "STORE_NAME"} if is_class else set()
        for ins in dis.get_instructions(co):
            # In a class body LOAD_NAME looks in the class namespace first.
            if ins.opname in ("LOAD_NAME", "LOAD_GLOBAL", "LOAD_FROM_DICT_OR_GLOBALS") and ins.argval not in own:
                loads.add(ins.argval)
        for c in co.co_consts:
            if isinstance(c, types.CodeType):
                # Class bodies (and annotation scopes) are compiled with LOAD_NAME semantics.
                walk(c, not (c.co_flags & 0x1))  # CO_OPTIMIZED is unset for class bodies

    walk(code, False)
    defines = {n for n in stores - imported if not n.startswith("_")}
    reads = {n for n in loads - every_store - imported - BUILTINS if not n.startswith("_")}
    return defines, reads, imported


def files(paths):
    for p in paths:
        if os.path.isdir(p):
            for root, dirs, fs in os.walk(p):
                dirs[:] = [d for d in dirs if d not in ("test", "tests", "idle_test", "__pycache__", "site-packages")]
                for f in fs:
                    if f.endswith(".py"):
                        yield os.path.join(root, f)
        else:
            yield p


def main():
    n = ok = skipped = 0
    bad_def, bad_read = [], []
    elapsed = 0.0
    lines = 0
    for path in files(sys.argv[1:]):
        try:
            with open(path, encoding="utf-8") as f:
                src = f.read()
            d_true, r_true, imported = truth(src)
        except (SyntaxError, UnicodeDecodeError, ValueError):
            skipped += 1
            continue
        t = time.perf_counter()
        s = scan(src)
        elapsed += time.perf_counter() - t
        lines += src.count("\n") + 1
        n += 1
        if "error" in s:
            bad_def.append((path, "scan error", s["error"]))
            continue
        d, r = set(s["defines"]), set(s["reads"]) - imported
        good = True
        if d != d_true:
            bad_def.append((path, sorted(d - d_true), sorted(d_true - d)))
            good = False
        if r != r_true:
            bad_read.append((path, sorted(r - r_true), sorted(r_true - r)))
            good = False
        ok += good
    print(f"{n} files ({lines:,} lines), {skipped} skipped; exact match: {ok} ({100 * ok / max(n, 1):.1f} %)")
    print(f"defines differ: {len(bad_def)}; reads differ: {len(bad_read)}; scan time {elapsed:.2f} s ({1000 * elapsed / max(n, 1):.2f} ms/file)")
    for title, bad in (("defines (scan-only, bytecode-only)", bad_def), ("reads (scan-only, bytecode-only)", bad_read)):
        if bad:
            print(f"\n{title}:")
            for path, extra, missing in bad[:15]:
                print(f"  {os.path.relpath(path, sys.argv[1]) if os.path.isdir(sys.argv[1]) else path}: +{extra} -{missing}")


if __name__ == "__main__":
    main()
