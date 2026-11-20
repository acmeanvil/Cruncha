---
name: cruncha-history
description: Use a Cruncha document's version history, which is kept automatically because every change is committed with no Save step. Covers listing versions and who made them (human, AI, CLI), diffing versions or tags, explaining what changed since a revision and which results moved, and proposing a rollback. Use it for questions like "what changed since rev A?", "who changed this load?", or "undo yesterday's edits".
---

# Document history

Every change to a Cruncha document is an atomic, versioned commit (ADR-0003):
- Versions are numbered `0, 1, 2…`.
- Each records its **actor** (`user:<name>`, `ai:<session>`, `cli`), a message, a timestamp, and a kind (`edit`, `restore`, `maintenance`).
- **Snapshots** (tags, or refs, in the store) name known-good versions between major changes, such as "Checked" or "Rev A, issued for review", with an optional note. Users take them from the History panel. Snapshot names are unique within a document. A snapshot is the natural baseline for "what changed since…?".

History is never rewritten:
- A rollback is a **new** version, so it can itself be undone.
- `maintenance` versions (compaction) don't change content. Leave them out of summaries.
- **Retention:** a document may have a history-retention policy, set in Preferences or in the document's settings (ADR-0011): a cap on saves, on size, or on days of *activity*, counted back from the last edit, not from today. Pruning happens only after new edits, so a document nobody is editing never loses history. Versions past the cutoff can be pruned, and a maintenance version marks where (its message reads "history before version *v* (*date*) pruned"). History then starts at the earliest available version, and asking for anything older fails with "pruned". **Snapshots (tags) are always kept**, and can still be viewed and restored after their version is trimmed. If a request reaches past the earliest available version, say so plainly and offer the nearest snapshot instead. Never imply the older history still exists.

## Tools

- `history(cell_id?, since?)`: the version list, or the versions that touched one cell.
- `history(diff: {from, to})`: per-cell changes (added, removed, modified), with source and result diffs. `from` and `to` can be version numbers or tag names.
- `propose_edit(restore: version | snapshot)`: proposes a rollback as a normal proposal, which the user accepts. Restoring adds a new version with the old content (message "restore to version *v*" or "restore to “*name*”").
- Agents don't take snapshots: a snapshot asserts that a human considers the version good. Suggest one when a check is complete instead.

## Answering "what changed?"

1. Resolve the baseline: a tag name, a date (the latest version at or before it), or "my last session".
2. Diff the baseline against the current version.
3. Report in this order:
   1. **Changed inputs.** Old → new value, with the actor and version.
   2. **Results that moved.** Old → new, with the percentage change. Lead with those that affect `@test`s, and any test that changed between pass and fail.
   3. Structural changes (cells added or removed), and prose edits (summarized).
4. Point out changes by `ai:*` actors that are still `ai:proposed`.

## Rollback

- Rolling back the whole document is a big step. Confirm the target version with the user, and list what will be lost compared with now.
- Prefer restoring **specific cells** to an older version over a whole-document rollback, when only a few cells are wrong.
- Never use rollback to hide a change. The history keeps both, and that's intended.
