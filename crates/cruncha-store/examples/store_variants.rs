// SPDX-License-Identifier: GPL-3.0-or-later
//! Spike S3b: compare `doc` table layouts on an identical, seeded editing workload.
//!
//! - `merge`    — the current `Store`: one row per cell, copy-on-write MERGE per commit.
//! - `append`   — option A: append-only log of cell revisions (`rev`, `deleted`), reduced to the
//!   latest row per id on read; squashed (overwritten with current rows) every N commits.
//! - `bucketed` — option B: one row per cell, partitioned by `hash(id) % 16`, MERGE with the
//!   bucket in the predicate so only touched partitions are rewritten.
//!
//! ```sh
//! cargo run -p cruncha-store --release --example store_variants -- <merge|append|bucketed|all> \
//!     [commits=1000] [cells=2000] [squash_every=500]
//! ```

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use cruncha_store::schema::{self, DOC_COLUMNS, DOC_SCHEMA, OP_COLUMN, OP_DELETE, OP_UPSERT};
use cruncha_store::{Cell, CellKind, Change, Store};
use deltalake::DeltaTable;
use deltalake::arrow::array::{
    Array, ArrayRef, BooleanArray, Int32Array, Int64Array, RecordBatch, StringArray, UInt32Array,
};
use deltalake::arrow::compute::{concat_batches, take_record_batch};
use deltalake::arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use deltalake::datafusion::prelude::{SessionContext, col, lit};
use deltalake::kernel::StructType;
use deltalake::kernel::engine::arrow_conversion::TryIntoKernel as _;
use deltalake::protocol::SaveMode;
use futures::TryStreamExt;
use url::Url;

type Res<T> = Result<T, Box<dyn std::error::Error>>;

const BUCKETS: u64 = 16;
const PROPS: [(&str, &str); 3] = [
    ("delta.enableExpiredLogCleanup", "false"),
    ("delta.logRetentionDuration", "interval 36500 days"),
    ("delta.deletedFileRetentionDuration", "interval 36500 days"),
];

// ---------------------------------------------------------------- workload

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

fn source_for(i: usize, rev: usize) -> String {
    match i % 5 {
        0 => format!(
            "Paragraph {i} rev {rev}: {}",
            "lorem ipsum dolor sit amet ".repeat(12)
        ),
        1 => format!("import polars as pl\ndf_{i} = pl.DataFrame({{'a': range({rev})}})\n"),
        _ => format!("M_{i} = w L^2 / 8 + {rev} = ?? kip·ft"),
    }
}

/// Deterministic ULID-like ids so every backend sees byte-identical data.
fn id_for(n: usize) -> String {
    format!("01JWORKLOAD{n:015}")
}

fn initial_cells(n: usize) -> Vec<Cell> {
    (0..n)
        .map(|i| {
            let kind = [
                CellKind::Prose,
                CellKind::Python,
                CellKind::Calc,
                CellKind::Calc,
                CellKind::Calc,
            ][i % 5];
            let mut c = Cell::new(kind, format!("a{i:06}"), source_for(i, 0));
            c.id = id_for(i);
            c
        })
        .collect()
}

/// The seeded sequence of change sets (90% edit, 7% insert, 3% delete; 1–3 cells each).
fn workload(cells: &mut Vec<Cell>, commits: usize) -> Vec<Vec<Change>> {
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    let mut next = cells.len();
    let mut out = Vec::with_capacity(commits);
    for rev in 1..=commits {
        let mut changes = Vec::new();
        let mut touched = std::collections::HashSet::new();
        for _ in 0..1 + rng.below(3) {
            let roll = rng.below(100);
            if roll < 7 {
                let mut c = Cell::new(CellKind::Calc, format!("a{next:06}"), source_for(next, rev));
                c.id = id_for(next);
                next += 1;
                touched.insert(c.id.clone());
                cells.push(c.clone());
                changes.push(Change::Upsert(c));
            } else if roll < 10 {
                let i = rng.below(cells.len());
                if touched.insert(cells[i].id.clone()) {
                    changes.push(Change::Delete(cells.swap_remove(i).id));
                }
            } else {
                let i = rng.below(cells.len());
                if touched.insert(cells[i].id.clone()) {
                    cells[i].source = source_for(i, rev);
                    changes.push(Change::Upsert(cells[i].clone()));
                }
            }
        }
        out.push(changes);
    }
    out
}

