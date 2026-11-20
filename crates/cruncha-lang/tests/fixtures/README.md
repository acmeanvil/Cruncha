# Golden fixtures from Hurmet

These files are extracted from the unit tests of Hurmet (<https://hurmet.org>,
`test/test.cjs`, commit `87f30ed0`):

- `parser_tex.json`: the `parserTests` array, as `[input, expected TeX]`, checked against `hurmet.parse(input)`.
- `calc_rpn.json`: the `calcTests` array, as `[parse input, expected RPN, original test input]`.
  The parse input is the original input sliced to before its last `=`, trimmed, with a leading
  `@test ` or `@@test ` removed, exactly as the Hurmet test harness does before calling
  `hurmet.parse(str, { decimalFormat: "1,000,000.", dateFormat: "yyy-mm-dd" }, true)`.
  RPN tokens are separated by U+00A0 (no-break space), or `§` inside lazily evaluated expressions.

The extraction checked that the array source contains only string literals and template literals
without `${}` and evaluated it with Node's `vm.runInNewContext`; Hurmet's own code was never run.

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
