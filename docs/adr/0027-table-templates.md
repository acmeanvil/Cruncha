# ADR-0027: Table templates: standard size tables you browse, pick a row from, and insert as calcs

**Status:** Accepted (2026-11-25)
**Date:** 2026-11-25
**Deciders:** acmeanvil

## Context

Twelve Lacewing Lab calculators are lookup tables, not calculations (research 02 §5). Eleven of them are tables of parts:
- hex, heavy hex, socket head, low head, button head and flat head screws;
- nuts and washers;
- bolt grades;
- disc spring sizes (EN 16983);
- standard pipe sizes (ASME B36.10M and B36.19).

The twelfth, the GD&T symbol page, is a reference page and stays out.

In Lacewing, each is a page of one or more size tables. Each table has:
- a standard;
- a reference to the table and page it was read from;
- one column per tabulated dimension, with its symbol, name, limit (max, min, ref) and kind (length, angle, count, mass, stress, text);
- one row per size;
- footnotes.

They come from generator scripts that read the owner's licensed copies (`engineering_core/lib/standards/`, `piping/`), and there are about 45 tables.

The ADR-0014 template format has no place for them, for two reasons:
- **A template is cells.** A table is data, and the user picks one row from it.
- **ADR-0014 §6 keeps such tables out:** "A table that can't be shown to be free to redistribute stays out, and the template says where to look it up." The owner has now asked for them (2026-11-25): "add the 12 lookup tables to some version of a template where you can look through the tables and then select an item and it gets added to the calc sheet. We don't need to include the GD&T symbols."

## Decision

### 1. A table template is a template with a `[table]` in its front matter

- **The file:** a `.cruncha.md` file as in ADR-0014. The front matter is the same manifest (`id`, `version`, `title`, `category`, `from`, `references`) plus a `[table]` section:
  - `unit` is `"in"` or `"mm"`: the length unit the table prints;
  - `row_label`: `"Size"`, or `"Grade"`;
  - `columns`: one entry per column. Each has:
    - `symbol`: as the standard writes it ("F", "dk");
    - `name`: "Width across flats";
    - `limit`: `max`, `min` or `ref`, if it has one;
    - `kind`: `length`, `angle`, `count`, `mass`, `stress` or `text`;
    - `unit`: for a count with a unit of its own (`N`, `MPa`, `1/in`);
    - `var`: the calc name it inserts as (`F_max`, `d_k`);
  - `rows`: one array per row, the designation first, then a value per column. `""` means the table gives none.
  - `notes`: the table's footnotes that matter for using it.
- **One file per table.** A Lacewing page with several tables (Nuts has seven) becomes several templates with the same `from`. The status table counts them as one calculator, as it does now.
- **The body** holds ordinary cells, which are inserted after the row's values and read them by their `var` names. For example, a pipe table's body computes ID, areas, I, Z and r from OD and t. Most tables have no body.
- **The values are stored as the table prints them.** A length is in the table's unit and a stress in ksi or MPa. A mass printed per 100 or per 1000 pieces is converted to the mass of one piece; the conversion is exact, a decimal shift.

### 2. Inserting a row

- **What the user does:**
  1. In the Templates tab, a table template opens as a table: its columns and rows, its standard, reference and notes. There is a filter box for sizes.
  2. They pick a row, and can untick columns they don't want; by default all columns are ticked.
  3. They choose Insert.
- **What is inserted:**
  - a prose line naming the part, size and source ("Hex cap screw 1/4 · ASME B18.2.1-2012 Table 6, pp. 10-11");
  - one calc per ticked column: `F_max = 0.438 in`, with the column's name as its description ("Width across flats, max");
  - then the body's cells, if there are any.
- **The rest is ADR-0014's insert path:**
  - the editor's name check and renames (a second screw's `F_max` becomes `F_max2`, or whatever the user types);
  - Imported provenance with the template's id and version, plus the row;
  - one commit.
