# ADR-0003: Hybrid Delta Lake container as the storage format

**Status:** Accepted. Spike S3 must confirm commit performance; see Action Items.
**Date:** 2026-11-18
**Deciders:** acmeanvil

## Context

Requirements:
- **Saving is optional:** every change is persisted atomically.
- Versioning, named snapshots and rollback.
- Data frames (Polars) as first-class content.
- Readable without Cruncha.
- Friendly to AI and git.

The requested starting point was Delta Lake on Parquet. A naive "Delta on every keystroke" design has four problems:
- it produces thousands of tiny files and commits per hour;
- a Delta table is a directory, not a single file you can email;
- `VACUUM` deletes the history that time travel needs, and so does Delta's default **30-day log retention**;
- Parquet doesn't diff in git.

## Decision

A **container directory** (`name.cruncha/`) of Delta tables with one job each, written by a single engine process:

- `doc/`: one row per cell. Each commit is a MERGE of the changed rows, and each Delta version is a document version.
- `results/`: cached results keyed by `(cell_id, input_hash)`.
- `data/<name>/`: one Delta table per data frame, read and written natively by Polars.
- `refs/`: named versions, append-only; they pin `doc` and data versions.
- `assets/`: content-addressed blobs.
- `cruncha.json`: manifest. `.lock`: single-writer lock.

Rules:
- **Commit cadence:** on cell submit, and after about 1–2 s of prose idle. Never per keystroke.
- **History is kept forever by default.** Tables are created with `delta.enableExpiredLogCleanup=false` and very long log and deleted-file retention.
- **Compaction** (`OPTIMIZE` plus checkpoint) runs in the background. It does not remove history.
- **Rollback writes a new version**; history is never rewritten.
- The actor (human, AI or CLI) and a message go in each commit's `commitInfo`.
- **Sharing:** `.crunchaz`, a zip of the container (latest only, or with full history).
- **Git and AI:** an auto-generated Markdown **text mirror** that includes results.

## Options Considered

### Option A: Pure Delta, commit on every edit
| Dimension | Assessment |
| --- | --- |
| Complexity | Low |
| Cost | Thousands of tiny files per hour; slow reads without constant compaction |
| Scalability | Poor for high-frequency small writes |
| Team familiarity | Medium |

### Option B: Hybrid Delta container (chosen)
**Pros:** atomic always-on persistence; time travel; Polars, DuckDB and Spark interoperability; data and document versioned together; AI and git through the mirror.
**Cons:** a directory, not a file (mitigated by the zip export); a write-latency floor from DataFusion MERGE (to be measured); we must manage retention ourselves.

### Option C: SQLite single file
**Pros:** one file; mature WAL atomicity.
**Cons:** Polars can't scan it natively, so data frames get copied; time travel must be hand-built.

### Option D: CRDT (Loro) document
**Pros:** real-time collaboration; fine-grained history.
**Cons:** not an analytics store; history is opaque to other tools. **Kept as a possible later in-memory layer** whose updates persist as Delta commits.

## Trade-off Analysis

B keeps what Delta is good at (atomic versions, analytics-native data) and limits its weak spot (tiny writes) with batching and compaction. The text mirror makes up for Parquet being binary.

## Consequences

- **Easier:** "no save", history UI, reproducible results tied to exact data versions, external analysis of documents.
- **Harder:** single-writer coordination (the UI, MCP agents and CLI all go through the engine process); retention policy for large data tables.
- **Revisit:** if S3 shows MERGE latency is too high, switch `doc` to append-only rows plus an in-memory "latest per id" view.

## Action Items

1. [x] `cruncha-store` v0: container create/open/lock, MERGE commits with actor metadata, time travel, history, diff, tags, restore, compact.
2. [x] Spike S3 benchmark: [results](../spikes/S3-delta-store.md). Latency passes (p50 51 ms, p95 62 ms; time travel under 50 ms). **Storage amplification** is about 17 KiB per commit for a 200-cell doc, because MERGE rewrites the whole doc file.
2b. [x] Spike S3b: append-only beats MERGE and bucketed layouts, and the store survives SIGKILL mid-commit. See [results](../spikes/S3-delta-store.md) and ADR-0010 (proposed).
3. [ ] `results/` and `data/<name>` tables; `assets/`.
4. [ ] Text-mirror export/import; `.crunchaz` zip.
5. [ ] Retention: "compact history" that materializes pinned refs before `VACUUM`.
