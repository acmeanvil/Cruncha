// SPDX-License-Identifier: GPL-3.0-or-later
//! Arrow schemas of the container's tables and conversion between rows and Rust types.

use std::sync::{Arc, LazyLock};

use deltalake::arrow::array::{
    Array, ArrayRef, Int64Array, ListArray, ListBuilder, RecordBatch, StringArray, StringBuilder,
    TimestampMicrosecondArray,
};
use deltalake::arrow::datatypes::{DataType, Field, Schema, SchemaRef, TimeUnit};

use crate::cell::{Cell, CellKind, Change, Provenance};
use crate::error::{Error, Result};

/// Extra column on merge sources: `"upsert"` or `"delete"`.
pub const OP_COLUMN: &str = "_op";
pub const OP_UPSERT: &str = "upsert";
pub const OP_DELETE: &str = "delete";

/// Columns of the `doc` table, in order (excluding the merge-only `_op` column).
pub const DOC_COLUMNS: [&str; 11] = [
    "id",
    "kind",
    "order_key",
    "parent_id",
    "source",
    "attrs",
    "defines",
    "reads",
    "provenance",
    "modified_by",
    "modified_at",
];

fn utc_us() -> DataType {
    DataType::Timestamp(TimeUnit::Microsecond, Some("UTC".into()))
}

fn str_list() -> DataType {
    DataType::List(Arc::new(Field::new("item", DataType::Utf8, true)))
}

fn doc_fields() -> Vec<Field> {
    vec![
        Field::new("id", DataType::Utf8, false),
        Field::new("kind", DataType::Utf8, true),
        Field::new("order_key", DataType::Utf8, true),
        Field::new("parent_id", DataType::Utf8, true),
        Field::new("source", DataType::Utf8, true),
        Field::new("attrs", DataType::Utf8, true),
        Field::new("defines", str_list(), true),
        Field::new("reads", str_list(), true),
        Field::new("provenance", DataType::Utf8, true),
        Field::new("modified_by", DataType::Utf8, true),
        Field::new("modified_at", utc_us(), true),
    ]
}

/// Schema of the `doc` table.
pub static DOC_SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| Arc::new(Schema::new(doc_fields())));

/// Schema of a merge source: the `doc` columns plus `_op`.
pub static DOC_MERGE_SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| {
    let mut fields = doc_fields();
    fields.push(Field::new(OP_COLUMN, DataType::Utf8, false));
    Arc::new(Schema::new(fields))
});

/// Schema of the `refs` table (named versions).
pub static REFS_SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| {
    Arc::new(Schema::new(vec![
        Field::new("name", DataType::Utf8, false),
        Field::new("doc_version", DataType::Int64, false),
        Field::new("message", DataType::Utf8, true),
        Field::new("actor", DataType::Utf8, true),
        Field::new("created_at", utc_us(), true),
    ]))
});

fn list_array(rows: impl Iterator<Item = Option<Vec<String>>>) -> ListArray {
    let mut b = ListBuilder::new(StringBuilder::new());
    for row in rows {
        match row {
            Some(items) => {
                for s in items {
                    b.values().append_value(s);
                }
                b.append(true);
            }
            None => b.append(false),
        }
    }
    b.finish()
}

/// Build a merge-source batch from a set of changes.
///
/// Delete rows carry only `id` and `_op`; their other columns are null and never written.
pub fn changes_to_batch(changes: &[Change], modified_by: &str, now_us: i64) -> Result<RecordBatch> {
    let ups = |f: &dyn Fn(&Cell) -> Option<String>| -> StringArray {
        changes
            .iter()
            .map(|c| match c {
                Change::Upsert(cell) => f(cell),
                Change::Delete(_) => None,
            })
            .collect()
    };
    let lists = |f: &dyn Fn(&Cell) -> Vec<String>| -> ListArray {
        list_array(changes.iter().map(|c| match c {
            Change::Upsert(cell) => Some(f(cell)),
            Change::Delete(_) => None,
        }))
    };

    let id: StringArray = changes.iter().map(|c| Some(c.id().to_owned())).collect();
    let modified_by_col: StringArray = changes
        .iter()
        .map(|c| matches!(c, Change::Upsert(_)).then(|| modified_by.to_owned()))
        .collect();
    let modified_at: TimestampMicrosecondArray = changes
        .iter()
        .map(|c| matches!(c, Change::Upsert(_)).then_some(now_us))
        .collect::<TimestampMicrosecondArray>()
        .with_timezone("UTC");
    let op: StringArray = changes
        .iter()
        .map(|c| {
            Some(match c {
                Change::Upsert(_) => OP_UPSERT,
                Change::Delete(_) => OP_DELETE,
            })
        })
        .collect();

    let columns: Vec<ArrayRef> = vec![
        Arc::new(id),
        Arc::new(ups(&|c| Some(c.kind.as_str().to_owned()))),
        Arc::new(ups(&|c| Some(c.order_key.clone()))),
        Arc::new(ups(&|c| c.parent_id.clone())),
        Arc::new(ups(&|c| Some(c.source.clone()))),
        Arc::new(ups(&|c| Some(c.attrs.to_string()))),
        Arc::new(lists(&|c| c.defines.clone())),
        Arc::new(lists(&|c| c.reads.clone())),
        Arc::new(ups(&|c| Some(c.provenance.to_json()))),
        Arc::new(modified_by_col),
        Arc::new(modified_at),
        Arc::new(op),
    ];
    Ok(RecordBatch::try_new(DOC_MERGE_SCHEMA.clone(), columns)?)
}

