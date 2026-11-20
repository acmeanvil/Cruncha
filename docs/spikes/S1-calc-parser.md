# Spike S1: port Hurmet's lexer and parser to Rust

*Date: 2026-11-19. Code: `crates/cruncha-lang` (lexer, parser, golden tests, `examples/parse_bench.rs`). Source: Hurmet `src/lexer.js` and `src/parser.js` at commit `87f30ed0`. Machine: Intel i7-1068NG7 MacBook, rustc 1.99.1, regex 1.13.2.*

## Method

- Port `lexer.js` and `parser.js` statement by statement, keeping Hurmet's single-pass, two-stack algorithm (see Findings 1) so that the output matches byte for byte.
- Golden data: the two arrays in Hurmet's `test/test.cjs`, extracted to JSON (`tests/fixtures/`) by a script that checks the array text contains only string literals and then evaluates it with Node's `vm`. No Hurmet code was run.
  - `parserTests`: input → TeX, as `hurmet.parse(input)` with the default formats.
  - `calcTests`: input → RPN. The harness slices off the last `=` and a leading `@test `, then calls `parse(str, {decimalFormat: "1,000,000.", dateFormat: "yyy-mm-dd"}, true)`. Results (evaluation) are out of scope.
- `tests/golden.rs` runs every case, prints every failure, and allows an explicit `KNOWN_GAPS` list. A second test parses every prefix and suffix of every fixture input (about 10,000 fragments, each in both modes) and checks that nothing panics.

```sh
cargo test -p cruncha-lang -- --nocapture
cargo run  -p cruncha-lang --release --example parse_bench -- 5000
```

## Results

| Metric | Value |
| --- | --- |
| **`parserTests` (TeX)** | **65 / 66 pass**, 1 known gap |
| **`calcTests` (RPN)** | **217 / 217 pass** |
| Fragment robustness | no panics; malformed input returns `Error::Malformed` where Hurmet would throw |
| JS ported | `lexer.js` 1,038 + `parser.js` 1,596 = 2,634 lines, plus about 350 lines of helpers from 5 other modules |
| Rust (`src/`, rustfmt, without unit tests) | about 4,460 lines: `parser.rs` 2,405, `lexer.rs` 1,315, helpers 740 |
| Of which data tables | about 950 lines (token tables and word lists: lexer 778, parser 175) |
| Dependencies | `regex` only (MIT/Apache-2.0); `serde_json` for tests |
| **Throughput (release)** | **about 25,000 statements/s for TeX (40 µs each) and 29,000 statements/s for TeX plus RPN (34 µs each)**, single-threaded, over the fixture inputs (18 bytes on average) |

The Rust is about 1.5× the line count of the JS. Most of the growth is rustfmt wrapping the long conditionals and the one-entry-per-line tables, plus explicit `Result` plumbing where JavaScript would throw a `TypeError`. The algorithm itself is the same size.

A profile shows the parse is allocation-bound (token strings, `format!`, regex capture slots). There is no hot spot. 34 µs per statement means a 1,000-statement document parses in about 35 ms. Interning token text as `&'static str` would cut most of the allocation if it ever matters.

### Known gap

| Suite | Input | Reason |
| --- | --- | --- |
| TeX | ``α = ``B	C	D\n7.0	9.5	11.5``["C"]`` | A data frame literal. Outside real-time mode, Hurmet renders it with `DataFrame.display(DataFrame.dataFrameFromTSV(…))`, which builds the data frame model and formats each number (`7.0` shows as `7`). That belongs with the evaluator. The port always uses `DataFrame.quickDisplay`, which Hurmet uses while typing. |

### Modules the parser depends on

The parser is not self-contained. Exact output needed parts of:

| Hurmet module | What | Why |
| --- | --- | --- |
| `utils.js` | `isValidIdentifier`, `addTextEscapes`, `unitTeXFromString`, `checkForNumericSubscript`, `verbatimArg`, `numeralFromSuperScript`, `tablessTrim` | identifiers, `\text{}` escaping, unit TeX, `x_2` → `x₂` in RPN |
| `rational.js` | `numberPattern`, `fromString`, `normalize` | the number regex and the `®n/d` RPN (decimals are not reduced: `1.2` → `®12/10`; mixed fractions are: `5 2/3` → `®17/3`) |
| `format.js` | `formattedDecimal`, `texFromMixedFraction` | thousands separators in TeX (`4500` → `4,500`) |
| `date.js` | `dateRPN`, `dateDisplayFromIsoString` | `'2025-01-20'` → `⌾1737331200` and its TeX |
| `dataframe.js` | `quickDisplay` (and `display`, not ported) | data frame literals |
| `constants.js` | `isnanMarker` | `isnan(x)` RPN |

The TeX output also depends on the document's decimal format, and a date's TeX depends on its date format. A cached TeX rendering is therefore keyed by formats as well as by source.

## Findings

