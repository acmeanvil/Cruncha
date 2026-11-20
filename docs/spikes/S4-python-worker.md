# Spike S4: the Python worker

*Date: 2026-11-20. Code: `python/cruncha_worker` (the worker), `python/cruncha` (what cells import), `crates/cruncha-kernel-py` (process, protocol, environment; `examples/py_bench.rs`, `examples/setup_env.rs`), `python/tools/scan_check.py`. Machine: Intel i7-1068NG7 MacBook, macOS 26, rustc 1.99.1. Python 3.13.18 (python-build-standalone via uv 0.13.9), NumPy 2.5.5, Polars 2.1.0, PyArrow 26.0.1, pint 0.26.2, SciPy 1.18.1, matplotlib 3.11.2.*

ADR-0005 asked for two things: what passing data costs (an Arrow IPC round trip of a 1M-row Polars frame with units in metadata), and how reliable the AST dependency scan is.

## Method

- **Worker:** one Python process per document, answering length-prefixed JSON messages over stdin/stdout. Large payloads (images, Arrow IPC streams) follow a header as raw binary frames, with no base64. At start-up the worker duplicates fds 0 and 1 for the protocol, then points fd 1 at stderr and fd 0 at `/dev/null`, so nothing a cell or a C extension prints can corrupt the channel. A test writes to fd 1 from a cell to prove it.
- **Environment:** `uv venv --managed-python --python 3.13`, then `uv pip install --require-hashes --compile-bytecode -r python/requirements.lock`, all under the app's data directory. If uv is missing, the app installs it from PyPI into its own venv; Homebrew's uv would have meant building Rust from source on this macOS.
- **`py_bench`** (release build) measures:
  - start-up, as the median of 5;
  - the first import of each package;
  - scan and trivial-cell round trips, as the median of 50;
  - echoing 1, 16 and 128 MB blobs;
  - a 1,000,000-row × 3-column frame (`id`, `load` in kip/ft, `span` in ft; units in Arrow field metadata `cruncha.unit`). Rust encodes it as an IPC stream, Python reads it with PyArrow, converts it to Polars, adds `moment = load · span² / 8` with unit `kip·ft`, and writes it back. Rust decodes it and checks the units.
- **`scan_check.py`** compares the scan against an independent ground truth, the compiled bytecode. Defines are the names the module's own code object stores (`STORE_NAME`/`STORE_GLOBAL`, minus imports and names whose last top-level action is a delete). Reads are names loaded as globals in any nested code object (class bodies resolve `LOAD_NAME` in their own namespace first). It ran over every `.py` file of Python 3.13's standard library outside the test suites, treating each file as one cell.

```sh
cargo run -p cruncha-kernel-py --example setup_env
cargo run -p cruncha-kernel-py --release --example py_bench -- 1000000
<env>/bin/python -I python/tools/scan_check.py <stdlib dir>
cargo test -p cruncha-kernel-py           # needs the environment; skips without it
<env>/bin/python python/tests/test_worker.py
```

## Results

| Metric | Value |
| --- | --- |
| Environment setup (download and install) | 6.9 s on this network; 5.2 s from uv's cache with bytecode compiled; 514 MB environment + 70 MB interpreter |
| Worker start-up (to `hello`) | **51–53 ms** |
| First import, warm OS caches | NumPy 77–82 ms, Polars 179–189, PyArrow 220–239, pint + Cruncha registry 312–357, `scipy.optimize` 437–495, `matplotlib.pyplot` 408–526 |
| First import right after installing | `scipy.optimize` 26 s, `matplotlib.pyplot` 12.8 s, PyArrow 4.1 s, Polars 2.9 s (macOS checks each new native library once) |
| First import without precompiled bytecode | `scipy.optimize` 20.4 s **on every worker start** (the worker runs with `PYTHONDONTWRITEBYTECODE`). Fixed by `--compile-bytecode` |
| Scan of a 4-line cell | 0.24–0.54 ms |
| Run of `y = 1` (round trip) | 0.10–0.24 ms |
| Pipe throughput (each way) | 820–850 MB/s at 1, 16 and 128 MB, after reading into a preallocated buffer (it was 120–350 MB/s with a growing one) |
| 1M-row frame, 24 MB → 32 MB IPC | **126–221 ms round trip** (median of 3 per run). Python: read IPC 0.1 ms, to Polars 0.15 ms, compute 11 ms, write IPC 23 ms. Rust: encode 30 ms, decode 5 ms |
| Units through Arrow metadata | survived both ways (`kip/ft`, `ft` in, `kip·ft` back) |
| **Scan vs. bytecode** (633 files, 311,006 lines) | **98.6 % exact** (624 files). Defines differ in 5 files and reads in 4. Scan time 6.6 ms per file |

