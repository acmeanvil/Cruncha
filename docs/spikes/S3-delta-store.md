# Spike S3: always-committed Delta container

*Date: 2026-11-18. Code: `crates/cruncha-store` (v0), benchmark `examples/store_bench.rs`. Machine: Intel i7-1068NG7 MacBook, APFS SSD. Build: release, deltalake 1.1.2 with DataFusion 55.*

## Method

- A document starts with 200 cells: a mix of prose (about 350 B), Python (about 80 B) and calc cells (about 40 B).
- 1,000 commits follow, each changing 1–3 cells: 90% edits, 7% inserts, 3% deletes.
- Each commit is a single DataFusion `MERGE` into `doc/`, carrying actor and message metadata.
- The PRNG is deterministic, so runs are comparable.

```sh
cargo run -p cruncha-store --release --example store_bench -- 1000 200
```

## Results

| Metric | Value |
| --- | --- |
| Create container | 15 ms |
| Initial 200-cell commit | 52 ms |
| **Commit latency** | **p50 51 ms · p95 62 ms · p99 136 ms · max 422 ms** |
| Latency drift | mean of first 50 commits 43 ms → last 50 commits 60 ms (log growth between checkpoints) |
| Throughput (back to back) | 18.7 commits/s |
| Read latest (269 cells) | 4.2 ms (2.8 ms after compaction) |
| Time travel to v500 / v2 | 43 ms / 31 ms |
| Full history (1,000 versions) | 65 ms |
| Compact (OPTIMIZE plus checkpoint) | 49 ms; the live table had only 7 files |
| Reopen plus first read | 34 ms + 3 ms |
| **On-disk growth** | **2 files and about 17 KiB per commit** (2,017 files and 16.9 MiB after 1,000 commits) |

## Findings

1. **Latency passes.** At the designed cadence (a commit on cell submit, or after 1–2 s of idle), about 50 ms per commit is invisible to users. Even back to back, it sustains about 19 commits/s. The p99 and max spikes (136 and 422 ms) line up with checkpoint writes. Commits must therefore run **off the UI thread**, which they will, since the engine process owns the store.
2. **Reads and time travel are fast:** under 50 ms for any version, so a history slider is practical.
3. **Storage amplification is the real cost.** MERGE is copy-on-write: each commit rewrites the whole Parquet file that holds the matched rows, and in a small document that file is the entire document. Growth is about **(compressed document size) × (number of commits)**:
   - this 200-cell document grew about 17 MiB per 1,000 commits;
   - a 2,000-cell document would grow about 170 MiB per 1,000 commits.

   History is kept forever (ADR-0003), so this has to improve before large documents are practical.
4. **Compaction does not help with that,** because it only rewrites the live table and old files stay for time travel. Compaction mainly keeps read and log-replay costs flat.

## Options for finding 3: storage amplification

Each option below is scored against a reference **2,000-cell document**:
- about 170 KiB compressed;
- a heavy editing day of 300 commits, each touching 1–3 cells.

The current design costs about **170 KiB per commit, or about 50 MiB a day**, and the whole cost is kept forever.

### A. Append-only change log, with periodic squash *(recommended)*

`doc/` stops being "the current document" and becomes a **log of cell revisions**. Each row has the doc columns plus:
- `rev`: the version this row was written in, known in advance because there is a single writer;
- `deleted`: a tombstone flag.

How it works:
- **Commit:** a *blind append* of only the changed rows. No scan and no DataFusion MERGE, so it should also be **faster** than today's 50 ms (estimate 10–20 ms, to be measured).
- **Read latest:** scan, then keep the row with the highest `rev` per `id` and drop tombstones. This is cheap, because a document's rows fit in memory.
- **Time travel:** Delta `load_version(v)` and the same reduction. Each Delta version is still a consistent snapshot of the log, so history semantics don't change.
- **Squash:** every N commits (for example 500), or on idle, overwrite the table with just the current rows. One Delta version writes one document-sized file. **Older versions remain readable**, because the old files are kept (we never vacuum, ADR-0003). This keeps read cost bounded.

