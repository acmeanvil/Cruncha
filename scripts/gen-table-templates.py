# SPDX-License-Identifier: GPL-3.0-or-later
"""Write the table templates (ADR-0027) under templates/mechanical/tables/ (standard parts) and
templates/structural/shapes/ (the AISC, EN, cold-formed and aluminium shape databases) from a
JSON dump of Lacewing Lab's tables:

    dart run --packages=$LACEWING/.dart_tool/package_config.json scripts/lacewing-tables/dump.dart > tables.json
    python3 scripts/gen-table-templates.py tables.json

($LACEWING is ~/Developer/Tension_Meter/App.) Lacewing's generators read the numbers from the
owner's licensed copies of the standards and record the table and page of each; the files
keep those citations. The column names and notes are Lacewing's own words.
"""

import json
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
OUT = ROOT / "templates/mechanical/tables"
VERSION = "1.0.0"
CATEGORY = ["Mechanical engineering", "Standard parts"]

# Lacewing calculator → (folder, subcategory, tags, its tables with each one's part name).
GROUPS = {
    "mechanical.hexScrews": ("screws", "Screws and bolts", ["hex bolt", "hex cap screw", "bolt dimensions", "fastener"], {
        "asme.hexCapScrew": "Hex cap screw", "asme.hexBolt": "Hex bolt", "iso.hexBolt": "Hex head bolt or screw"}),
    "mechanical.heavyHexScrews": ("screws", "Screws and bolts", ["heavy hex", "structural bolt", "bolt dimensions", "fastener"], {
        "asme.heavyHexScrew": "Heavy hex screw", "asme.heavyHexBolt": "Heavy hex bolt",
        "asme.structuralBolt": "Heavy hex structural bolt", "iso.heavyHexBolt": "Heavy hex structural bolt"}),
    "mechanical.socketHeadScrews": ("screws", "Screws and bolts", ["socket head cap screw", "SHCS", "fastener"], {
        "asme.socketHeadCapScrew": "Socket head cap screw", "iso.socketHeadCapScrew": "Socket head cap screw"}),
    "mechanical.lowHeadScrews": ("screws", "Screws and bolts", ["low head cap screw", "socket", "fastener"], {
        "asme.lowHeadCapScrew": "Low head socket cap screw", "din.lowHeadCapScrew": "Low head socket cap screw"}),
    "mechanical.buttonHeadScrews": ("screws", "Screws and bolts", ["button head cap screw", "BHCS", "fastener"], {
        "asme.buttonHeadCapScrew": "Button head socket cap screw", "iso.buttonHeadCapScrew": "Button head socket cap screw",
        "iso.flangedButtonHeadCapScrew": "Button head socket cap screw with collar"}),
    "mechanical.flatHeadScrews": ("screws", "Screws and bolts", ["flat head cap screw", "countersunk", "FHCS", "fastener"], {
        "asme.flatHeadCapScrew": "Flat countersunk head socket cap screw", "iso.flatHeadCapScrew": "Countersunk head socket cap screw"}),
    "mechanical.hexNuts": ("nuts", "Nuts", ["nut", "hex nut", "jam nut", "heavy hex nut", "fastener"], {
        "asme.hexNut": "Hex nut", "asme.hexJamNut": "Hex jam nut", "asme.heavyHexNut": "Heavy hex nut",
        "asme.heavyHexJamNut": "Heavy hex jam nut", "iso.hexNut": "Hex nut, style 1", "iso.thinNut": "Thin hex nut",
        "iso.heavyHexNut": "Heavy hex nut"}),
    "mechanical.washers": ("washers", "Washers", ["washer", "plain washer", "flat washer", "fastener"], {
        "asme.washerTypeAN": "Plain washer, Type A, N series", "asme.washerTypeAW": "Plain washer, Type A, W series",
        "asme.washerTypeBN": "Plain washer, Type B, narrow", "asme.washerTypeBR": "Plain washer, Type B, regular",
        "asme.washerTypeBW": "Plain washer, Type B, wide", "asme.washerTypeAOther": "Plain washer, Type A",
        "astm.f436Washer": "Hardened structural washer (F436)", "iso.washerSmall": "Plain washer, small series",
        "iso.washerNormal": "Plain washer, normal series", "iso.washerLarge": "Plain washer, large series",
        "iso.washerExtraLarge": "Plain washer, extra large series", "asme.metricWasherNarrow": "Metric plain washer, narrow",
        "asme.metricWasherRegular": "Metric plain washer, regular", "asme.metricWasherWide": "Metric plain washer, wide"}),
    "mechanical.boltGrades": ("grades", "Bolt grades", ["bolt grade", "property class", "proof strength", "tensile strength", "marking", "fastener"], {
        "boltGrades.sae": "SAE J429", "boltGrades.astmStructural": "ASTM", "boltGrades.astmPressure": "ASTM",
        "boltGrades.f593": "ASTM F593", "boltGrades.iso898": "ISO 898-1 property class"}),
    "mechanical.discSpringSizes": ("springs", "Disc springs", ["disc spring", "Belleville", "EN 16983", "DIN 2093", "spring"], {
        "en16983.seriesA": "Disc spring, series A", "en16983.seriesB": "Disc spring, series B",
        "en16983.seriesC": "Disc spring, series C"}),
}