## Findings

1. **Pipes are fast enough; shared memory can wait.** A 1M-row frame costs about a fifth of a second for the round trip, almost all of it copying 56 MB through pipes. ADR-0005 proposed shared memory. It would save about 0.1 s here, at the cost of platform-specific code and lifetime management. Revisit when a real workload moves hundreds of megabytes per run.
2. **Precompiled bytecode is mandatory.** Without it every worker start recompiles SciPy (20 s). Install with `--compile-bytecode`.
3. **The first launch after installing is slow** (26 s for SciPy), because macOS checks each new native library the first time it loads. It happens once per install. Setup could warm it up by importing each package once in the background before reporting "ready".
4. **The scan needed three fixes found by the comparison.**
   - Since Python 3.12 the symbol table reports comprehension variables at module level (PEP 709 inlining). They are not defines, but a walrus (`:=`) inside a comprehension is.
   - Names deleted at the top level (`del k, v`) are not defines. Neither are `except … as e` names.
   - `symtable` marks some parameters as global (stdlib `poplib`); parameters are never reads.

   Defines now come from walking the syntax tree, and reads from `symtable`.
5. **The remaining scan differences are dynamic cases**: `global` statements inside functions (warned about), conditional `del`, a `type` alias. For calculation cells these are rare and the warning covers them. `# cruncha: defines/reads` overrides the scan.
6. **Imports are local to each cell.** With imports as defines, `import numpy as np` in two cells would break rule 1 (one definition per name). Every cell imports what it uses; imports of cached modules are cheap.
7. **A fresh namespace per cell** (holding only what it reads) makes the dependency graph the whole truth: a name the scan didn't see is a `NameError`, never silently stale. Values are shared by reference between cells, so mutating an input changes it for its other readers. The skill tells authors not to.
8. **Interrupts:** SIGINT raises `KeyboardInterrupt` in the cell, and the worker survives. Code that blocks SIGINT (tested with `pthread_sigmask` plus `sleep`) gets the worker killed after a grace period. Every Python value is then recomputed.

## Decisions taken in the implementation

These are recorded in ADR-0005.

- **Two workers per document.** One runs cells; one only scans sources, so saving a cell never waits behind a long run.
- **When cells run.** Python runs only in **trusted** documents: created on this computer, or confirmed with "Trust and run". A cell re-runs on its own when its inputs change. An **edited** cell waits for Run (Shift+Enter), so half-typed code never runs.
- **Results are memoized.** The key is the cell's source plus its inputs: calc values, and the keys of upstream Python cells.
- **Values travel as cruncha-eval's `Plain` JSON.** Exact rationals are sent as strings and units as name, factor, gauge and dimensions. Python values calcs can't use (functions, large arrays, models) stay in the worker, available to other Python cells.
- **Output is structured data** (PNG images, table rows, text), never HTML from the worker.

## Not done yet

- Sandboxing the worker (macOS `sandbox-exec`/App Sandbox, seccomp on Linux).
- User environments with extra packages, and storing the lockfile in the document.
- Column units for Polars frames on the Python side. The Arrow metadata path works; the Python API (`cruncha.units(df)`) is open.
- Calling Python functions from calcs, and SymPy as the symbolic engine.
- Keeping Python results with the history (the planned `results/` table).
- Interrupts on Windows (SIGINT is Unix-only here).
- Warming up imports after setup.
