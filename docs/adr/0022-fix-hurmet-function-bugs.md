# ADR-0022: Fix the built-in function bugs inherited from Hurmet

**Status:** Proposed
**Date:** 2026-11-22
**Deciders:** acmeanvil
**Refines:** ADR-0002 (where Cruncha deliberately differs from Hurmet)

## Context

The calc evaluator (`crates/cruncha-eval`, spike S2b) is a faithful port of Hurmet's `src/` at commit `87f30ed0`, bugs included. Spike S2b treated that faithfulness as a feature: Hurmet's 250 golden tests pass unchanged.

Writing the function reference (`reference/functions.json`, the ƒx Functions palette) ran every built-in through the engine. A follow-up probe on 2026-11-22 confirmed these defects, all ported from Hurmet:

| Function | Today | Correct | Cause |
| --- | --- | --- | --- |
| `median` | `median(1, 2, 10)` = 5.5 | 2 | It computes the midrange `(max + min) / 2` (`functions.rs`, `reduce_fn`). |
| `mod` | `mod(-7, 3)` = 1; `mod(7.5, 2)` = 3 | 2; 1.5 | `Rational::modulo` takes the remainder of the quotient's numerator and denominator, then its absolute value. |
| `rem` | `rem(7.5, 2)` = 3 | 1.5 | The same mistake as `mod`, without the absolute value. Right only for integers. |
| `hypot` | `hypot(-3, -4)` = −5 | 5 | It takes no absolute values (`Rnl.hypot` does; this copy doesn't). |
| `rms` | `rms(3, 4)` = 5 | 3.5355… | An alias of `hypot`, not the root mean square its name promises. |
| `asec`, `acot`, `asecd`, `acotd` | `asec(2)` → error | 60° (1.0472) | A range check that is inverted (it rejects \|x\| ≥ 1). It then calls a missing `Math.atn`. The error text also says "arcecant". |
| `acsc`, `acscd`, `acsch` | "not supported" | `asin(1/x)` and so on | Hurmet calls a missing `Math.atn` (a TypeError). |
| `variance`, `stddev` | "not supported" | sample variance and standard deviation | Hurmet calls a missing `Rnl.pow`. |
| `trace` | "not supported" | the sum of the diagonal | Not implemented in Hurmet. |
| `Char` | "Function Char not found" (unit-aware) | `Char(65)` = "A" | Hurmet's golden `Char(87)` passes; the unit-aware path doesn't reach it. To be diagnosed. |
| `e` | "Variable e not found" | Euler's number | The evaluator has a branch for `e`, but the parser emits it as a variable. |
| `roundSig` | "does not understand the expression" | `roundSig(3.14159, 3)` = 3.14 | The parser knows it; the evaluator has no case for it. |

The worst row is `median`: it silently gives a plausible wrong answer. The reference's example happened to hide it (`median(5, 1, 3)` is 3 either way). The non-integer `mod`/`rem` and the negative `hypot` are also silent. The rest fail loudly.

For now, the reference and `skills/cruncha-calc-authoring` carry cautions for each row, and unavailable functions are marked as such.

What makes this a decision rather than a bug fix:

- **Calc results are never stored** (CLAUDE.md, ADR-0018); they are recomputed on open.
  - A fix changes the results shown for every version of every document, past versions and snapshots included.
  - It changes the reports printed from them. "Same version → byte-identical PDF" (ADR-0015) then holds only within one Cruncha release.
- **Python and Rust cells downstream re-run.** Their input hash includes the calc values, so the stored runs no longer match.
- **It departs from Hurmet on purpose.** ADR-0002 asks for those departures to be documented.

## Decision

1. **Fix them in the engine, in place, and say so.** The table's "Correct" column is the target. Specifically:
   - **`mod(a, b)`** (and the operator `a mod b`, which Hurmet refuses when unit-aware) is floored modulo, `a − b·⌊a/b⌋`, with the sign of `b` (as in Excel `MOD`, Python `%` and Mathcad `mod`). **`rem(a, b)`** is truncated, with the sign of `a`.
     - Both are exact on rationals, including non-integers. `b = 0` is an error.
     - The arguments share a unit, and the result is in that unit.
   - **`hypot(a, b, …)`** takes absolute values, any number of arguments, and is 0 when all are 0.
   - **`rms(v)`** becomes a reducer, like `mean`: `√(Σxᵢ² / n)` over its arguments or a vector's elements. Documents that used it as `hypot` must change.
   - **`median(v)`** is the middle value, or the mean of the two middle values.
   - **`variance(v)`** and **`stddev(v)`** are the *sample* statistics (n − 1), as Excel's `VAR.S` and `STDEV.S` and Python's `statistics` are. The variance's unit is the square of the arguments'.
   - **Inverse trigonometric functions:**
     - `asec(x) = acos(1/x)` and `acsc(x) = asin(1/x)`, for \|x\| ≥ 1 (an error otherwise, with the right name).
     - `acot(x) = atan(1/x)`, with `acot(0) = π/2`.
     - `acsch(x) = asinh(1/x)`.
     - The degree forms likewise.
     - All of them go through `cruncha_units::jsmath`.
   - **`trace(M)`** is the sum of a square matrix's diagonal. **`Char(n)`** works unit-aware.
   - **`e`** is Euler's number unless the document defines `e`; engineers often use `e` for eccentricity. A definition wins, as it does for any built-in name. The variable browser marks the name as shadowing a constant.
   - **`roundSig(x, n)`** rounds to n significant digits, as `roundn(x, "rn")` does. (An earlier draft dropped it, thinking the parser didn't know it; it does.)
2. **Tests guard each fix.**
   - Each fix gets a test in `cruncha-eval`, with negative, non-integer, unit and empty-argument cases.
   - The reference's examples are changed to inputs that tell the right answer from the wrong one: `median(1, 2, 10)`, `mod(-7, 3)`, `mod(7.5, 2)`, `hypot(-3, -4)`. Their cautions are removed.
   - Hurmet's golden tests stay green. Where a golden encodes a bug, its expectation changes, with a comment naming this ADR. None does today: the goldens use `hypot(3, 4)`, `Char(87)` and complex `asec`/`acot`, all correct.
3. **A list of differences from Hurmet** (`docs/design/03-hurmet-differences.md`) records this ADR's changes, alongside ADR-0004 (one definition per name) and ADR-0007 (always unit-aware). Importing a Hurmet document is the place that list is read.
4. **Results change without a migration.** The project is early and every document is the owner's. A report's provenance already names the Cruncha version that printed it (`cruncha_version`), so a re-printed revision shows why its numbers differ.
5. **The skill changes in the same commit** (`skills/cruncha-calc-authoring`, Built-in functions).

## Options Considered

### Option A: Fix in place (chosen)

| Dimension | Assessment |
| --- | --- |
| Complexity | Low: local changes in `functions.rs`, `rational.rs`, `eval.rs` and the parser's constants |
| Cost | Every document's results may change once, on upgrade |
| Scalability | No extra machinery |
| Team familiarity | Ordinary engine work |

**Pros:** correct answers everywhere, including old versions; no second set of semantics to maintain.
**Cons:** a past revision re-printed after the upgrade can differ from the PDF issued at the time, so the issued PDF remains the record.

### Option B: A semantics level per document

The meta cell records a calc-semantics level. Documents created before this change keep the old behaviour until the user upgrades them, with a list of affected cells.

| Dimension | Assessment |
| --- | --- |
| Complexity | Medium–high: two code paths per function, an upgrade flow, tests for both |
| Cost | Ongoing, for every later semantics change |
| Scalability | Needed once there are users with issued calculations |

**Pros:** old versions reproduce exactly.
**Cons:** keeps known-wrong answers alive, for little benefit while no one else has documents. Worth revisiting before the first public release, as a general mechanism rather than for these bugs.

### Option C: New names, old names unchanged

Keep `mod`, `median` and `rms` as Hurmet has them, and add `modulo`, `med` and `rmsq`.

**Pros:** nothing changes for existing calcs.
**Cons:** keeps a wrong `median` under its natural name, which is a trap for every new user and agent. Rejected.

## Trade-off Analysis

The choice is between reproducing past results exactly and being correct. While Cruncha has no issued calculations from other users, being correct costs almost nothing. The issued PDF stays the record of what was issued; history, reports and stored runs all follow the current engine.

A per-document semantics level (Option B) is the right tool once that changes. It belongs in its own ADR, applied to any future semantics change, not bolted onto these fixes.

## Consequences

- `median`, `mod`, `rem` and `hypot` stop giving silent wrong answers. The functions marked "not available yet" become available.
- Documents using `rms(a, b)` as `hypot` get different numbers. A lint could flag two-argument `rms` for a release or two.
- Stored Python and Rust runs downstream of a changed calc no longer match their input hash, so those cells re-run when they are next run. Nothing is lost: `results/` is append-only.
- Cruncha's engine and Hurmet's now differ on these functions. Hurmet documents imported later are read with Cruncha's meaning, and the differences list says so.
- **To revisit:** a calc-semantics level per document (Option B) before the first public release.

## Action Items

1. [x] Fix `mod` and `rem` (exact floored and truncated remainders on rationals, units, `b = 0`) in `cruncha-units` and `cruncha-eval`.
2. [x] Fix `hypot` (absolute values, n arguments, all zero) and make `rms` the root mean square, a reducer.
3. [x] Fix `median`; implement `variance` and `stddev` (sample; unit²).
4. [x] Implement `asec`, `acsc`, `acot`, their degree forms and `acsch` for real arguments, and fix the error name.
5. [x] Implement `trace`. Diagnose and fix `Char` on the unit-aware path.
6. [x] Make `e` Euler's number when the document doesn't define it, and mark the shadowing in the variable browser.
7. [x] Update `reference/functions.json`:
   - examples that distinguish right from wrong;
   - cautions removed;
   - `roundSig` implemented.

   Then update the skill and write `docs/design/03-hurmet-differences.md`.
8. [x] Keep Hurmet's goldens green, and mark any changed expectation with this ADR.

9. [x] Unit names and data (found while building the Units palette, 2026-11-22). A name in Hurmet's unit table is now that unit before a plural *s* or an SI prefix is stripped. Before, `Celsius`, `gauss`, `kilogram`, `kilobyte`, `centistoke`, `dekan` and `megaton TNT` were "unknown", and `mas` (milliarcsecond) read as milli-annum. Two rows of the table had wrong dimensions, corrected in `tools/gen_table.py`: tesla and gauss (kg·s⁻²·A⁻¹), and ampere-turn and gilbert (a current, not temperature × luminous intensity). `crates/cruncha-units/tests/catalog.rs` reads every name in the table.

Done 2026-11-22. Each change is covered by `crates/cruncha-eval/tests/functions.rs`, the reference's examples and an engine test for `e`. Hurmet's goldens needed no changes. Found along the way: `mod` dropped units, and `Rational::power` used the broken `mod` to detect odd roots; both are fixed and noted in `docs/design/03-hurmet-differences.md`.

### Second set: found 2026-11-22, to correct

A sweep of the port's "Hurmet bug, reproduced" comments, each probed in a calc (unit-aware, as in the app). These give wrong answers without an error, so they are fixed under this ADR's same decision (in place, documented, tests first). Until then, the reference and `skills/cruncha-calc-authoring` warn about each.

| Where | Today | Correct | Cause |
| --- | --- | --- | --- |
| Fixed display format (`format = "f2"`, `roundn(x, "f2")`, `string`) | −0.25 shows as **0.25**; any value between −1 and 0 loses its sign | −0.25 | `Rational::to_fixed`: the integer quotient is 0, so its sign is lost. |
| `×` (cross product) | `[1, 0, 0] × [0, 1, 0]` = `[]` | `[0, 0, 1]` | The vector branch checks the length of the first *element* (`x[0].length`, 2 for a Hurmet rational). |
| `sum`, `product`, `mean`, `min`, `max`, `range`, `median` of a matrix | `sum([1, 2; 3, 4])` = 0 | 10 (over every element; `sum(A, 1)` and `sum(A, 2)` already work) | Hurmet tests `args.dtype & MATRIX` on the argument list (always 0), so the rows are folded as numbers. |
| `product` units | `product(2 'm', 3 'm')` = 6 m | 6 m² | The exponents are summed and then discarded. |
| A vector in an offset unit | `[20; 30] '°C'` = [−166.52; −156.52] °F | [68; 86] °F | With a gauge, the vector conversion subtracts the gauge and drops the factor (a single value is right). |
| `zeros(m, 1)`, `ones(m, 1)` | `zeros(3, 1)` = `[0]` | a 3-element column | The column vector's length is n, not m. |
| `^` on a two-element vector | `[2, 3]^2` = 4 + j 9 | `[4, 9]` | Hurmet's `isComplex` accepts any pair of rationals. |
| `atan2`, `gcd` (and other two-argument functions) | `atan2(1 'm', 1 's')` = 0.785 | an error: the units differ | The unit check's error is discarded (`unwrap_or(None)`); `mod` and `rem` already check. |
| Matrix literal of column vectors, inside a larger expression | `2 + sum([x, y])` → "does not understand" | 2 + the sum | The vectors are read from the bottom of the stack, not from the first argument. Loud, not silent. |
| Data frames | to probe | — | Three reproduced bugs in `dataframe.rs`: a row range "all" uses the number of columns; a column range looks up `columnMap[NaN]`; a selection's column keys are the source's k-th headings. |

10. [ ] Fix the sign of fixed-format values between −1 and 0 (`to_fixed`), in results, `roundn` and `string`. This is the most urgent: it changes what a report prints.
11. [ ] Fix the cross product, and the reducers over a matrix (`sum`, `product`, `mean`, `min`, `max`, `range`, `median`).
12. [ ] Fix `product`'s unit, offset units on vectors, `zeros`/`ones` columns, and `[a, b]^n`.
13. [ ] Check units in every two-argument function that requires one unit (`atan2`, `gcd`, …), as `mod` does.
14. [ ] Fix the column-vector matrix literal, and probe and fix the three data-frame bugs.
15. [ ] For each: a case in `tests/functions.rs`, the caution removed from the reference, the skill and `docs/design/03-hurmet-differences.md` updated.
