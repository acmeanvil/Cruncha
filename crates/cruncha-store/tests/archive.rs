// SPDX-License-Identifier: GPL-3.0-or-later
//! Archiving snapshot groups (ADR-0023): oldest first, snapshots kept, every removed save in
//! the save log, and automatic prunes keep the log too.

use cruncha_store::{Cell, CellKind, Change, Error, Retention, Store};

async fn new_store(dir: &tempfile::TempDir) -> Store {
    Store::create(dir.path().join("t.cruncha"), "user:test").await.unwrap()
}

/// `n` edits, each rewriting X and adding a new cell.
async fn edits(store: &mut Store, n: u64, actor: &str) {
    for _ in 0..n {
        let v = store.version() + 1;
        let mut x = Cell::new(CellKind::Calc, "a0", format!("x = {v}"));
        x.id = "X".into();
        let mut y = Cell::new(CellKind::Calc, format!("b{v:06}"), format!("y{v} = x"));
        y.id = format!("Y{v}");
        store
            .commit(&[Change::Upsert(x), Change::Upsert(y)], actor, &format!("edit {v}"))
            .await
            .unwrap();
    }
}

/// Draft (5 edits), "Rev A" + 6 edits, "Rev B" + 4 edits, "Rev C" + 3 edits.
async fn revisions(store: &mut Store) -> (u64, u64, u64) {
    store.set_squash_interval(4); // squashes inside groups: the save log must see through them
    edits(store, 5, "user:a").await;
    let a = store.tag("Rev A", "issued", "user:a").await.unwrap().doc_version;
    edits(store, 6, "ai:claude").await;
    let b = store.tag("Rev B", "issued", "user:a").await.unwrap().doc_version;
    edits(store, 4, "user:b").await;
    let c = store.tag("Rev C", "issued", "user:a").await.unwrap().doc_version;
    edits(store, 3, "user:a").await;
    (a, b, c)
}

#[tokio::test]
async fn archive_oldest_groups_keeps_snapshots_and_logs_every_save() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = new_store(&dir).await;
    let (a, b, c) = revisions(&mut store).await;
    let at_a = store.cells_at(a).await.unwrap();
    let at_b = store.cells_at(b).await.unwrap();
    let latest = store.cells().await.unwrap();
    let saves_before_b = store
        .history()
        .await
        .unwrap()
        .into_iter()
        .filter(|v| v.version < b && v.kind != cruncha_store::CommitKind::Maintenance)
        .count();

    // Archive Draft and Rev A: everything before Rev B.
    let preview = store.archive_preview(b).await.unwrap();
    assert_eq!(preview.saves, saves_before_b);
    assert_eq!(preview.snapshots.iter().map(|r| r.name.as_str()).collect::<Vec<_>>(), ["Rev A"]);
    assert!(preview.bytes_freed > 0);
    let report = store.archive_before(b, "user:a").await.unwrap();
    assert_eq!(report.pruned_before, b);
    assert_eq!(report.pinned, ["Rev A"]);
    assert_eq!(store.earliest_version(), b);

    // The snapshots and everything from Rev B on are exactly as they were.
    let same = |x: &[Cell], y: &[Cell]| x.len() == y.len() && x.iter().zip(y).all(|(p, q)| p.id == q.id && p.same_content(q));
    assert!(same(&store.cells_at_ref("Rev A").await.unwrap(), &at_a));
    assert!(same(&store.cells_at(b).await.unwrap(), &at_b));
    assert!(same(&store.cells().await.unwrap(), &latest));
    assert!(matches!(store.cells_at(a).await, Err(Error::Pruned { .. })));

    // The save log: one row per removed save, with its group, actor and cells.
    let log = store.archived(None).await.unwrap();
    assert_eq!(log.len(), saves_before_b);
    assert_eq!(log[0].kind, "create");
    let first_edit = &log[1];
    assert_eq!((first_edit.snapshot.as_deref(), first_edit.actor.as_str()), (None, "user:a"));
    assert_eq!(first_edit.cells, ["X", &format!("Y{}", first_edit.version)]);
    let in_a: Vec<_> = log.iter().filter(|r| r.snapshot.as_deref() == Some("Rev A")).collect();
    // The group starts at the snapshot's own version (the last draft edit), as the History
    // tab groups it; Rev B's own version (the agent's last edit) is in Rev B's group.
    assert_eq!(in_a.len(), 6);
    assert_eq!((in_a[0].version, in_a[0].actor.as_str()), (a, "user:a"));
    assert!(in_a[1..].iter().all(|r| r.actor == "ai:claude"));
    assert!(in_a.iter().all(|r| r.removed_before == b && r.cells.len() == 2));
    assert!(log.iter().all(|r| r.version < b));
    // By cell.
    assert_eq!(store.archived(Some("Y3")).await.unwrap().len(), 1);
    assert_eq!(store.archived(Some("X")).await.unwrap().len(), saves_before_b - 1);

    // Then Rev B's group too; the log grows, and nothing is logged twice.
    store.archive_before(c, "user:a").await.unwrap();
    let log = store.archived(None).await.unwrap();
    assert_eq!(log.iter().filter(|r| r.snapshot.as_deref() == Some("Rev B")).count(), 4); // b, then 3 of user:b's
    assert!(log.windows(2).all(|w| w[0].version < w[1].version));
    assert!(same(&store.cells_at_ref("Rev B").await.unwrap(), &at_b));

    // Reopened, the log is still there.
    drop(store);
    let store = Store::open(dir.path().join("t.cruncha")).await.unwrap();
    assert_eq!(store.archived(None).await.unwrap().len(), log.len());
}