# Symbols with a name of their own (a word, or one a calc can't spell).
NAMES = {
    "Sizes": "sizes", "Hardness": "hardness", "Marking": "marking", "Material": "material", "Nut": "nut",
    "Washer": "washer", "Group": "group", "UNC": "n_UNC", "UNF": "n_UNF", "FIM": "FIM", "P fine": "P_fine",
    "b ≤125": "b_1", "b ≤200": "b_2", "b >200": "b_3", "B ≤100": "B_1", "B >100": "B_2",
    "σII/III": "σ_(II,III)", "σOM": "σ_OM", "De/t": "ratio_Dt", "h0/t": "ratio_ht", "t′": "t′",
}


def var(c):
    """The calc name of a column: the symbol's tail as a subscript (dk → d_k), the limit with it
    (d_(k,max), F_max)."""
    sym, limit = c["symbol"], (None if c["limit"] == "nominal" else c["limit"])
    if c["name"] == "Thread length, L over 6 in":
        return "L_(T,long)"
    if sym in NAMES:
        base, sub = NAMES[sym], None
        if "_" in base and limit:
            base, sub = base.split("_", 1)
            sub = sub.strip("()")
    elif len(sym) > 1 and re.fullmatch(r"[A-Za-zα-ω][A-Za-z0-9]+", sym):
        base, sub = sym[0], sym[1:]
    else:
        base, sub = sym, None
    if limit:
        sub = f"{sub},{limit}" if sub else limit
    if not sub:
        return base
    return f"{base}_({sub})" if "," in sub else f"{base}_{sub}"


def unit_of(c, table_unit):
    kind = c["kind"]
    if kind == "length":
        return table_unit
    if kind == "stress":
        return "ksi" if table_unit == "in" else "MPa"
    if kind == "angle":
        return "°"
    if kind == "count":
        return c["unit"]
    return None


def s(x):
    return json.dumps(x, ensure_ascii=False)


def value(v):
    if v is None:
        return '""'
    if isinstance(v, str):
        return s(v)
    if isinstance(v, float) and v.is_integer():
        return str(int(v))
    return repr(v)


def write(path, head, columns, rows, row_notes, notes, unit, row_label, part, source, body=""):
    lines = ["+++", "# Generated by scripts/gen-table-templates.py from Lacewing Lab's tables (ADR-0027). Do not edit by hand.",
             "cruncha-template = 1"]
    for k in ("id", "version", "title", "summary"):
        lines.append(f"{k} = {s(head[k])}")
    lines.append(f"category = {s(head['category'])}")
    lines.append(f"tags = {s(head['tags'])}")
    lines.append(f"from = {s(head['from'])}")
    lines.append('license = "GPL-3.0-or-later"')
    for r in head["references"]:
        lines += ["", "[[references]]", f"work = {s(r['work'])}", f"locator = {s(r['locator'])}"]
    lines += ["", "[table]", f"unit = {s(unit)}", f"row_label = {s(row_label)}", f"part = {s(part)}", f"source = {s(source)}"]
    if notes:
        lines.append("notes = [")
        lines += [f"  {s(n)}," for n in notes]
        lines.append("]")
    lines.append("rows = [")
    lines += ["  [" + ", ".join(value(v) for v in r) + "]," for r in rows]
    lines.append("]")
    for c in columns:
        lines += ["", "[[table.columns]]", f"symbol = {s(c['symbol'])}", f"name = {s(c['name'])}"]
        if c.get("limit"):
            lines.append(f"limit = {s(c['limit'])}")
        if c.get("kind", "number") != "number":
            lines.append(f"kind = {s(c['kind'])}")
        if c.get("unit"):
            lines.append(f"unit = {s(c['unit'])}")
        lines.append(f"var = {s(c['var'])}")
    if row_notes:
        lines += ["", "[table.row_notes]"]
        lines += [f"{s(k)} = {s(v)}" for k, v in row_notes.items()]
    lines.append("+++")
    text = "\n".join(lines) + "\n" + (("\n" + body.strip() + "\n") if body.strip() else "")
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text)


