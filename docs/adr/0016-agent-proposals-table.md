# ADR-0016: Agent proposals live in a `proposals/` table, applied by a human's commit

**Status:** Accepted
**Date:** 2026-11-20
**Deciders:** acmeanvil

## Context

The rules (`skills/cruncha-core`, ADR-0009):
- Agents never write a document. `propose_edit` creates changes a human accepts or rejects.
- Design 02 decision **D3** said a proposal would be "pending rows (`doc` rows with `proposal_id`) … Accepting commits them; rejecting deletes them."

Since D3, the `doc` table became an append-only revision log (ADR-0010):
- reads keep the highest `rev` per cell id;
- every commit is `version + 1`;
- history is never rewritten.

D3's pending rows conflict with all three:
- a pending row is the newest revision of its cell, so a plain read would show unaccepted content;
- "rejecting deletes them" rewrites history;
- each proposal would add versions to the document's own history before anyone agreed to it.

Proposals also need things `doc` rows don't have:
- who proposed and why;
- the version the agent looked at (to detect conflicts);
- a status (open, accepted, rejected);
- several cells changed as one unit.

## Decision

1. **A separate append-only Delta table, `proposals/`**, created the first time a document gets a proposal (as `pins/` is). It holds one row per state change of a proposal:

   | Column | Meaning |
   | --- | --- |
   | `proposal_id` | ULID |
   | `seq` | 0 when created, then +1 per status change; reads keep the highest |
   | `status` | `open`, `accepted`, `rejected`, `withdrawn`, `stale` |
   | `actor` | `ai:<session>`; for decisions, `user:<name>` |
   | `message` | the agent's description, which a reviewer reads |
   | `base_version` | the document version the agent read |
   | `changes` | JSON: `insert { after, cell }`, `modify { id, base_rev, source?, attrs? }`, `delete { id, base_rev }` |
   | `at_us` | Cruncha's clock |
   | `accepted_version` | set on `accepted` |

   Nothing is deleted when a proposal is decided; a status row is appended.
2. **Accepting is an ordinary commit by the human.**
   - `Store::commit` applies the changes with provenance `ai:accepted { session, by }` and actor `user:<name>`.
   - The commit metadata gets a new key, **`cruncha.proposal`** = the proposal id.
   - The `accepted` row is appended after the commit. If the app dies in between, opening the document finds the commit carrying `cruncha.proposal` and appends the missing row, so the recovery is idempotent.
   - The document's history shows exactly one version per accepted proposal and nothing for rejected ones.
3. **Conflicts are detected, never merged silently.** Each `modify` and `delete` records the `rev` of the cell it was based on. If that cell has changed since, the proposal is **stale**.
   - The card shows the current source next to the proposed one.
   - The user can reject it, ask the agent to redo it against the latest version, or accept it anyway, which replaces the current source.
   - Inserts anchor to the cell they follow. If that cell was deleted, the insert goes at the end of its section.
4. **What if?** The engine evaluates every open proposal on a scratch copy of the latest version, in memory, with no commit. The card and the agent's `propose_edit` reply show each affected cell's result before and after, labelled "if accepted".
   - Python cells changed by a proposal aren't run until accepted and Run, as with any edit (ADR-0005), so their results show as not calculated.
   - Names are checked on the combined document. A proposal that would create a duplicate name or a cycle is refused at `propose_edit`, with the graph's error.
5. **Granularity:** accept or reject a whole proposal. To accept only part, the user rejects it and asks for a smaller one; agents are told (skill) to make one coherent proposal per logical change.
6. **Retention:**
   - Open proposals are always kept.
   - Decided proposals follow the document's retention policy (ADR-0011). Their status rows are history like any other: pruned with the same cutoff, by the same `Store::prune` steps (VACUUM with the explicit retention, then expired logs).
   - A proposal accepted into a pinned version keeps its rows while the pin exists.
7. **Format:** container format **4**.
   - Older containers need no migration: no `proposals/` table means no proposals.
   - An older Cruncha refuses format 4 (existing rule), so it can't lose open proposals by ignoring the table.

## Options Considered

### Option A: pending rows in `doc/` (design 02 D3)
**Pros:** one table.
**Cons:**
- breaks the "highest rev wins" read rule and the version-per-commit invariant;
- rejecting rewrites history;
- unaccepted content leaks into every reader that doesn't know about proposals (mirror, reports, prune).

### Option B: Delta branches (a copy of `doc/` per proposal)
**Pros:** full isolation.
**Cons:**
- Delta has no native branching; it would mean a table per proposal and our own merge;
- heavy for a three-cell change.

### Option C: proposals only in memory
**Pros:** trivial.
**Cons:**
- lost when the app closes;
- no record of what an agent proposed and what was rejected, which a calc package's reviewer may need.

### Option D: a `proposals/` table (chosen)
| Dimension | Assessment |
| --- | --- |
| Complexity | Medium: one more append-only table, the same reduce pattern as `doc/` |
| Cost | Small: proposals are a few KB |
| Scalability | Fine: reads reduce by `proposal_id` |
| Team familiarity | Same machinery as `doc/`, `refs/`, `pins/` |

**Pros:**
- `doc/` stays the record of what humans agreed to;
- proposals have their own history, conflicts are detectable, and accepting is one atomic commit.

**Cons:**
- a format bump;
- an accept is two writes (commit, then status), made idempotent by `cruncha.proposal`.

## Consequences

- **Easier:**
  - tracked-change cards with result diffs;
  - an audit of agent activity ("what did the agent propose, who accepted it");
  - several agents proposing at once (each proposal is independent; staleness catches collisions).
- **Harder:**
  - the editor must show open proposals without making them content (ghost blocks or a side panel, never document nodes that DocSync would commit);
  - prune gains a table.
- **Changes elsewhere:**
  - design 02 D3 is superseded by this;
  - the store invariants in CLAUDE.md gain the proposals rules;
  - `skills/cruncha-core` documents `propose_edit`'s change format and the "if accepted" results.

## Action Items

1. [x] `cruncha-store`: `proposals/` schema and table (lazy), `propose`, `decide`, `open_proposals`, recovery on open, format 4, prune support; tests including a crash between the accept commit and the status row.
2. [x] App: what-if evaluation on a scratch engine; staleness check; `accept_proposal` and `reject_proposal` commands.
3. [x] `cruncha-mcp`: `propose_edit(changes, message)` with validation, returning the proposal id and the "if accepted" results; `withdraw` for the agent's own open proposals.
4. [x] UI: a Proposals tab (cards with source and result diffs, Accept / Reject, stale warnings) and markers in the document next to affected cells.
5. [x] Update design 02 (D3), CLAUDE.md invariants, and `skills/cruncha-core`.
