# Cruncha

An AI-forward engineering calculation and report tool. It combines Mathcad-style live, unit-aware, reviewable math with Jupyter-style Python (NumPy, SciPy, Polars) and Rust code blocks in one reactive document. Licensed GPL-3.0-or-later.

Status: early. `crates/cruncha-store` (the container, with retention and out-of-line content), the Tauri app skeleton in `app/` (a read-only viewer plus history settings), and `crates/cruncha-lang` (spike S1: Hurmet's lexer and parser ported, TeX + RPN output) exist. Everything else is design.

## Read first

- `docs/adr/`: accepted and proposed decisions. **Check these before proposing an architectural change.**
- `docs/design/02-document-model.md`: the document, execution and storage model. Planned crates are listed in §7, spikes in §8.
- `docs/research/01-landscape-survey.md`: why we use which open-source pieces.

## Layout

```
Cargo.toml                 workspace (edition 2024, resolver 3)
crates/cruncha-store/      Delta Lake document container (ADR-0003)
  src/cell.rs              Cell, CellKind, Change, Provenance, CellDiff
  src/schema.rs            Arrow schemas + row <-> struct conversion (doc log, refs, pins)
  src/store.rs             Store: create/open/lock, commit (append + squash), time travel, history, diff, tag, restore, compact, prune
  src/retention.rs         Retention policy, `_delta_log` commit reader, cutoff computation (ADR-0011)
  src/blobs.rs             content-addressed, zstd-compressed `blobs/` for large cell sources (ADR-0013)
  examples/store_bench.rs  spike S3 benchmark
  examples/store_variants.rs  spike S3b: merge vs append-only vs bucketed layouts
  tests/                   integration tests (tempdir containers): store, retention, blobs, crash
crates/cruncha-lang/       Hurmet lexer + parser port (spike S1; MIT notice in LICENSE-HURMET): parse_tex, parse_calc
  tests/golden.rs          Hurmet's golden TeX/RPN tests (tests/fixtures/*.json); keep green when adding an AST
app/                       desktop app (ADR-0008, ADR-0012): Svelte 5 + Vite + TypeScript web UI
  src/App.svelte           shell: toolbar, history panel, cell view, dialogs
  src/lib/api.ts           typed wrappers for the backend's Tauri commands (keep in sync with lib.rs)
  src/lib/*.svelte         HistoryPanel, CellList, RetentionControl, Preferences, DocumentSettings, …
  src-tauri/               Rust backend (crate `cruncha-app`): src/lib.rs commands, src/session.rs state
docs/spikes/               spike write-ups with measured numbers
docs/adr/  docs/design/  docs/research/
skills/                    Agent Skills (SKILL.md) for AI agents working inside Cruncha documents
```

## Commands

```sh
cargo build -p cruncha-store
cargo test  -p cruncha-store
cargo run   -p cruncha-store --release --example store_bench -- <commits>
cargo run   -p cruncha-store --release --example store_variants -- <store|append|bucketed|all> [commits] [cells] [squash_every]
cargo test  -p cruncha-store --test crash      # SIGKILLs a writer 8 times mid-commit and 8 times mid-prune (~40 s)
cargo clippy --all-targets -- -D warnings
cargo test  -p cruncha-app                     # backend session logic, no window needed
cargo test  -p cruncha-lang                    # Hurmet golden tests (65/66 TeX + 217/217 RPN; 1 known gap)
cargo run   -p cruncha-lang --release --example parse_bench

cd app && npm install                          # once
npm run tauri dev                              # run the app (Vite on :5173 + the Rust backend)
npm run check                                  # svelte-check (TypeScript)
npm run build                                  # front end to app/dist
```

The first build compiles DataFusion and takes about 10 minutes on this machine. Dependencies are built at `opt-level = 2`, even in dev builds. Rust here comes from Homebrew (no `rustup`), so extra targets such as `wasm32-wasip2` aren't installed yet.

## Conventions

- Every new source file starts with `// SPDX-License-Identifier: GPL-3.0-or-later` (or `#` for Python).
- **Dependency licenses must be GPL-3 compatible:** MIT, BSD, Apache-2.0, ISC, MPL-2.0, LGPL, and GPL "v2-or-later" or v3. **Never** GPL-2.0-only, proprietary, or non-commercial/source-available licenses (for example, Symbolica). Check before adding a crate or package (`npm view <pkg> license` for npm).
- Ported Hurmet code or data (MIT) keeps Hurmet's copyright notice.
- `deltalake` is used with `default-features = false, features = ["datafusion", "native-tls"]`. deltalake-core refuses to build without a TLS backend, and `native-tls` avoids building aws-lc from C. Use its re-exports (`deltalake::arrow`, `deltalake::datafusion`) instead of adding separate `arrow` or `datafusion` dependencies, so the versions stay in lockstep.
- Use async APIs on tokio.
- The app process is the single writer of a document it has open: its `Session` holds the `Store`. Tauri commands are thin wrappers over `Session` methods, so logic is tested without a window. It prunes (with the document's effective policy) only when an edited document closes, or when the user confirms "Trim history now".

## Store invariants (do not break)

- **History is never rewritten.** Rollback is a new commit (`Store::restore`). Never use Delta `RESTORE`. Only `Store::prune` (ADR-0011) may `VACUUM` or expire logs, with explicit parameters derived from the document's policy, after pinning refs. **Never change Delta's retention table properties**: they act on wall-clock time during unrelated commits. A document with no new edits must never lose history: `prune` acts only after edit commits since its last marker, and the activity window counts back from the last edit (`cruncha.at`), never from today.
- `doc/` is an **append-only revision log** (ADR-0010; container format 3). A commit appends the changed rows with `rev = version + 1`, and the store fails loudly if the committed version differs. Reads reduce to the highest `rev` per id. Every 50 commits the log is squashed (overwritten with current rows); old files stay, so time travel still works. Older containers are migrated on open by adding nullable columns (no data rewritten).
- Sources of at least 1 KiB live out of line (ADR-0013): the row has `source_hash` (SHA-256) and a null `source`; the text is in `blobs/ab/cdef….zst`. Blobs are immutable and written **before** the commit that references them. Only `Store::prune` deletes blobs, and only those no data file left in `doc/` or `pins/` references. Keep the layout seam (`commit_with` appends, `maintain` squashes) so a deletion-vector layout (S3 option C) can replace those two steps; `reduce_log` already reads a one-row-per-id table.
- Container tables are created with expired-log cleanup **disabled**. Delta's default 30-day log retention would silently destroy time travel. `Store::history` reads `doc/_delta_log/NNN.json` directly, from `manifest.earliest_version` to the latest version, and fails loudly on a gap.
- Pruning order (each step crash-safe): pin refs older than the cutoff into `pins/` → checkpoint at the cutoff → write `earliest_version` to the manifest (point of no return) → VACUUM (Lite, explicit retention, fixed clock) → `cleanup_expired_logs_for(cutoff, …)` → delete unreferenced blobs → marker squash carrying `cruncha.prunedBefore`. An interrupted prune is finished by the next call.
- There is a single writer per container, enforced by the `.lock` file through `File::try_lock`. Other writers (UI, MCP agents, CLI) must go through the process that holds the `Store`.
- One `Store::commit` is one atomic Delta version (a squash, when due, adds a second, maintenance version). Batch changes (on cell submit or idle), never per keystroke.
- Results do not live in `doc/`. They go in a separate `results/` table (planned).
- Table properties include `delta.dataSkippingNumIndexedCols = 1` (statistics on `id` only; spike S3b).
- Commit metadata keys are `cruncha.actor`, `cruncha.message`, `cruncha.kind`, `cruncha.at` (µs; Cruncha's clock), `cruncha.restoredFrom`, `cruncha.prunedBefore` and `cruncha.blobBytes` (new blob bytes, counted by the size limit) (see `store.rs`).

## Decisions and docs as we go

- Write an ADR (`docs/adr/NNNN-title.md`, using the template in `docs/adr/README.md`) for any significant, hard-to-reverse choice: a new dependency with lock-in, a file-format change, a semantics change, or a license-affecting change. Add it to the index table. Proposals start as **Proposed**. Only the project owner moves an ADR to **Accepted**.
- Accepted ADRs are immutable. Supersede them with a new ADR rather than editing.
- When a spike finishes, record its numbers in the relevant ADR or design doc.
- `skills/` is the agent-facing spec of the calc syntax and the MCP tools. When an implementation changes syntax, tools or semantics, update the matching skill **in the same change**.
- The agent interface reuses the user's own `~/Developer/looking_glass` (MIT; `lg-acp` and `lg-agent`). Keep the MIT notice on anything ported from it (ADR-0009).
- Keep this file current when layout, commands or invariants change.
