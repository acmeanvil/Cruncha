# ADR-0013: Large cell sources are stored out of line in content-addressed blobs

**Status:** Accepted (2026-11-19)
**Date:** 2026-11-19
**Deciders:** acmeanvil
**Refines:** ADR-0010 (the `doc` revision log) and ADR-0011 (pruning gains a blob step). Implements option D of spike S3 on top of option A, and leaves a seam for option C.

## Context

Spike S3 listed five ways to cut history growth. S3b measured **A** (an append-only revision log) against **B** (bucketed MERGE). A won on every axis and became ADR-0010. B was rejected. **C** (Delta deletion vectors) would keep "table = document" without copy-on-write, but delta-rs cannot write deletion vectors. That was checked again on 2026-11-19 against deltalake-core 1.1.4, still the newest release: DELETE and MERGE read them but never write them. **D** (out-of-line content) was left as a complement to whichever layout won.

Under A, a commit writes only the changed rows, so D cannot shrink an ordinary text edit: the new text has to be written somewhere. What D removes is every *other* copy of unchanged text:
- **Squashes:** every 50 commits, the log is rewritten with every current row, prose included.
- **Restores:** these re-upsert every cell that differs from the target version. Usually the text is identical to text already stored.
- **Moves and attribute-only edits:** a new `order_key` or new `attrs` currently re-writes the whole source.
- **Pins:** pruning copies whole named versions into `pins/`.
- **Duplicates:** the same text in several cells or versions.

In a prose-heavy document these copies dominate. ProseMirror JSON paragraphs are 1–4 KiB, and every squash copies all of them. See Results.

## Decision

1. **Threshold.** A cell source of at least **1 KiB** (`DEFAULT_BLOB_THRESHOLD`; `Store::set_blob_threshold`) is stored out of line. Smaller sources (calc statements, most code) stay inline, where Parquet compresses them in bulk and no extra file is needed. Both kinds of row read the same, so the threshold can change at any time.
2. **Blob store.** Blobs live at `blobs/ab/cdef….zst`. Each one is the zstd-compressed (level 3) UTF-8 source, named by the SHA-256 of the uncompressed text, with a two-character fan-out. Blobs are **immutable**, and identical text is stored once.
3. **Row format: container format 3.** A new nullable column, `source_hash`, goes **last** in `doc/` and `pins/`. When it is set, `source` is null. Format-2 containers are migrated on open by adding the column. No data is rewritten, and older rows read as inline.
4. **Write order.** Blobs are written (temp file, then rename) **before** the Delta commit that references them. A crash can therefore leave only unreferenced blobs, never a reference without its blob. This matches Delta's own durability for local files: neither fsyncs.
5. **Read.** After the log reduction, the store collects the referenced hashes and loads them, with a 64 MiB in-memory cache that can never be stale. It checks each blob against its hash. A missing or damaged blob is `Error::Corrupt`, never an empty cell.
6. **Garbage collection happens only in `Store::prune`** (ADR-0011 step 7). After VACUUM and log expiry, the store scans the `source_hash` column of every data file still on disk in `doc/` and `pins/`, and deletes every other blob. Files on disk are a superset of what the remaining versions read, so a needed blob is never deleted. "Keep everything" documents never lose a blob.
7. **Size accounting.** Each commit records the bytes of new blobs in `cruncha.blobBytes`. The size limit (ADR-0011) counts them, and `history_size_bytes` includes `blobs/`.
8. **External readers.** These already need the log reduction (ADR-0010 item 4). They now also need to resolve `source_hash`: read `blobs/<h[0..2]>/<h[2..]>.zst` and decompress it with zstd. The planned `read_doc` helper and the DuckDB recipe must do both. The Markdown mirror is unaffected.
9. **`blobs/` is separate from `assets/`** (design 02 §5.2, images and figures). Assets are referenced from inside cell content, which blob GC does not parse. Merging the two stores would need GC to understand those references first.

### Room for deletion vectors (option C)

C would make `doc/` one live row per cell again: a MERGE on `id` with deletion vectors, so old rows are masked rather than their files rewritten. This change prepares for it without building it:
- **The reader already works for both layouts.** `reduce_log` keeps the highest `rev` per id and drops tombstones. On a table with one live row per id it returns the table unchanged, so time travel, diff, restore, pins and blob resolution don't change.
- **The writer is isolated** in two steps of `Store`: `commit_with` (append the changed rows) and `maintain` (squash every N commits). A DV layout replaces them with a MERGE on `id` that sets `rev`, and with OPTIMIZE instead of squash.
- **Blobs are independent of the layout.** Under C, rows stay small, so DV MERGE and OPTIMIZE move kilobytes, not documents.
- **Migration path when delta-rs can write DVs:**
  1. squash, which leaves one row per id;
  2. enable the `deletionVectors` table feature (a one-way protocol upgrade, reader v3 and writer v7);
  3. bump the container format;
  4. switch the two writer steps.

  That needs its own ADR, and it should be measured against this one first. A's per-commit floor (about 6 KiB) comes from Delta's per-commit overhead, which C does not remove.

## Results (spike S3c)

Full write-up: [S3c](../spikes/S3-delta-store.md#spike-s3c-a-against-a--d-out-of-line-content). Real `Store`, 2,000 cells, 1,000 commits, squash every 50:

| Prose-heavy workload | A (inline) | A + D |
| --- | --- | --- |
| Growth per commit | 27.2 KiB | **9.05 KiB** (3.0× less) |
| Restore to the middle version | 385 KiB | **40 KiB** |
| Commit p50 | 21 ms | 20 ms |
| Read latest, warm / first read, cold | 12 / 16 ms | 7 / 40–88 ms |
| Initial 2,000-cell import | 29 ms | 160–215 ms |

On the default (calc-heavy) workload, no source reaches the threshold, and the results are identical to A.

## Options Considered

**D1. Out-of-line above a threshold, files per blob (chosen):** simple, deduplicated, immutable, and crash-safe by write order. One file per distinct large text.
**D2. Everything out of line:** every row is tiny, but each calc edit pays an extra file for a 40-byte source, and Parquet's bulk compression of small sources is lost.
**D3. Blobs in a Delta table (`blobs/` as an append-only table of hash and bytes):** one file per commit instead of per blob, but two Delta commits per edit, which loses single-commit atomicity, and GC would need a VACUUM of its own.
**D4. Packfiles (git-style):** the fewest files, but repacking would have to be crash-safe and history-aware. Revisit only if file counts become a problem.
**Do nothing:** acceptable for calc-heavy documents. Prose-heavy documents pay for every squash.

## Consequences

- **Easier:** squash, restore, move and pin costs no longer scale with prose size; identical text is stored once.
- **Harder:**
  - one more directory to keep consistent;
  - external readers must resolve hashes;
  - a prune must scan data files for references;
  - reading a cold document costs one small file read per distinct large source, after which the cache serves them.
- **Invariants added to CLAUDE.md:**
  - blobs are immutable and written before their commit;
  - only `prune` deletes them, and only those no data file left in `doc/` or `pins/` references.

## Action Items

1. [x] `cruncha-store`: `blobs.rs`, the `source_hash` column, format 3 and its migration, the threshold, the commit metadata key, GC in `prune`, and tests (`tests/blobs.rs`; `tests/crash.rs` now writes large sources).
2. [x] Spike S3c: `store_variants`, A (inline) against A + D, on the default and prose-heavy workloads.
3. [ ] `read_doc` helper and DuckDB recipe resolve `source_hash` (with ADR-0010 item 4).
4. [ ] Revisit C when delta-rs writes deletion vectors: re-run S3c with a DV layout behind the same writer seam.
