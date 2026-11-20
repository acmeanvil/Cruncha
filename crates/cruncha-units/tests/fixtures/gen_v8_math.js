// SPDX-License-Identifier: GPL-3.0-or-later
// Write test vectors of V8's Math functions: node gen_math.js <out.json>
// Deterministic inputs (an LCG), bit patterns as hex strings.
const fs = require("fs")
let seed = 12345
const rnd = () => { seed = (seed * 1103515245 + 12345) % 2147483648; return seed / 2147483648 }
const hex = x => { const b = Buffer.alloc(8); b.writeDoubleBE(x); return b.toString("hex") }
const inputs = []
for (let i = 0; i < 300; i++) {
  inputs.push((rnd() - 0.5) * 2)                       // (-1, 1)
  inputs.push(Math.round((rnd() - 0.5) * 2000) / 100)  // two decimals, (-10, 10)
  inputs.push((rnd() - 0.5) * 200)                     // (-100, 100)
  inputs.push((rnd() - 0.5) * 2e5)                     // (-1e5, 1e5)
}
inputs.push(2.34, -5.67, 5.67, 0.5235987755982989, 1.5, Math.PI / 2, Math.PI, 3 * Math.PI / 4)
const fns = {
  sin: Math.sin, cos: Math.cos, tan: Math.tan, exp: Math.exp, atan: Math.atan,
  log: Math.log, log10: Math.log10, log2: Math.log2, asin: Math.asin, acos: Math.acos,
}
const out = []
for (const [name, f] of Object.entries(fns)) {
  for (let x of inputs) {
    if (name === "exp") x = x / 100 * 7
    if (name.startsWith("log")) x = Math.abs(x) + 1e-3
    if (name === "asin" || name === "acos") x = Math.max(-1, Math.min(1, x / 10))
    out.push([name, hex(x), hex(f(x))])
  }
}
for (let i = 0; i < 400; i++) {
  const y = (rnd() - 0.5) * 20, x = (rnd() - 0.5) * 20
  out.push(["atan2", hex(y), hex(x), hex(Math.atan2(y, x))])
  const b = rnd() * 100 + 0.01, e = Math.round((rnd() - 0.5) * 1000) / 100
  out.push(["pow", hex(b), hex(e), hex(Math.pow(b, e))])
}
fs.writeFileSync(process.argv[2], JSON.stringify({ node: process.version, cases: out }) + "\n")
console.log(out.length, "cases", process.version)
