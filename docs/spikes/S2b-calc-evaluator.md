# Spike S2b: port Hurmet's calculation evaluator to Rust

*Date: 2026-11-19. Code: `crates/cruncha-units` (units, rationals, JavaScript numerics) and `crates/cruncha-eval` (evaluator, golden tests, `examples/eval_bench.rs`). Source: Hurmet `src/` at commit `87f30ed0`. Machine: Intel i7-1068NG7 MacBook, rustc 1.99.1.*

## Method

- Port the evaluator statement by statement, keeping Hurmet's algorithms and its dynamic operand model, so that the output matches byte for byte. The RPN that `cruncha_lang::parse_calc` produces (spike S1) is evaluated directly; S1 finding 3 says a typed tree comes later, and keeping the RPN lets Hurmet's goldens act as the conformance oracle for both steps.
- Ported modules: `compile.js` (statement preparation), `evaluate.js` (`evalRpn`, `evalCustomFunction`, `conditionResult`, `errorResult`), `operations.js`, `compare.js`, `functions.js`, `rational.js`, `complex.js`, `matrix.js`, `format.js`, `result.js`, `tuple.js`, `literal.js`, `constants.js`, `error.js`, `operand.js`, `echo.js`, `insertOneHurmetVar.js`, `module.js` (user-defined functions), `property.js`, `text.js`, `map.js`, most of `dataframe.js`, `date.js` and `units.js` (the unit table is generated from Hurmet's JSON by `crates/cruncha-units/tools/gen_table.py`).
- Golden data: the `assignmentTests` and `calcTests` arrays and the eight function definitions in Hurmet's `test/test.cjs`, extracted to `crates/cruncha-eval/tests/fixtures/*.json` by `extract.cjs`. It checks that the text contains only string literals, then evaluates just that text with Node's `vm`. No Hurmet code was run.
- `tests/golden.rs` replays the harness: assignments in order into one set of variables (with the `vars.format = "h15"` the harness sets), then the functions, then the 217 calculations, comparing each result *and* its RPN. It prints every failure and allows an explicit `KNOWN_GAPS` list.
- `tests/robustness.rs` evaluates every prefix and suffix of every golden input (27,144 fragments) in both unit-awareness modes against the golden variables, plus 23 inputs that would make JavaScript hang, overflow or throw.
- `tests/api.rs` covers the public API, the TeX output and ADR-0007's always-unit-aware mode.
- `crates/cruncha-units/tests/v8_math.rs` checks the JavaScript `Math` functions against 12,880 vectors printed by Node.js (see Findings 2).

```sh
cargo test -p cruncha-units -p cruncha-eval -- --nocapture
cargo run  -p cruncha-eval --release --example eval_bench -- 300
cargo run  -p cruncha-eval --example eval_demo
```

## Results

| Metric | Value |
| --- | --- |
| **`assignmentTests`** | **33 / 33 pass** (the 7 rows with an empty expected value only define variables) |
| **`calcTests`** (result and RPN) | **217 / 217 pass** |
| Known gaps | **none** in the goldens; features outside them are listed below |
| Fragment robustness | no panics on 27,144 fragments in debug (overflow-checked) builds; adversarial inputs return within limits |
| JS ported | about 10,300 lines across 25 modules (of which the unit table is 543), minus the parts listed as not ported |
| Rust (`src/`, rustfmt, without unit tests) | about 12,300 lines: `cruncha-eval` 9,800, `cruncha-units` 2,500 (unit table 560, rationals 800 of which 130 are factorial tables, JavaScript number formatting 270, fdlibm ports and `Math` wrappers 350) |
| Dependencies added | `num-bigint`, `num-integer`, `num-traits` (MIT/Apache-2.0), `libm` (MIT); `regex` and `serde_json` (tests) as before |
| **`prepare` (release)** | **19 µs per statement** (52,000 statements/s) over the 216 golden calculations, 17.7 bytes on average |
| **`evaluate` (release)** | **median 19 µs**; mean 79 µs, of which one statement (`1 - 0.9999375^1000`, an exact 23,000-bit rational) takes 10 ms; the mean of the other 215 is 32 µs |

The Rust is about 1.2× the JavaScript: rustfmt wraps long lines, `Result` plumbing replaces JavaScript's exceptions, and JavaScript's number formatting and fdlibm had to be written out, while the operator tables collapse into `match` arms. 32 µs per statement means a 1,000-statement document evaluates in about 30 ms on one thread, before any caching; `prepare` depends only on the source and the options, so the graph can cache it.

### Not ported (outside the goldens)

| Feature | Status |
| --- | --- |
| Drawings (`draw()`, `plot`, `startSvg`), beam diagrams, spreadsheets, `fetch()` | Report `Error. Cruncha does not support “…” yet.` (or Hurmet's own `fetch` error). They belong to the app and the report layer. |
| Remote modules, macros (`'''…'''`) | A macro is stored as a value but not expanded; `module` blocks are scanned but not imported from files. |
| Localized dates (`Intl.DateTimeFormat`) | English month and weekday names only. |
| `map & scalar` (Hurmet's `map.append`) | Hurmet calls a `Map` method that maps do not have; the port reports an error. |
| Multiple assignment from data frames and maps | Only from vectors, matrices, tuples and modules. |
| Saved-Markdown results (`stmt.md`) | Computed internally, not exposed. |
| `sin`/`cos` of \|x\| > 2¹⁹·π/2 | fdlibm's `__kernel_rem_pio2` is not ported; those arguments use the `libm` crate and may differ from V8 in the last bit. |

## Findings

1. **The evaluator is a dynamically typed interpreter, and matching it means keeping its value model.** An operand is `{ value, unit, dtype }`, where `dtype` is a sum of bit flags and the code does arithmetic on it (`arg.dtype - dt.RATIONAL + dt.STRING`). Values are JavaScript shapes: a rational is a pair of BigInts, a vector an array of pairs, a complex number an array of two pairs. Several results depend on that ambiguity (Finding 4). The port mirrors it with a small JavaScript-like `V` enum and Hurmet's flags, and keeps a typed public `Value`/`Kind` on top. A typed engine (ADR-0002) is still the goal; this port is its oracle, as S1's parser is for the AST.
2. **JavaScript's numerics are part of the specification.** Hurmet's 15-digit results depend on:
   - `Number(bigint)` (correct rounding), `toPrecision` (ties round up, where Rust rounds to even), `toExponential()` with the shortest round-trip digits, and `String(number)`'s switch to exponent form. `cruncha_units::jsnum` reimplements these.
   - **V8's `Math` functions, which are fdlibm, not the platform libm.** The first run failed one golden (`sec(z1)`, last digit) because macOS's `sin(2.34)` is correctly rounded (…263) and V8's is not (…261). The `libm` crate (musl) also differs: for 1–2 % of arguments in `sin` and `cos`, 1 % in `log` and 12 % in `log10`. `sin`, `cos`, `log` and `log10` are therefore ported from fdlibm 5.3 (with Sun's notice) in `cruncha_units::jsmath`; the others come from `libm`, which matches V8 on all 12,880 vectors.
   - **V8's `Math.pow` is the C library's `pow`**, so even Hurmet gives platform-dependent results: on macOS, fdlibm's `pow` differs from Node in the last bit for 41 of 400 arguments. Cruncha uses fdlibm's `pow` everywhere, for results that do not depend on the OS (desktop and WASM alike), and the test requires agreement with Node-on-macOS within one unit in the last place. The owner confirmed this on 2026-11-20: results stay the same on every platform rather than matching Node's per-platform `pow`.

   **Recommendation:** the typed engine should keep `jsmath` (deterministic, fdlibm-compatible) as its only source of transcendental functions. This matters more for Cruncha than for Hurmet, because results are saved and diffed (ADR-0003).
3. **Display is string surgery, as in the parser.** The TeX line is assembled from templates: the selector is replaced by a regex (`/[?%] *[?%]|[?%]/`, whose replacement string interprets `$$` and `$&`), `%` is a placeholder unit `\⦂` that is stripped at the end, and the echo replaces `〖name〗` marks with each variable's stored TeX, deciding on parentheses by looking at neighbouring characters. `Outcome.tex` reproduces this exactly. The typed engine should return structured parts (expression, echo, result, unit), so that the app and the Typst exporter lay them out themselves.
4. **Hurmet bugs that the port reproduces** (each is commented in the code). Some are pinned by goldens:
   - `|matrix|` returns the reciprocal of the determinant (Gauss–Jordan divides by each pivot): `|(2.1, 7.5; -15.3, 33)|` is 0.00543…, not 184.05. **Pinned by a golden.**
   - Exact powers are never reduced and `0.9999375^1000` is computed in 23,000-bit integers, then read back through a 20-digit string. **Pinned** (and the slowest statement).
   - `Cpx.isComplex` accepts any two-element vector of rationals, so `[2, 3]^2` is the complex number 4 + j 9.
   - A column vector times a row vector checks the length of the first *element* (2 for a rational), so the outer product works only when the row has two elements; `[2; 3] [4, 5]` passes (a golden), `[2; 3] [4, 5, 6]` is an error.
   - `[v1, v2]` of column vectors reads the evaluation stack from the bottom.
   - `Rnl.toString` loses the sign of numbers between -1 and 0 and rounds `-0.7` to `1`, so `round(-0.7)` is 1.
   - Converting a matrix from base units with a gauge (°F) drops the factor.
   - A function-body assignment `x=1` (no spaces) loses the last character of the name.
   - `"a" ∉ "xyz"` is always false for one-character strings; `binaryShapesOf` tests `o1.dtyp` (a typo), so a map never broadcasts on the left; `product()` discards its unit exponents; `not` of a vector reports a scalar type; a data frame's numeric index is 0-based for a single-row frame; `df[name, col]` with an unknown row name uses the number of columns as the number of rows.
5. **Where Hurmet throws a JavaScript `TypeError`, the port reports an error** instead of crashing: unknown operator/shape combinations, `NaN`/`±∞` from a floating-point step (`log(-1)`, which Hurmet turns into an exception in `Rnl.fromNumber`), calls to functions that do not exist in Hurmet (`Math.atn` in `asec`/`acot`/`acsc`, `Rnl.pow` in `variance`, `Cpx.gamma`, `Matrix.trace`), and `acsch` of a real (a typo in Hurmet's table). A few are fixed outright, which changes no golden: matrix × matrix (an empty function in Hurmet) is the matrix product, `√` of a negative real is typed complex, matrix-to-matrix comparison is element-wise, and the postfix `%` operator returns the percentage. The unit table's `Å` row has a ninth exponent in Hurmet; it is dropped.
6. **Limits replace hangs.** JavaScript would hang or exhaust memory on `[1:1e-7:1e5...]`, `while true`, or `"abc"[1:2:1e18]`. The port stops at 1,000,000 elements, 1,000,000 function steps or `∑` terms, 32 nested function calls, exact powers of 2²⁰ bits (Hurmet's JavaScript allows 2³⁰ and then falls back to floating point too), `n!` for n > 100,000 and `n!!` for n > 10,000.
7. **Variables carry more than values.** What a statement stores (Hurmet's cell attributes, the port's `Value`) holds the value as written *and* in SI base units, the unit, the dimension exponents, and the TeX that later echoes show. A non-unit-aware `?` statement reads the "as written" number, a `??` statement the base-unit number. An errored assignment still defines its name, with a missing value, so dependents report "Missing value for $x$" rather than "not found". The graph (ADR-0004) can store `Value`s as they are; `Statement`s are pure functions of source and options.
8. **ADR-0007 (always unit-aware) fits as a mode.** `UnitAware::Always` evaluates every statement unit-aware and shows a result with dimensions but no display unit in SI base units (`L = 3 ft`, `2 L = ?` → `1.8288 m`; Hurmet gives 6 for `?` and an error for `??`). The goldens run in `UnitAware::Hurmet`. Two consequences for the importer: `?` statements with units change their numbers (as ADR-0007 expects, so they go in the import report), and `??` without a unit stops being an error.
9. **Hurmet's RPN has implicit control flow that the typed tree must make explicit.** `cases` branches are lazily evaluated strings (`§`-separated); `isnan(…)` is bracketed by a marker token so that the argument can fail on its own; `.` and `[]` on a data frame look ahead in the token stream and fuse two accessors (`wideFlanges.W8X31.area` is one cell lookup, not a row then a column). Each of these needs its own AST node.

## Recommendation and next steps

1. **Accept the port as the evaluation reference** (S2b is answered: 250 of 250 golden cases, about 12,300 lines of Rust, 19 µs to prepare and a median 19 µs to evaluate a statement).
2. **Integrate it in the app** through `prepare`/`evaluate` (stable names; see the crate docs), caching `Statement` per cell source and storing each `Outcome.values` entry in the graph.
3. **Build the typed AST and evaluator** (S1 step 2) against both oracles: the goldens through `parse_calc` + `evaluate`, and differential tests that compare the typed engine with this port on random statements.
4. **Decide the bug list** in Finding 4 case by case for the typed engine. Each fix changes results, so each belongs in the Hurmet import notes (ADR-0002 "Revisit").
5. Data frames should move to Polars (design §3.2) once the typed engine exists; this port keeps Hurmet's string-cell frames so that the goldens stay exact.
