// SPDX-License-Identifier: GPL-3.0-or-later
//! Code-cell results (ADR-0018): the `results/` table and its records.
//!
//! The table is append-only and created by the first stored run. Each row is one finished run
//! of a code cell, identified by `(cell_id, input_hash)`: the hash of everything that produced
//! it (computed by the engine, not here). The same key at any document version means the same
//! inputs, so a row applies wherever its key matches. Results are a cache: dropping them never
//! loses an edit, so they never touch `doc/` and never count as edits for retention.
//!
//! Large strings in `values` and `outputs` (images as base64, figure specs) are moved to
//! `blobs/` and replaced by `{"$blob": "<hash>"}`; a large JSON document as a whole by
//! `{"$json": "<hash>"}`. Blobs are written before the row that references them, and each row
//! lists its blobs in the `blobs` column so [`crate::Store::prune`] keeps them.

use std::sync::{Arc, LazyLock};

use deltalake::arrow::array::{Array, ArrayRef, Int64Array, RecordBatch, StringArray};
use deltalake::arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::error::{Error, Result};
use crate::schema::conform;

/// The column listing a row's blobs (space-separated hashes).
pub(crate) const BLOBS_COLUMN: &str = "blobs";
/// Strings at least this long go to `blobs/`.
pub(crate) const BLOB_STRING_BYTES: usize = 4 << 10;
/// A `values` or `outputs` document still this long after its strings moved out goes whole.
pub(crate) const BLOB_JSON_BYTES: usize = 256 << 10;
/// `stdout`, `stderr` and `error` are cut to this many bytes.
pub const MAX_TEXT_BYTES: usize = 1 << 20;
/// A run whose values, outputs and text add up to more than this isn't stored (ADR-0018).
pub const MAX_RUN_BYTES: usize = 64 << 20;

/// How a run ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RunStatus {
    Done,
    Error,
    Interrupted,
}

impl RunStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            RunStatus::Done => "done",
            RunStatus::Error => "error",
            RunStatus::Interrupted => "interrupted",
        }
    }

    pub fn parse(s: &str) -> Option<RunStatus> {
        [RunStatus::Done, RunStatus::Error, RunStatus::Interrupted]
            .into_iter()
            .find(|k| k.as_str() == s)
    }
}

/// One stored run of a code cell.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunRecord {
    pub cell_id: String,
    /// The engine's hash of what produced the run (hex SHA-256).
    pub input_hash: String,
    pub status: RunStatus,
    /// Each defined name's value and summary (the engine's format).
    pub values: Value,
    /// The output list (text, tables, images, figure specs).
    pub outputs: Value,
    pub stdout: String,
    pub stderr: String,
    /// The error, if any (the engine's format, as text).
    pub error: Option<String>,
    pub duration_ms: u64,
    /// The environment label (`env-…`: interpreter and package lockfile).
    pub env: String,
    /// Cruncha's clock.
    pub computed_at_us: i64,
    /// `user:<name>@<host>`.
    pub computed_by: String,
    /// The latest document version when it ran.
    pub doc_version: u64,
}

impl RunRecord {
    pub fn key(&self) -> (String, String) {
        (self.cell_id.clone(), self.input_hash.clone())
    }
}

/// A stored run without its values and outputs (cheap to list).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunEntry {
    pub cell_id: String,
    pub input_hash: String,
    pub status: RunStatus,
    pub env: String,
    pub computed_at_us: i64,
    pub computed_by: String,
    pub doc_version: u64,
}

/// What [`crate::Store::put_results`] did.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PutReport {
    pub written: usize,
    /// Cells whose run was larger than [`MAX_RUN_BYTES`]: shown this session only.
    pub too_large: Vec<String>,
    /// Bytes of new blobs.
    pub blob_bytes: u64,
}

/// Schema of the `results` table.
pub static RESULTS_SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| {
    Arc::new(Schema::new(vec![
        Field::new("cell_id", DataType::Utf8, false),
        Field::new("input_hash", DataType::Utf8, false),
        Field::new("status", DataType::Utf8, false),
        Field::new("values", DataType::Utf8, true),
        Field::new("outputs", DataType::Utf8, true),
        Field::new("stdout", DataType::Utf8, true),
        Field::new("stderr", DataType::Utf8, true),
        Field::new("error", DataType::Utf8, true),
        Field::new("duration_ms", DataType::Int64, false),
        Field::new("env", DataType::Utf8, false),
        Field::new("computed_at_us", DataType::Int64, false),
        Field::new("computed_by", DataType::Utf8, false),
        Field::new("doc_version", DataType::Int64, false),
        Field::new(BLOBS_COLUMN, DataType::Utf8, true),
    ]))
});

/// One row as stored: `values` and `outputs` with their blob references unresolved.
#[derive(Debug, Clone)]
pub(crate) struct Row {
    pub entry: RunEntry,
    pub values: String,
    pub outputs: String,
    pub stdout: String,
    pub stderr: String,
    pub error: Option<String>,
    pub duration_ms: u64,
    pub blobs: Vec<String>,
}

