# SPDX-License-Identifier: GPL-3.0-or-later
# Generate the Rust unit table from Hurmet's units.js JSON literal.
# Usage: python3 -I tools/gen_table.py <Hurmet src/units.js at 87f30ed0> src/table.rs
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
out.append("//!")
out.append("//! Corrections to Hurmet's data (see `CORRECTIONS` in tools/gen_table.py):")
out.append("//! - the Fahrenheit gauge is 459.67, not 459 (0 °F = 459.67 °R exactly);")
out.append("//! - tesla and gauss are kg·s⁻²·A⁻¹, not m⁻²·kg⁻²·s⁻²·A⁻¹ (ADR-0022);")
out.append("//! - ampere-turn and gilbert are a current, not temperature × luminous intensity (ADR-0022).")
out.append("")
out.append("pub(crate) type Row = (\n    &'static str,\n    &'static str,\n    &'static str,\n    &'static str,\n    &'static str,\n    [i8; 8],\n);")
out.append("")
out.append("#[rustfmt::skip]")
out.append("pub(crate) static UNIT_TABLE: &[Row] = &[")
# Cruncha's deliberate departures from Hurmet's data: name -> (index, value, note).
# Index 2 is the gauge, 4 the exponents.
FLUX_DENSITY = (4, [0, 1, -2, -1, 0, 0, 0, 0], "corrected: Hurmet has [-2, -2, -2, -1, …] (ADR-0022)")
MMF = (4, [0, 0, 0, 1, 0, 0, 0, 0], "corrected: Hurmet has temperature × luminous intensity (ADR-0022)")
CORRECTIONS = {
    "Fahrenheit": (2, "459.67", "corrected: Hurmet has 459"),
    "°F": (2, "459.67", "corrected: Hurmet has 459"),
    "℉": (2, "459.67", "corrected: Hurmet has 459"),
    "T": FLUX_DENSITY, "tesla": FLUX_DENSITY, "G": FLUX_DENSITY, "gauss": FLUX_DENSITY,
    "At": MMF, "ampere turn": MMF, "Gi": MMF, "gilbert": MMF,
}
seen = {}
for k, v in pairs:
    if k in CORRECTIONS:
        i, val, _ = CORRECTIONS[k]
        v = list(v)
        v[i] = val
    ex = v[4]
    note = ""
    if len(ex) != 8:
        note = "  // Hurmet has a ninth exponent here (bug); dropped"
        ex = ex[:8]
    if k in seen:
        note += "  // duplicate key in Hurmet's JSON: the last one wins"
    if k in CORRECTIONS:
        note += "  // " + CORRECTIONS[k][2]
    seen[k] = True
    out.append("    (%s, %s, %s, %s, %s, [%s]),%s" % (
        json.dumps(k, ensure_ascii=False), json.dumps(v[0]), json.dumps(v[1]),
        json.dumps(v[2]), json.dumps(v[3]), ", ".join(str(x) for x in ex), note))
out.append("];")
open(sys.argv[2], "w", encoding="utf-8").write("\n".join(out) + "\n")
print(len(pairs), "rows", len(seen), "unique")