def standard_tables(dump):
    by_id = {t["id"]: t for t in dump["tables"]}
    written = []
    for calc, (folder, sub, tags, tables) in GROUPS.items():
        for tid, part in tables.items():
            t = by_id[tid]
            cols = []
            for c in t["columns"]:
                cols.append({
                    "symbol": c["symbol"], "name": c["name"],
                    "limit": None if c["limit"] == "nominal" else c["limit"],
                    "kind": "text" if c["kind"] == "text" else "number",
                    "unit": unit_of(c, t["unit"]), "var": var(c),
                })
            # Two columns that would share a name get a number in the subscript.
            seen = {}
            for c in cols:
                n = seen.get(c["var"], 0)
                seen[c["var"]] = n + 1
                if n:
                    v = c["var"]
                    c["var"] = (v[:-1] + f",{n + 1})") if v.endswith(")") else (
                        f"{v.split('_', 1)[0]}_({v.split('_', 1)[1]},{n + 1})" if "_" in v else f"{v}_{n + 1}")
            rows = [[r["size"], *r["values"]] for r in t["rows"]]
            row_notes = {r["size"]: r["note"].rstrip(".") for r in t["rows"] if r["note"]}
            head = {
                "id": f"mechanical.tables.{tid}",
                "version": VERSION,
                "title": f"{t['title']}, {t['standard']}" if t["standard"] not in t["title"] else t["title"],
                "summary": f"{t['title']}: {len(rows)} {t['rowLabel'].lower()}s from {t['standard']}, each "
                           f"{t['rowLabel'].lower()} inserted as named values: "
                           + ", ".join(dict.fromkeys(c["name"].split(" (")[0].lower() for c in cols[:8])) + ".",
                "category": CATEGORY + [sub],
                "tags": tags + [t["standard"]],
                "from": f"lacewing:{calc}",
                "references": [{"work": t["standard"], "locator": t["reference"]}],
            }
            path = OUT / folder / f"mechanical.tables.{tid}.cruncha.md"
            write(path, head, cols, rows, row_notes, t["notes"], t["unit"], t["rowLabel"], part, t["reference"])
            written.append(path)
    return written


PIPE_BODY = """
```calc {{description="Inside diameter"}}
d = D - 2 t = ?? {u}
```

```calc {{description="Metal area"}}
A = π/4 (D^2 - d^2) = ?? {u}²
```

```calc {{description="Bore (flow) area"}}
A_b = π/4 d^2 = ?? {u}²
```

```calc {{description="Second moment of area"}}
I = π/64 (D^4 - d^4) = ?? {u}⁴
```

```calc {{description="Polar second moment (torsion constant)"}}
J = 2 I = ?? {u}⁴
```

```calc {{description="Elastic section modulus"}}
S = I / (D / 2) = ?? {u}³
```

```calc {{description="Plastic section modulus"}}
Z = (D^3 - d^3) / 6 = ?? {u}³
```

```calc {{description="Radius of gyration"}}
r = sqrt(I / A) = ?? {u}
```
"""