/// Cut `s` to at most `max` bytes on a character boundary, saying so.
pub(crate) fn cap(mut s: String, max: usize) -> String {
    if s.len() <= max {
        return s;
    }
    let mut end = max;
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    s.truncate(end);
    s.push_str("\n… (cut: more than 1 MB)");
    s
}

/// The approximate stored size of a run, for [`MAX_RUN_BYTES`].
pub(crate) fn run_bytes(r: &RunRecord) -> usize {
    fn strings(v: &Value) -> usize {
        match v {
            Value::String(s) => s.len(),
            Value::Array(a) => a.iter().map(strings).sum::<usize>() + a.len(),
            Value::Object(o) => o.iter().map(|(k, v)| k.len() + strings(v)).sum(),
            _ => 8,
        }
    }
    strings(&r.values)
        + strings(&r.outputs)
        + r.stdout.len().min(MAX_TEXT_BYTES)
        + r.stderr.len().min(MAX_TEXT_BYTES)
        + r.error.as_ref().map_or(0, |e| e.len().min(MAX_TEXT_BYTES))
}

/// Replace long strings in `v` by blob references, collecting the strings to store.
pub(crate) fn take_strings(v: Value, out: &mut Vec<String>) -> Value {
    match v {
        Value::String(s) if s.len() >= BLOB_STRING_BYTES => {
            let h = crate::blobs::hash(&s);
            out.push(s);
            json!({ "$blob": h })
        }
        Value::Array(a) => Value::Array(a.into_iter().map(|x| take_strings(x, out)).collect()),
        Value::Object(o) => Value::Object(o.into_iter().map(|(k, x)| (k, take_strings(x, out))).collect()),
        v => v,
    }
}

/// `v` as stored: long strings out, and the whole document out if it's still long.
pub(crate) fn externalize(v: Value, out: &mut Vec<String>) -> Result<String> {
    let text = serde_json::to_string(&take_strings(v, out))?;
    if text.len() < BLOB_JSON_BYTES {
        return Ok(text);
    }
    let h = crate::blobs::hash(&text);
    out.push(text);
    Ok(json!({ "$json": h }).to_string())
}

/// The single key and hash of a blob reference (`$blob` or `$json`), if `o` is one.
fn reference(o: &Map<String, Value>) -> Option<(&str, &str)> {
    if o.len() != 1 {
        return None;
    }
    let (k, v) = o.iter().next()?;
    match (k.as_str(), v.as_str()) {
        (k @ ("$blob" | "$json"), Some(h)) => Some((k, h)),
        _ => None,
    }
}

/// Every blob hash referenced in a stored document.
#[cfg(test)]
pub(crate) fn references(v: &Value, out: &mut std::collections::BTreeSet<String>) {
    match v {
        Value::Array(a) => a.iter().for_each(|x| references(x, out)),
        Value::Object(o) => match reference(o) {
            Some((_, h)) => {
                out.insert(h.to_owned());
            }
            None => o.values().for_each(|x| references(x, out)),
        },
        _ => {}
    }
}

/// Put the blobs back into a stored document. `get` loads one blob's text.
pub(crate) fn internalize(v: Value, get: &dyn Fn(&str) -> Result<Arc<str>>) -> Result<Value> {
    Ok(match v {
        Value::Array(a) => Value::Array(a.into_iter().map(|x| internalize(x, get)).collect::<Result<_>>()?),
        Value::Object(o) => match reference(&o) {
            Some(("$blob", h)) => Value::String(get(h)?.to_string()),
            Some((_, h)) => internalize(serde_json::from_str(&get(h)?)?, get)?,
            None => Value::Object(
                o.into_iter()
                    .map(|(k, x)| Ok((k, internalize(x, get)?)))
                    .collect::<Result<_>>()?,
            ),
        },
        v => v,
    })
}

pub(crate) fn rows_to_batch(rows: &[Row]) -> Result<RecordBatch> {
    let s = |f: &dyn Fn(&Row) -> Option<&str>| -> ArrayRef { Arc::new(rows.iter().map(f).collect::<StringArray>()) };
    let i = |f: &dyn Fn(&Row) -> i64| -> ArrayRef { Arc::new(rows.iter().map(|r| Some(f(r))).collect::<Int64Array>()) };
    let blobs: Vec<Option<String>> = rows
        .iter()
        .map(|r| (!r.blobs.is_empty()).then(|| r.blobs.join(" ")))
        .collect();
    let columns = vec![
        s(&|r| Some(r.entry.cell_id.as_str())),
        s(&|r| Some(r.entry.input_hash.as_str())),
        s(&|r| Some(r.entry.status.as_str())),
        s(&|r| Some(r.values.as_str())),
        s(&|r| Some(r.outputs.as_str())),
        s(&|r| Some(r.stdout.as_str())),
        s(&|r| Some(r.stderr.as_str())),
        s(&|r| r.error.as_deref()),
        i(&|r| r.duration_ms as i64),
        s(&|r| Some(r.entry.env.as_str())),
        i(&|r| r.entry.computed_at_us),
        s(&|r| Some(r.entry.computed_by.as_str())),
        i(&|r| r.entry.doc_version as i64),
        Arc::new(blobs.iter().map(|b| b.as_deref()).collect::<StringArray>()),
    ];
    Ok(RecordBatch::try_new(RESULTS_SCHEMA.clone(), columns)?)
}

