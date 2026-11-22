// SPDX-License-Identifier: GPL-3.0-or-later
//! The 10,000-version follow-up (ADR-0010 item 3, ADR-0011): how reopen and time travel grow
//! with the number of versions, and how many `remove` tombstones the checkpoint carries.
//!
//! ```sh
//! cargo run -p cruncha-store --release --example history_scale -- <dir> [versions=10000] [cells=200] [every=1000]
//! ```
//!
//! Each step commits 1–3 cell edits (as `store_bench` does). At every `every` versions it
//! reopens the container and measures: open, read latest, time travel to v1, to the middle and
//! to `latest - 10`; the latest checkpoint's tombstones and size; the size of `_delta_log`.

use std::path::Path;
use std::time::{Duration, Instant};

use cruncha_store::{Cell, CellKind, Change, Store};
use deltalake::arrow::array::Array;
use deltalake::parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;

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

fn dir_bytes(p: &Path) -> u64 {
    std::fs::read_dir(p)
        .map(|rd| rd.flatten().map(|e| if e.path().is_dir() { dir_bytes(&e.path()) } else { e.metadata().map(|m| m.len()).unwrap_or(0) }).sum())
        .unwrap_or(0)
}

/// (tombstones, adds, bytes) of the latest checkpoint of `doc/`.
fn checkpoint_stats(doc: &Path) -> (usize, usize, u64) {
    let log = doc.join("_delta_log");
    let Ok(last) = std::fs::read_to_string(log.join("_last_checkpoint")) else { return (0, 0, 0) };
    let v: serde_json::Value = serde_json::from_str(&last).unwrap();
    let version = v["version"].as_u64().unwrap();
    let file = log.join(format!("{version:020}.checkpoint.parquet"));
    let bytes = std::fs::metadata(&file).map(|m| m.len()).unwrap_or(0);
    let reader = ParquetRecordBatchReaderBuilder::try_new(std::fs::File::open(&file).unwrap()).unwrap().build().unwrap();
    let (mut removes, mut adds) = (0, 0);
    for batch in reader {
        let b = batch.unwrap();
        let count = |name: &str| b.column_by_name(name).map(|c| c.len() - c.null_count()).unwrap_or(0);
        removes += count("remove");
        adds += count("add");
    }
    (removes, adds, bytes)
}

fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1e3
}

/// Median of three runs of `f`.
async fn median3<F, Fut>(mut f: F) -> f64
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Duration>,
{
    let mut v = vec![ms(f().await), ms(f().await), ms(f().await)];
    v.sort_by(f64::total_cmp);
    v[1]
}

#[tokio::main]
async fn main() -> cruncha_store::Result<()> {
    let mut args = std::env::args().skip(1);
    let dir = std::path::PathBuf::from(args.next().expect("usage: history_scale <dir> [versions] [cells] [every]"));
    let versions: u64 = args.next().map(|s| s.parse().unwrap()).unwrap_or(10_000);
    let cells: usize = args.next().map(|s| s.parse().unwrap()).unwrap_or(200);
    let every: u64 = args.next().map(|s| s.parse().unwrap()).unwrap_or(1000);
    let path = dir.join("scale.cruncha");
    let _ = std::fs::remove_dir_all(&path);
    let mut store = Store::create(&path, "user:bench").await?;
    let mut ids = Vec::new();
    let mut seed: Vec<Change> = Vec::new();
    for i in 0..cells {
        let mut c = Cell::new(CellKind::Calc, format!("a{i:05}"), format!("x{i} = {i} m"));
        c.id = format!("C{i:05}");
        ids.push(c.id.clone());
        seed.push(Change::Upsert(c));
    }
    store.commit(&seed, "user:bench", "seed").await?;
    let mut rng = Rng(0x9E3779B97F4A7C15);
    println!("versions | open ms | latest ms | v1 ms | mid ms | latest-10 ms | tombstones | adds | checkpoint KiB | _delta_log MiB | commit p50 ms");
    let mut commit_ms = Vec::new();
    while store.version() < versions {
        let n = 1 + rng.below(3);
        let mut picked: Vec<usize> = (0..n).map(|_| rng.below(ids.len())).collect();
        picked.sort_unstable();
        picked.dedup();
        let changes: Vec<Change> = picked
            .into_iter()
            .map(|i| {
                let id = &ids[i];
                let mut c = Cell::new(CellKind::Calc, "a", format!("y = {} m", rng.next() % 1000));
                c.id = id.clone();
                c.order_key = format!("a{}", &id[1..]);
                Change::Upsert(c)
            })
            .collect();
        let t = Instant::now();
        store.commit(&changes, "user:bench", "edit").await?;
        commit_ms.push(ms(t.elapsed()));
        let v = store.version();
        if v % every == 0 || v == versions {
            drop(store);
            let open = median3(|| async {
                let t = Instant::now();
                drop(Store::open(&path).await.unwrap());
                t.elapsed()
            })
            .await;
            store = Store::open(&path).await?;
            let s = &store;
            let latest = median3(|| async {
                let t = Instant::now();
                s.cells().await.unwrap();
                t.elapsed()
            })
            .await;
            let at = |ver: u64| async move {
                median3(|| async move {
                    let t = Instant::now();
                    s.cells_at(ver).await.unwrap();
                    t.elapsed()
                })
                .await
            };
            let (v1, mid, recent) = (at(1).await, at(v / 2).await, at(v.saturating_sub(10)).await);
            let (removes, adds, cp_bytes) = checkpoint_stats(&path.join("doc"));
            commit_ms.sort_by(f64::total_cmp);
            let p50 = commit_ms.get(commit_ms.len() / 2).copied().unwrap_or(0.0);
            commit_ms.clear();
            println!(
                "{v:>8} | {open:>7.1} | {latest:>9.1} | {v1:>5.1} | {mid:>6.1} | {recent:>12.1} | {removes:>10} | {adds:>4} | {:>14.0} | {:>14.1} | {p50:>13.1}",
                cp_bytes as f64 / 1024.0,
                dir_bytes(&path.join("doc/_delta_log")) as f64 / (1 << 20) as f64
            );
        }
    }
    Ok(())
}