// ---------------------------------------------------------------- helpers

fn now_us() -> i64 {
    chrono::Utc::now().timestamp_micros()
}

fn dir_stats(p: &Path) -> (u64, u64) {
    let (mut files, mut bytes) = (0, 0);
    for e in std::fs::read_dir(p).unwrap().flatten() {
        let m = e.metadata().unwrap();
        if m.is_dir() {
            let (f, b) = dir_stats(&e.path());
            files += f;
            bytes += b;
        } else {
            files += 1;
            bytes += m.len();
        }
    }
    (files, bytes)
}

fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1e3
}

fn pct(sorted: &[Duration], p: f64) -> Duration {
    sorted[((sorted.len() - 1) as f64 * p).round() as usize]
}

fn fnv_bucket(id: &str) -> i32 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in id.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    (h % BUCKETS) as i32
}

fn with_columns(base: &SchemaRef, extra: Vec<Field>) -> SchemaRef {
    let mut f: Vec<Field> = base.fields().iter().map(|f| f.as_ref().clone()).collect();
    f.extend(extra);
    Arc::new(Schema::new(f))
}

async fn create(path: &Path, schema: &Schema, partitions: &[&str]) -> Res<DeltaTable> {
    std::fs::create_dir_all(path)?;
    let kernel: StructType = schema.try_into_kernel()?;
    let url = Url::from_directory_path(path.canonicalize()?).unwrap();
    let t = DeltaTable::try_from_url(url).await?;
    let t = t
        .create()
        .with_columns(kernel.fields().cloned())
        .with_partition_columns(partitions.iter().map(|s| s.to_string()))
        .with_configuration(PROPS.iter().map(|(k, v)| (*k, Some(*v))))
        .with_configuration(stats_cols().map(|n| ("delta.dataSkippingNumIndexedCols", Some(n))))
        .await?;
    Ok(t)
}

/// `CRUNCHA_BENCH_STATS_COLS=n` limits per-file column statistics to the first n columns.
fn stats_cols() -> Option<String> {
    std::env::var("CRUNCHA_BENCH_STATS_COLS").ok()
}

async fn scan(t: &DeltaTable) -> Res<Vec<RecordBatch>> {
    let (_, s) = t.clone().scan_table().await?;
    Ok(s.try_collect().await?)
}

fn decode(batches: &[RecordBatch]) -> Res<Vec<Cell>> {
    let mut cells = Vec::new();
    for b in batches {
        schema::batch_to_cells(b, &mut cells)?;
    }
    cells.sort_by(|a, b| a.order_key.cmp(&b.order_key).then_with(|| a.id.cmp(&b.id)));
    Ok(cells)
}

// ---------------------------------------------------------------- option A: append-only log

static LOG_SCHEMA: std::sync::LazyLock<SchemaRef> = std::sync::LazyLock::new(|| {
    with_columns(
        &DOC_SCHEMA,
        vec![
            Field::new("rev", DataType::Int64, false),
            Field::new("deleted", DataType::Boolean, false),
        ],
    )
});

/// Merge-source batch → log rows (`rev`, `deleted`).
fn to_log_batch(changes: &[Change], rev: i64) -> Res<RecordBatch> {
    let m = schema::changes_to_batch(changes, "user:bench", now_us())?;
    let op = m
        .column_by_name(OP_COLUMN)
        .unwrap()
        .as_any()
        .downcast_ref::<StringArray>()
        .unwrap();
    let mut cols: Vec<ArrayRef> = DOC_COLUMNS
        .iter()
        .map(|c| m.column_by_name(c).unwrap().clone())
        .collect();
    cols.push(Arc::new(Int64Array::from(vec![rev; m.num_rows()])));
    cols.push(Arc::new(BooleanArray::from_iter(
        op.iter().map(|o| Some(o == Some(OP_DELETE))),
    )));
    Ok(RecordBatch::try_new(LOG_SCHEMA.clone(), cols)?)
}

