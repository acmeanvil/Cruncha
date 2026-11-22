# ADR-0010: The `doc` table is an append-only revision log with periodic squash

**Status:** Accepted (2026-11-18)
**Date:** 2026-11-18
**Deciders:** acmeanvil
**Refines:** ADR-0003. The container, cadence, retention and refs rules there are unchanged; only the layout of `doc/` changes.

## Context

ADR-0003 stored `doc/` as one row per cell, updated by Delta MERGE. Spikes S3 and S3b ([results](../spikes/S3-delta-store.md)) measured it. MERGE is copy-on-write: every commit rewrites the file holding the changed rows, which in practice is the whole document. With history kept forever, growth is (document size) × (number of commits): **57 KiB per commit for a 2,000-cell document**, about 17 MiB a day for a heavy user.

S3b compared three layouts on identical workloads, at 2,000 cells and 1,000 commits:

| Layout | Commit p50 | Growth per commit | Worst read latest |
| --- | --- | --- | --- |
| MERGE (current) | 55 ms | 56.9 KiB | 10 ms |
| **Append-only, squash every 50, stats on `id`** | **17 ms** | **6.9 KiB** | **7 ms** |
| Bucketed MERGE (16 partitions) | 106 ms | 18.8 KiB | 54 ms |

Delta deletion vectors would avoid the rewrite while keeping the one-row-per-cell layout, but delta-rs 1.1.4 cannot write them.

## Decision

`doc/` becomes an **append-only log of cell revisions**:

1. **Schema:** the existing cell columns plus two more:
   - `rev: i64`, the Delta version that wrote the row;
   - `deleted: bool`, a tombstone flag.
2. **Commit:** a blind `write(Append)` of only the changed rows, with `rev = current version + 1`. The single-writer lock (ADR-0003) guarantees this. After each commit, the store **asserts** that the committed Delta version equals `rev` and refuses to continue otherwise.
3. **Read** (latest or any version): scan, keep the highest-`rev` row per `id`, and drop tombstones. Time travel is Delta `load_version(v)` followed by the same reduction.
4. **Squash:** after every 50 commits (tunable), or on idle, overwrite the table with the current rows, keeping their `rev`. This is one Delta version with unchanged content. Old files are never deleted (no VACUUM), so every earlier version stays readable. History marks squashes as `maintenance`.
5. **Statistics only on `id`** (`delta.dataSkippingNumIndexedCols = 1`). This is already applied.
6. **External readers:** Polars, DuckDB and Spark see a log, so we ship the reduction:
   - a `cruncha` Python helper (`read_doc(path, version=None)`);
   - a documented DuckDB query (`QUALIFY row_number() OVER (PARTITION BY id ORDER BY rev DESC) = 1 AND NOT deleted`).

   The Markdown text mirror stays the human and AI view.
7. The public `Store` API (`commit`, `cells`, `cells_at`, `history`, `diff`, `restore`, `tag`, `compact`) is **unchanged**. `compact` gains the squash.

## Options Considered

**A. Append-only plus squash (chosen):** 3.2× faster commits and 8.2× less growth; growth hardly depends on document size; same read and time-travel latency.
**Cons:** the table is a log, not the document, so external tools need the reduction. There is one more invariant (`rev == version`).

**B. Bucketed MERGE:** keeps "table = document". Rejected because it is slower than plain MERGE and its reads degrade.

**C. Deletion vectors:** the best of both, but not writable in delta-rs 1.1.4. Revisit when it is. That would be a new ADR, and since the logical content is the same, migration is a rewrite.

**Keep MERGE:** simplest, but storage cost is not acceptable for large documents with full history.

## Consequences

- **Easier:** cheap history for large documents; faster commits leave more headroom for an every-cell-submit cadence.
- **Harder:** external readers must reduce the log; one more invariant to test.
- **Container format** `format_version` goes 1 → 2. Version-1 containers are migrated on open by writing the current rows as a log at a new version, with all history kept.
- **Watch:** time travel and reopen latency grow with total versions (119 ms and 73 ms at 3,000 versions). The likely cause is remove tombstones kept forever by `deletedFileRetentionDuration = 100 years`. A follow-up must check whether a short tombstone retention is safe, given that we never VACUUM.

## Action Items

1. [x] Implement in `cruncha-store`: log schema, append commit with the `rev` assertion, reduction, squash in `compact` and every N commits, and the format-version-2 migration.
2. [x] Re-run `tests/crash.rs` and `store_variants` against the new `Store`. Add a test that squash leaves every earlier version readable and identical.
3. [x] Follow-up spike: tombstone retention versus time-travel and reopen latency at 10,000 versions. **Done 2026-11-21** ([S3c](../spikes/S3c-history-scale.md)): tombstones are *not* the cause. Dropping them all leaves the timings unchanged. The cost is delta-rs listing and stat-ing every file in `_delta_log`, on each commit and each time travel: 66 ms at 10,000 versions, so commit p50 is 84–99 ms (20 ms at 1,000), and time travel 170–375 ms. A pruned log (1,016 files) is back to 23 ms commits. The remedy is the subject of a new decision.
4. [ ] Python `read_doc` helper and the DuckDB recipe in the docs.
