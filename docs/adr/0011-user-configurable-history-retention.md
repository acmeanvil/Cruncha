# ADR-0011: User-configurable history retention

**Status:** Accepted (2026-11-18)
**Date:** 2026-11-18. Revised the same day to add the saves and size limits and to measure time against document activity, never the calendar. Revised 2026-11-19, at the owner's request, to match the implementation: the pruning order and its point of no return, the blob step from ADR-0013, and a correction to the tombstone consequence, which measurement disproved.
**Deciders:** acmeanvil
**Refines:** ADR-0003 (which keeps history forever by default) and ADR-0010.

## Context

ADR-0003 keeps every version forever. After ADR-0010 that costs about 7 KiB per commit, roughly 2 MiB a day for a heavy user. That's cheap, but it's unbounded, and some users or firms will want to limit it. Others, under document-retention policies for regulated engineering work, need the guarantee that nothing is ever lost.

The owner asked for a **settings/preferences page** where the user sets a retention limit, which may be **a length of time, a size, or a number of saves**.

**Hard requirement: a document must never lose history just because time passed.** A finished study may sit unopened for years. When it's opened again, its history must be exactly as it was left. The app must not destroy history on a timer.

Delta's built-in retention is the wrong tool, because it is wall-clock based and runs by itself:
- `delta.logRetentionDuration` plus `delta.enableExpiredLogCleanup` delete old `_delta_log` entries automatically whenever a checkpoint is written. That would happen during an unrelated edit, judged by *today's* date.
- `VACUUM` deletes files that stopped being live more than `delta.deletedFileRetentionDuration` ago, again by wall-clock age.

Named versions (`refs`, such as "rev A, issued for review") **must survive any retention setting**.

## Decision

### 1. Policy

The policy is **Keep everything** (the default), or one or more **limits**:

| Limit | Meaning | Example |
| --- | --- | --- |
| **Saves** | Keep at most the last *N* versions | 1,000 saves |
| **Size** | Keep history within *S* on disk: `doc/` and its log, plus blobs (ADR-0013), excluding pins | 500 MB |
| **Activity window** | Keep versions made within *D* days **before the document's most recent edit**, not before today | 90 days of activity |

How the limits work:
- When several limits are set, **the strictest wins**. The user is setting caps, and each one is honored.
- **Floor:** whatever the limits say, the newest **100 versions** are always kept. This is configurable, but never below 10.
- **Named versions are never pruned** (see §3).

The policy is stored per document in the container manifest (`cruncha.json`, `retention` field), written atomically like the format version. A missing field means "use the app default". The **app-wide default** for new and unconfigured documents lives on the Preferences page (Tauri UI, ADR-0008) in the app's own `preferences.json`, and each document's settings can override it.

### 2. Time never prunes on its own

- **Activity-relative time.** Every commit records Cruncha's own timestamp, `cruncha.at` (µs). The activity window is measured back from the latest *edit*, restore or create commit. Maintenance commits (squash, prune, migration) and tags don't count as activity. A document untouched for five years has exactly the history it had when it was last edited.
- **Pruning needs new edits.** `Store::prune()` acts only when there are edit or restore commits after the last prune marker. The one exception is finishing a prune that was interrupted. Opening, reading, exporting, time travel, tagging and compaction never prune. So a dormant document can never be pruned, by any limit.
- **Delta's own retention stays disabled permanently.** The table properties stay as in ADR-0003: cleanup off, 100-year windows. Cruncha computes a **cutoff version** itself and passes explicit parameters to the deletion steps. Never derive Delta table properties from the policy.

### 3. Pruning procedure (`Store::prune()`)

Pruning is an explicit maintenance operation, never part of a commit. The app runs it in two cases: when a document that was edited in this session closes, at most once per session, and when the user confirms **Trim history now**.

1. **Compute the cutoff version.** Versions before the cutoff become unavailable. Each limit yields the oldest version it allows:
   - *Saves:* `latest + 1 − N`.
   - *Size:* walk versions newest-first, summing each version's log-entry size, the data files it added, and the new blobs it wrote (`cruncha.blobBytes`), while the total fits the budget. The latest version is always kept.
   - *Activity window:* the oldest version whose `cruncha.at` is at least `last_edit − D`.

   The cutoff is the **newest** of these (the strictest limit), capped at `latest + 1 − floor`. If the cutoff is at or before the earliest available version, stop: there is nothing to do.
2. **Pin refs.** Every ref older than the cutoff that isn't pinned yet is copied into `pins/`, a Delta table holding that version's cells. Large sources stay in `blobs/` and the pin rows reference them. `cells_at_ref` reads pins when the version itself is gone. Pins are excluded from the size limit. Later, pins will also need the referenced `data/*` versions (item 4).
3. **Checkpoint at the cutoff**, so every version from the cutoff on can still be loaded once older log entries are gone.
4. **Write `earliest_version = cutoff` to the manifest.** This is the point of no return: from now on, versions before the cutoff report `Error::Pruned`, even if a later step is interrupted.
5. **Delete data files.** Run VACUUM (Lite mode) on `doc/` with an *explicit* retention of `now − (oldest Delta timestamp among versions ≥ cutoff)`, a fixed clock, and the retention check disabled. A file removed by a commit at or after the cutoff was removed after every kept version was written, so it survives.
6. **Expire log entries** before the cutoff, by calling `cleanup_expired_logs_for(cutoff, …)` with explicit arguments. It deletes only entries older than the checkpoint from step 3.
7. **Delete unreferenced blobs** (ADR-0013): blobs that no data file left in `doc/` or `pins/` references. Files on disk are a superset of what the kept versions read, so this never removes a needed blob.
8. **Marker commit:** a squash whose message is "history before version *v* (*date*) pruned; policy: …" and whose metadata carries `cruncha.prunedBefore = v`. A checkpoint follows it. History then shows a clear marker instead of silently missing versions.

