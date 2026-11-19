# ADR-0006: Rust cells compile to WASM and run in wasmtime

**Status:** Proposed
**Date:** 2026-11-18
**Deciders:** acmeanvil

## Context

Users want Rust blocks for performance-critical or reusable logic, and functions defined in them should be callable from calc cells. Compiling and loading native code from a document is a security risk: opening a document must not run arbitrary native code. Rust also has no stable ABI.

## Decision

- Rust cells compile with `cargo` to **`wasm32-wasip2`** and run in **wasmtime**, cached by source hash.
- A `cruncha-prelude` crate provides `Quantity`, `Array`, an Arrow `Frame` view, and `#[cruncha::export]`. Exported functions and statics become graph names.
- Dependencies come from a per-document allowlist of crates that build for WASM.
- **Native `cdylib` builds** are a later opt-in, behind an explicit "trust this document" prompt.

## Options Considered

### Option A: Native dylib (`cdylib`) loaded with `libloading`
**Pros:** full speed and any crate.
**Cons:** arbitrary code runs on open; no ABI stability; platform-specific toolchains.

### Option B: WASM plus wasmtime (proposed)
**Pros:** sandboxed, deterministic, portable, and runs the same in the web build.
**Cons:** compile latency of seconds; a smaller set of usable crates; about 1.5–3× slower than native.

### Option C: An interpreted Rust subset (for example `rhai` or `evcxr`)
**Pros:** instant feedback.
**Cons:** not real Rust (rhai), or native and unsafe (evcxr).

## Consequences

- Requires a Rust toolchain with the WASM target on the user's machine, or a bundled one. We need a "compiling…" UX.

## Action Items

1. [ ] Spike S5: compile time and call overhead for a prelude-based cell.
