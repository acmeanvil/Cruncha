# SPDX-License-Identifier: GPL-3.0-or-later
# Generate the Rust unit table from Hurmet's units.js JSON literal.
# Usage: python3 -I gen_units.py <units.js> <out.rs>
import json, sys
src = open(sys.argv[1], encoding="utf-8").read()
marker = "export const unitTable = Object.freeze(JSON.parse(`"
start = src.index(marker) + len(marker)
end = src.index("`))", start)
text = src[start:end].replace("\\\\", "\\")  # template literal: \\ -> \
pairs = json.loads(text, object_pairs_hook=lambda p: p)
out = []
out.append("// SPDX-License-Identifier: GPL-3.0-or-later")
out.append("// Ported from Hurmet (https://hurmet.org), Copyright (c) 2026 Ronald Kok, MIT License; see LICENSE-HURMET.")
out.append("// Generated from Hurmet `src/units.js` (`unitTable`, commit 87f30ed0) by a script; do not edit by hand.")
out.append("")
out.append("//! Hurmet's unit table: name, factor numerator, factor denominator, gauge, misc, exponents.")
out.append("//! `misc` is `siSymbol`/`siWord` (SI prefixes allowed), a 3-letter currency code, or `0`.")
out.append("//! Exponents: length, mass, time, current, temperature, amount, luminous intensity, money.")
out.append("")
out.append("pub(crate) type Row = (&'static str, &'static str, &'static str, &'static str, &'static str, [i8; 8]);")
out.append("")
out.append("#[rustfmt::skip]")
out.append("pub(crate) static UNIT_TABLE: &[Row] = &[")
seen = {}
for k, v in pairs:
    ex = v[4]
    note = ""
    if len(ex) != 8:
        note = "  // Hurmet has a ninth exponent here (bug); dropped"
        ex = ex[:8]
    if k in seen:
        note += "  // duplicate key in Hurmet's JSON: the last one wins"
    seen[k] = True
    out.append("    (%s, %s, %s, %s, %s, [%s]),%s" % (
        json.dumps(k, ensure_ascii=False), json.dumps(v[0]), json.dumps(v[1]),
        json.dumps(v[2]), json.dumps(v[3]), ", ".join(str(x) for x in ex), note))
out.append("];")
open(sys.argv[2], "w", encoding="utf-8").write("\n".join(out) + "\n")
print(len(pairs), "rows", len(seen), "unique")
