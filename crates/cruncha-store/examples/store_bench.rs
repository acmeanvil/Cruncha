// SPDX-License-Identifier: GPL-3.0-or-later
//! Spike S3: is an always-committed Delta container fast enough for interactive editing?
//!
//! Simulates an editing session on a document of `cells` cells: `commits` commits, each changing
//! 1–3 cells (90% edits, 7% inserts, 3% deletes). Reports commit latency, on-disk growth,
//! read and time-travel latency before/after compaction, and reopen time.
//!
//! ```sh
//! cargo run -p cruncha-store --release --example store_bench -- [commits=500] [cells=200] [dir]
//! ```

use std::path::Path;
use std::time::{Duration, Instant};

use cruncha_store::{Cell, CellKind, Change, Store};

/// Tiny deterministic PRNG (xorshift) so runs are comparable.
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

fn pct(sorted: &[Duration], p: f64) -> Duration {
    sorted[((sorted.len() - 1) as f64 * p).round() as usize]
}

fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1e3
}

fn source_for(i: usize, rev: usize) -> String {
    // Realistic-ish sizes: calc statements are short, some prose/code cells are longer.
    match i % 5 {
        0 => format!(
            "Paragraph {i} rev {rev}: {}",
            "lorem ipsum dolor sit amet ".repeat(12)
        ),
        1 => format!("import polars as pl\ndf_{i} = pl.DataFrame({{'a': range({rev})}})\n"),
        _ => format!("M_{i} = w L^2 / 8 + {rev} = ?? kip·ft"),
    }
}

async fn time<T>(f: impl std::future::Future<Output = T>) -> (T, Duration) {
    let t = Instant::now();
    let r = f.await;
    (r, t.elapsed())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let commits: usize = args.get(1).map(|s| s.parse()).transpose()?.unwrap_or(500);
    let n_cells: usize = args.get(2).map(|s| s.parse()).transpose()?.unwrap_or(200);
    let tmp = tempfile::tempdir()?;
    let path = args
        .get(3)
        .map(Into::into)
        .unwrap_or_else(|| tmp.path().join("bench.cruncha"));
    let path: std::path::PathBuf = path;

    println!(
        "# store_bench: {commits} commits on a {n_cells}-cell document\n# {}",
        path.display()
    );
    let (store, t_create) = time(Store::create(&path, "user:bench")).await;
    let mut store = store?;

    // Initial document in one commit.
    let mut cells: Vec<Cell> = (0..n_cells)
        .map(|i| {
            let kind = [
                CellKind::Prose,
                CellKind::Python,
                CellKind::Calc,
                CellKind::Calc,
                CellKind::Calc,
            ][i % 5];
            Cell::new(kind, format!("a{i:06}"), source_for(i, 0))
        })
        .collect();
    let initial: Vec<Change> = cells.iter().cloned().map(Change::Upsert).collect();
    let (r, t_initial) = time(store.commit(&initial, "user:bench", "initial")).await;
    r?;
    println!(
        "create: {:.1} ms, initial {n_cells}-cell commit: {:.1} ms",
        ms(t_create),
        ms(t_initial)
    );

    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    let mut lat = Vec::with_capacity(commits);
    let mut next_key = n_cells;
    let started = Instant::now();
    for rev in 1..=commits {
        let mut changes = Vec::new();
        let mut touched = std::collections::HashSet::new();
        for _ in 0..1 + rng.below(3) {
            let roll = rng.below(100);
            if roll < 7 || cells.len() < 10 {
                let c = Cell::new(
                    CellKind::Calc,
                    format!("a{next_key:06}"),
                    source_for(next_key, rev),
                );
                next_key += 1;
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
        let (r, d) = time(store.commit(&changes, "user:bench", "edit")).await;
        r?;
        lat.push(d);
        if rev % 100 == 0 {
            let (files, bytes) = dir_stats(&path);
            println!(
                "  after {rev:>5} commits: {files:>6} files, {:>8.1} KiB on disk",
                bytes as f64 / 1024.0
            );
        }
    }
    let wall = started.elapsed();
    let mut sorted = lat.clone();
    sorted.sort();
    let first: Duration = lat.iter().take(50).sum::<Duration>() / 50.min(lat.len()).max(1) as u32;
    let last: Duration =
        lat.iter().rev().take(50).sum::<Duration>() / 50.min(lat.len()).max(1) as u32;
    println!(
        "commit latency: p50 {:.1} ms, p95 {:.1} ms, p99 {:.1} ms, max {:.1} ms; mean first-50 {:.1} ms vs last-50 {:.1} ms; {:.1} commits/s",
        ms(pct(&sorted, 0.5)),
        ms(pct(&sorted, 0.95)),
        ms(pct(&sorted, 0.99)),
        ms(*sorted.last().unwrap()),
        ms(first),
        ms(last),
        commits as f64 / wall.as_secs_f64()
    );

    let v = store.version();
    let (r, t_read) = time(store.cells()).await;
    let n_now = r?.len();
    let (r, t_tt_mid) = time(store.cells_at(v / 2)).await;
    r?;
    let (r, t_tt_early) = time(store.cells_at(2)).await;
    r?;
    let (r, t_hist) = time(store.history()).await;
    r?;
    println!(
        "before compact: read latest ({n_now} cells) {:.1} ms; time travel v{} {:.1} ms, v2 {:.1} ms; history {:.1} ms",
        ms(t_read),
        v / 2,
        ms(t_tt_mid),
        ms(t_tt_early),
        ms(t_hist)
    );

    let (stats, t_compact) = time(store.compact("system")).await;
    let stats = stats?;
    let (files, bytes) = dir_stats(&path);
    println!(
        "compact: {:.1} ms (live doc files {} -> {}); container now {files} files, {:.1} KiB",
        ms(t_compact),
        stats.files_before,
        stats.files_after,
        bytes as f64 / 1024.0
    );
    let (r, t_read2) = time(store.cells()).await;
    r?;
    let (r, t_tt_mid2) = time(store.cells_at(v / 2)).await;
    r?;
    println!(
        "after compact: read latest {:.1} ms; time travel v{} {:.1} ms",
        ms(t_read2),
        v / 2,
        ms(t_tt_mid2)
    );

    // Post-compaction commit latency (should drop back to the small-table baseline).
    let mut post = Vec::new();
    for rev in 0..20 {
        let i = rev % cells.len();
        cells[i].source = source_for(i, commits + rev);
        let (r, d) =
            time(store.commit(&[Change::Upsert(cells[i].clone())], "user:bench", "edit")).await;
        r?;
        post.push(d);
    }
    post.sort();
    println!(
        "commit latency after compact: p50 {:.1} ms",
        ms(pct(&post, 0.5))
    );

    drop(store);
    let (r, t_open) = time(Store::open(&path)).await;
    let store = r?;
    let (r, t_read3) = time(store.cells()).await;
    r?;
    println!(
        "reopen: {:.1} ms, first read {:.1} ms",
        ms(t_open),
        ms(t_read3)
    );
    Ok(())
}
