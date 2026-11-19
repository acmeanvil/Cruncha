# Cruncha

An AI-forward engineering calculation and report tool. It combines Mathcad-style live, unit-aware, reviewable math with Jupyter-style Python (NumPy, SciPy, Polars) and Rust code blocks in one reactive document. Licensed GPL-3.0-or-later.

Status: early. Only `crates/cruncha-store` exists. Everything else is design.

## Read first

- `docs/adr/`: accepted and proposed decisions. **Check these before proposing an architectural change.**
- `docs/design/02-document-model.md`: the document, execution and storage model. Planned crates are listed in §7, spikes in §8.
- `docs/research/01-landscape-survey.md`: why we use which open-source pieces.

## Layout

```
Cargo.toml                 workspace (edition 2024, resolver 3)
crates/cruncha-store/      Delta Lake document container (ADR-0003)
  src/cell.rs              Cell, CellKind, Change, Provenance, CellDiff
  src/schema.rs            Arrow schemas + row <-> struct conversion
  src/store.rs             Store: create/open/lock, commit (MERGE), time travel, history, diff, tag, restore, compact
  examples/store_bench.rs  spike S3 benchmark
  examples/store_variants.rs  spike S3b: merge vs append-only vs bucketed layouts
  tests/                   integration tests (tempdir containers)
docs/spikes/               spike write-ups with measured numbers
docs/adr/  docs/design/  docs/research/
skills/                    Agent Skills (SKILL.md) for AI agents working inside Cruncha documents
```

## Commands

```sh
cargo build -p cruncha-store
cargo test  -p cruncha-store
cargo run   -p cruncha-store --release --example store_bench -- <commits>
cargo run   -p cruncha-store --release --example store_variants -- <merge|append|bucketed|all> [commits] [cells] [squash_every]
cargo test  -p cruncha-store --test crash      # SIGKILLs a writer process 8 times (~20 s)
cargo clippy --all-targets -- -D warnings
```

The first build compiles DataFusion and takes about 10 minutes on this machine. Dependencies are built at `opt-level = 2`, even in dev builds. Rust here comes from Homebrew (no `rustup`), so extra targets such as `wasm32-wasip2` aren't installed yet.

## Conventions

- Every new source file starts with `// SPDX-License-Identifier: GPL-3.0-or-later` (or `#` for Python).
- **Dependency licenses must be GPL-3 compatible:** MIT, BSD, Apache-2.0, ISC, MPL-2.0, LGPL, and GPL "v2-or-later" or v3. **Never** GPL-2.0-only, proprietary, or non-commercial/source-available licenses (for example, Symbolica). Check before adding a crate or package.
- Ported Hurmet code or data (MIT) keeps Hurmet's copyright notice.
- `deltalake` is used with `default-features = false, features = ["datafusion", "native-tls"]`. deltalake-core refuses to build without a TLS backend, and `native-tls` avoids building aws-lc from C. Use its re-exports (`deltalake::arrow`, `deltalake::datafusion`) instead of adding separate `arrow` or `datafusion` dependencies, so the versions stay in lockstep.
- Use async APIs on tokio.

## Store invariants (do not break)

- **History is never rewritten.** Rollback is a new commit (`Store::restore`). Never use Delta `RESTORE` or `VACUUM` on `doc/` without the retention work in ADR-0003's action items.
- Container tables are created with expired-log cleanup **disabled**. Delta's default 30-day log retention would silently destroy time travel. `Store::history` relies on every version being present.
- There is a single writer per container, enforced by the `.lock` file through `File::try_lock`. Other writers (UI, MCP agents, CLI) must go through the process that holds the `Store`.
- One `Store::commit` is one atomic Delta version. Batch changes (on cell submit or idle), never per keystroke.
- Results do not live in `doc/`. They go in a separate `results/` table (planned).
- Table properties include `delta.dataSkippingNumIndexedCols = 1` (statistics on `id` only; spike S3b).
- Commit metadata keys are `cruncha.actor`, `cruncha.message`, `cruncha.kind` and `cruncha.restoredFrom` (see `store.rs`).

## Decisions and docs as we go

- Write an ADR (`docs/adr/NNNN-title.md`, using the template in `docs/adr/README.md`) for any significant, hard-to-reverse choice: a new dependency with lock-in, a file-format change, a semantics change, or a license-affecting change. Add it to the index table. Proposals start as **Proposed**. Only the project owner moves an ADR to **Accepted**.
- Accepted ADRs are immutable. Supersede them with a new ADR rather than editing.
- When a spike finishes, record its numbers in the relevant ADR or design doc.
- `skills/` is the agent-facing spec of the calc syntax and the MCP tools. When an implementation changes syntax, tools or semantics, update the matching skill **in the same change**.
- The agent interface reuses the user's own `~/Developer/looking_glass` (MIT; `lg-acp` and `lg-agent`). Keep the MIT notice on anything ported from it (ADR-0009).
- Keep this file current when layout, commands or invariants change.
