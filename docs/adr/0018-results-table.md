# ADR-0018: Code-cell results are kept in a `results/` table, keyed by what produced them

**Status:** Accepted
**Date:** 2026-11-21
**Deciders:** acmeanvil

## Context

Results don't live in `doc/` (ADR-0003, design 02 §5.2), and so far they live nowhere: the app keeps them in memory. For calc cells that's fine, because the engine recomputes the whole document in milliseconds and the result is deterministic (`jsmath`, spike S2b). For Python cells it's a real gap:
- **Reopening** a document runs every Python cell again. That can take minutes, and needs Python set up and the document trusted.
- **Past versions and snapshots** show Python cells as NOT CALCULATED ("Python results aren't kept for past versions yet"). A report of snapshot "Rev A" can't be printed again (ADR-0015, action item 5), and the History view can't show what a plot looked like.
- **A document sent to someone** who hasn't set up Python, or doesn't trust it, shows nothing for its Python cells, not even the figures that were reviewed.
- **Agents and readers outside Cruncha** (Polars, DuckDB) can't see Python outcomes.
- **Rust cells** (ADR-0006, spike S5, next) have the same need.
- **Figures** (ADR-0017) need somewhere to live with the result.

Today the engine memoizes a Python run by a key: a `DefaultHasher` hash of the source, the names it reads, each calc input's `Debug` text, and the upstream Python cells' keys. That hash isn't stable across Rust versions or platforms, so it can't be stored.

## Decision

1. **A `results/` Delta table**, append-only and created lazily (like `pins/` and `proposals/`). It holds one row per **run of a code cell** (Python now, Rust later):

   | Column | Meaning |
   | --- | --- |
   | `cell_id` | the cell |
   | `input_hash` | SHA-256 of what produced it (item 2), hex |
   | `status` | `done`, `error`, `interrupted` |
   | `values` | JSON: each defined name → its `Plain` value (`cruncha_eval::exchange`) and summary |
   | `outputs` | JSON: the output list (text, tables, figure specs); images and large specs by blob hash |
   | `stdout`, `stderr`, `error` | text, capped at 1 MB each |
   | `duration_ms` | |
   | `env` | the environment label (`env-9a97…`: the Python version and package lockfile) |
   | `computed_at_us`, `computed_by` | Cruncha's clock; `user:<name>@<host>` |
   | `doc_version` | the latest document version when it ran |

   - Images and figure specs go in `blobs/` (ADR-0013: content-addressed, written before the row that references them), so the same figure across versions is stored once.
   - Rows are written when a run finishes, a batch per background evaluation. That is one commit to `results/`, never to `doc/`, so **results never add document versions** and never count as edits for retention.
2. **A stable input hash.** SHA-256 over a canonical encoding of:
   - the cell's kind and source;
   - each name it reads, in order, with its calc value as canonical `Plain` JSON (exact numbers, units), or the upstream code cell's `input_hash`;
   - the environment label (a different lockfile is a different result);
   - the engine's result-format version.

   This replaces `DefaultHasher` for Python memoization too, so the in-memory memo and the table agree.
3. **Reading results back.** A result applies wherever its hash matches. The same hash at any version means the same inputs.
   - **Latest version:** when a Python cell would run, the engine first looks up `(cell_id, input_hash)`. On a hit it uses the stored values and outputs and doesn't run the cell. Its later reads then see those values. A Python cell whose inputs nobody changed doesn't re-run on reopen.
   - **Past versions, snapshots and reports of them:** computed the same way, read-only, never running anything. A cell with no stored result is NOT CALCULATED, as now.
   - Calc cells are **not** stored. They are recomputed, which is cheaper than reading and can't go stale.
4. **Results are data, never trusted code, and never pass silently as checks.**
   - Stored results are shown in **untrusted** documents (showing data runs nothing), marked "computed on <host>, <date>; not run on this computer".
   - In a report, such cells print with the same note, so a reviewer knows the numbers weren't reproduced there. A `@test` that depends on them still evaluates (calcs always run), and the summary notes the source.
   - Results whose `env` differs from this computer's environment are used for display but are **stale for running**. With Python set up and the document trusted, they re-run, as an edited input would.