def pipe_tables(dump):
    written = []
    sets = [
        ("steel", "Wrought steel pipe, NPS 1/8 to 14 (ASME B36.10M)", "Steel pipe, NPS",
         [p for p in dump["steelPipe"] if p["source"] == "machinerys"], "in"),
        ("steelLarge", "Wrought steel pipe, NPS 16 to 30 (ASME B36.10)", "Steel pipe, NPS",
         [p for p in dump["steelPipe"] if p["source"] == "crane"], "mm"),
        ("stainless", "Stainless steel pipe (ASME B36.19)", "Stainless steel pipe, NPS", dump["stainlessPipe"], "mm"),
    ]
    for key, title, part, sizes, wall_unit in sets:
        assert all(w["unit"] == wall_unit for p in sizes for w in p["walls"]), key
        weight = any(w["weightLbFt"] is not None for p in sizes for w in p["walls"])
        cols = [
            {"symbol": "Schedule", "name": "Schedule or weight designation", "kind": "text", "var": "schedule"},
            {"symbol": "D", "name": "Outside diameter", "unit": "in", "var": "D"},
            {"symbol": "t", "name": "Wall thickness", "unit": wall_unit, "var": "t"},
        ]
        if weight:
            cols.append({"symbol": "w", "name": "Plain-end weight, carbon steel", "unit": "lb/ft", "var": "w"})
        rows = []
        for p in sizes:
            for w in p["walls"]:
                label = " / ".join(w["designations"]) if w["designations"] else ("API 5L" if w["api5L"] else "")
                t = value(w["printed"])
                size = f"{p['nps']} × {t} {wall_unit}" + (f" ({label})" if label else "")
                row = [size, label, round(p["odIn"], 4), w["printed"]]
                if weight:
                    row.append(w["weightLbFt"])
                rows.append(row)
        source = sizes[0]["reference"]
        notes = [
            "A size is NPS × wall thickness, with its schedule or weight designation (STD, XS, XXS; 5S to 80S for stainless) "
            "where it has one; API 5L marks a wall only API 5L lists.",
            "The outside diameter is the NPS's; the wall is as the source prints it (inches in Machinery's, millimetres in Crane).",
            "The cells after the values compute the inside diameter and the section: metal and bore areas, I, J, the elastic "
            "and plastic section moduli and the radius of gyration.",
        ]
        if weight:
            notes.append("The weight is the tabulated plain-end weight of carbon steel pipe in lb/ft (B36.10M: "
                         "W = 10.68 (D − t) t, D and t in inches).")
        head = {
            "id": f"mechanical.tables.pipe.{key}",
            "version": VERSION,
            "title": title,
            "summary": f"{title}: {len(rows)} sizes (NPS and wall), each inserted with its outside diameter and wall, then "
                       "the inside diameter, areas, second moments, section moduli and radius of gyration.",
            "category": CATEGORY + ["Pipe"],
            "tags": ["pipe", "schedule", "NPS", "pipe size", "section properties", "B36.10", "B36.19"],
            "from": "lacewing:mechanical.pipeSizes",
            "references": [{"work": "ASME B36.19" if key == "stainless" else "ASME B36.10M", "locator": source}],
        }
        u = "in" if wall_unit == "in" else "mm"
        path = OUT / "pipe" / f"mechanical.tables.pipe.{key}.cruncha.md"
        write(path, head, cols, rows, {}, notes, "in", "Size", part, source, PIPE_BODY.format(u=u))
        written.append(path)
    return written


SHAPES = ROOT / "templates/structural/shapes"

# Shape symbols a calc can't spell as they are, or that read better as AISC writes them.
SHAPE_NAMES = {
    "OD": "OD", "ID": "ID", "x": "x̄", "y": "y\u0304", "bf/2tf": "λ_f", "b/t": "λ_b", "b/tdes": "λ_(b,des)",
    "h/tw": "λ_w", "h/tdes": "λ_(h,des)", "D/t": "λ_D", "tan(α)": "tanα", "Wel.y": "W_(el,y)", "Wpl.y": "W_(pl,y)",
    "Wel.z": "W_(el,z)", "Wpl.z": "W_(pl,z)", "alpha": "α", "clsMyS355": "cls_(y,355)", "clsMyS460": "cls_(y,460)",
    "clsNS355": "cls_(N,355)", "clsNS460": "cls_(N,460)", "beta": "β", "Rt": "ratio_Rt", "bt": "λ_b", "slope": "slope",
    "W": "W", "G": "G", "J": "J", "A": "A", "H": "H", "C": "C",
}

US = {"length": "in", "area": "in²", "length3": "in³", "length4": "in⁴", "length6": "in⁶", "ratio": None, "weight": "lbf/ft"}


def shape_var(sym):
    if sym in SHAPE_NAMES:
        return SHAPE_NAMES[sym]
    if len(sym) > 1 and re.fullmatch(r"[A-Za-zα-ω][A-Za-z0-9]+", sym):
        return f"{sym[0]}_{sym[1:]}"
    return sym


def scaled(v, k):
    """v × 10^k, exactly as a decimal shift."""
    from decimal import Decimal
    x = Decimal(repr(v)).scaleb(k)
    return int(x) if x == x.to_integral_value() else float(x)


def shape_table(path, head, fields, rows, part, source, notes):
    """fields: (symbol, label, unit, scale exponent); rows: (designation, values by symbol)."""
    used = [f for f in fields if any(f[0] in v for _, v in rows)]
    cols = [{"symbol": sym, "name": label, "kind": "number", "unit": unit, "var": shape_var(sym)} for sym, label, unit, _ in used]
    out = []
    for name, v in rows:
        r = [name]
        for sym, _, _, k in used:
            x = v.get(sym)
            r.append(None if x is None else (scaled(x, k) if k else x))
        out.append(r)
    write(path, head, cols, out, {}, notes, "in" if "in" in {c["unit"] for c in cols} else "mm", "Shape", part, source)


