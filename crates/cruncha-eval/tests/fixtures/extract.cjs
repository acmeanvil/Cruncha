// SPDX-License-Identifier: GPL-3.0-or-later
// Extract Hurmet's evaluator golden tests as JSON. Usage: node extract.cjs <hurmet-clone> <outdir>
// Only literal text is evaluated (checked first); no Hurmet code is run.
const fs = require("fs"), vm = require("vm"), path = require("path")
const [clone, outDir] = process.argv.slice(2)
const src = fs.readFileSync(path.join(clone, "test/test.cjs"), "utf8")
function slice(startMarker, endMarker) {
  const s = src.indexOf(startMarker); if (s < 0) throw new Error("no " + startMarker)
  const a = src.indexOf("[", s)
  const e = src.indexOf(endMarker, a); if (e < 0) throw new Error("no end " + endMarker)
  return src.slice(a, src.lastIndexOf("]", e) + 1)
}
function checkLiteral(text) {
  let rest = "", i = 0
  while (i < text.length) {
    const c = text[i]
    if (c === "/" && text[i + 1] === "/") { i = text.indexOf("\n", i); if (i < 0) break; continue }
    if (c === '"' || c === "'" || c === "`") {
      let j = i + 1
      while (text[j] !== c) {
        if (text[j] === "\\") j++
        if (c === "`" && text[j] === "$" && text[j + 1] === "{") throw new Error("template interpolation at " + j)
        j++
        if (j >= text.length) throw new Error("unterminated string")
      }
      i = j + 1; rest += "S"; continue
    }
    rest += c; i++
  }
  if (!/^[\s\[\],S]*$/.test(rest)) throw new Error("non-literal content: " + rest.replace(/[\s\[\],S]/g, "").slice(0, 80))
}
const assignText = slice("const assignmentTests = [", "const tex2CalcTests")
const calcText = slice("const calcTests = [", "const testRegEx")
checkLiteral(assignText); checkLiteral(calcText)
const assignmentTests = vm.runInNewContext("(" + assignText + ")", Object.create(null))
const calcTests = vm.runInNewContext("(" + calcText + ")", Object.create(null))
// The function definitions: hurmet.calculate(`function ...end`, vars) between the two arrays.
const fStart = src.indexOf("// Write a few functions into the vars object")
const fEnd = src.indexOf("// Calculations.", fStart)
const fText = src.slice(fStart, fEnd)
const re = /hurmet\.calculate\((`[^`]*`), vars\)/g
const functions = []
let m
while ((m = re.exec(fText)) !== null) {
  checkLiteral("[" + m[1] + "]")
  functions.push(vm.runInNewContext("(" + m[1] + ")", Object.create(null)))
}
const w = (f, v) => fs.writeFileSync(path.join(outDir, f), JSON.stringify(v, null, 1) + "\n")
w("assignments.json", assignmentTests)
w("functions.json", functions)
w("calculations.json", calcTests)
console.log("assignments", assignmentTests.length, "functions", functions.length, "calc", calcTests.length)