#[tokio::test]
async fn only_whole_groups_and_never_the_newest() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = new_store(&dir).await;
    let (a, _, c) = revisions(&mut store).await;
    // Not a snapshot's version.
    assert!(matches!(store.archive_before(a + 2, "user:a").await, Err(Error::BadArchive(_))));
    assert!(matches!(store.archive_preview(store.version()).await, Err(Error::BadArchive(_))));
    // Version 0 is the earliest: nothing before it.
    assert!(matches!(store.archive_before(0, "user:a").await, Err(Error::BadArchive(_))));
    // The newest group is everything from Rev C on: archiving up to Rev C is the most allowed.
    store.archive_before(c, "user:a").await.unwrap();
    assert!(matches!(store.archive_before(c, "user:a").await, Err(Error::BadArchive(_))));
}

#[tokio::test]
async fn a_dormant_document_archives_on_request() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = new_store(&dir).await;
    let (_, b, _) = revisions(&mut store).await;
    // Automatic pruning waits for edits after a prune; an archive is the user's choice.
    store.archive_before(b, "user:a").await.unwrap();
    store.set_clock_offset_us(5 * 365 * 86_400_000_000);
    let c = store.refs().await.unwrap()[2].doc_version;
    store.archive_before(c, "user:a").await.unwrap();
    assert_eq!(store.earliest_version(), c);
}

#[tokio::test]
async fn automatic_prunes_write_the_save_log_too() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = new_store(&dir).await;
    store.set_squash_interval(5);
    edits(&mut store, 30, "user:a").await;
    let policy = Retention { saves: Some(12), size_bytes: None, activity_days: None, floor: 10 };
    let r = store.prune(&policy, "system").await.unwrap().unwrap();
    let log = store.archived(None).await.unwrap();
    let removed = store_saves_before(&log, r.pruned_before);
    assert_eq!(removed, log.len());
    assert!(log.iter().filter(|s| s.kind == "edit").all(|s| s.cells.len() == 2));
    assert_eq!(log.iter().filter(|s| s.kind == "edit").count() as u64 + 1, log.len() as u64);
}

fn store_saves_before(log: &[cruncha_store::ArchivedSave], cutoff: u64) -> usize {
    log.iter().filter(|s| s.version < cutoff).count()
}
