# ADR-0019: Stored results from another Python environment are found by an environment-free hash

**Status:** Proposed
**Date:** 2026-11-21
**Deciders:** acmeanvil
**Refines:** ADR-0018

## Context

ADR-0018 keys a stored run by `input_hash`, which includes the environment label (§2: "a different lockfile is a different result"). It also says results from another environment are "used for display but are stale for running" (§4). The two contradict each other: a lookup by `input_hash` never finds a run from another environment, so §4 can't happen.

The case is real:
- the environment label is the hash of `python/requirements.lock`, so every Cruncha release that changes a package version gets a new label;
- a document sent to a colleague on an older or newer Cruncha would show its Python cells as not calculated, even though the reviewed output is in the file;
- the same happens to the sender after they upgrade Cruncha.

Reusing such a run as if it were current would be wrong: a package upgrade can change the numbers. Showing it, labelled, is what ADR-0018 intended.

## Decision

1. **A second hash, `base_hash`**, over the same encoding as `input_hash` but without the environment label, under its own domain tag (`cruncha-result-base/<RESULT_FORMAT>`).
   - An upstream Python input contributes its **base** hash, so `base_hash` is free of the environment all the way up a chain.
   - `input_hash` stays exactly as ADR-0018 defines it; nothing already stored changes meaning.
2. **A nullable `base_hash` column** in `results/`. Rows written before it have none and are found only by `input_hash`. Adding a nullable column is the store's existing migration (no data rewritten), so the container format stays **5**.
3. **Lookup order** for a Python cell with no outcome for its inputs:
   1. `(cell_id, input_hash)`: a current result, used as ADR-0018 says;
   2. otherwise `(cell_id, base_hash)`, newest first: a result from **another environment**.
4. **Using a result from another environment:**
   - **If the cell may run here** (Python set up, document trusted, source run before), it runs, as an edited input would. The stored output isn't shown meanwhile; the new run replaces it and is stored under the current environment.
   - **Otherwise** (no Python, not trusted, a past version, an edited cell waiting for Run), the stored output is shown, labelled "computed with a different Python environment (‹env›)", and with the host note when it came from another computer.
   - Calcs read its values, so a dependent check is evaluated. In a report, the cell prints with the note, and the summary counts it as not reproduced here.
5. **Retention:** `retain_after_prune` keeps a row if either of its hashes is looked up by a retained version.

## Options Considered

### Option A: drop the environment from `input_hash`
**Pros:** one hash, one lookup.
**Cons:**
- results after a package upgrade would be reused silently, which is the risk ADR-0018 §2 guards against;
- it changes the meaning of every row already stored.

### Option B: match by cell id only and compare inputs at read time
**Pros:** no new column.
**Cons:**
- inputs aren't stored, so they can't be compared;
- storing them duplicates the document's values in every row.

### Option C: an environment-free `base_hash` column (chosen)
| Dimension | Assessment |
| --- | --- |
| Complexity | Low: a second hash beside the first, one fallback lookup |
| Cost | 64 bytes per row |
| Compatibility | Additive nullable column; existing rows keep working |

**Pros:**
- §4 of ADR-0018 works as written;
- exact reuse stays strict;
- documents survive Cruncha upgrades and travel between versions.

**Cons:**
- two hashes to keep in step: both bump with `RESULT_FORMAT`.

## Consequences

- **Easier:** sending documents between Cruncha versions; upgrading packages without losing the reviewed output of untrusted or past documents.
- **Harder:** the engine computes and threads two hashes per Python cell.
- **Changes elsewhere:**
  - `engine::py_input_hash` gains a sibling;
  - `PyView.stored` says when the environment differs;
  - the report note and the `cruncha-code-cells` skill mention it;
  - CLAUDE.md's results invariant names both hashes.

## Action Items

1. [ ] `cruncha-store`: nullable `base_hash` column (migrated on open), `RunRecord.base_hash`, a lookup by `(cell_id, base_hash)`; tests.
2. [ ] Engine: `base_hash` beside `input_hash` (golden test); fall back to it when the cell can't run here; the label.
3. [ ] App and report: the "different Python environment" note; retention keeps either hash.
4. [ ] Update CLAUDE.md and `skills/cruncha-code-cells`.
