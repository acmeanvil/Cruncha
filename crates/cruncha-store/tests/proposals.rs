// SPDX-License-Identifier: GPL-3.0-or-later
//! Agent proposals (ADR-0016): proposing leaves the document alone, accepting is one human
//! commit carrying `cruncha.proposal`, decisions are appended, an interrupted accept is
//! finished on open, and prune drops only old decided proposals.

use cruncha_store::{
    Cell, CellKind, Change, Error, META_PROPOSAL, ProposalStatus, ProposedCell, ProposedChange,
    Retention, Store,
};
use serde_json::json;

async fn new_store(dir: &tempfile::TempDir) -> Store {
    Store::create(dir.path().join("t.cruncha"), "user:test")
        .await
        .unwrap()
}

fn cell(id: &str, order: &str, src: &str) -> Cell {
    let mut c = Cell::new(CellKind::Calc, order, src);
    c.id = id.into();
    c
}

async fn seeded(dir: &tempfile::TempDir) -> Store {
    let mut s = new_store(dir).await;
    s.commit(
        &[Change::Upsert(cell("A", "a0", "L = 20 ft")), Change::Upsert(cell("B", "a1", "M = L"))],
        "user:test",
        "seed",
    )
    .await
    .unwrap();
    s
}

fn modify_b(rev: u64) -> Vec<ProposedChange> {
    vec![
        ProposedChange::Modify { id: "B".into(), base_rev: rev, source: Some("M = 2 L".into()), attrs: None },
        ProposedChange::Insert {
            after: Some("B".into()),
            cell: ProposedCell { id: "C".into(), kind: CellKind::Calc, source: "N = M".into(), attrs: json!({}) },
        },
    ]
}

#[tokio::test]
async fn proposing_leaves_the_document_and_its_history_alone() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = seeded(&dir).await;
    let v = s.version();
    let revs = s.cell_revs().await.unwrap();
    assert_eq!(revs["B"], v);
    let p = s.propose(modify_b(revs["B"]), "double the moment", "ai:s1").await.unwrap();
    assert_eq!(p.status, ProposalStatus::Open);
    assert_eq!(p.base_version, v);
    assert_eq!(p.proposed_by, "ai:s1");
    assert_eq!(p.changes, modify_b(v));
    assert_eq!(s.version(), v, "no document version for a proposal");
    assert_eq!(s.cells().await.unwrap().len(), 2);
    assert!(dir.path().join("t.cruncha/proposals/_delta_log").is_dir());
}

#[tokio::test]
async fn accepting_is_one_commit_with_the_proposal_id() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = seeded(&dir).await;
    let p = s.propose(modify_b(1), "double the moment", "ai:s1").await.unwrap();
    let changes = [
        Change::Upsert(cell("B", "a1", "M = 2 L")),
        Change::Upsert(cell("C", "a2", "N = M")),
    ];
    let v = s.accept(&p.id, &changes, "user:test", "accept: double the moment").await.unwrap();
    assert_eq!(v, 2);
    let p = s.proposal(&p.id).await.unwrap();
    assert_eq!(p.status, ProposalStatus::Accepted);
    assert_eq!(p.accepted_version, Some(2));
    assert_eq!(p.decided_by.as_deref(), Some("user:test"));
    let h = s.history().await.unwrap();
    assert_eq!(h[0].version, 2);
    assert_eq!(h[0].actor, "user:test");
    // Deciding again is refused.
    let e = s.accept(&p.id, &changes, "user:test", "again").await.unwrap_err();
    assert!(matches!(e, Error::ProposalDecided { .. }), "{e}");
    assert_eq!(s.version(), 2);
    // The commit carries the id.
    let log = std::fs::read_to_string(dir.path().join("t.cruncha/doc/_delta_log/00000000000000000002.json")).unwrap();
    assert!(log.contains(META_PROPOSAL) && log.contains(&p.id), "{log}");
}

