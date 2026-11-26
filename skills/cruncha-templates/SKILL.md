---
name: cruncha-templates
description: How to find, insert and write Cruncha calculation templates and snippets (ADR-0014). Covers the templates and propose_template tools (search the built-in and user libraries, read a template, insert it as a proposal with inputs bound to the document's values and clashing names renamed) and the .cruncha.md format (TOML front matter with inputs, outputs, references and worked-example tests, then Markdown prose and fenced calc, Python and Rust cells). Also table templates (ADR-0027): standard size tables of screws, nuts, washers, bolt grades, disc springs and pipe, one row inserted as named values. Use it when the user asks for a standard calculation ("add a bolt check", "is there a template for pipe friction"), or asks you to draft a snippet or template.
---

# Templates and snippets

A **template** is a reviewed, tested calculation: a bolt check to AISC 360, a pipe friction factor, a beam's reactions and diagrams. A **snippet** is the user's own reusable group of cells. Both are `.cruncha.md` files in one of two libraries:
- **built-in**: bundled with Cruncha, read-only;
- **user**: the user's own folder, which may be shared with a team.

Prefer a template to writing the same calculation from scratch. It carries its references and worked examples, which have already been checked.

## Finding and reading templates

| Tool | Purpose |
| --- | --- |
| `templates(query?)` | Search both libraries. Every word must match a word in the title, summary, tags, category, id, or input and output names. Omit `query` to list them all. |
| `templates(id)` | Read one template: its validation, the outcome of its worked examples, and the whole file |
| `propose_template(id, bindings?, rename?, after?, message, row?, columns?, document?)` | Insert a template into the document as one proposal; a table template needs `row` |

Before you propose a template, read it with `templates(id)`. Check four things:
- what it assumes (the prose);
- its inputs and outputs;
- its references;
- that its worked examples pass.

A failing or skipped example is something to tell the engineer about.

## Inserting: `propose_template`

The insert is a **proposal** (see `cruncha-core`). Nothing changes until the engineer accepts it, and the accept is one version. Its message is `Insert template <title> <version>: <your message>`.

- **Inputs** are the template's `_` placeholders (`d_b = 0.75 in_`). Unless you bind them, they arrive as placeholders for the engineer to fill in. `check` lists unfilled ones as `PLACEHOLDER`.
- **`bindings`**: use a value the document already has. Map input → the document's name: `{"V_u": "V"}`, or `{"L": "L"}` when the name is the same. The input's definition cell is left out, and the template's cells read the document's name instead.
  - Bind only when the document's value really is that quantity (same meaning, same units dimension).
  - Mention every binding in your message.
  - The document must define the name.
  - The input must be defined in a cell of its own.
- **`rename`**: for any other name the document already defines, map old → new (`{"ϕ": "ϕ2"}`).
  - The rename applies to definitions and uses, in calc, Python and Rust cells.
  - Every rename is re-scanned and checked; a rename that can't be done safely is refused.
- **Clashes are refused, never merged.** If the template defines a name the document already has, the call fails. The error lists the names, a free name for each (`"M": "M2"`), and which ones are inputs you could bind. Call again with `rename` or `bindings`.
- **`after`**: the cell to insert after. Omit it for the start of the document. The template's cells keep their order.
- **Code cells arrive not run.** Python and Rust cells in the template run only when the engineer presses Run in a trusted document. Their results are pending until then, so say so.

Example: "add a bolt check for the shear we already have":
1. `templates(query: "bolt shear")`.
2. `templates(id: "structural.bolts.shearTension")`.
3. `read_doc`, to find the shear `V` and the cell to insert after.
4. `propose_template(id: "structural.bolts.shearTension", bindings: {"V_u": "V"}, after: "<cell>", message: "Bolt check for the connection shear V; d_b, n_s, F_nv, F_nt and T_u left for you to fill in")`.
5. If that's refused because the document already defines `ϕ`, call again with `rename: {"ϕ": "ϕ2"}`.

## Table templates (standard parts)

Some templates are **tables**: a standard's size table, such as hex cap screws (ASME B18.2.1), ISO hex nuts, plain washers, bolt grades (SAE J429, ASTM, ISO 898-1), disc springs (EN 16983) or pipe (ASME B36.10M, B36.19), or a shape database: AISC (W, HSS, angles…), EN (IPE, HE, UB…), SFIA cold-formed studs and tracks, and Aluminum Design Manual shapes. The listing marks them `table:`. Their ids start `mechanical.tables.` or `structural.tables.`; search with words like `nut`, `washer`, `pipe`, `socket head`, `W shape`, `IPE`. The user finds them in the app's Tables tab (⌘⇧L).

- **Reading one:** `templates(id)` shows the file. `[table]` has the `part`, the `source` (standard, table and page), `notes`, `rows` (the designation first, then a value per column, `""` where the standard gives none) and `[[table.columns]]`. Each column has its `symbol`, `name`, `limit` (max, min, ref), `unit` and `var`, the calc name it inserts as (`F_max`, `d_(k,max)`, `n_UNC`).
- **Inserting a row:** `propose_template(id, row: "M8", columns?: ["s_max", "m_max"], …)`.
  - `row` is the designation exactly as the first value of the row gives it: `"1/2"`, `"M8"`, `"2 × 0.154 in (40 / STD)"`, `"Grade 5, 1/4-1"`, `"W14X22"`, `"IPE 200"`.
  - `columns` keeps only those `var`s. Omit it for every column. Insert only the values the calculation needs.
  - What arrives:
    1. a prose line naming the part, size and source;
    2. one calc per column the row has a value for (`s_max = 13 mm`), with the column's name as its description;
    3. the template's own cells, if any. A pipe table adds the inside diameter, areas, I, J, section moduli and radius of gyration.
  - The proposal's message is `Insert template <title> <version>: <row>` followed by yours.
- **Values are the standard's, as printed:** lengths in the table's unit (inches or millimetres; EN section properties in cm, cm², cm⁴), stresses in ksi or MPa, percentages with `%`. An AISC shape's `W` is its nominal weight in lbf/ft; an EN section's `G` is its mass in kg/m. Threads per inch (`n_UNC`) are plain numbers. Quote the row and the source line when you use a value.
- **Clashes:** the column names are short (`d`, `P`, `F`), so a second part often clashes with the first. Rename as for any template (`rename: {"d": "d_nut"}`).
- A bolt grade's strengths (`S_p`, `S_y`, `S_(ut,min)`) are minimums from its specification. Say which grade and size range you picked.

You can't save templates or snippets to a library, edit them, or delete them: that is the user's action, in the Templates tab (⌘⇧T). If the user wants a new snippet, propose its cells into the document. Then tell them to select those cells and choose "Save selection as snippet…". If they want the file itself, give them its text to import.

## The file format

An abridged, LRFD-only version of the built-in bolt template (the real one, version 1.1.0, also does ASD, with `R_(a,nv)` and an `LRFD = 1_` input):

````markdown
+++
cruncha-template = 1
id = "structural.bolts.shearTension"     # dotted, stable: documents' provenance points to it
version = "1.0.0"                         # major.minor.patch
title = "Bolt shear and tension, AISC 360 (LRFD)"
summary = "Design shear and tension strength of one bolt, and tension reduced by shear."
category = ["Structural engineering", "Connections"]
tags = ["bolt", "AISC 360", "LRFD"]
license = "GPL-3.0-or-later"
inputs = ["d_b", "n_s", "F_nv", "F_nt", "V_u", "T_u"]
outputs = ["A_b", "ϕR_nv", "ϕR_nt"]

[[references]]
work = "ANSI/AISC 360-22 Specification for Structural Steel Buildings"
edition = "2022"
locator = "Sec. J3.7, Eq. J3-1; Table J3.2"

[[tests]]
name = "3/4 in. A325-N, single shear"
source = "By hand: A_b = π 0.75²/4 = 0.4418 in², ϕR_nv = 0.75 × 54 × 0.4418 = 17.89 kips"
inputs = { d_b = "0.75 in", F_nv = "54 ksi" }
expect = { A_b = "0.4418 in²", "ϕR_nv" = "17.89 kips" }
tolerance = "0.1%"
checks = "pass"
+++

## Bolt shear and tension

Prose is ordinary Markdown: one paragraph, heading, list or quote per cell.

```calc {description="Nominal bolt diameter"}
d_b = 0.75 in_
```

```calc {description="Nominal unthreaded body area"}
A_b = π d_b^2 / 4 = ?? in²
```

```calc
> Required shear stress
f_rv = V_u / (A_b n_s) = ?? ksi
> Design tensile strength with shear, Eq. J3-2
ϕR_ntc = ϕ F_nt′ A_b = ?? kips
```

```python {description="Darcy friction factor (Colebrook)"}
f = brentq(lambda x: …, 1e-4, 1.0)
```
````

### Front matter (the manifest)

- **Required:** `cruncha-template = 1`, `id`, `version` and `title`. A snippet needs nothing more.
- **Optional:** `summary`, `category` (a path), `tags`, `from` (where it was converted from, e.g. `lacewing:structural.bolts`), and `license`.
- **For templates:** `inputs`, `outputs`, `[[references]]` and `[[tests]]`.
- **For table templates:** a `[table]` with `unit`, `row_label`, `part`, `source`, `notes`, `rows`, `[[table.columns]]` and `[table.row_notes]` (see Table templates above). It has no `inputs` or `[[tests]]`; validation checks every row instead. The built-in ones are generated (`scripts/gen-table-templates.py`), so don't edit them by hand.
- **Unknown keys are errors.**
- Write a manifest name as a calc reads it: `λ_1` is `λ₁`. Quote keys that aren't plain ASCII: `"ϕR_nv" = "…"`.

### Cells

- **Prose** is Markdown between the fences.
- **Calc, Python, Rust and TeX cells** are fences whose info string names the kind (`calc`, `python`, `rust`, `tex`).
- **The one attribute** is `{description="…"}`, a calc's one-line description. Escape `"` and `\` inside it as `\"` and `\\`. If the description has a backtick, use a `~~~` fence.
- **One calc fence is one calc,** with these exceptions:
  - a data frame literal or a `function … end` stays together in one fence;
  - a **group** is the steps to one answer, with each step's description on a `>` line before it.
- **An input** is a calc whose value ends in `_` (`h = 4 in_`). The value is the default, so make it sensible: it is what the template evaluates with until the user types theirs.
- **An output** is a name the template defines and promises. Show key results with `= ?? unit`, so the report prints them with units and an echo.
- **A check** is an `@test` calc (`@test V_u ≤ ϕR_nv`).
- Calc syntax is in `cruncha-calc-authoring`; Python and Rust cells are in `cruncha-code-cells`.

### Worked examples (`[[tests]]`)

Each case has the following keys:

| Key | What it holds |
| --- | --- |
| `name` | The case's name |
| `source` | Where the numbers come from: a work, edition and example, or "By hand: …" with the arithmetic |
| `inputs` | Values that replace the placeholders, as calc literals (`"0.75 in"`). Inputs left out keep their defaults. |
| `expect` | Outputs, as calc literals. They are compared in SI within `tolerance`: `"0.5%"`, or a relative number such as `"1e-9"` (the default). |
| `checks` | `"pass"` (every `@test` holds) or `"fail"` (at least one doesn't) |
| `figures` | How many figures the code cells must show |

Cruncha runs every case with its own engine. Calcs always run. Python cells run where the Python environment exists.

A converted or published template carries at least one case from a cited source, as well as hand checks.

### Sources and copyright

- **Facts go in:** formulas, worked-example numbers, and citations (work, edition, clause, table or example).
- **Never copy** the text, figures or tables of a copyrighted standard or book. Describe what the clause does in your own words, and cite it.
- **Lookup tables** (materials, bolt grades, section properties) need their own provenance. The built-in table templates (ADR-0027) are generated from Lacewing Lab's tables, which were read from licensed copies, each with its table and page; the owner decided to include them. Don't type in other tables from a standard: if a table can't be shown to be free to redistribute, leave it out and say where to look the values up.

## Rules

- **The engine computes:** quote a template's results from `run` or the proposal's "if accepted" results, never from the file's test values.
- **Say what you did:** in the proposal's message and your reply, give the template's id and version, each binding and rename, and the inputs left as placeholders.
- **Don't change a template's equations while inserting it.** If it doesn't fit, say why, and propose ordinary cells with `propose_edit` instead.
