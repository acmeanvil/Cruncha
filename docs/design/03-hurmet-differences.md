# Where Cruncha's calc language differs from Hurmet

Cruncha's calc engine (`crates/cruncha-lang`, `crates/cruncha-units`, `crates/cruncha-eval`) is a port of Hurmet's `src/` at commit `87f30ed0` (spikes S1 and S2b). Where Cruncha behaves differently, on purpose, it's listed here. Hurmet documents imported later are read with Cruncha's meanings.

## Document model

- **One definition per name** (ADR-0004). Hurmet lets a later cell redefine a name; Cruncha reports a duplicate.
- **Always unit-aware** (ADR-0007). Hurmet's unit-aware mode is a per-document switch; Cruncha has no switch.

## Built-in functions (ADR-0022)

| Function | Hurmet | Cruncha |
| --- | --- | --- |
| `mod(a, b)`, `a mod b` | `\|rem(a, b)\|`, and wrong for non-integers (`mod(7.5, 2)` = 3). The operator is refused when unit-aware. | Floored modulo, `a − b·⌊a/b⌋`, with the sign of `b`. Exact for rationals; the arguments share a unit; `b = 0` is an error. |
| `rem(a, b)` | Wrong for non-integers. | Truncated remainder, with the sign of `a`. |
| `hypot(a, b, …)` | No absolute values: `hypot(-3, -4)` = −5. | Absolute values; 0 when all are 0. |
| `rms` | `rms(a, b)` = `hypot(a, b)`. | The root mean square of its arguments or a vector's elements, `√(Σx²/n)`. |
| `median` | The midrange, `(max + min) / 2`. | The middle value, or the mean of the two middle values. |
| `variance`, `stddev` | TypeError (missing `Rnl.pow`). | Sample statistics (n − 1); the variance is in the unit squared. |
| `asec`, `acot`, their degree forms | An inverted range check, then a TypeError (missing `Math.atn`). | `acos(1/x)` and `atan(1/x)`; `acot(0)` is π/2. |
| `acsc`, `acscd`, `acsch` | TypeError. | `asin(1/x)` and `asinh(1/x)`. |
| `trace` | Not implemented (missing `Matrix.trace`). | The sum of a square matrix's diagonal. |
| `Char` | Not found when unit-aware. | Works. |
| `roundSig(x, n)` | Known to the parser, not to the evaluator. | x rounded to n significant digits. |
| Fixed format (`format = "f2"`, `roundn(x, "f2")`, `string(x, "f2")`, `round`) | A value between −1 and 0 loses its minus sign: −0.25 shows as 0.25, `round(-0.7)` is 1. | The sign is kept: −0.25, `round(-0.7)` = −1. A value that rounds to zero shows as 0, without a sign. |
| `×` (cross product) | `[]` for any two 3-vectors. | The cross product, shaped like the first vector. |
| `sum`, `product`, `mean`, `min`, `max`, `range`, `median` of a matrix | 0. | Over every element. |
| `\|A\|` of a square matrix | 1/det(A), and row swaps didn't change the sign. Hurmet's golden `\|matrix\|` expects 0.0054333; Cruncha gives 184.05. | det(A), in the unit to the nth power; 0 when singular. |
| Column × row (outer product) | "Mis-matched number of elements" unless the row has 2 elements. | Any lengths. |
| `product` | The unit of the first argument (`6 m` for `product(2 'm', 3 'm')`). | The product of the units (`6 m²`). |
| A vector in an offset unit (°C, °F) | Converted wrongly (`[20; 30] °C` = [−166.52; −156.52] °F). | As a single value ([68; 86] °F). |
| `zeros(m, 1)`, `ones(m, 1)` | One element. | A column of m. |
| `[a, b]^n`, `[a; b]^n` | Read as a complex number. | Element-wise. |
| Two-argument functions (`atan2`, `gcd`, `hypot`, `sum`, …) | Unit checks discarded: `atan2(1 'm', 1 's')` = 0.785. | Different units are an error; `gcd` takes no unit. |
| `[x, y]` of column vectors inside an expression | "does not understand the expression". | The matrix of those columns. |
| `M[rows, a:step:b]` (stepped columns) | Every element undefined. | Every step-th column. |
| Data frames | A column name first in `df[…, …]` spanned the number of columns as rows; a column range looked up `columnMap[NaN]`; columns by a row of names were undefined; a selection's columns were keyed by the source's first headings; a frame without units printed a row of "undefined". | Each selects what it says; a missing column on a frame without row names says "no column named". |
| `e` | Not a constant ("Variable e not found"). | Euler's number unless the document defines `e`; a definition wins and gets a warning (as for `j`). |

## Statements

| Statement | Hurmet | Cruncha |
| --- | --- | --- |
| A chained comparison, `0 ≤ x ≤ 1` (unit-aware) | "Comparing incompatible units", even for plain numbers: the second comparison checked the first one's result, which has no unit. | Each comparison checks its own pair: `x` against 0, then `x` against 1. |
| A name with a comma in its subscript, `X_(CO2,dry) = … = ?` | Read as a multiple assignment (`a, b = …`) and refused; only a literal assignment worked. | One name, as the lexer reads it; a multiple assignment still needs names that are valid on their own. |

## Units (ADR-0022)

- **A unit's own name wins.** Hurmet stripped a plural *s* and an SI prefix before looking a name up, so names in its own table were unknown: `Celsius`, `gauss`, `kilogram`, `kilobyte`, `centistoke`, `dekan` and `megaton TNT`. And `mas` (milliarcsecond) read as milli-annum. Cruncha looks up the whole name first, then the singular, then prefixes.
- **Corrected table rows** (`crates/cruncha-units/tools/gen_table.py`, `CORRECTIONS`):
  - The Fahrenheit gauge is 459.67, not 459.
  - Tesla and gauss are kg·s⁻²·A⁻¹; Hurmet had m⁻²·kg⁻²·s⁻²·A⁻¹.
  - Ampere-turn and gilbert are a current; Hurmet had temperature × luminous intensity.
- Unchanged, but worth knowing: `kilobyte` is 1024 bytes (`kB` and `KiB` are separate units), and `kpph` is kilopascals per hour.

Internally, `Rational::power` for a negative base and a fractional exponent p/q now returns the real root only when q is odd (Hurmet's test went through its broken `mod`). Calcs aren't affected: a negative base takes the complex principal root there, as before (`(-8)^(1/3)` = 1 + j 1.732).

Hurmet's golden tests still pass: none of them encoded these bugs. `crates/cruncha-eval/tests/functions.rs` covers each row.
