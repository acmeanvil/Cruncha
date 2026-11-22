#!/usr/bin/env node
// SPDX-License-Identifier: GPL-3.0-or-later
// Every npm package the app installs must have a GPL-3 compatible license (ADR-0001).
//   node scripts/check-npm-licenses.mjs [app/package-lock.json]
import { readFileSync } from "node:fs";

const ALLOW = new Set([
  "MIT", "MIT-0", "ISC", "BSD-2-Clause", "BSD-3-Clause", "0BSD", "Apache-2.0", "MPL-2.0", "Zlib",
  "CC0-1.0", "Unlicense", "BlueOak-1.0.0", "Python-2.0", "PSF-2.0",
  "LGPL-2.1-or-later", "LGPL-3.0-or-later", "GPL-3.0-or-later", "GPL-3.0-only", "GPL-2.0-or-later",
]);

/** An SPDX expression is allowed if every AND term, and some OR alternative, is. */
function allowed(expr) {
  const e = expr.replace(/[()]/g, " ").trim();
  return e.split(/\s+OR\s+/).some((alt) => alt.split(/\s+AND\s+/).every((t) => ALLOW.has(t.trim())));
}

const path = process.argv[2] ?? new URL("../app/package-lock.json", import.meta.url).pathname;
const lock = JSON.parse(readFileSync(path, "utf8"));
const bad = [];
let n = 0;
for (const [where, p] of Object.entries(lock.packages ?? {})) {
  if (!where) continue; // the app itself
  n += 1;
  const lic = typeof p.license === "string" ? p.license : p.license?.type;
  if (!lic || !allowed(lic)) bad.push(`${where.replace(/^.*node_modules\//, "")}@${p.version}: ${lic ?? "no license field"}`);
}
if (bad.length) {
  console.error(`npm packages without an allowed license (${bad.length} of ${n}):\n  ` + bad.join("\n  "));
  process.exit(1);
}
console.log(`npm licenses ok (${n} packages)`);