- **Empty cells:** a cell the table leaves empty inserts nothing. Text columns insert strings, and counts insert plain numbers or numbers with their unit.
- **Agents:**
  - `templates(id)` returns the table template's text;
  - `propose_template` gains `row` (the designation) and `columns` (the `var`s to keep; all of them if not given). The rest of its bindings and renames work as they do now.

### 3. Data and its provenance

The owner decides to include these tables, superseding the table clause of ADR-0014 §6 for them:

- **What goes in:** the tabulated numbers, as Lacewing's generators read them from licensed copies, with:
  - each table's standard and edition;
  - the table and page each number was read from;
  - the corrections the generators made (in the notes).
- **What stays out:** no text, figures or layout of a standard. The column names and notes are Lacewing's own words.
- **Regeneration:** `scripts/gen-table-templates.py` writes the files from a JSON dump of Lacewing's tables. A small Dart program, `scripts/lacewing-tables/dump.dart`, runs against `engineering_core` to make the dump. The files say "generated; do not edit by hand".
- **Licence:** the files carry the same licence line as the other templates (GPL-3.0-or-later).
- **The status table** marks these calculators as having templates.

### 4. Checks

`cruncha-templates` validates a table template:
- every row has one value per column;
- numbers are in number columns and strings in text columns;
- `var`s are calc names and are unique;
- each value inserts as a calc that evaluates to its own number in its unit;
- max ≥ min wherever a column pair has both;
- the body's calcs evaluate for the first row and the last row.

`tests/library.rs` runs these checks on every built-in table.

## Options Considered

### Option A: Table templates: data in the front matter, per-column calcs on insert (chosen)

| Dimension | Assessment |
| --- | --- |
| Complexity | Low to medium: a `[table]` section in the manifest, a table view in the Templates tab, a row-to-cells step before the existing insert |
| Cost | A generator; the data already exists in Lacewing |
| Scalability | About 45 tables and a few thousand rows; each file stays small |
| Team familiarity | The same file format, library, insert path and tests as other templates |

**Pros:**
- **Readable values:** the inserted values are ordinary named calcs with descriptions, so they print in reports and show in the variable browser.
- **The row is recorded:** provenance says which table and row each value came from.
- **Agents use the same path** as other templates.

**Cons:**
- **Changing size means a new insert:** the inserted values are a copy, and editing them doesn't change size.
- **The data sits in TOML,** which is less pleasant to read than a Markdown table.

### Option B: Insert the whole table as a data frame calc, and look a size up from it

Hurmet's data frames can hold the table, and `T["1/4"].F_max` would read a value from it.

**Pros:** changing size is one edit.
**Cons:**
- every document carries the whole table (30 rows × 19 columns);
- reports print a large frame or hide it;
- the lookups are harder to review than `F_max = 0.438 in`;
- data frame literals with unit-carrying columns hit the matrix-literal gaps noted in `docs/TODO.md`.

This could be offered later as a second insert option.

### Option C: A separate "Tables" palette outside the template library

**Pros:** a dedicated view.
**Cons:**
- it needs a second library, search, hiding, user folders and MCP tools for what templates already do;
- provenance and renames would have to be duplicated.

Rejected.

### Option D: Keep them out (ADR-0014 §6 as written)

**Pros:** no redistribution question.
**Cons:** the owner wants them, and Lacewing already has the data with its sources.

Rejected by the owner.

## Trade-off Analysis

The tables are data, but what the user wants in the document is a few named values with their source. Making the table a template keeps the browser, search, library, provenance, renames and agent tools that templates already have. The new parts are a `[table]` section and a row picker.

Copying values (A) rather than embedding the table (B) favours a reviewable report over one-edit size changes. B can be added later without changing the file format.

## Consequences

