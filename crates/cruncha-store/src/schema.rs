// SPDX-License-Identifier: GPL-3.0-or-later
//! Arrow schemas of the container's tables and conversion between rows and Rust types.

use std::sync::{Arc, LazyLock};

use std::collections::HashMap;

use deltalake::arrow::array::{
    Array, ArrayRef, BooleanArray, Int64Array, ListArray, ListBuilder, RecordBatch, StringArray,
    StringBuilder, TimestampMicrosecondArray, UInt32Array, new_null_array,
};
use deltalake::arrow::compute::{concat_batches, take_record_batch};
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

/// Revision column of the `doc` log (format 2, ADR-0010): the Delta version that wrote the row.
pub const REV_COLUMN: &str = "rev";
/// Tombstone column of the `doc` log: `true` marks the cell deleted as of `rev`.
pub const DELETED_COLUMN: &str = "deleted";

/// The columns the revision log adds to the cell columns. Nullable, so that rows written by
/// format-1 containers (one row per cell, no revisions) remain readable: a null `rev` reads as
/// -1 and a null `deleted` as `false`.
pub fn log_extra_fields() -> Vec<Field> {
    vec![
        Field::new(REV_COLUMN, DataType::Int64, true),
        Field::new(DELETED_COLUMN, DataType::Boolean, true),
    ]
}

/// Out-of-line content column (format 3, ADR-0013): the SHA-256 of a `source` stored in
/// `blobs/`. When set, `source` is null.
pub const SOURCE_HASH_COLUMN: &str = "source_hash";

/// The column format 3 adds. Last in every schema, so migrated tables (which gain it at the
/// end) and new ones have the same column order.
pub fn blob_fields() -> Vec<Field> {
    vec![Field::new(SOURCE_HASH_COLUMN, DataType::Utf8, true)]
}

