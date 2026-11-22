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
| `e` | Not a constant ("Variable e not found"). | Euler's number unless the document defines `e`; a definition wins and gets a warning (as for `j`). |

Internally, `Rational::power` for a negative base and a fractional exponent p/q now returns the real root only when q is odd (Hurmet's test went through its broken `mod`). Calcs aren't affected: a negative base takes the complex principal root there, as before (`(-8)^(1/3)` = 1 + j 1.732).

Hurmet's golden tests still pass: none of them encoded these bugs. `crates/cruncha-eval/tests/functions.rs` covers each row.
