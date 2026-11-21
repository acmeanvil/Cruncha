// SPDX-License-Identifier: GPL-3.0-or-later
//! Stored code-cell results (ADR-0018): runs are kept apart from `doc/` (no document
//! versions), large outputs go to `blobs/`, the newest run per key wins, prune keeps the
//! blobs results reference, and dropped results take their blobs with them.

use std::collections::HashSet;

use cruncha_store::{Cell, CellKind, Change, MAX_RUN_BYTES, Retention, RunRecord, RunStatus, Store};
use serde_json::json;

fn path(dir: &tempfile::TempDir) -> std::path::PathBuf {
    dir.path().join("t.cruncha")
}

async fn seeded(dir: &tempfile::TempDir) -> Store {
    let mut s = Store::create(path(dir), "user:test").await.unwrap();
    let mut c = Cell::new(CellKind::Python, "a0", "x = 1");
    c.id = "P".into();
    s.commit(&[Change::Upsert(c)], "user:test", "seed").await.unwrap();
    s
}

fn png(seed: u8) -> String {
    // Base64-ish text well over the blob threshold.
    (0..20_000).map(|i| (b'A' + ((i as u8).wrapping_add(seed) % 26)) as char).collect()
}

fn run(cell: &str, hash: &str, at: i64, image: &str) -> RunRecord {
    RunRecord {
        cell_id: cell.into(),
        input_hash: hash.into(),
        status: RunStatus::Done,
        values: json!([{ "name": "x", "summary": "1", "plain": { "kind": "number", "value": "1" } }]),
        outputs: json!([{ "kind": "image", "mime": "image/png", "data": image, "width": 10, "height": 10 },
                        { "kind": "text", "text": "hello" }]),
        stdout: "printed\n".into(),
        stderr: String::new(),
        error: None,
        duration_ms: 12,
        env: "env-test".into(),
        computed_at_us: at,
        computed_by: "user:test@host".into(),
        doc_version: 1,
    }
}

fn keys(k: &[(&str, &str)]) -> HashSet<(String, String)> {
    k.iter().map(|(a, b)| (a.to_string(), b.to_string())).collect()
}

fn blob_count(dir: &tempfile::TempDir) -> usize {
    let Ok(rd) = std::fs::read_dir(path(dir).join("blobs")) else { return 0 };
    rd.flatten()
        .filter(|e| e.path().is_dir())
        .map(|e| std::fs::read_dir(e.path()).unwrap().count())
        .sum()
}

#[tokio::test]
async fn runs_round_trip_without_adding_versions() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = seeded(&dir).await;
    let (version, history) = (s.version(), s.history().await.unwrap().len());
    let history_bytes = s.history_size_bytes().unwrap();

    let image = png(0);
    let r = s.put_results(vec![run("P", "h1", 10, &image)], "user:test").await.unwrap();
    assert_eq!(r.written, 1);
    assert!(r.blob_bytes > 0);
    assert_eq!(s.version(), version, "a stored run is not a document version");
    assert_eq!(s.history().await.unwrap().len(), history);
    assert_eq!(s.history_size_bytes().unwrap(), history_bytes, "results are counted apart");
    assert!(s.results_size_bytes().unwrap() > 0);

    let got = s.results_for(&keys(&[("P", "h1"), ("P", "nope")])).await.unwrap();
    assert_eq!(got.len(), 1);
    assert_eq!(got[&("P".into(), "h1".into())], run("P", "h1", 10, &image));

    // The same figure in another run is stored once.
    let before = blob_count(&dir);
    s.put_results(vec![run("P", "h2", 20, &image)], "user:test").await.unwrap();
    assert_eq!(blob_count(&dir), before);

    drop(s);
    let s = Store::open(path(&dir)).await.unwrap();
    let got = s.results_for(&keys(&[("P", "h1"), ("P", "h2")])).await.unwrap();
    assert_eq!(got.len(), 2);
    assert_eq!(s.result_entries().await.unwrap().len(), 2);
}