/// Latest non-deleted row per id (log schema). Ties cannot happen: one row per id per rev.
fn reduce_log(batches: &[RecordBatch]) -> Res<RecordBatch> {
    let conformed = batches
        .iter()
        .map(|b| schema::conform(b, &LOG_SCHEMA))
        .collect::<Result<Vec<_>, _>>()?;
    let all = concat_batches(&LOG_SCHEMA, &conformed)?;
    let ids = all
        .column_by_name("id")
        .unwrap()
        .as_any()
        .downcast_ref::<StringArray>()
        .unwrap();
    let revs = all
        .column_by_name("rev")
        .unwrap()
        .as_any()
        .downcast_ref::<Int64Array>()
        .unwrap();
    let del = all
        .column_by_name("deleted")
        .unwrap()
        .as_any()
        .downcast_ref::<BooleanArray>()
        .unwrap();
    let mut win: HashMap<&str, (i64, u32)> = HashMap::with_capacity(all.num_rows());
    for i in 0..all.num_rows() {
        let e = win.entry(ids.value(i)).or_insert((i64::MIN, 0));
        if revs.value(i) > e.0 {
            *e = (revs.value(i), i as u32);
        }
    }
    let mut idx: Vec<u32> = win
        .into_values()
        .filter(|(_, i)| !del.value(*i as usize))
        .map(|(_, i)| i)
        .collect();
    idx.sort_unstable();
    Ok(take_record_batch(&all, &UInt32Array::from(idx))?)
}

// ---------------------------------------------------------------- option B: bucketed merge

static BUCKET_MERGE_SCHEMA: std::sync::LazyLock<SchemaRef> = std::sync::LazyLock::new(|| {
    with_columns(
        &schema::DOC_MERGE_SCHEMA,
        vec![Field::new("bucket", DataType::Int32, false)],
    )
});
static BUCKET_SCHEMA: std::sync::LazyLock<SchemaRef> = std::sync::LazyLock::new(|| {
    with_columns(
        &DOC_SCHEMA,
        vec![Field::new("bucket", DataType::Int32, false)],
    )
});

fn to_bucket_batch(changes: &[Change]) -> Res<RecordBatch> {
    let m = schema::changes_to_batch(changes, "user:bench", now_us())?;
    let mut cols: Vec<ArrayRef> = m.columns().to_vec();
    cols.push(Arc::new(Int32Array::from_iter_values(
        changes.iter().map(|c| fnv_bucket(c.id())),
    )));
    Ok(RecordBatch::try_new(BUCKET_MERGE_SCHEMA.clone(), cols)?)
}

// ---------------------------------------------------------------- backends

enum Backend {
    Merge(Store),
    Append {
        table: DeltaTable,
        squash_every: usize,
        since_squash: usize,
        squashes: Vec<Duration>,
    },
    Bucketed {
        table: DeltaTable,
        ctx: SessionContext,
    },
}

impl Backend {
    async fn create(kind: &str, path: &Path, squash_every: usize) -> Res<Backend> {
        Ok(match kind {
            "merge" => Backend::Merge(Store::create(path, "user:bench").await?),
            "append" => Backend::Append {
                table: create(&path.join("doc"), &LOG_SCHEMA, &[]).await?,
                squash_every,
                since_squash: 0,
                squashes: Vec::new(),
            },
            "bucketed" => Backend::Bucketed {
                table: create(&path.join("doc"), &BUCKET_SCHEMA, &["bucket"]).await?,
                ctx: SessionContext::new(),
            },
            _ => return Err(format!("unknown backend {kind}").into()),
        })
    }

    async fn reopen(kind: &str, path: &Path, squash_every: usize) -> Res<Backend> {
        let open = || async {
            let url = Url::from_directory_path(path.join("doc").canonicalize()?).unwrap();
            Res::Ok(deltalake::open_table(url).await?)
        };
        Ok(match kind {
            "merge" => Backend::Merge(Store::open(path).await?),
            "append" => Backend::Append {
                table: open().await?,
                squash_every,
                since_squash: 0,
                squashes: Vec::new(),
            },
            _ => Backend::Bucketed {
                table: open().await?,
                ctx: SessionContext::new(),
            },
        })
    }

    fn version(&self) -> u64 {
        match self {
            Backend::Merge(s) => s.version(),
            Backend::Append { table, .. } | Backend::Bucketed { table, .. } => {
                table.version().unwrap_or(0)
            }
        }
    }

