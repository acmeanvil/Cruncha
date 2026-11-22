# Spike S3c: 10,000 versions

*Date: 2026-11-21. Code: `crates/cruncha-store/examples/history_scale.rs`, `examples/history_variants.rs`. Machine: Intel i7-1068NG7 MacBook, macOS 26, APFS, deltalake 1.1 (object_store local file system). Answers ADR-0010 action item 3 and the ADR-0011 tombstone watch item.*

## Question

Spike S3 saw time travel and reopen latency grow with the number of versions: 119 ms and 73 ms at 3,000. ADR-0010 and ADR-0011 guessed the cause was remove tombstones, which `deletedFileRetentionDuration = 100 years` keeps in every checkpoint. Is that the cause, and what happens at 10,000 versions?

## Method

- `history_scale` builds a 200-cell document. Each commit edits 1–3 cells; the store squashes every 50 commits and Delta checkpoints every 100.
- Every 1,000 versions, it reopens the container and measures (median of three):
  - open;
  - reading the latest version;
  - time travel to v1, to the middle and to `latest − 10`;
  - the latest checkpoint's tombstones and size;
  - the size of `_delta_log`;
  - commit p50 over that stretch.
- `history_variants` copies the 10,000-version container and measures the same reads, plus 100 more commits, after one change:
  - **baseline:** nothing;
  - **tombstones:** `deletedFileRetentionDuration` set to 0 (spike only), then a checkpoint, which drops every tombstone;
  - **prune:** the existing ADR-0011 prune, keeping 1,000 saves.

## Results

Growth (`history_scale`):

| Versions | Open | Latest | v1 | Middle | Latest − 10 | Tombstones | Checkpoint | `_delta_log` | Commit p50 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1,000 | 16.5 ms | 13.3 ms | 38 ms | 66 ms | 140 ms | 968 | 91 KiB | 1.5 MiB | 20 ms |
| 3,000 | 31.4 | 15.2 | 111 | 148 | 240 | 2,957 | 237 KiB | 6.7 MiB | 37 ms |
| 5,000 | 46.6 | 3.6 | 252 | 190 | 348 | 4,997 | 378 KiB | 14.6 MiB | 77 ms |
| 10,000 | 80.5 | 4.4 | 375 | 321 | 376 | 9,995 | 743 KiB | 47.0 MiB | 99 ms |

Variants at 10,000 versions (`history_variants`):

| Variant | Files in `_delta_log` | Tombstones | Checkpoint | Open | Time travel: earliest / middle / latest − 10 | Commit p50 / p95 |
| --- | --- | --- | --- | --- | --- | --- |
| baseline | 10,102 | 9,995 | 743 KiB | 49.5 ms | 203 / 169 / 169 ms | 84 / 127 ms |
| tombstones expired | 10,104 | 0 | 15 KiB | 41.4 ms | 203 / 165 / 150 ms | 83 / 95 ms |
| pruned to 1,000 saves | 1,016 | 10,001 | 742 KiB | 16.4 ms | 46 / 57 / 130 ms | 23 / 35 ms |

Listing `_delta_log` at 10,000 versions: 11 ms for the names alone, **66 ms with each entry's metadata**. object_store's local file system lists that way, even for a listing from an offset.

## Conclusions

1. **Tombstones aren't the cause.** Dropping all of them shrinks the checkpoint 50-fold but leaves open, time travel and commit timings within noise. The ADR-0010 and ADR-0011 hypothesis is wrong, and changing the retention property (which needs an ADR) would buy disk space only.
2. **The cause is the number of files in `_delta_log`.**
   - delta-rs lists the whole directory, stat-ing every entry, on every commit (to find the next version) and on every time travel (to find the nearest checkpoint).
   - At 10,000 versions that listing alone is about 66 ms, matching the commit slowdown from 20 to 84–99 ms.
   - With the log trimmed to 1,016 files and every tombstone still in place, the numbers are back to their 1,000-version values.
3. **The cost is linear in versions ever kept,** so "Keep everything" documents slow down without bound; documents with any save limit stay bounded.
4. **Disk:** at 10,000 versions, checkpoints are most of `_delta_log` (100 checkpoints growing to 743 KiB, about 37 of 47 MiB).
5. **Reading the latest version is unaffected** (4–20 ms): the open `Store` holds its snapshot.

## After ADR-0021 (indexed listing)

At 10,000 versions on the same container:
- commit p50 is 15 ms (84 before);
- time travel to the earliest version and the middle takes 36 / 48 ms (203 / 169 before);
- reopen takes 42 ms.

The growth curve is flat for commits and old versions. Time travel to a recent version still grows with the newest checkpoint's tombstones; see ADR-0021 item 5.
