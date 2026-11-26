// SPDX-License-Identifier: GPL-3.0-or-later

//! Table templates (ADR-0027): a standard's size table in the front matter (`[table]`), from
//! which the user picks a row; the row inserts as one calc per column, after a prose line
//! naming the part, size and source, then the template's own cells (which read the columns).

use serde::{Deserialize, Serialize};

use crate::{Cell, Template};

/// A size table: its columns, and one row per size (the designation first).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Table {
    /// The length unit the table prints (`in` or `mm`).
    pub unit: String,
    /// What a row is: `Size`, or `Grade`.
    #[serde(default = "size_label")]
    pub row_label: String,
    /// The part, as the prose line names it ("Hex cap screw").
    pub part: String,
    /// Where the numbers were read: standard, table and page.
    pub source: String,
    pub columns: Vec<Column>,
    /// The designation, then one value per column; `""` where the table gives none.
    pub rows: Vec<Vec<Value>>,
    /// A designation's own footnote.
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub row_notes: std::collections::BTreeMap<String, String>,
    /// The table's footnotes that matter for using it.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
}

fn size_label() -> String {
    "Size".into()
}

/// One column.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Column {
    /// As the standard writes it: `F`, `dk`.
    pub symbol: String,
    /// "Width across flats".
    pub name: String,
    /// `max`, `min` or `ref`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<String>,
    /// `number` (the default) or `text`.
    #[serde(default = "number_kind")]
    pub kind: String,
    /// The value's unit as a calc writes it (`in`, `ksi`, `°`, `N`, `%`); none for a plain number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    /// The calc name it inserts as (`F_max`, `d_(k,max)`).
    pub var: String,
}

fn number_kind() -> String {
    "number".into()
}

impl Column {
    /// "Width across flats, max".
    pub fn description(&self) -> String {
        match self.limit.as_deref() {
            Some(l) => format!("{}, {l}", self.name),
            None => self.name.clone(),
        }
    }

    /// The column heading: symbol and limit ("F max").
    pub fn heading(&self) -> String {
        match self.limit.as_deref() {
            Some(l) => format!("{} {l}", self.symbol),
            None => self.symbol.clone(),
        }
    }
}

/// A table cell: a number, or text (`""`: none given).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Value {
    Number(f64),
    Text(String),
}

impl Value {
    pub fn is_empty(&self) -> bool {
        matches!(self, Value::Text(s) if s.is_empty())
    }

    /// As the table shows it.
    pub fn display(&self) -> String {
        match self {
            Value::Number(x) => number(*x),
            Value::Text(s) => s.clone(),
        }
    }
}

/// The shortest decimal that reads back as `x` (`20`, `0.4375`).
fn number(x: f64) -> String {
    format!("{x}")
}

impl Table {
    /// The row designated `size`.
    pub fn row(&self, size: &str) -> Option<&[Value]> {
        self.rows.iter().find(|r| matches!(r.first(), Some(Value::Text(s)) if s == size)).map(|r| &r[1..])
    }

    /// Every designation, in order.
    pub fn sizes(&self) -> impl Iterator<Item = &str> {
        self.rows.iter().filter_map(|r| match r.first() {
            Some(Value::Text(s)) => Some(s.as_str()),
            _ => None,
        })
    }

    /// The calc a value inserts as (`F_max = 0.438 in`); none for an empty cell.
    pub fn calc(&self, column: &Column, value: &Value) -> Option<String> {
        if value.is_empty() {
            return None;
        }
        Some(match value {
            Value::Text(s) => format!("{} = \"{}\"", column.var, s.replace('"', "'")),
            Value::Number(x) => match column.unit.as_deref() {
                None | Some("") => format!("{} = {}", column.var, number(*x)),
                Some(u @ ("%" | "°")) => format!("{} = {}{u}", column.var, number(*x)),
                Some(u) => format!("{} = {} {u}", column.var, number(*x)),
            },
        })
    }
}

/// The cells a table template's row inserts (ADR-0027 §2): the prose line naming the part,
/// size and source; a calc per column in `columns` (by `var`; every column when `None`) that
/// the row has a value for; then the template's own cells.
pub fn table_cells(t: &Template, size: &str, columns: Option<&[String]>) -> Result<Vec<Cell>, String> {
    let table = t.manifest.table.as_ref().ok_or_else(|| format!("{} isn't a table template", t.manifest.id))?;
    let row = table.row(size).ok_or_else(|| {
        let sizes: Vec<&str> = table.sizes().take(8).collect();
        format!("{} has no {} {size:?} (it has {}, …)", t.manifest.id, table.row_label.to_lowercase(), sizes.join(", "))
    })?;
    if let Some(cols) = columns {
        for c in cols {
            if !table.columns.iter().any(|k| &k.var == c) {
                let vars: Vec<&str> = table.columns.iter().map(|k| k.var.as_str()).collect();
                return Err(format!("{} has no column {c} (its columns: {})", t.manifest.id, vars.join(", ")));
            }
        }
    }
    let mut line = format!("{} {size} · {}.", table.part, table.source);
    if let Some(n) = table.row_notes.get(size) {
        line.push_str(&format!(" Note: {n}."));
    }
    let text = cruncha_prose::Inline::Text { text: line, marks: Default::default() };
    let mut out = vec![Cell::Prose { source: cruncha_prose::to_json(&cruncha_prose::Block::Paragraph(vec![text])) }];
    for (col, value) in table.columns.iter().zip(row) {
        if columns.is_some_and(|cols| !cols.contains(&col.var)) {
            continue;
        }
        if let Some(source) = table.calc(col, value) {
            out.push(Cell::Calc { source, description: Some(col.description()) });
        }
    }
    out.extend(t.cells.iter().cloned());
    Ok(out)
}

