# V8 `Math` test vectors

`v8_math.json` holds 12,880 argument/result pairs of JavaScript's `Math.sin`, `cos`, `tan`,
`exp`, `atan`, `atan2`, `log`, `log10`, `log2`, `asin`, `acos` and `pow`, printed by Node.js
v24.21.0 (V8) on macOS (x86-64). Doubles are written as 16-digit hex bit patterns.
`tests/v8_math.rs` checks that `cruncha_units::jsmath` reproduces every result bit for bit.

They were made with `gen_v8_math.js` (`node gen_v8_math.js v8_math.json`), which runs only
Node's built-in `Math` on deterministic pseudo-random inputs: values in (-1, 1), two-decimal
values in (-10, 10), and values up to ±100 and ±100,000.

`Math.pow` in V8 calls the C library's `pow`, so its vectors are platform-specific. `jsmath::pow`
uses fdlibm's algorithm on every platform instead, and the test only requires it to be within one
unit in the last place of these. (The other functions are compared bit for bit.) On another
platform, the vectors can be regenerated there.