| | |
| --- | --- |
| Growth per commit | changed rows only, about 1–2 KiB, plus a 170 KiB squash every 500 commits (about 0.3 KiB per commit) → **about 2 KiB** (roughly 85× smaller) |
| Commit latency | expected to be lower (append only) |
| Read latest | O(rows since last squash); 500 × 2 rows is trivial |
| External readers | Polars/DuckDB see a log, not the document. They need the reduction (document it; ship a `cruncha.read_doc()` helper and a DuckDB view). The Markdown mirror is unaffected. |
| Code change | moderate: `commit` becomes `write(Append)`, and `collect_cells` gains the reduction. Diff, history and restore keep their APIs. |
| Risks | `rev` must equal the Delta version actually committed, which holds under the single-writer lock. Verify after each commit, and fail loudly if it doesn't. |

### B. Bucketed table (keeps "table = current document")

- Add `bucket = hash(id) % B` and partition by it.
- Add `t.bucket = s.bucket` to the MERGE predicate so DataFusion prunes untouched partitions.
- A commit then rewrites only the 1–3 buckets it touches.

| | |
| --- | --- |
| Growth per commit (B = 16) | about 2 buckets × 170/16 KiB → **about 21 KiB** (8× smaller) |
| Commit latency | still about 50 ms (DataFusion MERGE overhead dominates) |
| Read latest | at least B files to open; slightly slower, still a few ms |
| External readers | unchanged and trivial |
| Code change | small |
| Risks | B is fixed at creation; changing it means a rewrite. Growth is still O(document size), just divided by B. Small documents end up with many tiny files. |

### C. Deletion vectors (best of both, but not available yet)

With deletion vectors, MERGE would mark old rows deleted in a small bitmap file and append the new rows, instead of rewriting the file. Growth would be close to option A while keeping B's "table = document" property. **delta-rs 1.1.4 reads deletion vectors but has no write path for them in MERGE or DELETE** (checked in the source). Revisit this when delta-rs adds writes.

### D. Store large content out of line (complements A, B or C)

- Move large `source` values (prose ProseMirror JSON, long code) into content-addressed `assets/` blobs, keyed by sha256, and keep only the hash in `doc` rows.
- Unchanged text is then never rewritten, and rows shrink to about 150 B. That alone cuts the current design's growth by about 3–10× in prose-heavy documents.
- Costs: one indirection on read, and blob garbage collection must respect history (blobs can never be deleted while history references them).

### E. Coarser commit cadence

This reduces the **number** of commits, not the cost per commit. The design already commits on submit or idle, not per keystroke. Lengthening the prose idle window (2 s → 5 s) might cut commits by 30–50% in prose-heavy sessions. It's worth tuning, but it is not a fix.

### Summary

| Option | Growth per commit (2k cells) | Commit latency | Table = document? | Available now |
| --- | --- | --- | --- | --- |
| Today (MERGE) | about 170 KiB | about 50 ms | yes | yes |
| **A. Append-only + squash** | **about 2 KiB** | **about 10–20 ms (est.)** | no (log + reduction) | yes |
| B. Bucketed MERGE (B = 16) | about 21 KiB | about 50 ms | yes | yes |
| C. Deletion vectors | about 2–5 KiB | about 50 ms | yes | **no** (delta-rs) |
| D. Out-of-line content | divides any of the above by 3–10 | +1 lookup | — | yes |

**Recommendation:**
- Benchmark **A** against **B** in spike S3b, with the same harness at 2,000 cells and 100k accumulated rows.
- Choose A if reading the latest version stays under about 20 ms between squashes. It is the only option that makes history cost proportional to *what changed*.
- Add D later if prose-heavy documents need it, and move to C if delta-rs gains deletion-vector writes.
- Switching `doc/` from "current document" to "log" changes the table semantics, so it needs a new ADR that refines ADR-0003.