1. **Hurmet has no grammar and no AST.** `parse()` is one left-to-right pass with a TeX operator stack and an RPN operator stack. It writes both outputs as strings and edits them after the fact: it inserts `\dfrac{` at a remembered byte offset when it meets `/`, deletes a numerator's parentheses after the fact, and pastes `\left`/`\middle` in later. The only specification of the syntax is this code plus the golden tests. Matching it exactly means porting the algorithm, quirks included. A clean grammar would not match (`(w L)/8` drops the parentheses but `(b^2/2)` keeps them, `\left( … \right)`, depending on what follows the `)`). We should keep this port as the **reference implementation and conformance oracle**. It answers the S1 question, "Can we match its syntax?": yes, exactly.
2. **Whitespace and adjacency are semantic.** This is the main thing for ADR-0002's "harder" list:
   - `5m` is a quantity (`applyUnit m`), but `5 m` is 5 × the variable `m`.
   - `f(x)` is a call, but `f (x)` is `f × x`. `x[2]` is an index, `(x)[2]` and `(x) [2]` are multiplication by a 1×1 matrix.
   - `(a, b)` (comma plus space) is a 1×2 matrix, but `(a,b)` is an argument list.
   - `5 2/3` is the mixed number 17/3, but `5 (2/3)` is a product.
   - `a!` is a factorial, but `a !b` is logical not.
   - `-2^2` lexes `-2` as one number literal, so it is `(-2)² = 4`, unlike most languages.
   - `/` is stacked division that binds a run of factors: `a/b c` is `(a/b)·c`, but `sin x/8` is `sin(x)/8`.

   Each of these is pinned by a golden test or follows from one. A Cruncha AST must preserve the token adjacency (or the source spans) so that the echo and an exporter can round-trip it.
3. **The RPN is a lossy string format.** Tokens are separated by U+00A0. Inside `{… if …}` they are separated by `§`, so that the branches can be cut out and evaluated lazily. Strings are not escaped: `"a ""string"""` becomes the RPN token `"a "string""`. Anonymous functions and `∑` bodies are re-quoted with `§` by regex surgery after the fact (`sumRegEx`). This confirms ADR-0002's "replace string-encoded RPN": the evaluator should consume a typed tree. The RPN port is still worth keeping as a conformance target for the evaluator port.
4. **The echo template is part of the parse.** In calc mode the TeX wraps each variable in `〖name〗`, so that its value can be substituted later. The parser also returns the dependency list (`¿name`). The blue echo is therefore a parser feature, not a renderer feature, and the AST needs to carry what the echo needs.
5. **Hurmet bugs that the port reproduces** (the golden tests don't cover them; each is commented in the code):
   - `⋆` has the type `"tt.BIN"` (a string), and `⇒` is missing a column, so it has the type `""` and no close delimiter.
   - The thousands separator for `1,0000,0000.` inserts a literal `$1,` (a regex with no group 1).
   - A `#` comment drops the character after `#` and leaves the comment's last character as a token: `x = 2 # note` gives `\#note` followed by a variable `e`.
   - `delims[…].symbol` is read but never set, so `(a to b)` and `[a to b]` both become `lognormal`, and `{a : b}` never gets `\colon`.
   - `prevToken.length` is `undefined`, so the implicit-multiplication test looks at the first character of the previous token, not the last.
   - An unclosed rich-text backtick recurses forever; JavaScript fails with a stack overflow. The port returns an error at depth 32.
   - `\dots` and `\cr` are in the misc-symbol table but can never be reached.

   For the engine proper, we should decide case by case whether to fix these. A fix changes output, so each one goes in the import notes (ADR-0002 "Revisit").
6. **String positions.** Hurmet indexes strings in UTF-16 code units. The port uses byte offsets into its own output, plus a `skip_utf16` helper wherever the JS advances the input by a token's `.length`. (The en dash `–` lexes to input `-`: same UTF-16 length, different UTF-8 length.) The fragment test exists to catch char-boundary panics from this.
7. **D2 (inline calc syntax in the text mirror).** Calc source uses most of Markdown's special characters. Of the 283 fixture inputs, 88 contain at least one of `` ` | * _ $ # [ < \ ``, and 68 contain non-ASCII characters. They use `` ` `` and ``` `` ``` themselves (rich text and data frames), `'` for units and dates, and `"` for strings. So the inline form must be **opaque to Markdown**, and the only robust CommonMark construct for that is a code span with a variable-length backtick fence. Hurmet's own Markdown does exactly this, with `` ¢`…` `` and longer fences when the content has backticks. Recommendation: keep Hurmet's scheme with an ASCII marker in place of `¢`. For example, an inline code span whose content starts with a fixed prefix, such as `` `=M = w L^2/8` ``, with a longer backtick fence whenever the content contains backticks, as CommonMark allows. Avoid a new bracket pair such as `‹calc …›`: it is not ASCII-typeable, and Markdown would still interpret `*`, `_` and `|` inside it.

## Recommendation and next steps

1. **Accept the port as the syntax reference** (S1 is answered: exact match on 282 of 283 golden cases; the parser is about 2,600 lines of JS and about 4,500 lines of Rust).
2. **Add a typed AST beside the string outputs.** Change the parser to build an AST with source spans, and generate TeX and RPN from it. The goldens act as the regression net: they must stay green while the string surgery moves into tree-to-string emitters. Expect the hard parts to be `/` numerator capture, the paren-hiding rules and the `cases` reordering.
3. **Evaluator port order**, each step against the `calcTests` results column (element 2) and the remaining sections of `test.cjs`:
   1. `rational.js` (exact rationals; BigInt → a bignum crate) and the number formatting in `format.js`;
   2. `units.js` plus the unit data (ADR-0002 action item 2), and `applyUnit`;
   3. the scalar operations and functions (`operations.js`, `functions.js`), then complex numbers;
   4. matrices and `cases`;
   5. data frames (`dataframe.js`, closing the known gap above) and maps;
   6. user-defined functions (`module.js`, `compile.js`).
4. **Make D2 decision** using Finding 7.
5. Re-run `examples/parse_bench.rs` once the AST exists. Interning token text is the first optimization if parse time ever matters (it is about 35 ms per 1,000 statements now).