/// Schema of the `doc` table: an append-only log of cell revisions (ADR-0010), with large
/// sources out of line (ADR-0013).
pub static DOC_LOG_SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| {
    let mut fields = doc_fields();
    fields.extend(log_extra_fields());
    fields.extend(blob_fields());
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

/// Pin column of the `pins` table: the name of the ref whose cells the row preserves.
pub const PIN_COLUMN: &str = "pin";

/// Schema of the `pins` table (ADR-0011): full copies of named versions whose history was
/// pruned. One row per cell per pinned ref.
pub static PINS_SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| {
    // `id` is nullable here: a ref of an empty document is pinned as a single all-null row.
    let mut fields = vec![Field::new(PIN_COLUMN, DataType::Utf8, false)];
    fields.extend(
        doc_fields()
            .into_iter()
            .map(|f| if f.name() == "id" { f.with_nullable(true) } else { f }),
    );
    fields.extend(blob_fields());
    Arc::new(Schema::new(fields))
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

/// Build revision-log rows for one commit at version `rev`. Deletes become tombstone rows
/// (`deleted = true`, other columns null). `hashes[i]`, when set, is the blob holding change
/// `i`'s source: the row then stores the hash and a null `source`.
pub fn changes_to_log_batch(
    changes: &[Change],
    hashes: &[Option<String>],
    modified_by: &str,
    now_us: i64,
    rev: i64,
) -> Result<RecordBatch> {
    assert_eq!(changes.len(), hashes.len());
    let m = changes_to_batch(changes, modified_by, now_us)?;
    let mut columns: Vec<ArrayRef> = DOC_COLUMNS
        .iter()
        .map(|c| {
            let col = m
                .column_by_name(c)
                .expect("merge batch has doc columns")
                .clone();
            if *c == "source" {
                null_where_hashed(&col, hashes)
            } else {
                col
            }
        })
        .collect();
    columns.push(Arc::new(Int64Array::from(vec![rev; changes.len()])));
    columns.push(Arc::new(
        changes
            .iter()
            .map(|c| Some(matches!(c, Change::Delete(_))))
            .collect::<BooleanArray>(),
    ));
    columns.push(Arc::new(hashes.iter().cloned().collect::<StringArray>()));
    Ok(RecordBatch::try_new(DOC_LOG_SCHEMA.clone(), columns)?)
}

fn null_where_hashed(source: &ArrayRef, hashes: &[Option<String>]) -> ArrayRef {
    let s = source
        .as_any()
        .downcast_ref::<StringArray>()
        .expect("source is Utf8");
    Arc::new(
        (0..s.len())
            .map(|i| (hashes[i].is_none() && s.is_valid(i)).then(|| s.value(i)))
            .collect::<StringArray>(),
    )
}

/// Rows of the `pins` table preserving `cells` exactly (including who modified them and when)
/// under the ref name `pin`. `hashes` as for [`changes_to_log_batch`].
pub fn cells_to_pin_batch(
    pin: &str,
    cells: &[Cell],
    hashes: &[Option<String>],
) -> Result<RecordBatch> {
    assert_eq!(cells.len(), hashes.len());
    if cells.is_empty() {
        let mut columns: Vec<ArrayRef> = vec![Arc::new(StringArray::from(vec![pin]))];
        columns.extend(
            PINS_SCHEMA.fields()[1..]
                .iter()
                .map(|f| new_null_array(f.data_type(), 1)),
        );
        return Ok(RecordBatch::try_new(PINS_SCHEMA.clone(), columns)?);
    }
    let strs = |f: &dyn Fn(&Cell) -> Option<String>| -> ArrayRef {
        Arc::new(cells.iter().map(f).collect::<StringArray>())
    };
    let columns: Vec<ArrayRef> = vec![
        Arc::new(StringArray::from(vec![pin; cells.len()])),
        strs(&|c| Some(c.id.clone())),
        strs(&|c| Some(c.kind.as_str().to_owned())),
        strs(&|c| Some(c.order_key.clone())),
        strs(&|c| c.parent_id.clone()),
        Arc::new(
            cells
                .iter()
                .zip(hashes)
                .map(|(c, h)| h.is_none().then(|| c.source.clone()))
                .collect::<StringArray>(),
        ),
        strs(&|c| Some(c.attrs.to_string())),
        Arc::new(list_array(cells.iter().map(|c| Some(c.defines.clone())))),
        Arc::new(list_array(cells.iter().map(|c| Some(c.reads.clone())))),
        strs(&|c| Some(c.provenance.to_json())),
        strs(&|c| Some(c.modified_by.clone())),
        Arc::new(
            cells
                .iter()
                .map(|c| Some(c.modified_at_us))
                .collect::<TimestampMicrosecondArray>()
                .with_timezone("UTC"),
        ),
        Arc::new(hashes.iter().cloned().collect::<StringArray>()),
    ];
    Ok(RecordBatch::try_new(PINS_SCHEMA.clone(), columns)?)
}

/// The names pinned in scanned `pins` rows, and the rows pinned under `wanted` (if given).
/// Decode the rows with [`batch_to_cells_with`].
pub fn read_pins(
    batches: &[RecordBatch],
    wanted: Option<&str>,
    rows: &mut Vec<RecordBatch>,
) -> Result<std::collections::BTreeSet<String>> {
    let mut names = std::collections::BTreeSet::new();
    for b in batches {
        let b = conform_filling(b, &PINS_SCHEMA)?;
        let pin = column::<StringArray>(&b, PIN_COLUMN)?;
        names.extend(pin.iter().flatten().map(str::to_owned));
        if let Some(w) = wanted {
            let id = column::<StringArray>(&b, "id")?;
            let mask: BooleanArray = (0..b.num_rows())
                .map(|i| Some(pin.value(i) == w && id.is_valid(i)))
                .collect();
            rows.push(deltalake::arrow::compute::filter_record_batch(&b, &mask)?);
        }
    }
    Ok(names)
}

/// Like [`conform`], but columns missing from `batch` are filled with nulls when the target
/// field is nullable (used for rows predating a schema extension).
fn conform_filling(batch: &RecordBatch, schema: &SchemaRef) -> Result<RecordBatch> {
    let columns = schema
        .fields()
        .iter()
        .map(|f| match batch.column_by_name(f.name()) {
            Some(col) if col.data_type() == f.data_type() => Ok(col.clone()),
            Some(col) => Ok(deltalake::arrow::compute::cast(col, f.data_type())?),
            None if f.is_nullable() => Ok(new_null_array(f.data_type(), batch.num_rows())),
            None => Err(Error::Corrupt(format!("column `{}` missing", f.name()))),
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(RecordBatch::try_new(schema.clone(), columns)?)
}

/// Reduce scanned revision-log rows to the document: the highest-`rev` row of each id,
/// minus tombstones. Returns rows in the log schema (order unspecified).
pub fn reduce_log(batches: &[RecordBatch]) -> Result<RecordBatch> {
    let conformed = batches
        .iter()
        .map(|b| conform_filling(b, &DOC_LOG_SCHEMA))
        .collect::<Result<Vec<_>>>()?;
    let all = concat_batches(&DOC_LOG_SCHEMA, &conformed)?;
    let ids = column::<StringArray>(&all, "id")?;
    let revs = column::<Int64Array>(&all, REV_COLUMN)?;
    let deleted = column::<BooleanArray>(&all, DELETED_COLUMN)?;
    let rev_at = |i: usize| if revs.is_valid(i) { revs.value(i) } else { -1 };

    let mut winners: HashMap<&str, usize> = HashMap::with_capacity(all.num_rows());
    for i in 0..all.num_rows() {
        let w = winners.entry(ids.value(i)).or_insert(i);
        if rev_at(i) > rev_at(*w) {
            *w = i;
        } else if *w != i && rev_at(i) == rev_at(*w) {
            return Err(Error::Corrupt(format!(
                "two rows for cell {} at rev {}",
                ids.value(i),
                rev_at(i)
            )));
        }
    }
    let mut keep: Vec<u32> = winners
        .into_values()
        .filter(|&i| !(deleted.is_valid(i) && deleted.value(i)))
        .map(|i| i as u32)
        .collect();
    keep.sort_unstable();
    Ok(take_record_batch(&all, &UInt32Array::from(keep))?)
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

/// The blob hashes referenced by `batch` (rows whose source is out of line).
pub fn blob_hashes(batch: &RecordBatch) -> Result<Vec<String>> {
    let Some(col) = batch.column_by_name(SOURCE_HASH_COLUMN) else {
        return Ok(Vec::new());
    };
    let col = deltalake::arrow::compute::cast(col, &DataType::Utf8)?;
    let col = col
        .as_any()
        .downcast_ref::<StringArray>()
        .expect("cast to Utf8");
    Ok(col.iter().flatten().map(str::to_owned).collect())
}

/// Decode `doc` table rows whose sources are all inline. Column lookup is by name, so column
/// order on disk doesn't matter.
pub fn batch_to_cells(batch: &RecordBatch, out: &mut Vec<Cell>) -> Result<()> {
    batch_to_cells_with(batch, &HashMap::new(), out)
}

/// Decode `doc` (or `pins`) rows, taking out-of-line sources from `blobs` (by hash; see
/// [`blob_hashes`]).
pub fn batch_to_cells_with(
    batch: &RecordBatch,
    blobs: &HashMap<String, Arc<str>>,
    out: &mut Vec<Cell>,
) -> Result<()> {
    let hashes = match batch.column_by_name(SOURCE_HASH_COLUMN) {
        Some(c) => Some(deltalake::arrow::compute::cast(c, &DataType::Utf8)?),
        None => None,
    };
    let hashes = hashes
        .as_ref()
        .map(|c| c.as_any().downcast_ref::<StringArray>().expect("Utf8"));
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
            source: match hashes.filter(|h| h.is_valid(i)) {
                Some(h) => blobs
                    .get(h.value(i))
                    .ok_or_else(|| Error::Corrupt(format!("blob {} not loaded", h.value(i))))?
                    .to_string(),
                None => source.value(i).to_owned(),
            },
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