5. **Size and retention.**
   - Results are a cache: deleting them never loses work that trust and Python can't rebuild. "Clear stored results" in Document settings → History drops the table (with confirmation).
   - Stored results are counted and shown apart from history size. They don't count toward ADR-0011's size limit.
   - `Store::prune` keeps results that the latest version, any version from the cutoff on, or a snapshot (pinned or not) can use, and drops the rest. "Can use" is decided by recomputing those versions' input hashes, which is calc evaluation only, run in the background. Per-row outputs over 64 MB aren't stored; the cell shows its output this session only.
6. **Format:** container format **5**. Older containers need nothing: no table means no stored results. An older Cruncha refuses format 5 (existing rule).
7. **Outside Cruncha:**
   - `results/` is plain Delta and Parquet: Polars or DuckDB can read every Python outcome with its cell id and environment.
   - The text mirror (`read_doc`) gains nothing new; it already shows the engine's current results, which now include stored ones.

## Options Considered

### Option A: keep results in memory only (today)
**Pros:** nothing to build; never stale.
**Cons:** every problem in the Context; reports of snapshots can't be reproduced.

### Option B: results as columns of `doc/` rows
**Pros:** one table, versioned with the cell.
**Cons:**
- a run would be a document version, flooding history and retention with non-edits;
- results would change `doc/` without a human's edit, which breaks "doc/ is what humans agreed to" (ADR-0016);
- re-running after a package upgrade would rewrite the document.

### Option C: results keyed by `(cell_id, doc_version)`
**Pros:** a simple join for "results at version v".
**Cons:**
- a cell unchanged across 500 versions would need 500 rows, or interval bookkeeping;
- it misses the real identity (inputs), so identical runs can't be shared.

### Option D: an append-only `results/` table keyed by input hash (chosen)
| Dimension | Assessment |
| --- | --- |
| Complexity | Medium: a stable hash, a lookup before running, background writes, a prune step |
| Cost | Mostly figures, stored once each in `blobs/`; values and text are small |
| Scalability | One row per distinct run; reads by `(cell_id, input_hash)` with statistics on `cell_id` |
| Team familiarity | The same append-only pattern as `doc/`, `pins/` and `proposals/` |

**Pros:**
- Python outcomes survive reopening and travel with the document;
- past versions and snapshots show and print their Python results;
- memoization and storage use one key.

**Cons:**
- a format bump;
- prune gains a step that evaluates the retained versions;
- stored results from elsewhere need clear labelling.

## Consequences

- **Easier:**
  - reopening without re-running;
  - reports of any snapshot (ADR-0015, action item 5);
  - History showing past plots;
  - sending a document to someone without Python;
  - the same machinery for Rust cells (S5) and figures (ADR-0017).
- **Harder:**
  - the input hash must be canonical (exact `Plain` numbers, ordered keys) and versioned;
  - an engine change that alters a value's encoding must bump the result-format version, or old results would be wrongly reused.
- **Changes elsewhere:**
  - the engine's Python memo uses the stable hash;
  - `Engine` gets a results source (lookup) and sink (new runs);
  - the report prints the "computed on <host>" note;
  - the CLAUDE.md store invariants gain the results rules;
  - `skills/cruncha-code-cells` says when cells don't re-run.

## Action Items

1. [ ] `cruncha-store`: `results/` schema (lazy), `put_results` (blobs first), `results_for(cell_ids)`, `clear_results`, format 5, prune step; tests including a crash between the blob writes and the row.
2. [ ] Stable input hash in the engine (canonical `Plain` JSON, env label, format version), replacing `DefaultHasher`; tests that it is stable and order-independent where it should be.
3. [ ] App: look up before queuing a Python job; write finished runs in the background; past versions and reports read stored results; the "not run on this computer" label; Clear stored results.
4. [ ] Report: print stored Python results for past versions and snapshots, with their provenance.
5. [ ] Update CLAUDE.md, design 02 §5.2 (columns) and `skills/cruncha-code-cells`.