    async fn commit(&mut self, changes: &[Change]) -> Res<()> {
        match self {
            Backend::Merge(s) => {
                s.commit(changes, "user:bench", "edit").await?;
            }
            Backend::Append {
                table,
                squash_every,
                since_squash,
                squashes,
            } => {
                let rev = table.version().unwrap_or(0) as i64 + 1;
                let t = table
                    .clone()
                    .write([to_log_batch(changes, rev)?])
                    .with_save_mode(SaveMode::Append)
                    .await?;
                assert_eq!(t.version(), Some(rev as u64), "single-writer rev invariant");
                *table = t;
                *since_squash += 1;
                if *since_squash >= *squash_every {
                    let t0 = Instant::now();
                    let current = reduce_log(&scan(table).await?)?;
                    let t = table
                        .clone()
                        .write([current])
                        .with_save_mode(SaveMode::Overwrite)
                        .await?;
                    *table = t;
                    *since_squash = 0;
                    squashes.push(t0.elapsed());
                }
            }
            Backend::Bucketed { table, ctx } => {
                let source = ctx.read_batch(to_bucket_batch(changes)?)?;
                let s = |c: &str| col(format!("s.{c}"));
                let (t, _) = table
                    .clone()
                    .merge(
                        source,
                        col("t.bucket")
                            .eq(col("s.bucket"))
                            .and(col("t.id").eq(col("s.id"))),
                    )
                    .with_source_alias("s")
                    .with_target_alias("t")
                    .when_matched_delete(|d| d.predicate(s(OP_COLUMN).eq(lit(OP_DELETE))))?
                    .when_matched_update(|mut u| {
                        u = u.predicate(s(OP_COLUMN).eq(lit(OP_UPSERT)));
                        for c in &DOC_COLUMNS[1..] {
                            u = u.update(*c, s(c));
                        }
                        u
                    })?
                    .when_not_matched_insert(|mut i| {
                        i = i.predicate(s(OP_COLUMN).eq(lit(OP_UPSERT)));
                        for c in DOC_COLUMNS.iter().chain(["bucket"].iter()) {
                            i = i.set(*c, s(c));
                        }
                        i
                    })?
                    .await?;
                *table = t;
            }
        }
        Ok(())
    }

    async fn cells_at(&self, version: Option<u64>) -> Res<Vec<Cell>> {
        let load = |t: &DeltaTable| {
            let mut t = t.clone();
            async move {
                if let Some(v) = version {
                    t.load_version(v).await?;
                }
                Res::Ok(t)
            }
        };
        match self {
            Backend::Merge(s) => Ok(match version {
                Some(v) => s.cells_at(v).await?,
                None => s.cells().await?,
            }),
            Backend::Append { table, .. } => {
                decode(&[reduce_log(&scan(&load(table).await?).await?)?])
            }
            Backend::Bucketed { table, .. } => decode(&scan(&load(table).await?).await?),
        }
    }
}

// ---------------------------------------------------------------- run

async fn timed<T>(f: impl std::future::Future<Output = T>) -> (T, Duration) {
    let t = Instant::now();
    let r = f.await;
    (r, t.elapsed())
}

/// Best of 3, to keep one-off OS noise out of read numbers.
async fn read_latency(b: &Backend, version: Option<u64>) -> Res<(usize, Duration)> {
    let mut best = Duration::MAX;
    let mut n = 0;
    for _ in 0..3 {
        let (r, d) = timed(b.cells_at(version)).await;
        n = r?.len();
        best = best.min(d);
    }
    Ok((n, best))
}