/// Cast a scanned batch to `schema` (by column name). Scans may return equivalent but
/// different physical types than were written (e.g. DataFusion's `Utf8View` for `Utf8`).
pub fn conform(batch: &RecordBatch, schema: &SchemaRef) -> Result<RecordBatch> {
    let columns = schema
        .fields()
        .iter()
        .map(|f| {
            let col = batch
                .column_by_name(f.name())
                .ok_or_else(|| Error::Corrupt(format!("column `{}` missing", f.name())))?;
            Ok(if col.data_type() == f.data_type() {
                col.clone()
            } else {
                deltalake::arrow::compute::cast(col, f.data_type())?
            })
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(RecordBatch::try_new(schema.clone(), columns)?)
}

fn column<'a, T: 'static>(batch: &'a RecordBatch, name: &str) -> Result<&'a T> {
    batch
        .column_by_name(name)
        .and_then(|c| c.as_any().downcast_ref::<T>())
        .ok_or_else(|| Error::Corrupt(format!("doc table column `{name}` missing or mistyped")))
}

fn opt_str(a: &StringArray, i: usize) -> Option<String> {
    a.is_valid(i).then(|| a.value(i).to_owned())
}

fn str_vec(a: &ListArray, i: usize) -> Result<Vec<String>> {
    if a.is_null(i) {
        return Ok(Vec::new());
    }
    let values = a.value(i);
    let values = values
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| Error::Corrupt("list column is not a list of strings".into()))?;
    Ok(values.iter().flatten().map(str::to_owned).collect())
}

/// Decode `doc` table rows. Column lookup is by name, so column order on disk doesn't matter.
pub fn batch_to_cells(batch: &RecordBatch, out: &mut Vec<Cell>) -> Result<()> {
    let batch = &conform(batch, &DOC_SCHEMA)?;
    let id = column::<StringArray>(batch, "id")?;
    let kind = column::<StringArray>(batch, "kind")?;
    let order_key = column::<StringArray>(batch, "order_key")?;
    let parent_id = column::<StringArray>(batch, "parent_id")?;
    let source = column::<StringArray>(batch, "source")?;
    let attrs = column::<StringArray>(batch, "attrs")?;
    let defines = column::<ListArray>(batch, "defines")?;
    let reads = column::<ListArray>(batch, "reads")?;
    let provenance = column::<StringArray>(batch, "provenance")?;
    let modified_by = column::<StringArray>(batch, "modified_by")?;
    let modified_at = column::<TimestampMicrosecondArray>(batch, "modified_at")?;

    for i in 0..batch.num_rows() {
        let kind_s = kind.value(i);
        let kind = CellKind::parse(kind_s)
            .ok_or_else(|| Error::Corrupt(format!("unknown cell kind `{kind_s}`")))?;
        out.push(Cell {
            id: id.value(i).to_owned(),
            kind,
            order_key: order_key.value(i).to_owned(),
            parent_id: opt_str(parent_id, i),
            source: source.value(i).to_owned(),
            attrs: serde_json::from_str(attrs.value(i))?,
            defines: str_vec(defines, i)?,
            reads: str_vec(reads, i)?,
            provenance: Provenance::from_json(provenance.value(i))?,
            modified_by: modified_by.value(i).to_owned(),
            modified_at_us: if modified_at.is_valid(i) {
                modified_at.value(i)
            } else {
                0
            },
        });
    }
    Ok(())
}

/// A named version (tag).
#[derive(Debug, Clone, PartialEq)]
pub struct Ref {
    pub name: String,
    pub doc_version: u64,
    pub message: String,
    pub actor: String,
    pub created_at_us: i64,
}

pub fn ref_to_batch(r: &Ref) -> Result<RecordBatch> {
    let columns: Vec<ArrayRef> = vec![
        Arc::new(StringArray::from(vec![r.name.clone()])),
        Arc::new(Int64Array::from(vec![r.doc_version as i64])),
        Arc::new(StringArray::from(vec![r.message.clone()])),
        Arc::new(StringArray::from(vec![r.actor.clone()])),
        Arc::new(TimestampMicrosecondArray::from(vec![r.created_at_us]).with_timezone("UTC")),
    ];
    Ok(RecordBatch::try_new(REFS_SCHEMA.clone(), columns)?)
}

pub fn batch_to_refs(batch: &RecordBatch, out: &mut Vec<Ref>) -> Result<()> {
    let batch = &conform(batch, &REFS_SCHEMA)?;
    let name = column::<StringArray>(batch, "name")?;
    let doc_version = column::<Int64Array>(batch, "doc_version")?;
    let message = column::<StringArray>(batch, "message")?;
    let actor = column::<StringArray>(batch, "actor")?;
    let created_at = column::<TimestampMicrosecondArray>(batch, "created_at")?;
    for i in 0..batch.num_rows() {
        out.push(Ref {
            name: name.value(i).to_owned(),
            doc_version: doc_version.value(i) as u64,
            message: opt_str(message, i).unwrap_or_default(),
            actor: opt_str(actor, i).unwrap_or_default(),
            created_at_us: if created_at.is_valid(i) {
                created_at.value(i)
            } else {
                0
            },
        });
    }
    Ok(())
}
