# Golden fixtures from Hurmet

These files are extracted from the unit tests of Hurmet (<https://hurmet.org>,
`test/test.cjs`, commit `87f30ed0`):

- `assignments.json`: the `assignmentTests` array, as `[entry, template, expected]`. The harness
  runs `hurmet.calculate(entry, vars)` and then compares `hurmet.calculate(template, vars, true)`
  with `expected` (skipped when `expected` is empty; the entry still defines its variable).
- `functions.json`: the eight user-defined functions (`isIn`, `testFor`, `reverse`, `sumVec`,
  `testWhile`, `testBreak`, `testThrow`, `newton`) that the harness defines with
  `hurmet.calculate(source, vars)` between the two arrays.
- `calculations.json`: the `calcTests` array, as `[input, expected RPN, expected result]`, checked
  with `hurmet.calculate(input, vars, true)` after the assignments and functions, in order, with
  the same `vars`. RPN tokens are separated by U+00A0 (no-break space), or `§` inside lazily
  evaluated expressions.

Hurmet's `vars` starts as `{ format: { value: "h15" }, currencies: { USD: 1, CAD: 1.33 } }`, and
its harness also loads `site/currencyRates.json` into `globalThis.currencyRates`, which is where
`units.js` reads exchange rates. `tests/golden.rs` uses two of those rates (USD 1.1014, CAD 1.5734
per euro); only their ratio matters to the tests.

`extract.cjs` produced the files (`node extract.cjs <hurmet-clone> <outdir>`). It checks that the
array source and the function sources contain only string literals and template literals without
`${}`, and evaluates just that text with Node's `vm.runInNewContext`; Hurmet's own code was never
run.

The test data is from Hurmet and is used under its MIT license:

```
The MIT License (MIT)

Copyright (c) 2026 Ronald Kok

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```
