# Spike S5: Rust cells as WASM components

*Date: 2026-11-21. Code: `crates/cruncha-kernel-rs` (`wit/cell.wit`, `guest/cruncha-prelude`, `src/lib.rs`, `examples/rs_bench.rs`, `tests/cell.rs`). Machine: Intel i7-1068NG7 MacBook, macOS 26. Toolchain: rustc 1.99.1 with `wasm32-wasip2`, wasmtime 51 (Cranelift), wit-bindgen 0.57.*

ADR-0006 asked: is compiling a Rust cell to WASM and running it in wasmtime fast enough for an interactive document, and does the sandbox hold?

## Toolchain on this machine

| Where | What | Fit |
| --- | --- | --- |
| `/usr/local/Cellar/rust/1.99.1` (on PATH) | Homebrew rustc | no WASM targets, no rustup to add one |
| `/opt/local/bin/rustc` | MacPorts rustc 1.98.1 (source build) | no WASM targets |
| `~/Library/Caches/puccinialin/` | a full rustup install (rustup 1.29.1, stable 1.99.1), a Python build helper's cache | can add targets, but its owner may clear it |
| `~/.cargo/bin` | rustup's links without the `rustup` binary | broken leftovers |

Cruncha got **its own toolchain**, as the app has its own Python:
- `RUSTUP_HOME`/`CARGO_HOME` under `~/Library/Application Support/io.github.acmeanvil.cruncha/rust`;
- rustc 1.99.1, minimal profile, plus `wasm32-wasip2`;
- 651 MB, all prebuilt downloads;
- installed with the cached `rustup-init`;
- PATH, Homebrew and the shell profile untouched.

## Method

- A cell is a crate written to disk: `cdylib`, depending on the guest prelude (wit-bindgen bindings for `wit/cell.wit`, serde_json, Plain helpers). All cells share one target directory, so the dependencies compile once.
- The host compiles the component with Cranelift and instantiates it with an **empty WASI context**: no preopened directories, no environment, no sockets, stdout and stderr discarded.
- `rs_bench` builds cold (empty target directory), then after four edits (release and debug). It then compiles, caches and loads the component, instantiates it, and times three kinds of call:
  - one with no data;
  - one passing a 100k-element `list<f64>`;
  - the JSON `run` with two inputs, the Plain exchange format the Python cells use.
- `tests/cell.rs`:
  - a cell that tries to read `/etc/hosts`, read `HOME` and open a TCP connection;
  - a cell stuck in an infinite loop, stopped by an epoch deadline.

## Results

| Measure | Value |
| --- | --- |
| Cold build, release (prelude, wit-bindgen, serde_json and the cell) | 30.5 s, once per toolchain |
| Cold build, debug | 36.2 s |
| Rebuild after editing the cell, release | 0.80–0.84 s |
| Rebuild after editing the cell, debug | 0.45–0.64 s |
| Component size, release | 187 KB |
| Cranelift compile of the component | 52 ms |
| Precompile for the cache / load it back | 43 ms (426 KB) / 0.55 ms |
| Instantiate | 0.09 ms (first 0.29 ms) |
| Call, no data | 0.97 µs |
| Call with 100k `f64` | 0.16 ms (≈ 1.6 ns per element) |
| `run`, JSON with two inputs | 7.3 µs |
| Read a file, read `HOME`, connect to a socket | all fail inside the cell |
| Infinite loop with a 200 ms deadline | trapped at the deadline; a fresh instance works |

## Conclusions

1. **The UX is acceptable.**
   - An edited cell is ready in under a second: rebuild plus Cranelift is about 0.9 s in release, 0.5 s in debug.
   - Reopening a document with an unchanged cell loads its cached code in under a millisecond.
   - The 30 s cold build happens once per toolchain or prelude version, not per document. It could be done in the background when Rust is set up.
2. **Calls are cheap** next to the calc engine: about 1 µs, and arrays at memory speed. The cost is JSON, about 7 µs for a small call. Calling Rust functions from calcs per element is therefore fine for scalars; large arrays should go as `list<f64>`, not JSON.
3. **The sandbox holds with no extra work.** WASI 0.2 with an empty context gives no files, environment or network. Epoch interruption stops runaway cells, at the cost of a new instance.
4. **Dependencies:** each crate allowed for cells adds to the one-time cold build. The allowlist (ADR-0006) should start with the prelude only.
5. **Still to do** for Rust cells (after ADR-0006 is accepted):
   - toolchain setup in the app: rustup-init from static.rust-lang.org, checksum-verified, like `uv` for Python;
   - `#[cruncha::export]` and the scan of defines and reads;
   - `Quantity` and arrays with units in the prelude;
   - the engine's job queue and results (ADR-0018 already keys by kind and source);
   - compile errors mapped to cell lines.
