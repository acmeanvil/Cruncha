# ADR-0023: Archive major versions from the History tab

**Status:** Accepted (2026-11-22)
**Date:** 2026-11-22
**Deciders:** acmeanvil
**Refines:** ADR-0011 (retention and pruning). It uses ADR-0016's and ADR-0018's prune rules unchanged.

## Context

Every save is a version, and ADR-0011 keeps them all by default. That's about 7 KiB per save, so a document worked on for months reaches thousands of versions and tens of MB. The History tab also gets long. And the reopen and time-travel cost grows with the total number of versions (ADR-0011 item 5).

ADR-0011's limits (saves, size, activity window) trim automatically from a policy, and "Trim history now" applies that policy once. Neither one lets the user say what they actually mean: *"Rev A through Rev C are issued and done; keep the issued documents, drop the hundreds of saves in between."*

The owner asked for archiving from the History tab:
- The user picks major versions to archive, or everything older than a chosen major version.
- Archived versions leave the History tab but stay visible to agents.

A **major version** is a snapshot group, as the History tab shows it (app/src/lib/history.ts). It is a snapshot ("Rev A") and the saves after it until the next snapshot ("Rev A · 1" … "Rev A · n"). Saves before the first snapshot are the "Draft" group.

**Constraint:** Delta can only drop a *prefix* of versions (ADR-0011 option D). Removing a group from the middle of history would mean rewriting it, which the store forbids ("History is never rewritten").

## Decision

### 1. What can be archived: the oldest groups, in order