def shape_head(tid, title, sub, calc, tags, work, locator, n, cols_hint):
    return {
        "id": tid, "version": VERSION, "title": title,
        "summary": f"{title}: {n} shapes, each inserted as named values: {cols_hint}.",
        "category": ["Structural engineering", "Shapes", sub], "tags": tags, "from": f"lacewing:{calc}",
        "references": [{"work": work, "locator": locator}],
    }


def slug(s):
    """An id segment: "HE AA" → heAA, "IPE" → ipe, "Non-structural studs" → nonStructuralStuds."""
    t = re.findall(r"[A-Za-z0-9]+", s.replace("HSSround", "HSS round"))
    return t[0].lower() + "".join(w if w.isupper() else w.capitalize() for w in t[1:])


def shape_tables(dump):
    sh = dump["shapes"]
    written = []
    # AISC Shapes Database v16.0: one table per family (HSS split into rectangular and round).
    a = sh["aisc"]
    fields = [(f["symbol"], f["label"], US[f["unit"]], 0) for f in a["fields"]]
    groups = {}
    for r in a["rows"]:
        g, title = r["group"], r["groupTitle"]
        if g == "HSS":
            rect = r["name"].count("X") == 2
            g, title = ("HSS", "HSS, rectangular and square") if rect else ("HSSround", "HSS, round")
        groups.setdefault((g, title), []).append((r["name"], r["values"]))
    for (g, title), rows in groups.items():
        tid = f"structural.tables.aisc.{slug(g)}"
        head = shape_head(tid, f"{title}, AISC", "AISC shapes", "structural.aiscShapes",
                          ["AISC", "shape", "section properties", "steel", title.split(" (")[0]], "AISC Shapes Database",
                          f"v16.0 (Aug. 2023), {title}", len(rows), "area, depth, flange and web, I, S, Z, r, J, Cw")
        notes = ["Values as the AISC Shapes Database v16.0 tabulates them (consistent with the Steel Construction Manual, "
                 "16th ed.), in US units; W is the nominal weight in lbf/ft.",
                 "Columns the family doesn't tabulate are left out."]
        path = SHAPES / "aisc" / f"{tid}.cruncha.md"
        shape_table(path, head, fields, rows, "AISC", a["source"], notes)
        written.append(path)
    # ArcelorMittal (EN 10365, EN 10056-1): one table per family; units as printed (mm, cm², cm⁴…).
    e = sh["en"]
    def en_unit(f):
        u, k = f["unit"], f["factor"]
        sym = f["symbol"]
        if u == "weight":
            return "kg/m", 0
        if u == "length":
            return ("mm" if abs(k - 1e-3) < 1e-12 else "cm"), 0
        if u in ("area", "length3", "length4"):
            return {"area": "cm²", "length3": "cm³", "length4": "cm⁴"}[u], 0
        if u == "length6":
            return "cm⁶", 3  # tabulated in 10³ cm⁶
        if sym.startswith("ApV"):
            return "1/m", 0
        return {"AL": "m²/m", "AG": "m²/tonne", "alpha": "°"}.get(sym), 0
    fields = []
    for f in e["fields"]:
        unit, k = en_unit(f)
        label = f["label"] + (" (tabulated in 10³ cm⁶)" if k else "")
        fields.append((f["symbol"], label, unit, k))
    groups = {}
    for r in e["rows"]:
        groups.setdefault(r["group"], []).append((r["name"], r["values"]))
    for fam, rows in groups.items():
        title, pages = e["families"][fam]["title"], e["families"][fam]["pages"]
        tid = f"structural.tables.en.{slug(fam)}"
        head = shape_head(tid, f"{title}, EN", "EN sections", "structural.enShapes",
                          ["EN 10365", "Eurocode", "shape", "section properties", "steel", fam], "ArcelorMittal Europe, Sections and Merchant Bars",
                          f"sales programme V2026-1, {title}, {pages}", len(rows), "mass, dimensions, A, I, W_el, W_pl, i, I_t, I_w, classes")
        notes = ["Dimensions to EN 10365:2017 (angles EN 10056-1:2017); properties and cross-section classes to EN 1993-1-1:2005, "
                 "as ArcelorMittal's sales programme V2026-1 tabulates them: mm, cm², cm³, cm⁴, radii and positions in cm.",
                 "Iw is tabulated in 10³ cm⁶; here it is in cm⁶ (the tabulated value × 1000).",
                 "Classes are numbers (1 to 4) for bending about the strong axis and for compression, in S355 and S460.",
                 "Columns the family doesn't tabulate are left out."]
        path = SHAPES / "en" / f"{tid}.cruncha.md"
        shape_table(path, head, fields, rows, "EN section", f"{e['source']}, {title}, {pages}", notes)
        written.append(path)
    # SFIA cold-formed steel framing: one table per group.
    c = sh["sfia"]
    allow = {"Ma": "in·kip", "Mad": "in·kip", "Vag": "lbf", "Vanet": "lbf", "Va": "lbf", "Lu": "in"}
    groups = {}
    for r in c["rows"]:
        groups.setdefault(r["group"], []).append((r["name"], r["values"]))
    for g, rows in groups.items():
        fields = []
        for f in c["fields"]:
            if f["unit"] == "allowable":
                unit = "ft·lbf" if (f["symbol"] == "Ma" and g == "Hat furring channels") else allow[f["symbol"]]
                label = f["label"].split(" (")[0]
            else:
                unit = "ksi" if f["symbol"] == "Fy" else US[f["unit"]]
                label = f["label"].replace(" (ksi)", "")
            fields.append((f["symbol"], label, unit, 0))
        tid = f"structural.tables.sfia.{slug(g)}"
        head = shape_head(tid, f"Cold-formed steel {g.lower()}, SFIA", "Cold-formed steel", "structural.coldFormedShapes",
                          ["SFIA", "cold-formed", "stud", "track", "AISI S100", "section properties"], "SFIA Technical Guide for Cold-Formed Steel Framing Products",
                          f"v2025.1, {g.lower()}", len(rows), "thickness, F_y, A, I, S, effective properties, allowables")
        notes = ["As the SFIA Technical Guide v2025.1 tabulates them (properties to AISI S100-16 and S2-20), in US units; a "
                 "designation's yield strength is in brackets.",
                 "Allowable moments and shears are ASD values from the guide; J is in in⁴ (the guide tabulates J × 1000)."]
        path = SHAPES / "coldFormed" / f"{tid}.cruncha.md"
        shape_table(path, head, fields, rows, "Cold-formed", f"{c['source']}, {g.lower()}", notes)
        written.append(path)
    # Aluminum Design Manual 2015 Part V: one table per ADM table.
    m = sh["adm"]
    fields = [(f["symbol"], f["label"], ("°" if f["symbol"] == "alpha" else US[f["unit"]]), 0) for f in m["fields"]]
    fields.append(("slope", "Flange slope", None, 0))
    od = {round(p["odIn"], 3): p["nps"] for p in dump["steelPipe"]}
    groups = {}
    for r in m["rows"]:
        name = r["name"]
        if r["group"] == "22":  # Lacewing's pipe labels lost their schedules: name by NPS and wall
            name = f"NPS {od[round(r['values']['OD'], 3)]} × {value(r['values']['t'])} in"
        groups.setdefault(r["group"], []).append((name, r["values"]))
    for n, rows in groups.items():
        title = m["tables"][n]
        tid = f"structural.tables.adm.table{n}"
        head = shape_head(tid, f"Aluminum {title[0].lower() + title[1:]}, ADM Table {n}", "Aluminum shapes", "structural.admShapes",
                          ["Aluminum Design Manual", "ADM", "aluminum", "shape", "section properties"], "Aluminum Association, Aluminum Design Manual",
                          f"2015, Part V, Table {n} ({title})", len(rows), "dimensions, A, I, S, Z, r, J, Cw")
        notes = ["As the Aluminum Design Manual 2015 Part V tabulates them, in US units; W is the weight of 6061 in lbf/ft."]
        if n == "22":
            notes.append("Each pipe is named by its NPS and wall: the schedules in Lacewing's data were misread, so they are left out.")
        path = SHAPES / "aluminum" / f"{tid}.cruncha.md"
        shape_table(path, head, fields, rows, "Aluminum", f"{m['source']}, Table {n} ({title})", notes)
        written.append(path)
    return written


def main():
    dump = json.loads(pathlib.Path(sys.argv[1]).read_text())
    written = standard_tables(dump) + pipe_tables(dump) + shape_tables(dump)
    print(f"{len(written)} table templates")


if __name__ == "__main__":
    main()