#[tokio::test]
async fn rejecting_and_withdrawing_append_rows_and_never_touch_doc() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = seeded(&dir).await;
    let a = s.propose(modify_b(1), "one", "ai:s1").await.unwrap();
    let b = s.propose(modify_b(1), "two", "ai:s1").await.unwrap();
    let a = s.decide(&a.id, ProposalStatus::Rejected, "user:test", "wrong load case").await.unwrap();
    let b = s.decide(&b.id, ProposalStatus::Withdrawn, "ai:s1", "").await.unwrap();
    assert_eq!(a.status, ProposalStatus::Rejected);
    assert_eq!(a.decision_note.as_deref(), Some("wrong load case"));
    assert_eq!(a.seq, 1);
    assert_eq!(b.status, ProposalStatus::Withdrawn);
    assert_eq!(s.version(), 1);
    assert!(s.decide(&a.id, ProposalStatus::Rejected, "user:test", "").await.is_err());
    assert!(s.decide(&a.id, ProposalStatus::Open, "user:test", "").await.is_err());
    // Survives reopening.
    drop(s);
    let s = Store::open(dir.path().join("t.cruncha")).await.unwrap();
    let all = s.proposals().await.unwrap();
    assert_eq!(all.len(), 2);
    assert_eq!(all[0].message, "one");
    assert_eq!(all[1].status, ProposalStatus::Withdrawn);
}

#[tokio::test]
async fn an_accept_interrupted_after_its_commit_is_finished_on_open() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = seeded(&dir).await;
    let p = s.propose(modify_b(1), "double", "ai:s1").await.unwrap();
    let v = s
        .commit_for_proposal(&p.id, &[Change::Upsert(cell("B", "a1", "M = 2 L"))], "user:test", "accept")
        .await
        .unwrap();
    assert_eq!(s.proposal(&p.id).await.unwrap().status, ProposalStatus::Open);
    drop(s); // the app died here
    let s = Store::open(dir.path().join("t.cruncha")).await.unwrap();
    let got = s.proposal(&p.id).await.unwrap();
    assert_eq!(got.status, ProposalStatus::Accepted);
    assert_eq!(got.accepted_version, Some(v));
    assert_eq!(got.decided_by.as_deref(), Some("user:test"));
    drop(s);
    // Idempotent: no second row.
    let s = Store::open(dir.path().join("t.cruncha")).await.unwrap();
    assert_eq!(s.proposal(&p.id).await.unwrap().seq, 1);
}

#[tokio::test]
async fn prune_drops_old_decided_proposals_and_keeps_open_ones() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = seeded(&dir).await;
    let old = s.propose(modify_b(1), "old", "ai:s1").await.unwrap();
    s.decide(&old.id, ProposalStatus::Rejected, "user:test", "").await.unwrap();
    let open = s.propose(modify_b(1), "still open", "ai:s1").await.unwrap();
    for i in 0..30 {
        let mut x = cell("X", "b0", &format!("x = {i}"));
        x.order_key = "b0".into();
        s.commit(&[Change::Upsert(x)], "user:test", "edit").await.unwrap();
    }
    let recent = s.propose(modify_b(1), "recent", "ai:s1").await.unwrap();
    s.decide(&recent.id, ProposalStatus::Rejected, "user:test", "").await.unwrap();

    let policy = Retention { saves: Some(10), size_bytes: None, activity_days: None, floor: 10 };
    let r = s.prune(&policy, "system").await.unwrap().expect("pruned");
    assert_eq!(r.proposals_deleted, 1);
    let ids: Vec<String> = s.proposals().await.unwrap().into_iter().map(|p| p.id).collect();
    assert_eq!(ids, vec![open.id.clone(), recent.id.clone()]);
    drop(s);
    let s = Store::open(dir.path().join("t.cruncha")).await.unwrap();
    assert_eq!(s.proposals().await.unwrap().len(), 2);
}
