# ADR-0025: Fix the calc engine gaps found while converting templates

**Status:** Accepted (2026-11-24)
**Date:** 2026-11-24
**Deciders:** acmeanvil
**Refines:** ADR-0022 (fixing inherited bugs in place), ADR-0002 (where Cruncha differs from Hurmet)

## Context

Converting Lacewing Lab's Mechanical › Stress and failure group (ADR-0014 item 7, 22 templates) ran the calc engine harder than the pilot did. Each template works around what it hit. Each problem below was reproduced on 2026-11-24 with `prepare`/`evaluate`, unit-aware as in the app.

**Wrong or missing results (bugs):**

| # | Calc | Today | Correct | Cause |
| --- | --- | --- | --- | --- |
| 1 | `sqrt(0)`, `0^0.5`, `sqrt(a^2)` with `a = 0 ksi` | "Error while taking root." | 0 | `Rational::power`: a zero base isn't positive, and an even root denominator falls through to `None` (`0^(1/3)` works) |
| 2 | `@test 0.1 ≤ g ≤ 40`, `@test (0.1 ≤ g) and (g ≤ 40)` | shows `true`/`false`; the check is unset | a check that passes or fails | `result.rs` sets `check` only when the RPN ends in a single comparison, so the template runner, `check` and reports see "not evaluated" |
| 3 | `{1 if 0.1 ≤ g ≤ 40; 2 otherwise}` | "Logic operation “if” on a non-boolean value" | 1 | A chained comparison is a boolean in `@test` (row 2) but not in a piecewise condition |
| 4 | `k_1 = …` then `return k_1` inside `function … end` | "Variable k₁ not found" | the local value | The local is stored under the name as typed and read back as `k₁` |
| 5 | `w = [403] MPa`, `length(w)` | 0 (a loop over it does nothing) | 1 | A one-entry literal becomes a scalar |
| 5b | `vcat([1; 2], [3])`, `vcat([1; 2], 3)` | "does not understand the expression" | `[1; 2; 3]` | The same, and `vcat` takes only vectors |
| 6 | `x = t / r = ??` (mm/mm), then `[1; sqrt(x); x]` | "Matrix elements cannot contain a unit" | a plain-number vector | A quantity whose units cancel keeps a unit tag |

Row 5 is the worst: it gives a silent wrong answer. A one-level Miner's-rule sum gives D = 0.

**Not in Hurmet, but needed by ordinary engineering calcs (extensions):**

| # | Need | Today | Example |
| --- | --- | --- | --- |
| 7 | A user function calls another user function | "Unrecognized function name" | A Kt fit function calling a shared polynomial helper |
| 8 | Units with half-integer exponents | `MPa·√m`, `ksi·√in`, `m^0.5` aren't unit names; a result in them can't be shown ("No units specified") | Stress intensity K, fracture toughness K_Ic. The fracture templates carry K as a plain number in "σ_K·√l_K". |
| 9 | ∞ | "Variable ∞ not found"; `1/0` is an error | "Infinite life" below the endurance limit |
| 10 | Assigning an element inside a function (`out[i] = …`) | Not supported | Building a vector of per-level lives |

**Silent mistakes the engine could flag (lints):**

| # | Calc | Today | Proposed |
| --- | --- | --- | --- |
| 11 | `x = t / r`, an expression with no display selector | Defines nothing, with no message (Hurmet: only a literal assigns without `= ?`) | A warning: "x isn't calculated: add = ?? (or = !) to define it" |
| 12 | `K_Ic,req = n K_I = ??` | A multiple assignment of `K_Ic` and `req`, silently | The document already refuses a second definition (ADR-0004). Template validation should too, and the warning can suggest `K_(Ic,req)` |

**Units and docs:**

