---
name: cruncha-calc-review
description: Review a Cruncha calculation package the way a checking engineer would. Covers unit and dimensional sanity, traceable inputs and assumptions, governing load cases, independent re-derivation of key results with a second method, test coverage and AI provenance, reported as findings tied to cell IDs. Use it when asked to check, review, QA, or "sanity check" a calc, or before a document is tagged for issue.
---

# Reviewing a calc package

You are the second set of eyes. **Report findings; don't fix silently.** Fixes are separate `propose_edit` proposals, made only if the user asks for them.

## Procedure

1. **Read and run.** Use `read_doc`, then `run()` so every result is current, then `check()` for the automatic findings: unit errors, names used before they're defined, unfilled placeholders, failing tests and cycles.
2. **Inventory the inputs.** List every input cell: its value, unit, and stated source. Flag:
   - values with no source;
   - placeholders (`_`) still present;
   - inputs that look out of range for the stated material or code. A typical range is a heuristic: say "unusual", not "wrong".
3. **Check the assumptions in the prose.** Support conditions, load combinations, code edition, units system, and load paths. Flag missing or contradictory assumptions.
4. **Trace the governing result.** For each `@test` and each key result, `explain(cell_id)` and follow its dependencies back to the inputs. Confirm that the formula matches the cited clause.
5. **Re-derive independently.** For the most important results, compute them a second way in a **scratch proposal**:
   - a SymPy derivation;
   - an alternative formula;
   - a hand-check simplification;
   - or a Python cross-check.

   Compare the engine results. A disagreement beyond rounding is a finding. Never quote your own arithmetic as the check; run it.
6. **Coverage.** Every acceptance criterion should be an `@test`. Missing tests are findings.
7. **Provenance.** Count results that depend on cells still marked `ai:proposed` or `imported`, and list them. They need human acceptance before the document is issued.

## Findings format

Report a table, most severe first:

| # | Severity | Cell | Finding | Evidence | Suggested action |
| --- | --- | --- | --- | --- | --- |
| 1 | **Blocker** | 01JC3… `ϕM_n` | Uses `F_y = 50 ksi` but prose specifies A36 steel | `explain` shows input cell 01JC1…; prose ¶2 | Confirm grade; change one or the other |

Severity levels:
- **Blocker:** a wrong or unsupported result that affects an acceptance criterion.
- **Major:** a missing check, an untraceable input, or an untested criterion.
- **Minor:** clarity, presentation, or naming.
- **Note:** an observation with no action needed.

End with a one-paragraph summary:
- whether the package is ready to issue;
- how many blockers remain;
- what share of key results was re-derived and agreed.
