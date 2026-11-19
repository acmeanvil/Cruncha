// SPDX-License-Identifier: GPL-3.0-or-later

use cruncha_store::{Cell, CellDiff, CellKind, Change, CommitKind, Error, Provenance, Store};
use serde_json::json;

fn calc(order: &str, src: &str, defines: &[&str], reads: &[&str]) -> Cell {
    let mut c = Cell::new(CellKind::Calc, order, src);
    c.defines = defines.iter().map(|s| s.to_string()).collect();
    c.reads = reads.iter().map(|s| s.to_string()).collect();
    c.attrs = json!({"display": "?"});
    c
}

fn sources(cells: &[Cell]) -> Vec<&str> {
    cells.iter().map(|c| c.source.as_str()).collect()
}

async fn new_store(dir: &tempfile::TempDir) -> Store {
    Store::create(dir.path().join("t.cruncha"), "user:test")
        .await
        .unwrap()
}

#[tokio::test]
async fn create_is_empty_at_version_zero() {
    let dir = tempfile::tempdir().unwrap();
    let store = new_store(&dir).await;
    assert_eq!(store.version(), 0);
    assert!(store.cells().await.unwrap().is_empty());
    let h = store.history().await.unwrap();
    assert_eq!(h.len(), 1);
    assert_eq!(h[0].kind, CommitKind::Create);
}

#[tokio::test]
async fn create_refuses_existing_path() {
    let dir = tempfile::tempdir().unwrap();
    let err = Store::create(dir.path(), "user:test").await.err().unwrap();
    assert!(matches!(err, Error::AlreadyExists(_)));
}

#[tokio::test]
async fn commit_round_trips_all_fields_and_orders_by_key() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = new_store(&dir).await;

    let mut prose = Cell::new(CellKind::Prose, "a0", r#"{"type":"paragraph"}"#);
    prose.provenance = Provenance::AiProposed {
        session: "s1".into(),
    };
    let mut inline = calc("a1", "L = 3.1 m", &["L"], &[]);
    inline.parent_id = Some(prose.id.clone());
    let m = calc("a2", "M = w L^2 / 8 = ?? kip·ft", &["M"], &["w", "L"]);
    let w = calc("a1V", "w = 100 lbf/ft", &["w"], &[]);

    let v = store
        .commit(
            &[
                Change::Upsert(m.clone()),
                Change::Upsert(prose.clone()),
                Change::Upsert(inline.clone()),
                Change::Upsert(w.clone()),
            ],
            "user:test",
            "first",
        )
        .await
        .unwrap();
    assert_eq!(v, 1);

    let cells = store.cells().await.unwrap();
    assert_eq!(
        sources(&cells),
        vec![
            prose.source.as_str(),
            "L = 3.1 m",
            "w = 100 lbf/ft",
            m.source.as_str()
        ]
    );
    let got_m = cells.iter().find(|c| c.id == m.id).unwrap();
    assert!(got_m.same_content(&m));
    assert_eq!(got_m.modified_by, "user:test");
    assert!(got_m.modified_at_us > 0);
    let got_inline = cells.iter().find(|c| c.id == inline.id).unwrap();
    assert_eq!(got_inline.parent_id.as_deref(), Some(prose.id.as_str()));
    let got_prose = cells.iter().find(|c| c.id == prose.id).unwrap();
    assert_eq!(got_prose.provenance, prose.provenance);
    assert!(got_prose.defines.is_empty());
}

#[tokio::test]
async fn update_delete_and_time_travel() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = new_store(&dir).await;
    let a = calc("a0", "x = 1", &["x"], &[]);
    let b = calc("a1", "y = x + 1 = ?", &["y"], &["x"]);
    let v1 = store
        .commit(
            &[Change::Upsert(a.clone()), Change::Upsert(b.clone())],
            "user:test",
            "add",
        )
        .await
        .unwrap();

    let mut a2 = a.clone();
    a2.source = "x = 2".into();
    let v2 = store
        .commit(
            &[Change::Upsert(a2.clone()), Change::Delete(b.id.clone())],
            "ai:s1",
            "edit",
        )
        .await
        .unwrap();
    assert_eq!(v2, v1 + 1);

    assert_eq!(sources(&store.cells().await.unwrap()), vec!["x = 2"]);
    assert_eq!(
        sources(&store.cells_at(v1).await.unwrap()),
        vec!["x = 1", "y = x + 1 = ?"]
    );
    assert!(store.cells_at(0).await.unwrap().is_empty());
    assert!(matches!(
        store.cells_at(v2 + 1).await,
        Err(Error::NoSuchVersion(_))
    ));

    let diff = store.diff(v1, v2).await.unwrap();
    assert_eq!(diff.len(), 2);
    assert!(diff.iter().any(|d| matches!(d, CellDiff::Modified { before, after } if before.source == "x = 1" && after.source == "x = 2")));
    assert!(
        diff.iter()
            .any(|d| matches!(d, CellDiff::Removed(c) if c.id == b.id))
    );

    let h = store.history().await.unwrap();
    assert_eq!(h[0].version, v2);
    assert_eq!(h[0].actor, "ai:s1");
    assert_eq!(h[0].message, "edit");
    assert_eq!(h[0].kind, CommitKind::Edit);
}

