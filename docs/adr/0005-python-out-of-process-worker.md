# ADR-0005: Python cells run in an out-of-process worker

**Status:** Accepted
**Date:** 2026-11-18
**Deciders:** acmeanvil

## Context

Python cells need NumPy, SciPy, Polars, pint, SymPy and matplotlib, and they exchange values (quantities, arrays, frames) with the Rust engine. Python can crash the interpreter (C extensions), hang, or run for minutes. The Python worker also hosts SymPy, our first symbolic engine.

## Decision

Use **one Python worker process per document** (plus a second one that only scans sources):
- **Runtime:** a `python-build-standalone` CPython 3.13 that `uv` installs, with the packages pinned in `python/requirements.lock` (with hashes, bytecode precompiled), under the app's data directory. One environment per lockfile. If uv is missing, the app installs it from PyPI. Optional user `uv` environments come later.
- **Channel:** length-prefixed JSON messages over the worker's stdin/stdout, with binary payloads (images, Arrow IPC streams) as raw frames. *Revised after spike S4:* the proposal was shared memory, but pipes move about 830 MB/s and a 1M-row frame round-trips in about 0.2 s, so shared memory is deferred until a workload needs it.
- **Values:** calc values travel as cruncha-eval's `Plain` JSON. Quantities become `pint.Quantity` from Cruncha's registry (`cruncha.ureg`), exact values `fractions.Fraction`, vectors and matrices NumPy arrays, frames `polars.DataFrame`. Bulk frames use Arrow IPC with column units in field metadata (`cruncha.unit`). Python values that calcs can't use stay in the worker for other Python cells.
- **Dependencies:** the worker's `ast` scan. Names bound at the top level are defines (imports are local to each cell, names starting with `_` are private), and free names read are reads. A `# cruncha: defines …` / `# cruncha: reads …` comment overrides the scan for dynamic code, which gets a warning.
- **Isolation of cells:** each cell runs in a fresh namespace holding only what it reads, so the dependency graph is the whole truth.
- **When code runs:** only in documents this computer trusts (created here, or confirmed by the user). A cell re-runs on its own when its inputs change, but an edited cell waits for an explicit Run. Results are memoized by source and inputs. Stop sends SIGINT; a worker that ignores it is killed and restarted.
- **Output:** structured data (PNG figures, table rows, text), never HTML from the worker.

## Options Considered

### Option A: Embedded CPython through PyO3 (in-process)
| Dimension | Assessment |
| --- | --- |
| Complexity | Medium |
| Cost | Lowest latency; zero-copy Arrow through the C Data Interface |
| Scalability | One interpreter per process (the GIL); shared fate with the app |
| Team familiarity | Medium |

**Cons:** a segfault in a C extension kills the app; real cancellation is hard; no sandbox.

### Option B: Out-of-process worker (proposed)
**Pros:** crash isolation; kill-to-cancel; can be sandboxed; one environment per document.
**Cons:** IPC overhead (to be measured in S4); process startup cost (mitigated by a warm pool).

### Option C: Pyodide (WASM) everywhere
**Pros:** sandboxed; the same runtime as the web build.
**Cons:** slow cold start; no native SciPy performance; package limits.

## Trade-off Analysis

Engineering users will run heavy SciPy and Polars work, so isolation and cancellation matter more than microseconds. Pyodide remains the runtime for the web build only.

## Consequences

- We need a worker protocol crate (`cruncha-kernel-py`, now in place) and environment management (`cruncha_kernel_py::env`).
- The pip-installable `cruncha` package reuses the same worker code for headless runs.

## Spike S4 results (2026-11-20)

From [docs/spikes/S4-python-worker.md](../spikes/S4-python-worker.md):

| Metric | Value |
| --- | --- |
| Worker start-up | 51–53 ms; first imports with warm OS caches 0.08–0.5 s per package |
| Trivial cell round trip | 0.10–0.24 ms; scan of a short cell 0.24–0.54 ms |
| Pipes | 820–850 MB/s each way |
| 1M-row × 3-column frame with units, Arrow IPC both ways | 126–221 ms round trip; units survive |
| Scan vs. bytecode, Python 3.13 stdlib (633 files) | 98.6 % exact; the rest are `global` statements (warned), conditional `del`, a `type` alias |
| Without precompiled bytecode | `scipy.optimize` takes 20 s to import on every worker start, so install with `--compile-bytecode` |

## Action Items

1. [x] Spike S4: an Arrow IPC round trip of a 1M-row Polars frame with units; reliability of the AST dependency scan.
2. [x] Sandbox the worker (macOS App Sandbox / `sandbox-exec`, seccomp on Linux). Done on macOS 2026-11-21 (`cruncha-kernel-py/src/sandbox.rs`): deny-by-default `sandbox-exec` profile; reads only the environment, the worker package and system libraries and fonts; writes only a scratch directory; no network or subprocesses. Seccomp on Linux remains.
3. [ ] User environments with extra packages; the lockfile stored in the document.
4. [ ] Column units for Polars frames on the Python side; calling Python functions from calcs; SymPy as the symbolic engine.
5. [x] Keep Python results with the history (the `results/` table).
