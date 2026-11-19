# ADR-0005: Python cells run in an out-of-process worker

**Status:** Proposed
**Date:** 2026-11-18
**Deciders:** acmeanvil

## Context

Python cells need NumPy, SciPy, Polars, pint, SymPy and matplotlib, and they exchange values (quantities, arrays, frames) with the Rust engine. Python can crash the interpreter (C extensions), hang, or run for minutes. The Python worker also hosts SymPy, our first symbolic engine.

## Decision

Use **one Python worker process per document**:
- The runtime is a bundled `python-build-standalone` interpreter, plus optional user `uv` environments.
- Values travel as **Arrow IPC over shared memory**. Frames become `polars.DataFrame` with column units in field metadata; quantities become `pint.Quantity`; exact values become `fractions.Fraction`.
- Dependencies are found with an `ast` scan of top-level assignments and free reads. A `# cruncha: defines …` comment overrides the scan for dynamic code.

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

- We need a worker protocol crate (`cruncha-kernel-py`) and environment management.
- The pip-installable `cruncha` package reuses the same worker code for headless runs.

## Action Items

1. [ ] Spike S4: an Arrow IPC round trip of a 1M-row Polars frame with units; reliability of the AST dependency scan.