#[tokio::test]
async fn deleting_a_missing_id_is_harmless_and_empty_commit_is_noop() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = new_store(&dir).await;
    let v = store.commit(&[], "user:test", "nothing").await.unwrap();
    assert_eq!(v, 0);
    store
        .commit(&[Change::Delete("nope".into())], "user:test", "del")
        .await
        .unwrap();
    assert!(store.cells().await.unwrap().is_empty());
}

#[tokio::test]
async fn duplicate_ids_in_one_commit_are_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = new_store(&dir).await;
    let a = calc("a0", "x = 1", &["x"], &[]);
    let err = store
        .commit(
            &[Change::Upsert(a.clone()), Change::Delete(a.id.clone())],
            "user:test",
            "dup",
        )
        .await
        .err()
        .unwrap();
    assert!(matches!(err, Error::DuplicateId(_)));
    assert_eq!(store.version(), 0);
}

#[tokio::test]
async fn persists_across_reopen_and_lock_is_exclusive() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("t.cruncha");
    let a = calc("a0", "x = 1", &["x"], &[]);
    {
        let mut store = Store::create(&path, "user:test").await.unwrap();
        store
            .commit(&[Change::Upsert(a.clone())], "user:test", "add")
            .await
            .unwrap();
        assert!(matches!(Store::open(&path).await, Err(Error::Locked(_))));
    }
    let store = Store::open(&path).await.unwrap();
    assert_eq!(store.version(), 1);
    assert_eq!(sources(&store.cells().await.unwrap()), vec!["x = 1"]);
}

#[tokio::test]
async fn open_rejects_non_container() {
    let dir = tempfile::tempdir().unwrap();
    assert!(matches!(
        Store::open(dir.path()).await,
        Err(Error::NotAContainer(_))
    ));
}

#[tokio::test]
async fn restore_is_a_new_version_and_can_be_undone() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = new_store(&dir).await;
    let a = calc("a0", "x = 1", &["x"], &[]);
    let b = calc("a1", "y = 2", &["y"], &[]);
    let v1 = store
        .commit(&[Change::Upsert(a.clone())], "user:test", "a")
        .await
        .unwrap();
    let mut a2 = a.clone();
    a2.source = "x = 10".into();
    let v2 = store
        .commit(
            &[Change::Upsert(a2), Change::Upsert(b)],
            "user:test",
            "a2+b",
        )
        .await
        .unwrap();

    let v3 = store.restore(v1, "user:test").await.unwrap();
    assert_eq!(v3, v2 + 1);
    assert_eq!(sources(&store.cells().await.unwrap()), vec!["x = 1"]);
    let h = store.history().await.unwrap();
    assert_eq!(h[0].kind, CommitKind::Restore);
    assert_eq!(h[0].restored_from, Some(v1));

    // Undo the restore by restoring to v2.
    store.restore(v2, "user:test").await.unwrap();
    assert_eq!(
        sources(&store.cells().await.unwrap()),
        vec!["x = 10", "y = 2"]
    );
    // Restoring to the current state is a no-op.
    let v = store.version();
    assert_eq!(store.restore(v, "user:test").await.unwrap(), v);
}

#[tokio::test]
async fn tags_pin_versions() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = new_store(&dir).await;
    let a = calc("a0", "x = 1", &["x"], &[]);
    store
        .commit(&[Change::Upsert(a.clone())], "user:test", "a")
        .await
        .unwrap();
    let r = store
        .tag("rev A", "issued for review", "user:test")
        .await
        .unwrap();
    assert!(matches!(
        store.tag("rev A", "", "user:test").await,
        Err(Error::RefExists(_))
    ));

    let mut a2 = a.clone();
    a2.source = "x = 2".into();
    store
        .commit(&[Change::Upsert(a2)], "user:test", "edit")
        .await
        .unwrap();

    assert_eq!(
        sources(&store.cells_at_ref("rev A").await.unwrap()),
        vec!["x = 1"]
    );
    let refs = store.refs().await.unwrap();
    assert_eq!(refs, vec![r]);
    assert!(matches!(
        store.cells_at_ref("rev B").await,
        Err(Error::NoSuchRef(_))
    ));
}

#[tokio::test]
async fn compact_preserves_content_and_history() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = new_store(&dir).await;
    let mut cells: Vec<Cell> = (0..20)
        .map(|i| calc(&format!("a{i:03}"), &format!("x{i} = {i}"), &[], &[]))
        .collect();
    for c in &cells {
        store
            .commit(&[Change::Upsert(c.clone())], "user:test", "add")
            .await
            .unwrap();
    }
    cells[3].source = "x3 = 33".into();
    let before_compact = store
        .commit(&[Change::Upsert(cells[3].clone())], "user:test", "edit")
        .await
        .unwrap();

    let stats = store.compact("system").await.unwrap();
    assert!(stats.files_removed > 1, "{stats:?}");
    assert_eq!(store.version(), before_compact + 1);

    let now = store.cells().await.unwrap();
    assert_eq!(now.len(), 20);
    assert_eq!(now[3].source, "x3 = 33");
    assert_eq!(store.cells_at(5).await.unwrap().len(), 5);
    let h = store.history().await.unwrap();
    assert_eq!(h[0].kind, CommitKind::Maintenance);
    assert_eq!(h.len() as u64, store.version() + 1);

    drop(store);
    let reopened = Store::open(dir.path().join("t.cruncha")).await.unwrap();
    assert_eq!(reopened.cells().await.unwrap().len(), 20);
}