/// What's wrong with a table (ADR-0027 §4).
pub(crate) fn validate_table(t: &Template, table: &Table) -> Vec<String> {
    let mut problems = Vec::new();
    let mut vars: Vec<&str> = Vec::new();
    for c in &table.columns {
        if vars.contains(&c.var.as_str()) {
            problems.push(format!("column {:?}: var {} is used twice", c.heading(), c.var));
        }
        vars.push(&c.var);
        if c.kind != "number" && c.kind != "text" {
            problems.push(format!("column {:?}: kind is \"number\" or \"text\", not {:?}", c.heading(), c.kind));
        }
        if let Some(l) = &c.limit
            && !matches!(l.as_str(), "max" | "min" | "ref")
        {
            problems.push(format!("column {:?}: limit is max, min or ref, not {l:?}", c.heading()));
        }
    }
    if table.rows.is_empty() {
        problems.push("the table has no rows".into());
    }
    let mut seen: Vec<&str> = Vec::new();
    for (i, r) in table.rows.iter().enumerate() {
        let size = match r.first() {
            Some(Value::Text(s)) if !s.is_empty() => s.as_str(),
            _ => {
                problems.push(format!("row {}: starts with its designation (a string)", i + 1));
                continue;
            }
        };
        if seen.contains(&size) {
            problems.push(format!("row {size:?} is listed twice"));
        }
        seen.push(size);
        if r.len() != table.columns.len() + 1 {
            problems.push(format!("row {size:?}: {} values for {} columns", r.len() - 1, table.columns.len()));
            continue;
        }
        for (c, v) in table.columns.iter().zip(&r[1..]) {
            match (c.kind.as_str(), v) {
                ("number", Value::Text(s)) if !s.is_empty() => {
                    problems.push(format!("row {size:?}, column {:?}: {s:?} isn't a number", c.heading()))
                }
                ("text", Value::Number(x)) => problems.push(format!("row {size:?}, column {:?}: {x} isn't text", c.heading())),
                _ => {}
            }
        }
        // max ≥ min where a column has both.
        for (a, c) in table.columns.iter().enumerate() {
            if c.limit.as_deref() != Some("max") {
                continue;
            }
            let min = table.columns.iter().position(|k| k.symbol == c.symbol && k.name == c.name && k.limit.as_deref() == Some("min"));
            if let Some(b) = min
                && let (Value::Number(hi), Value::Number(lo)) = (&r[1 + a], &r[1 + b])
                && hi < lo
            {
                problems.push(format!("row {size:?}: {} {hi} is less than its min {lo}", c.heading()));
            }
        }
    }
    if table.row_notes.keys().any(|k| !seen.contains(&k.as_str())) {
        problems.push("a row note names a designation the table doesn't have".into());
    }
    if !problems.is_empty() {
        return problems;
    }
    // Every value inserts as a calc that evaluates (a number checked once per column: another
    // number with the same name and unit reads the same way; text always, for its characters);
    // the body evaluates for the first and the last row.
    let o = crate::opts();
    let mut checked = vec![false; table.columns.len()];
    for (i, size) in table.sizes().enumerate() {
        let row = table.row(size).expect("a listed size");
        for (k, (c, v)) in table.columns.iter().zip(row).enumerate() {
            if matches!(v, Value::Number(_)) && std::mem::replace(&mut checked[k], true) {
                continue;
            }
            let Some(src) = table.calc(c, v) else { continue };
            match cruncha_eval::prepare(&src, &o) {
                Err(e) => problems.push(format!("row {size:?}: {src:?} doesn't parse: {e}")),
                Ok(stmt) => {
                    if stmt.defines != [cruncha_lang::hurmet::check_for_numeric_subscript(&c.var)] {
                        problems.push(format!("row {size:?}: {src:?} defines {:?}, not {}", stmt.defines, c.var));
                    }
                    if let Some(e) = cruncha_eval::evaluate(&stmt, &cruncha_eval::Vars::new(), &o).error {
                        problems.push(format!("row {size:?}: {src:?}: {e}"));
                    }
                }
            }
        }
        if i == 0 || i + 1 == table.rows.len() {
            let cells = table_cells(t, size, None).expect("a listed size");
            let mut manifest = t.manifest.clone();
            manifest.table = None;
            for p in crate::validate(&Template { manifest, cells }) {
                problems.push(format!("row {size:?}: {p}"));
            }
        }
    }
    problems
}
