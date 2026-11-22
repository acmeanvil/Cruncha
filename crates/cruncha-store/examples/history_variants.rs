// SPDX-License-Identifier: GPL-3.0-or-later
//! Options for the 10,000-version follow-up (ADR-0010 item 3), measured on a copy of a
//! container that `history_scale` built:
//!
//! - `baseline`: nothing changed;
//! - `tombstones`: tombstones expire at once (`delta.deletedFileRetentionDuration`), then a
//!   checkpoint, which then drops them (SPIKE ONLY: changing a retention property breaks a
//!   store invariant today, so this needs an ADR before any real use);
//! - `prune`: the existing retention prune, keeping the last 1,000 saves.
//!
//! ```sh
//! cargo run -p cruncha-store --release --example history_variants -- <container> <baseline|tombstones|prune>
//! ```

use std::path::Path;
use std::time::{Duration, Instant};

use cruncha_store::{Cell, CellKind, Change, Retention, Store};
use deltalake::arrow::array::Array;
use deltalake::parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;

fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1e3
}

fn checkpoint_stats(doc: &Path) -> (usize, u64) {
    let log = doc.join("_delta_log");
    let last: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(log.join("_last_checkpoint")).unwrap()).unwrap();
    let file = log.join(format!("{:020}.checkpoint.parquet", last["version"].as_u64().unwrap()));
    let bytes = std::fs::metadata(&file).unwrap().len();
    let reader = ParquetRecordBatchReaderBuilder::try_new(std::fs::File::open(&file).unwrap()).unwrap().build().unwrap();
    let removes = reader
        .map(|b| {
            let b = b.unwrap();
            b.column_by_name("remove").map(|c| c.len() - c.null_count()).unwrap_or(0)
        })
        .sum();
    (removes, bytes)
}

async fn median3<F, Fut>(mut f: F) -> f64
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Duration>,
{
    let mut v = [ms(f().await), ms(f().await), ms(f().await)];
    v.sort_by(f64::total_cmp);
    v[1]
}

#[tokio::main]
async fn main() -> cruncha_store::Result<()> {
    let mut args = std::env::args().skip(1);
    let path = std::path::PathBuf::from(args.next().expect("usage: history_variants <container> <variant>"));
    let variant = args.next().unwrap_or_else(|| "baseline".into());
    let doc = path.join("doc");
    match variant.as_str() {
        "baseline" => {}
        "tombstones" => {
            let url = url::Url::from_directory_path(&doc).unwrap();
            let table = deltalake::open_table(url).await?;
            let props = [("delta.deletedFileRetentionDuration".to_string(), "interval 0 seconds".to_string())].into();
            let table = table.set_tbl_properties().with_properties(props).await?;
            deltalake::checkpoints::create_checkpoint(&table, None).await?;
        }
        "prune" => {
            let mut s = Store::open(&path).await?;
            // An edit commit first: prune acts only after edits since its last marker.
            let mut c = Cell::new(CellKind::Calc, "a00000", "x0 = 1 m");
            c.id = "C00000".into();
            s.commit(&[Change::Upsert(c)], "user:bench", "edit").await?;
            let t = Instant::now();
            let policy = Retention { saves: Some(1000), size_bytes: None, activity_days: None, floor: 10 };
            s.prune(&policy, "system").await?;
            println!("prune took {:.0} ms", ms(t.elapsed()));
        }
        v => panic!("unknown variant {v}"),
    }
    let (removes, cp_bytes) = checkpoint_stats(&doc);
    let log_files = std::fs::read_dir(doc.join("_delta_log")).unwrap().count();
    let open = median3(|| async {
        let t = Instant::now();
        drop(Store::open(&path).await.unwrap());
        t.elapsed()
    })
    .await;
    let mut s = Store::open(&path).await?;
    let latest = s.version();
    let earliest = s.earliest_version().max(1);
    let st = &s;
    let at = |v: u64| async move {
        median3(|| async move {
            let t = Instant::now();
            st.cells_at(v).await.unwrap();
            t.elapsed()
        })
        .await
    };
    let (first, mid, recent) = (at(earliest).await, at((earliest + latest) / 2).await, at(latest - 10).await);
    let mut commit = Vec::new();
    for i in 0..100 {
        let mut c = Cell::new(CellKind::Calc, "a00001", format!("x1 = {i} m"));
        c.id = "C00001".into();
        let t = Instant::now();
        s.commit(&[Change::Upsert(c)], "user:bench", "edit").await?;
        commit.push(ms(t.elapsed()));
    }
    commit.sort_by(f64::total_cmp);
    println!(
        "{variant}: versions {earliest}..{latest}, log files {log_files}, tombstones {removes}, checkpoint {:.0} KiB | open {open:.1} ms | \
         earliest {first:.1} · mid {mid:.1} · latest-10 {recent:.1} ms | commit p50 {:.1} p95 {:.1} ms",
        cp_bytes as f64 / 1024.0,
        commit[50],
        commit[95]
    );
    Ok(())
}
