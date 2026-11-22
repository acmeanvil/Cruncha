---
name: cruncha-calc-authoring
description: How to write Cruncha calc cells. Covers the Hurmet-style statement form (name = expression = ?? unit), the display selectors ?, %, @ and !, units and dimensional rules, placeholders, data frames with a units row, user functions, and @test assertions. Use it whenever you create or edit a calc cell or turn a hand calculation into a Cruncha calc.
---

# Authoring calc cells

Calc syntax follows Hurmet's (see ADR-0002), with Cruncha's differences called out.

## Statement forms

| Form | Example | Meaning |
| --- | --- | --- |
| Assignment | `L = 3.1 m` | Defines `L` from a literal |
| Placeholder | `p = 35 psi_` | A template input that a human must confirm; highlighted until they do |
| Calculation | `M = w L^2 / 8 = ?? kip·ft` | Defines `M`, shows the full working, and displays the result in kip·ft |
| Anonymous result | `2 L = ?` | Evaluates and displays the result without defining a name |
| Multiple assignment | `A, I_x, w_s = W12x26["A", "Ix", "weight"] = !` | Unpacks a single row of a data frame |

The display unit after the selector (`kip·ft`) must be dimensionally compatible with the result. If it isn't, the cell is an error; never pick a different unit to make the error go away.

## Display selectors

| Selector | Shows | Use for |
| --- | --- | --- |
| `?` | Formula, **blue echo** with every input value substituted, and the result | **Default for engineering checks.** Reviewers need the echo. |
| `%` | Formula and result, no echo | Only when the inputs are defined directly above |
| `@` | Result only | Table-like summaries |
| `!` | Nothing (computed and stored) | Data frames and large intermediate values |

Hurmet's doubled forms (`??`, `%%`) are accepted. **Cruncha is always unit-aware** (ADR-0007), so `?` and `??` behave the same.

## Units

- Write units after the number: `100 lbf/ft`, `25 MPa`, `3 ft 6 in`. Use `·` or a space for products, and `/` and `^` (or superscripts) in compound units.
- Mixing systems is fine (`2 m × 4 ft`); conversion is automatic. Pick a display unit that suits the reader's practice.
- Temperatures: `°C` and `°F` are absolute, and `Δ°C` is a difference. Don't add two absolute temperatures.
- Unitless ratios (utilization, `ϕ`) are plain numbers. A result of `0.87` that should be unitless but carries `in/ft` means the formula is wrong.

## Names

- Names can be Unicode with subscripts: `f_c′`, `ϕM_n`, `L_b`, `Δ_max`. Case and boldface matter.
- **Each name is defined exactly once in the whole document** (ADR-0004). For a second check of the same quantity, use a qualified name (`M_u,2`, `V_u,end`), not a redefinition.
- Define inputs above their use. Upward references still compute, but they get a lint badge.

## Descriptions

- A calc can carry a short description of what it defines: `attrs.description` on the cell (a string, e.g. `"Span, centre to centre of bearings"`). It is shown beside the typeset result, in the Variables panel (searchable), and in the right-hand column of the PDF report. A cell that defines several names shares one description.
- It is not part of the statement, so it never changes the result. Keep it to one line and say what the quantity *is* (and where it comes from, if not obvious), not its value: `"Factored moment, LC3"`, not `"M = 412 kip·ft"`.
- When you propose a calc, give every input and every key result a description. Keep the other `attrs` (such as `display`) as they were.

## Data frames

A literal data frame has headings in row 1, an optional **units row** in row 2, and data after that. `#` in the first heading makes column 1 the row key.

```
beam = ``#name	A	Ix	weight
	in²	in⁴	lbf/ft
W12x26	7.65	204	26
W14x30	8.85	291	30`` = !
```

Access: `beam.W12x26.Ix`, `beam["W14x30", "A"]`, `beam.Ix` (the whole column). For anything beyond lookups (filtering, joins, large tables), use a Python cell with Polars (see `cruncha-code-cells`).

## Functions and tests

```
function utilization(demand, capacity)
    return demand / capacity
end
```

- Functions are pure (they see only their parameters) and can be defined anywhere.
- Assertions: `@test ϕM_n ≥ M_u`. Every pass/fail criterion of a check should be an `@test`, so CI can catch regressions. The report's cover lists every `@test` with OK or NOT OK and the sheet it is on. A test that couldn't be evaluated (an error, or an input from Python that hasn't run) is listed as ERROR or NOT CALCULATED, never as OK.
- Units in a test's comparison value are quoted: `@test V_u < 25 'kips'`.

## Built-in functions

The full list, each with an example whose result is checked by a test, is `crates/cruncha-eval/reference/functions.json` (the app's ƒx Functions palette). Units inside an expression are quoted: `hypot(3 'm', 4 'm')`. Watch for these, inherited from Hurmet (ADR-0022 proposes fixing them):

- `median` is the midrange, `(max + min) / 2`: `median(1, 2, 10)` is 5.5, not 2. Don't use it; use a Python cell.
- `mod(a, b)` is `|rem(a, b)|`: `mod(-7, 3)` is 1, not 2. `mod` and `rem` are wrong for non-integers (`mod(7.5, 2)` is 3).
- `hypot` doesn't take absolute values: `hypot(-3, -4)` is -5.
- `rms(a, b)` is `√(a² + b²)` (the same as `hypot`), not the root mean square.
- Not evaluated yet: `asec`, `acsc`, `acot` (and their degree forms), `acsch`, `variance`, `stddev`, `Char`, `trace`, `roundSig`. Use `acos(1/x)`, `asin(1/x)`, `atan(1/x)`, `asinh(1/x)`, or a Python cell for statistics.
- `e` isn't a constant: write `exp(1)`. `π` and `j` (the imaginary unit) work.

## Checklist before proposing a calc

- [ ] Every input has a unit, and a cited source or a placeholder `_`.
- [ ] Inputs and key results have a description (`attrs.description`).
- [ ] `?` on every check result, so the echo is visible.
- [ ] No duplicate names (`read_doc` / `check`).
- [ ] An `@test` for each acceptance criterion.
- [ ] Prose states the assumptions (support conditions, load combination, code edition).