Archiving always takes a contiguous run of groups starting from the oldest unarchived one. Ticking a group ticks every older group too, so "these major versions" and "everything older than this one" are the same action (owner's choice, 2026-11-22).

The newest group (the one being edited) can't be archived.

Archiving groups up to and including group *G* uses one cutoff: the version of the snapshot that starts the group after *G*. That snapshot, and everything after it, stays as it is.

### 2. What is kept of an archived group: its snapshot and a save log

- **The snapshot**, as the document was at that version. Prune already pins it into `pins/` (ADR-0011 step 2). It stays restorable (`Store::restore_ref`), printable as a report, and readable by agents.
- **A save log**: one row per archived save, with its version, `cruncha.at`, actor, message, kind (edit, restore, accepted proposal) and the IDs of the cells it changed. It's a record of who changed what and when, and it is much smaller than the versions it describes.

The individual saves between snapshots can't be opened any more. This can't be undone, and the confirmation says so.

### 3. Storage: the `archive/` table (container format 6)

`archive/` is a new append-only Delta table, created by the first archive or prune. It has one row per removed save:

| Column | Type | Note |
| --- | --- | --- |
| `version` | u64 | key |
| `at_us` | i64 | `cruncha.at` |
| `actor`, `message`, `kind` | string | from the commit metadata |
| `snapshot` | string, null | the group's snapshot; null for Draft |
| `cells` | string | JSON array of the IDs of the cells the save wrote (its rows with `rev` = its version) |
| `removed_before` | u64 | the cutoff of the prune that removed it |

- **Every prune writes it**, whether it's an archive or an automatic retention prune. Trimming by policy therefore no longer erases the record of who changed what.
- **The rows are written after pinning and before the point of no return** (ADR-0011 step 4). Writing is idempotent by `version`, so an interrupted prune that resumes doesn't duplicate them.
- **It is never pruned itself.** At about 100 bytes per save, 10,000 saves come to roughly 1 MB.
- **Older containers** are bumped to format 6 when this version opens them (no data rewritten), as with formats 4 and 5. The table itself is created by the first prune or archive that removes a save.

### 4. Store API

`Store::archive_before(cutoff, actor)` runs the same pruning steps as `prune`, with three differences:
- **The cutoff is explicit.** It must be a snapshot's version, after the earliest available version, and not inside the newest group.
- **It doesn't wait for new edits.** ADR-0011's "pruning needs new edits" rule protects against *time* pruning a dormant document. An archive is an explicit act by the user, confirmed in a dialog, so the rule doesn't apply.
- **The floor of 100 versions doesn't apply.** The user chose this cutoff. The confirmation lists exactly what goes.

The marker commit says "archived Rev A–Rev C (versions *a*–*b*)" and carries `cruncha.prunedBefore` as before.

New read APIs:
- `Store::archive_preview(cutoff)` returns the groups, the number of saves, the bytes freed and the snapshots pinned.
- `Store::archived(cell_id?)` returns save-log rows.

### 5. UI

- **History tab:**
  - "Archive…" puts a checkbox on each group header except the newest. Ticking one ticks every older group.
  - The confirmation shows the groups, the save count, the space freed, that snapshots stay restorable, and that the saves themselves can't be reopened.
  - Archived groups leave the tab. One line at the bottom remains: "Archived: Draft – Rev C · 1,234 saves · Show". It opens the archive in a dialog, not in the tab: archived snapshots (open read-only, restore, report) and the save log, searchable by cell.
- **Document settings → History:** the size, and "Trim history now" (ADR-0011), as before.

### 6. Agents

- `history` takes `archived: true` to list the save log and archived snapshots. Without it, it lists available versions, as now. A last line of the normal listing says how many saves were archived and which snapshots were kept.
- `read_doc` takes `snapshot: "Rev A"` to read any snapshot, archived or not, as read-only mirror text with the results stored for it. Calc results come from evaluating the pinned cells; stored code-cell results are looked up as in ADR-0018.
- `skills/cruncha-history` documents both.

## Options Considered

### A. Archive oldest groups, keep snapshot + save log (chosen)

| Dimension | Assessment |
| --- | --- |
| Complexity | Low to medium: reuses prune, pins and the marker; adds one table and an explicit cutoff |
| Space | Freed immediately: all intermediate saves of archived groups |
| Integrity | Never rewrites history. Issued snapshots stay exact and restorable. A record of every save remains |

### B. Archive any groups; hide middle ones now, free their space later

This was offered to the owner and declined. It is more flexible, but a hidden group still takes its full space until everything older goes, and "archived but not freed" is hard to explain.

### C. Move archived saves into a separate archive file

This keeps every save openable, but it only moves the space, and it adds a second container to carry along with the document.

### D. Squash history (rewrite)

This can drop any range, but it breaks "history is never rewritten" and every version number after the range. Rejected.

## Trade-off Analysis

The owner preferred a simple rule with immediate savings (A) over freedom of choice with deferred savings (B). Keeping the snapshot and a save log, rather than every save, matches how engineering records work. The issued revisions must be exact. Between them, an audit needs who changed what and when, not every intermediate state.

## Consequences

- **The History tab shows recent work, and space is freed when the user decides.** Automatic limits (ADR-0011) still work as before, and now also leave a save log.
- **An archived save can't be reopened**, only its snapshot. "Restore to version *n*" for archived *n* is refused with `Error::Pruned`, as today.
- **Agents can read old revisions for the first time**, through `read_doc(snapshot)`, whether archived or not.
- **The tombstone growth noted in ADR-0011 is not fixed by this.** Archiving deletes files and log entries, but checkpoint tombstones stay (ADR-0011 item 5).
- **Container format 6.** Older Cruncha builds refuse to open a document once this version has opened it, as with every format bump.

## Action Items

**Implementation notes (2026-11-22):** a save's group is the latest snapshot at or before it, so a snapshot's own version belongs to its group (`snapshot` = that snapshot), as the History tab labels it. The cells a save wrote are read with one table load per maintenance-bounded window (between squashes `doc/` is only appended to), about one load per 50 versions.

1. [x] `cruncha-store`:
   - the `archive/` table and format 6;
   - save-log rows written in `prune`, idempotent, before the point of no return;
   - `archive_before`, `archive_preview` and `archived`;
   - tests: oldest-first validation, the newest group refused, a dormant document archives on request, the crash test extended to an interrupted archive, and automatic prunes write the log.
2. [x] App: History tab archive mode, the confirmation, the archive line and dialog (snapshots read-only, restore, report; the save log by cell).
3. [x] MCP: `history(archived)` and `read_doc(snapshot)`; update `skills/cruncha-history` and `skills/cruncha-core`.
4. [x] CLAUDE.md: the `archive/` table in the store invariants and in the pruning order.