A crash at any point leaves a readable container. Before step 4, nothing is lost, and the next prune recomputes. After step 4, the manifest is ahead of the newest marker, and the next `prune()` call finishes the job even without new edits. This is covered by a SIGKILL test (`tests/crash.rs`).

### 4. API

- `Store::earliest_version()`. `cells_at()` and `restore()` below it return `Error::Pruned { earliest }`, not `NoSuchVersion`. `history()` starts at it.
- The invariant "history length = version + 1" becomes "history length = version − earliest + 1".
- `Store::retention_preview(policy)` returns, without deleting anything:
  - the cutoff version and date;
  - the estimated bytes freed;
  - the refs it would pin;
  - `waits_for_edit`, when no edits have been made since the last prune.

  The UI uses it.
- `Store::prune(policy, actor)` returns a `PruneReport`: the new earliest version, the refs pinned, and the data files, log files and blobs deleted.

### 5. UI

- **Preferences → History:** "Keep everything" (default), or checkboxes for *last N saves*, *up to S MB*, *D days of activity*. Helper text: "Time limits count back from your last edit; documents you aren't editing are never trimmed. Named versions are always kept."
- **Document settings:** the same control, with "Use app default" as the first option, plus the current history size, the earliest version, and **Trim history now…**.
- **Confirmation on tightening:** if a new policy would prune at the next run, show `retention_preview`: what becomes unavailable (up to which version and date), the space freed, the refs that will be pinned, and that this **cannot be undone**.

## Options Considered

**A. Wall-clock age, Delta-native (rejected):** set Delta's retention properties from the policy. It's simplest, but finished studies that sit unopened lose their history, and cleanup runs during unrelated commits. That fails the hard requirement.
**B. One global setting only:** simple, but one setting can't fit both scratch calcs and issued deliverables.
**C. Per-document limits on saves, size, and activity-relative time, with an app default (chosen):** honors the user's caps, never acts on a timer, and protects named versions.
**D. Thinning instead of a hard cutoff (keep hourly, then daily, then weekly versions, like Time Machine):** nicer, but Delta only truncates a prefix of versions; it can't drop the ones in between. Thinning would need history to be squashed and rewritten. Revisit if users ask for it.

## Consequences

- Retention is fully under Cruncha's control: no Delta property ever deletes anything by itself.
- **Pruning frees disk space but does not relieve the S3b tombstone watch item.** Measured on 2026-11-19, on a 200-commit document pruned to 20 saves: tombstones went from 239 to 240. VACUUM deletes the files, but their `remove` actions stay in every checkpoint until `deletedFileRetentionDuration` passes, and we never change that property. Reopen and time-travel cost therefore still grow with the total number of versions ever written, for pruned and "Keep everything" documents alike. The follow-up spike (item 5) owns this.
  - One direction for that spike: tombstones only tell VACUUM which files to delete. Time travel to a version replays the log up to that version and never needs later tombstones. Dropping expired tombstones from checkpoints, with `prune` switching to Full-mode VACUUM (which finds unreferenced files by listing), may be safe. It would mean changing a Delta retention property, which this ADR forbids, so it needs its own ADR.
- CLAUDE.md's invariant: "Only `Store::prune` may VACUUM or expire logs, with explicit parameters derived from the document's policy, after pinning refs. Never change Delta's retention table properties."

## Action Items

1. [x] `cruncha-store`:
   - `Retention` type in the manifest (`saves`, `size_bytes`, `activity_days`, `floor`);
   - cutoff computation;
   - `retention_preview`;
   - `prune()` with ref pinning and the explicit VACUUM and log-cleanup parameters;
   - `Error::Pruned` and an `earliest_version` API.
2. [x] Tests:
   - each limit on its own, and combined (strictest wins);
   - the floor;
   - **a dormant document never prunes**: simulate a last edit 5 years ago and run prune;
   - prune only runs after new edits;
   - refs are readable after pruning;
   - the history marker;
   - a crash (SIGKILL) during prune leaves a readable container.
3. [x] Tauri Preferences and document-settings controls, and the confirmation dialog backed by `retention_preview`.
4. [ ] `data/*` tables follow the same policy (once those tables exist), and pins carry the referenced `data/*` versions.
5. [ ] Follow-up spike: reopen and time-travel latency at 10,000 versions, comparing "Keep everything" with a saves limit. It must cover the tombstone growth described under Consequences.
6. [x] Blob cleanup (step 7) in `prune()`, with tests (ADR-0013).