async fn run(kind: &str, commits: usize, n_cells: usize, squash_every: usize) -> Res<Vec<Cell>> {
    let tmp = tempfile::tempdir()?;
    let path = tmp.path().join("v.cruncha");
    let mut cells = initial_cells(n_cells);
    let initial: Vec<Change> = cells.iter().cloned().map(Change::Upsert).collect();
    let plan = workload(&mut cells, commits);

    let mut b = Backend::create(kind, &path, squash_every).await?;
    b.commit(&initial).await?;
    let (_, base_bytes) = dir_stats(&path);
    let v_initial = b.version();

    println!("\n## {kind}: {commits} commits on a {n_cells}-cell document");
    let mut lat = Vec::with_capacity(commits);
    let mut reads = Vec::new();
    let started = Instant::now();
    for (i, changes) in plan.iter().enumerate() {
        let (r, d) = timed(b.commit(changes)).await;
        r?;
        lat.push(d);
        // Sample one commit *before* each multiple of commits/5: for append with a squash
        // interval dividing it, that is the worst case (most rows since the last squash).
        if (i + 2) % (commits / 5).max(1) == 0 {
            let (n, rd) = read_latency(&b, None).await?;
            let (files, bytes) = dir_stats(&path);
            reads.push(rd);
            println!(
                "  after {:>5} commits: read latest ({n} cells) {:>6.1} ms; {files:>6} files, {:>8.1} KiB",
                i + 1,
                ms(rd),
                bytes as f64 / 1024.0
            );
        }
    }
    let wall = started.elapsed();
    let (_, end_bytes) = dir_stats(&path);
    let mut sorted = lat.clone();
    sorted.sort();
    println!(
        "  commit: p50 {:.1} ms, p95 {:.1} ms, p99 {:.1} ms, max {:.1} ms; {:.1} commits/s (incl. squashes)",
        ms(pct(&sorted, 0.5)),
        ms(pct(&sorted, 0.95)),
        ms(pct(&sorted, 0.99)),
        ms(*sorted.last().unwrap()),
        commits as f64 / wall.as_secs_f64()
    );
    if let Backend::Append { squashes, .. } = &b
        && !squashes.is_empty()
    {
        let mean = squashes.iter().sum::<Duration>() / squashes.len() as u32;
        println!(
            "  squashes: {} (mean {:.1} ms each)",
            squashes.len(),
            ms(mean)
        );
    }
    println!(
        "  growth: {:.2} KiB/commit (initial doc commit {:.1} KiB)",
        (end_bytes - base_bytes) as f64 / 1024.0 / commits as f64,
        base_bytes as f64 / 1024.0
    );
    let mid = v_initial + (b.version() - v_initial) / 2;
    let (_, tt) = read_latency(&b, Some(mid)).await?;
    let (_, tt0) = read_latency(&b, Some(v_initial)).await?;
    println!(
        "  time travel: v{mid} {:.1} ms, v{v_initial} {:.1} ms",
        ms(tt),
        ms(tt0)
    );

    let final_cells = b.cells_at(None).await?;
    drop(b);
    let (r, t_open) = timed(Backend::reopen(kind, &path, squash_every)).await;
    let b = r?;
    let (r, t_read) = timed(b.cells_at(None)).await;
    assert_eq!(r?.len(), final_cells.len());
    println!(
        "  reopen {:.1} ms + first read {:.1} ms",
        ms(t_open),
        ms(t_read)
    );

    // Correctness: the final state must equal the workload's model of the document.
    let mut expected = cells;
    expected.sort_by(|a, b| a.order_key.cmp(&b.order_key).then_with(|| a.id.cmp(&b.id)));
    assert_eq!(final_cells.len(), expected.len(), "{kind}: cell count");
    for (got, want) in final_cells.iter().zip(&expected) {
        assert!(got.same_content(want), "{kind}: cell {} differs", want.id);
    }
    println!(
        "  final state verified against the workload model ({} cells)",
        expected.len()
    );
    Ok(final_cells)
}

#[tokio::main]
async fn main() -> Res<()> {
    let a: Vec<String> = std::env::args().collect();
    let which = a.get(1).map(String::as_str).unwrap_or("all");
    let commits: usize = a.get(2).map(|s| s.parse()).transpose()?.unwrap_or(1000);
    let cells: usize = a.get(3).map(|s| s.parse()).transpose()?.unwrap_or(2000);
    let squash: usize = a.get(4).map(|s| s.parse()).transpose()?.unwrap_or(500);
    let kinds: Vec<&str> = if which == "all" {
        vec!["merge", "append", "bucketed"]
    } else {
        vec![which]
    };
    println!(
        "# store_variants: squash_every={squash}, buckets={BUCKETS}, stats_cols={:?}",
        stats_cols()
    );
    for k in kinds {
        run(k, commits, cells, squash).await?;
    }
    Ok(())
}