| # | Today | Proposed |
| --- | --- | --- |
| 13 | `kpsi` and `Mpsi` (Shigley's spelling) are unknown | Aliases of `ksi` and a new `Mpsi` (10⁶ psi) |
| 14 | `Δ°C` and `Δ°F` are unknown; `skills/cruncha-calc-authoring` says `Δ°C` is a difference | Add them: Δ°C = 1 K, Δ°F = 5/9 K, no offset |
| 15 | `(…)^2 'in'` is refused, because `2 'in'` is read as the exponent (Hurmet's grammar) | Keep the grammar. Make the error say to write `'in' × (…)^2`, and say so in the skill. |

What makes this a decision rather than a bug fix is what ADR-0022 said:
- Calc results are recomputed on open, so a fix changes past versions' results and re-printed reports.
- Every row turns an error, or a silent wrong value, into a value. No correct result changes.
- Rows 7 to 10 extend Hurmet's language and need a place in `docs/design/03-hurmet-differences.md`.

## Decision

1. **Fix rows 1–6 in place**, under ADR-0022's decision: tests first, documented in the differences list, no migration.
   - Row 1: zero to any positive power is zero; to a negative power it's "Divide by zero".
   - Row 2: an `@test` whose value is a boolean sets the check, whatever its last operator. The display keeps "ok ✓" or "n.g." for a single comparison, and shows the condition with "ok ✓" or "n.g." otherwise.
   - Row 3: a chained comparison means the same in a condition as in `@test` (`a ≤ x ≤ b` is `a ≤ x and x ≤ b`).
   - Row 4: a function's locals are stored and read under one canonical name.
   - Rows 5 and 5b: `[403]` is a one-entry vector; `vcat` takes vectors and scalars.
   - Row 6: a quantity whose dimensions are all zero counts as a plain number in a matrix literal.
2. **Extend the language with rows 7 and 8:**
   - **Row 7:** a user function may call any user function the document defines.
     - The graph adds the dependency, so editing the callee recomputes the caller's users.
     - Recursion, direct or mutual, is refused as a cycle (ADR-0004's rule), at definition, not as a runtime stack overflow.
   - **Row 8:** unit expressions accept `√u` and `u^(1/2)` / `u^0.5`, so `MPa·√m` and `ksi·√in` are units.
     - Dimensions are already floating point (`Dims = [f64; 8]`). The change is in parsing unit names and in displaying a result in such a unit.
     - Only half-integer exponents are accepted: they cover every engineering use found (stress intensity, √Hz noise densities) and keep units readable.
3. **Leave rows 9 and 10 out for now.**
   - **∞** has no exact rational representation, and it would spread through every operator. "Infinite life" is honest as a branch: no damage, with prose saying why.
   - **Element assignment inside functions** is a larger change to how functions run. A Python cell is the right tool for building arrays element by element. Both are listed in `docs/TODO.md` under Later.
4. **Add the lints (rows 11 and 12)** to the engine's warnings, so the editor, `check` and agents see them. Template validation reports any warning a template's calcs produce.
5. **Add the units (rows 13 and 14), and the error text and skill note (row 15).**
6. **Update the templates afterwards**, as their workarounds allow:
   - the fracture templates carry K in MPa·√m;
   - `hypot` stays where it's the natural form;
   - range checks become single chained `@test`s where that reads better.

   Each changed template gets a patch version and keeps its worked examples.

## Options Considered

### Option A: Fix the bugs, extend where needed, defer ∞ and element assignment (chosen)

| Dimension | Assessment |
| --- | --- |
| Complexity | Medium: `rational.rs`, `result.rs`, `eval.rs`, the unit parser, the graph's function dependencies |
| Cost | Past versions' results change only where they were errors or row 5's silent zero |
| Scalability | Templates stop needing workarounds; agents write natural calcs |
| Team familiarity | The same work as ADR-0022 |

**Pros:** correct and natural calcs; every change turns a failure into a value.
**Cons:** Cruncha's language grows beyond Hurmet's (rows 7, 8). That's noted for importing Hurmet documents.

### Option B: Fix only the bugs (rows 1–6)

**Pros:** no language extensions.
**Cons:** fracture mechanics stays without units on K, which is the kind of unchecked number unit-aware calcs exist to prevent. Shared helper functions stay copy-pasted. Rejected.

### Option C: Also add ∞ and element assignment now

**Pros:** the fatigue templates could say "infinite" and build vectors.
**Cons:** ∞ needs a non-rational value everywhere rationals flow (formatting, comparison, units, exchange with Python). Element assignment changes function evaluation from expressions to mutable state. Both are bigger than the templates' need. Deferred, not rejected.

## Trade-off Analysis

Rows 1–6 are plainly bugs: the choice is only whether to fix them, and ADR-0022 already settled that. Rows 7 and 8 are where Cruncha deliberately goes beyond Hurmet. Both come up in the first group of templates, and each workaround weakens what the calc shows: duplicated formulas, or a K value whose units the engine can't check. Rows 9 and 10 have honest workarounds today and a high cost. They wait until a template can't be written without them.

## Consequences

- The 22 Stress and failure templates can drop their workarounds, and later groups don't need them.
- Calcs that errored on zero (`sqrt(0)`), on compound checks or on one-entry vectors now give values. Reports re-printed from past versions show those values instead of NOT CALCULATED.
- Calc results change, so downstream Python and Rust stored runs re-run when next run, as in ADR-0022.
- The differences list gains a "Language extensions" section (rows 7, 8). A Hurmet document never uses them, so importing one is unaffected.
- **To revisit:** ∞ and element assignment inside functions; a per-document semantics level before the first public release (ADR-0022's open item).

## Action Items

1. [x] Rows 1–6, each with a test in `crates/cruncha-eval/tests/` (zero, negative and unit cases), Hurmet's goldens kept green. *(Done 2026-11-24: `tests/adr0025.rs`; the goldens needed no change. Row 2's cause was a chain adding the boolean flag twice, so the same fix gave row 3. A one-entry literal of a vector or matrix (`[v]`) is still that value. Later the same day, found by the Strain and Fastening templates and fixed the same way: a piecewise without `otherwise`, display units with parentheses and `(…)^n` units, `1/u` units, `Ω` as an input unit, a range loop's index unit, and `″` in names.)*
2. [x] Row 7: user functions calling user functions, the graph dependency, recursion refused as a cycle. *(Done 2026-11-24: a function reads the functions it calls, and so does a calc, which also fixes an older gap: a calc didn't recompute when a function it used changed. The app's memo follows calls transitively. Self-recursion is refused when the function is prepared, since the graph doesn't count a cell reading its own name; mutual recursion is a cycle.)*
3. [x] Row 8: `√u` and half-integer powers in unit names, and results shown in them (`MPa·√m`, `ksi·√in`). *(Done 2026-11-24: `√u`, `√(…)`, `u^0.5`, `u^(n/2)`, in literals and display units; other decimals refused. Unquoted after a number in an expression, a unit stays a variable, as before: quote it.)*
4. [x] Rows 11–12: the two lints, and template validation reporting calc warnings. *(Done 2026-11-24: `cruncha_eval::lints`, shown as the cell's warnings; template validation also reports a second definition.)*
5. [x] Rows 13–15: `kpsi`, `Mpsi`, `Δ°C`, `Δ°F`; the error text for a unit after a power. *(Done 2026-11-24: also `ΔK`; Cruncha's units are a table of their own in `units.rs`, beside Hurmet's generated one. The error suggests `(…)^2 × 1 'in'`, since `'in' × (…)^2` doesn't parse either.)*
6. [x] Update `docs/design/03-hurmet-differences.md`, `skills/cruncha-calc-authoring` and `reference/functions.json` in the same changes. *(Done 2026-11-24: a Language extensions section in the differences list.)*
7. [x] Remove the Stress and failure templates' workarounds (patch versions, examples unchanged). *(Done 2026-11-24: 8 templates at 1.0.1. The fracture templates carry K in MPa·√m and ksi·√in; Paris-law constants stay a point on the growth line (rate r_o at ΔK_o), since C's unit depends on m. Range checks are chained @tests; the fatigue-life template takes a one-level duty cycle. Every expected value is unchanged.)*