fn col<'a, T: 'static>(b: &'a RecordBatch, name: &str) -> Result<&'a T> {
    b.column_by_name(name)
        .and_then(|c| c.as_any().downcast_ref::<T>())
        .ok_or_else(|| Error::Corrupt(format!("results column `{name}` missing or mistyped")))
}

pub(crate) fn batches_to_rows(batches: &[RecordBatch]) -> Result<Vec<Row>> {
    let mut out = Vec::new();
    for b in batches {
        let b = conform(b, &RESULTS_SCHEMA)?;
        let cell = col::<StringArray>(&b, "cell_id")?;
        let hash = col::<StringArray>(&b, "input_hash")?;
        let status = col::<StringArray>(&b, "status")?;
        let values = col::<StringArray>(&b, "values")?;
        let outputs = col::<StringArray>(&b, "outputs")?;
        let stdout = col::<StringArray>(&b, "stdout")?;
        let stderr = col::<StringArray>(&b, "stderr")?;
        let error = col::<StringArray>(&b, "error")?;
        let ms = col::<Int64Array>(&b, "duration_ms")?;
        let env = col::<StringArray>(&b, "env")?;
        let at = col::<Int64Array>(&b, "computed_at_us")?;
        let by = col::<StringArray>(&b, "computed_by")?;
        let ver = col::<Int64Array>(&b, "doc_version")?;
        let blobs = col::<StringArray>(&b, BLOBS_COLUMN)?;
        let s = |a: &StringArray, i: usize| a.is_valid(i).then(|| a.value(i).to_owned());
        for i in 0..b.num_rows() {
            out.push(Row {
                entry: RunEntry {
                    cell_id: cell.value(i).to_owned(),
                    input_hash: hash.value(i).to_owned(),
                    status: RunStatus::parse(status.value(i))
                        .ok_or_else(|| Error::Corrupt(format!("unknown run status {}", status.value(i))))?,
                    env: env.value(i).to_owned(),
                    computed_at_us: at.value(i),
                    computed_by: by.value(i).to_owned(),
                    doc_version: ver.value(i) as u64,
                },
                values: s(values, i).unwrap_or_else(|| "null".into()),
                outputs: s(outputs, i).unwrap_or_else(|| "[]".into()),
                stdout: s(stdout, i).unwrap_or_default(),
                stderr: s(stderr, i).unwrap_or_default(),
                error: s(error, i),
                duration_ms: ms.value(i).max(0) as u64,
                blobs: s(blobs, i)
                    .map(|b| b.split_whitespace().map(str::to_owned).collect())
                    .unwrap_or_default(),
            });
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn long_strings_and_documents_round_trip_through_blobs() {
        let png = "A".repeat(BLOB_STRING_BYTES + 10);
        let outputs = json!([{ "kind": "image", "data": png, "width": 4 }, { "kind": "text", "text": "short" }]);
        let mut stored = Vec::new();
        let text = externalize(outputs.clone(), &mut stored).unwrap();
        assert_eq!(stored, vec![png.clone()]);
        let v: Value = serde_json::from_str(&text).unwrap();
        let mut refs = std::collections::BTreeSet::new();
        references(&v, &mut refs);
        assert_eq!(refs.into_iter().collect::<Vec<_>>(), vec![crate::blobs::hash(&png)]);

        // A big document of small strings goes out whole.
        let many = Value::Array((0..40_000).map(|i| json!(format!("x{i}"))).collect());
        let mut stored2 = Vec::new();
        let text2 = externalize(many.clone(), &mut stored2).unwrap();
        assert!(text2.starts_with("{\"$json\""));

        let all: std::collections::HashMap<String, Arc<str>> = stored
            .into_iter()
            .chain(stored2)
            .map(|s| (crate::blobs::hash(&s), Arc::from(s)))
            .collect();
        let get = |h: &str| all.get(h).cloned().ok_or_else(|| Error::Corrupt(h.into()));
        assert_eq!(internalize(v, &get).unwrap(), outputs);
        assert_eq!(internalize(serde_json::from_str(&text2).unwrap(), &get).unwrap(), many);
    }

    #[test]
    fn text_is_capped_on_a_char_boundary() {
        let s = "é".repeat(MAX_TEXT_BYTES); // 2 bytes each
        let c = cap(s, MAX_TEXT_BYTES + 1);
        assert!(c.ends_with("(cut: more than 1 MB)"));
        assert!(c.len() < MAX_TEXT_BYTES + 40);
    }
}