- **Templates:** eleven Lacewing calculators get templates, about 45 table templates in all. The status table leaves only the GD&T symbol page.
- **The format:** a template may now be a table. Readers of the format (`cruncha-templates`, the app's library, the MCP tools, the skill) learn `[table]`. Old templates are unchanged, and `cruncha-template = 1` stays.
- **The redistribution question** for standards' numbers is the owner's decision, recorded here. If a table has to come out later, its file is deleted, and documents that used it keep their copied values and provenance.
- **To revisit:**
  - a data frame insert (Option B);
  - user-made tables (CSV import into the user's library);
  - drawings of the parts with the symbols on them, as Lacewing shows (`DimFigure`).

## Action Items

1. [x] `cruncha-templates`: the `[table]` manifest section, validation (§4), and `table_cells(t, row, columns)`, which makes the cells a row inserts. *(2026-11-25: `src/table.rs`, `tests/table.rs`. Two details settled while building it: §1's `part` and `source` name the part and give the citation for the prose line, and a designation's own footnote goes in `[table.row_notes]`; validation evaluates every text value but one number per column, since another number with the same name and unit reads the same way, so a table template opens quickly.)*
2. [x] `scripts/lacewing-tables/dump.dart` and `scripts/gen-table-templates.py`: the fastener, nut, washer, bolt grade, disc spring and pipe tables as templates under `templates/mechanical/tables/`; the pipe tables with a body for section properties. *(2026-11-25: 48 templates. Pipe is three tables, because each column has one unit: steel NPS 1/8-14 from Machinery's (inches), steel NPS 16-30 and stainless from Crane (walls in mm); a size is `NPS × wall (schedule)`. No table prints a mass, so §1's conversion wasn't needed. `tests/table.rs` checks 2 in Schedule 40 against B36.10M: ID 2.067 in, A 1.075 in², I 0.666 in⁴, S 0.561 in³.)*
3. [x] The app: the table view in the Templates tab (filter, row pick, column ticks), and the insert through ADR-0014's path with the row in its provenance. *(2026-11-25: `TablePicker.svelte`, the `template_row` command (`Library::row_detail`); cells are Imported from `template:<id>@<version>#<row>`, and the message is "Insert template <title> <version>: <row>". The owner then chose a **Tables** tab of its own (⌘⇧L, and a Tables button in the toolbar) beside ƒx Functions, Units and Templates: it lists only table templates, grouped by the kind of part, and the Templates tab no longer lists them.)*
4. [x] MCP: `propose_template` takes `row` and `columns`; `skills/cruncha-templates` documents table templates. *(2026-11-25; `templates` marks table templates in its listing.)*
5. [x] `scripts/lacewing-status.py`: drop the `NOT_TEMPLATES` entries these replace; ADR-0014 item 7 note. *(2026-11-25: only the GD&T symbol page is left out. The owner also chose to include Structural engineering's four shape databases the same way: the AISC Shapes Database v16.0 (14 tables, HSS split into rectangular and round), ArcelorMittal's EN sections (21), SFIA cold-formed framing (5) and the Aluminum Design Manual 2015 Part V (19), under `templates/structural/shapes/`, each column in the unit its source prints (EN's Iw, tabulated in 10³ cm⁶, in cm⁶; SFIA's J × 1000 in in⁴). Lacewing's ADM pipe labels lost their schedules (NPS 1/8 Schedule 80 reads "Pipe 8 Sch 0"), so those pipes are named by NPS and wall. Later the same day the owner added ASCE 7-10's coefficient tables (8, `templates/structural/tables/asce7/`): importance factors, snow Ce and Ct, site coefficients Fa and Fv, Table 12.2-1's systems (R, Ω0, Cd with Table 12.8-2's Ct and x), Table 12.8-2 and Cu. Their columns are named as the loads templates' inputs (`F_a`, `C_e`, `R`, …) so a row binds to them, and where ASCE interpolates each tabulated point is a row. Checked against the library's ASCE 7-10.)*
