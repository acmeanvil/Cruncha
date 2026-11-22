# ADR-0021: The store lists `_delta_log` from an index it keeps, not from the directory

**Status:** Accepted
**Date:** 2026-11-21
**Deciders:** acmeanvil
**Refines:** ADR-0010, ADR-0011 (resolves their latency watch item)

## Context

Spike S3c ([write-up](../spikes/S3c-history-scale.md)) measured containers up to 10,000 versions:

- **Commits slowed five-fold** (20 → 84–99 ms p50), and **time travel** reached 170–375 ms.
- **The cause wasn't tombstones**, as ADR-0010 and ADR-0011 guessed: removing every one left the timings unchanged.
- **The cause is the number of files in `_delta_log`.**
  - delta-rs lists that directory on every commit (to find the latest version) and on every time travel (to find the nearest checkpoint).
  - object_store's local file system lists by walking the whole directory: 11 ms for 10,000 names, 66 ms with their metadata.
- Pruning (ADR-0011) bounds the cost, but only for documents with a limit. "Keep everything" documents slow down in proportion to their age.

## Decision

1. **The store's tables go through `LogIndexStore`**, a thin `ObjectStore` over the local file system (`crates/cruncha-store/src/logindex.rs`).
   - It keeps, per table, the names in `_delta_log`, read once per `Store`.
   - It stats a file only the first time a listing returns it.
   - Its writes keep the index current: puts, multipart uploads (checkpoints), copies, renames, deletes.
   - Listings of a `_delta_log` are answered from the index; every other call goes straight to the file system.
2. **This is safe because the container has a single writer** (the `.lock` file).
   - Each `Store` builds its own index, so a reopen always starts from the disk.
   - A delta-rs call that writes through its own storage (the URL-based checkpoint in `prune`) is followed by `forget()`, which makes the next listing read the directory again.
3. **Nothing on disk changes.** The container format is still 5; external readers (Polars, DuckDB, delta-rs itself) see an ordinary Delta table, and pay the ordinary listing cost.
4. **A history limit is offered, not imposed.** Document settings → History suggests "the last 2,000 saves" when a document keeps everything and has more than 5,000 saves. Editing stays fast either way; a limit saves disk, and makes older versions quicker to open.

## Options Considered

### Option A: expire tombstones (short `deletedFileRetentionDuration`)
**Pros:** checkpoints shrink 50-fold.
**Cons:** no speed gain (measured); it breaks the rule that Delta's retention properties never change, which exists because they act on wall-clock time.

### Option B: archive old log files into another folder
**Pros:** a short live log for every reader.
**Cons:**
- Cruncha would reconstruct old versions itself;
- outside readers lose old time travel;
- a container format change.

### Option C: a default retention limit
**Pros:** already built; measured back to 1,000-version speed.
**Cons:** "Keep everything" is a real choice for calc packages under review; it should stay fast.

### Option D: an indexed listing in the store (chosen)
| Dimension | Assessment |
| --- | --- |
| Complexity | Low: one `ObjectStore` wrapper, about 250 lines, unit and crash tested |
| Risk | delta-rs gaining a listing path the index doesn't cover: such a listing would still be correct, only slow |
| Format | Unchanged |

## Results (10,000 versions, the same container)

| | Before | Indexed |
| --- | --- | --- |
| Commit p50 / p95 | 84 / 127 ms | 15 / 25 ms |
| Time travel: earliest / middle / latest − 10 | 203 / 169 / 169 ms | 36 / 48 / 127 ms |
| Reopen | 50 ms | 42 ms |

Growth with the index (`history_scale`, run while other builds competed for the machine, so noisy):

| Versions | Commit p50 | Time travel v1 / middle | Latest − 10 |
| --- | --- | --- | --- |
| 2,000 | 14.7 ms | 26 / 47 ms | 129 ms |
| 10,000 | 19.6 ms (99 before) | 50 / 64 ms (375 / 321 before) | 410 ms |

Commits and old versions are now flat. Time travel to a *recent* version still grows: it loads the newest checkpoint, which carries every tombstone (742 KiB at 10,000 versions), so for this one path tombstones do matter. It stays near 130 ms on a quiet machine at 10,000 versions. It replays up to 99 commits after the last checkpoint, which doesn't depend on history length; checkpointing more often would trade disk for it.

## Consequences

- **Easier:** long-lived documents keep their editing speed; "Keep everything" is a reasonable default.
- **Harder:**
  - any write to a table that bypasses `Store`'s storage must call `forget()`;
  - a delta-rs upgrade must be re-run against `history_variants`.
- **Disk still grows with history:** checkpoints are about 37 of the 47 MiB at 10,000 versions. Deleting old checkpoints (they are only needed for time travel near them) is a possible follow-up.

## Action Items

1. [x] `LogIndexStore` (lazy stat), every table through it, `forget()` after the URL-based checkpoint; unit test; store and crash tests green.
2. [x] Re-measure with `history_variants` (above) and `history_scale` (S3c).
3. [x] Document settings: offer a limit for long "Keep everything" histories.
4. [ ] Consider removing checkpoints older than the newest few (disk only).
5. [ ] Recent-version time travel: measure tombstone expiry (Option A, needs its own ADR, since it changes a retention property) against checkpointing more often, on a quiet machine.
