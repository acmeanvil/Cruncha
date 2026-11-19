# ADR-0011: User-configurable history retention

**Status:** Accepted (2026-11-18)
**Date:** 2026-11-18 (revised the same day: added saves and size limits; time is measured against document activity, never the calendar)
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
| **Size** | Keep history within *S* on disk (`doc/` and its log, excluding pins) | 500 MB |
| **Activity window** | Keep versions made within *D* days **before the document's most recent edit**, not before today | 90 days of activity |

How the limits work:
- When several limits are set, **the strictest wins**. The user is setting caps, and each one is honored.
- **Floor:** whatever the limits say, the newest **100 versions** are always kept. This is configurable, but never below 10.
- **Named versions are never pruned** (see §3).

The policy is stored per document in the container manifest (`cruncha.json`, `retention` field), written atomically like the format version. The **app-wide default** for new documents lives on the Preferences page (Tauri UI, ADR-0008), and each document's settings can override it.

### 2. Time never prunes on its own

- **Activity-relative time.** The activity window is measured back from the timestamp of the document's latest *edit* commit. Maintenance commits (squash, prune, migration) don't count as activity. A document untouched for five years has exactly the history it had when it was last edited.
- **Pruning needs new edits.** `Store::prune()` runs only when there are edit commits since the last prune. Opening, reading, exporting, time travel, and tagging never prune. So a dormant document can never be pruned, by any limit.
- **Delta's own retention stays disabled permanently.** The table properties stay as in ADR-0003: cleanup off, 100-year windows. Cruncha computes a **cutoff version** itself and passes explicit parameters to the deletion steps. Never derive Delta table properties from the policy.

### 3. Pruning procedure (`Store::prune()`)

Pruning is an explicit maintenance operation, never part of a commit. The app runs it when idle, after editing activity, at most once per session per document.

1. **Compute the cutoff version.** Each limit yields the oldest version it allows:
   - *Saves:* `latest − N`.
   - *Size:* walk versions newest-first, summing each version's added bytes (`add.size` from the log plus its log-entry size) until the budget is reached.
   - *Activity window:* the oldest version whose timestamp is at least `last_edit − D`.

   The cutoff is the **newest** of these (the strictest limit), capped by the floor so the newest 100 versions survive. If the cutoff is at or before the earliest available version, stop: there is nothing to do.
2. **Pin refs.** Every ref older than the cutoff is materialized into `pins/`, a Delta table holding a full copy of that version's cells plus the referenced `data/*` versions. `cells_at_ref` reads pins when the version itself has been pruned. Pins are excluded from the size limit. If pins alone exceed it, the UI says so instead of failing.
3. **Delete data files.** VACUUM `doc/` with an *explicit* retention equal to `now − timestamp(cutoff)`. The window is computed at prune time from the cutoff version, so only files removed before the cutoff can be deleted.
4. **Expire log entries** before the cutoff (write a fresh checkpoint at or after the cutoff first, then clean up the log up to the cutoff version, passing explicit arguments).
5. **Record a maintenance commit:** "history before version *v* (*date*) pruned; policy: …". History then shows a clear marker instead of silently missing versions.

The steps are ordered so that a crash at any point leaves a readable container. At worst, some pruning is left undone and repeats on the next prune.

### 4. API changes

- `Store::history()` and `cells_at()` report the **earliest available version**. A request for a pruned version returns `Error::Pruned { earliest }`, not `NoSuchVersion`.
- The invariant "history length = version + 1" becomes "history length = version − earliest + 1".
- `Store::retention_preview(policy)` returns the cutoff version and date, the bytes it would free, and the refs it would pin, without deleting anything. The UI uses it.

### 5. UI

- **Preferences → History:** "Keep everything" (default), or checkboxes for *last N saves*, *up to S MB*, *D days of activity*. Helper text: "Time limits count back from your last edit; documents you aren't editing are never trimmed. Named versions are always kept."
- **Document settings:** the same control, with "Use app default" as the first option. It shows the current history size and earliest version.
- **Confirmation on tightening:** if a new policy would prune at the next run, show `retention_preview`: what becomes unavailable (up to which version and date), the space freed, the refs that will be pinned, and that this **cannot be undone**.

## Options Considered

**A. Wall-clock age, Delta-native (rejected):** set Delta's retention properties from the policy. It's simplest, but finished studies that sit unopened lose their history, and cleanup runs during unrelated commits. That fails the hard requirement.
**B. One global setting only:** simple, but one setting can't fit both scratch calcs and issued deliverables.
**C. Per-document limits on saves, size, and activity-relative time, with an app default (proposed):** honors the user's caps, never acts on a timer, and protects named versions.
**D. Thinning instead of a hard cutoff (keep hourly, then daily, then weekly versions, like Time Machine):** nicer, but Delta only truncates a prefix of versions; it can't drop the ones in between. Thinning would need history to be squashed and rewritten. Revisit if users ask for it.

## Consequences

- Retention is fully under Cruncha's control: no Delta property ever deletes anything by itself.
- The S3b watch item (tombstones accumulating, which slows reopen and time travel) is relieved for documents with a limit, because pruning expires old log entries and their tombstones. For "Keep everything" documents the follow-up spike stays open.
- CLAUDE.md's invariant becomes: "Only `Store::prune` may VACUUM or expire logs, with explicit parameters derived from the document's policy, after pinning refs. Never change Delta's retention table properties."

## Action Items

1. [ ] `cruncha-store`:
   - `Retention` type in the manifest (`saves`, `size_bytes`, `activity_days`, `floor`);
   - cutoff computation;
   - `retention_preview`;
   - `prune()` with ref pinning and the explicit VACUUM and log-cleanup parameters;
   - `Error::Pruned` and an `earliest_version` API.
2. [ ] Tests:
   - each limit on its own, and combined (strictest wins);
   - the floor;
   - **a dormant document never prunes**: simulate a last edit 5 years ago and run prune;
   - prune only runs after new edits;
   - refs are readable after pruning;
   - the history marker;
   - a crash (SIGKILL) during prune leaves a readable container.
3. [ ] Tauri Preferences and document-settings controls, and the confirmation dialog backed by `retention_preview`.
4. [ ] `data/*` tables follow the same policy (once those tables exist).
5. [ ] Follow-up spike: reopen and time-travel latency at 10,000 versions, comparing "Keep everything" with a saves limit.