#[tokio::test]
async fn the_newest_run_of_a_key_wins() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = seeded(&dir).await;
    let mut old = run("P", "h", 10, "small");
    old.stdout = "old".into();
    let mut new = run("P", "h", 20, "small");
    new.stdout = "new".into();
    s.put_results(vec![new], "user:test").await.unwrap();
    s.put_results(vec![old], "user:test").await.unwrap();
    let got = s.results_for(&keys(&[("P", "h")])).await.unwrap();
    assert_eq!(got[&("P".into(), "h".into())].stdout, "new");
}

#[tokio::test]
async fn oversized_runs_are_skipped_and_text_is_capped() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = seeded(&dir).await;
    let mut big = run("P", "big", 10, "x");
    big.outputs = json!([{ "kind": "text", "text": "y".repeat(MAX_RUN_BYTES + 1) }]);
    let mut chatty = run("Q", "chatty", 10, "x");
    chatty.stdout = "z".repeat(3 << 20);
    let r = s.put_results(vec![big, chatty], "user:test").await.unwrap();
    assert_eq!((r.written, r.too_large), (1, vec!["P".to_string()]));
    let got = s.results_for(&keys(&[("Q", "chatty")])).await.unwrap();
    let out = &got[&("Q".into(), "chatty".into())].stdout;
    assert!(out.len() < (1 << 20) + 64 && out.ends_with("more than 1 MB)"));
}

#[tokio::test]
async fn retain_drops_runs_and_their_blobs() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = seeded(&dir).await;
    s.put_results(vec![run("P", "keep", 10, &png(1)), run("P", "drop", 20, &png(2))], "user:test")
        .await
        .unwrap();
    assert_eq!(blob_count(&dir), 2);
    let dropped = s.retain_results(|_, h| h == "keep", "system").await.unwrap();
    assert_eq!(dropped, 1);
    assert_eq!(blob_count(&dir), 1);
    assert_eq!(s.results_for(&keys(&[("P", "keep")])).await.unwrap().len(), 1);

    assert_eq!(s.clear_results("system").await.unwrap(), 1);
    assert_eq!(blob_count(&dir), 0);
    assert!(s.result_entries().await.unwrap().is_empty());
    // A cleared table takes new runs.
    s.put_results(vec![run("P", "again", 30, "x")], "user:test").await.unwrap();
    drop(s);
    let s = Store::open(path(&dir)).await.unwrap();
    assert_eq!(s.result_entries().await.unwrap().len(), 1);
}

#[tokio::test]
async fn prune_keeps_result_blobs_and_removes_orphans_of_a_crash() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = seeded(&dir).await;
    let image = png(3);
    s.put_results(vec![run("P", "h", 10, &image)], "user:test").await.unwrap();
    // A crash after a run's blobs were written but before its row: the blob is unreferenced.
    let written = s.put_result_blobs_only(vec![run("P", "lost", 20, &png(4))]).await.unwrap();
    assert!(written > 0);
    drop(s);
    let mut s = Store::open(path(&dir)).await.unwrap();
    assert!(s.results_for(&keys(&[("P", "lost")])).await.unwrap().is_empty());
    assert_eq!(blob_count(&dir), 2);

    for i in 0..30 {
        let mut c = Cell::new(CellKind::Calc, "b0", format!("y = {i}"));
        c.id = "C".into();
        s.commit(&[Change::Upsert(c)], "user:test", "edit").await.unwrap();
    }
    let policy = Retention { saves: Some(10), size_bytes: None, activity_days: None, floor: 10 };
    s.prune(&policy, "system").await.unwrap().expect("pruned");
    assert_eq!(blob_count(&dir), 1, "the orphan goes, the stored run's figure stays");
    let got = s.results_for(&keys(&[("P", "h")])).await.unwrap();
    assert_eq!(got[&("P".into(), "h".into())].outputs[0]["data"], json!(image));
}
