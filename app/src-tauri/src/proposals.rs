// SPDX-License-Identifier: GPL-3.0-or-later
//! Agent proposals in the app (ADR-0016): turning an agent's edits into a stored proposal,
//! applying one to the latest cells (for "what if" and for accepting), staleness, and the
//! views the Proposals tab and the agent's reply show.

use std::collections::HashMap;

use cruncha_mcp::Edit;
use cruncha_store::{Cell, CellKind, Change, Proposal, ProposalStatus, ProposedCell, ProposedChange, Provenance};
use serde::Serialize;
use serde_json::{Value, json};

use crate::engine::CellResult;

/// A key strictly between `a` and `b` in byte order (`None`: unbounded): the Rust twin of
/// the editor's `keyBetween` (`app/src/lib/editor/ids.ts`), so keys made here sort like
/// keys made there.
pub fn key_between(a: Option<&str>, b: Option<&str>) -> Result<String, String> {
    const GEN_LO: i32 = b'1' as i32;
    const GEN_HI: i32 = b'z' as i32;
    const MIN_CHAR: i32 = b'!' as i32;
    let lower = a.unwrap_or("").as_bytes();
    let mut upper = b.map(str::as_bytes);
    if let Some(u) = upper
        && lower >= u
    {
        return Err(format!("key_between: {a:?} is not below {b:?}"));
    }
    let mut out: Vec<u8> = Vec::new();
    let mut i = 0;
    loop {
        let ca = lower.get(i).map(|&c| c as i32).unwrap_or(-1);
        let cb = upper.and_then(|u| u.get(i)).map(|&c| c as i32).unwrap_or(1000);
        let lo = (ca + 1).max(GEN_LO);
        let hi = (cb - 1).min(GEN_HI);
        if lo <= hi {
            let pick = if upper.is_none() { (lo + 8).min(hi) } else { (lo + hi) / 2 };
            out.push(pick as u8);
            return Ok(String::from_utf8(out).expect("ASCII"));
        }
        if ca >= 0 {
            out.push(lower[i]);
            if ca < cb {
                upper = None;
            }
        } else if cb > MIN_CHAR {
            out.push((cb - 1) as u8);
            upper = None;
        } else {
            let u = upper.expect("bounded");
            if i + 1 >= u.len() {
                return Err(format!("key_between: no key below {b:?}"));
            }
            out.push(u[i]);
        }
        i += 1;
    }
}

/// An agent's prose (one paragraph or heading, Markdown-ish) as the editor's ProseMirror
/// JSON: `## Title` is a heading, line breaks are hard breaks, text is plain.
pub fn prose_source(text: &str) -> String {
    let text = text.trim();
    let (node, body) = match text.find(' ') {
        Some(n) if n <= 6 && text[..n].bytes().all(|c| c == b'#') => {
            (json!({"type": "heading", "attrs": {"level": n}}), text[n + 1..].trim())
        }
        _ => (json!({"type": "paragraph"}), text),
    };
    let mut content = Vec::new();
    for (i, line) in body.lines().enumerate() {
        if i > 0 {
            content.push(json!({"type": "hard_break"}));
        }
        if !line.is_empty() {
            content.push(json!({"type": "text", "text": line}));
        }
    }
    let mut node = node;
    if !content.is_empty() {
        node["content"] = Value::Array(content);
    }
    node.to_string()
}

/// The kinds an agent may add.
fn parse_kind(kind: &str) -> Result<CellKind, String> {
    match kind {
        "calc" => Ok(CellKind::Calc),
        "python" => Ok(CellKind::Python),
        "rust" => Ok(CellKind::Rust),
        "prose" => Ok(CellKind::Prose),
        other => Err(format!("an agent can add calc, python, rust or prose cells, not {other:?}")),
    }
}

/// The agent's edits as stored changes, checked against the latest cells. `revs` are the
/// latest cells' revisions; `base_version` is the version the agent read (if it said), and a
/// cell changed after it is refused, so the agent re-reads instead of overwriting.
pub fn to_changes(
    edits: Vec<Edit>,
    cells: &[Cell],
    revs: &HashMap<String, u64>,
    base_version: Option<u64>,
) -> Result<Vec<ProposedChange>, String> {
    if edits.is_empty() {
        return Err("a proposal needs at least one change".into());
    }
    let by_id: HashMap<&str, &Cell> = cells.iter().map(|c| (c.id.as_str(), c)).collect();
    let mut new_ids: Vec<String> = Vec::new();
    let mut touched: Vec<String> = Vec::new();
    let check_existing = |id: &str| -> Result<u64, String> {
        let c = by_id.get(id).ok_or_else(|| format!("no cell {id} in the latest version"))?;
        if c.kind == CellKind::Meta || c.parent_id.is_some() {
            return Err(format!("cell {id} can't be changed by a proposal"));
        }
        let rev = revs.get(id).copied().unwrap_or(0);
        if let Some(base) = base_version
            && rev > base
        {
            return Err(format!(
                "cell {id} changed in version {rev}, after version {base} that you read; call read_doc again"
            ));
        }
        Ok(rev)
    };
    let mut out = Vec::with_capacity(edits.len());
    for e in edits {
        let id = match &e {
            Edit::Insert { .. } => None,
            Edit::Modify { id, .. } | Edit::Delete { id } => Some(id.clone()),
        };
        if let Some(id) = &id {
            if touched.contains(id) {
                return Err(format!("cell {id} is changed twice in one proposal"));
            }
            touched.push(id.clone());
        }
        out.push(match e {
            Edit::Insert { after, kind, source, description } => {
                let kind = parse_kind(&kind)?;
                if let Some(a) = &after
                    && !new_ids.contains(a)
                {
                    check_existing(a)?;
                }
                let id = ulid::Ulid::generate().to_string();
                new_ids.push(id.clone());
                let mut attrs = json!({});
                if let Some(d) = description.filter(|d| !d.trim().is_empty()) {
                    attrs["description"] = json!(d.trim());
                }
                let source = if kind == CellKind::Prose { prose_source(&source) } else { source };
                ProposedChange::Insert { after, cell: ProposedCell { id, kind, source, attrs } }
            }
            Edit::Modify { id, source, description } => {
                let base_rev = check_existing(&id)?;
                if source.is_none() && description.is_none() {
                    return Err(format!("modify {id}: give a new source, a description, or both"));
                }
                let prose = by_id[id.as_str()].kind == CellKind::Prose;
                let source = source.map(|s| if prose { prose_source(&s) } else { s });
                let attrs = description.map(|d| json!({"description": d.trim()}));
                ProposedChange::Modify { id, base_rev, source, attrs }
            }
            Edit::Delete { id } => {
                let base_rev = check_existing(&id)?;
                ProposedChange::Delete { id, base_rev }
            }
        });
    }
    Ok(out)
}

/// Cells of a proposal whose base revision is gone: changed or deleted since.
pub fn stale_ids(p: &Proposal, cells: &[Cell], revs: &HashMap<String, u64>) -> Vec<String> {
    let mut out = Vec::new();
    for c in &p.changes {
        match c {
            ProposedChange::Modify { id, base_rev, .. } | ProposedChange::Delete { id, base_rev } => {
                if revs.get(id) != Some(base_rev) {
                    out.push(id.clone());
                }
            }
            ProposedChange::Insert { after: Some(a), .. } => {
                let inserted_here = p.changes.iter().any(|o| matches!(o, ProposedChange::Insert { cell, .. } if &cell.id == a));
                if !inserted_here && !cells.iter().any(|c| &c.id == a) {
                    out.push(a.clone());
                }
            }
            ProposedChange::Insert { after: None, .. } => {}
        }
    }
    out
}

/// The latest cells with `p` applied, and the store changes that do it. Cells are marked
/// with `provenance` (`ai:proposed` for a preview, `ai:accepted` for the commit). Names are
/// left for the caller to compute, as with any commit.
pub fn apply(cells: &[Cell], p: &Proposal, provenance: &Provenance) -> (Vec<Cell>, Vec<Change>) {
    let mut doc: Vec<Cell> = cells.to_vec();
    let mut changes: Vec<Change> = Vec::new();
    // Top-level blocks in order (where inserts go).
    let is_block = |c: &Cell| c.parent_id.is_none() && c.kind != CellKind::Meta;
    for change in &p.changes {
        match change {
            ProposedChange::Modify { id, source, attrs, .. } => {
                let Some(c) = doc.iter_mut().find(|c| &c.id == id) else { continue };
                if let Some(s) = source {
                    c.source = s.clone();
                }
                if let Some(Value::Object(a)) = attrs {
                    if !c.attrs.is_object() {
                        c.attrs = json!({});
                    }
                    for (k, v) in a {
                        c.attrs[k] = v.clone();
                    }
                }
                c.provenance = provenance.clone();
                changes.push(Change::Upsert(c.clone()));
            }
            ProposedChange::Delete { id, .. } => {
                if doc.iter().any(|c| &c.id == id) {
                    doc.retain(|c| &c.id != id);
                    changes.push(Change::Delete(id.clone()));
                }
            }
            ProposedChange::Insert { after, cell } => {
                let blocks: Vec<&Cell> = doc.iter().filter(|c| is_block(c)).collect();
                // After its anchor; at the start for none; at the end if the anchor is gone.
                let pos = match after {
                    None => 0,
                    Some(a) => blocks.iter().position(|c| &c.id == a).map(|i| i + 1).unwrap_or(blocks.len()),
                };
                let lo = pos.checked_sub(1).map(|i| blocks[i].order_key.as_str());
                let hi = blocks.get(pos).map(|c| c.order_key.as_str());
                let key = key_between(lo, hi)
                    .or_else(|_| key_between(lo, None))
                    .unwrap_or_else(|_| "z".into());
                let mut c = Cell::new(cell.kind, key, cell.source.clone());
                c.id = cell.id.clone();
                c.attrs = if cell.attrs.is_object() { cell.attrs.clone() } else { json!({}) };
                c.provenance = provenance.clone();
                changes.push(Change::Upsert(c.clone()));
                doc.push(c);
                doc.sort_by(|a, b| a.order_key.cmp(&b.order_key).then_with(|| a.id.cmp(&b.id)));
            }
        }
    }
    (doc, changes)
}

/// The agent session part of an actor (`ai:claude-code` → `claude-code`).
pub fn session_of(actor: &str) -> String {
    actor.strip_prefix("ai:").unwrap_or(actor).to_owned()
}

/// One change of a proposal, for the card.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeItem {
    /// `insert`, `modify` or `delete`.
    pub op: &'static str,
    pub id: String,
    pub kind: String,
    /// For inserts: the cell it goes after.
    pub after: Option<String>,
    /// The cell's latest source (modify, delete).
    pub before: Option<String>,
    /// The proposed source (insert, modify with a source).
    pub source: Option<String>,
    pub description_before: Option<String>,
    pub description: Option<String>,
    /// The cell changed (or went) since the agent read it.
    pub stale: bool,
}

/// A cell whose result the proposal would change.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Effect {
    pub id: String,
    pub kind: String,
    /// `None`: the cell is new.
    pub before: Option<CellResult>,
    /// `None`: the cell would be deleted.
    pub after: Option<CellResult>,
}

/// A proposal as the Proposals tab shows it.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProposalView {
    pub id: String,
    pub status: ProposalStatus,
    pub proposed_by: String,
    pub message: String,
    pub base_version: u64,
    pub created_at_ms: i64,
    pub decided_by: Option<String>,
    pub decided_at_ms: Option<i64>,
    pub decision_note: Option<String>,
    pub accepted_version: Option<u64>,
    /// Some cell changed since the agent read it (open proposals only).
    pub stale: bool,
    pub changes: Vec<ChangeItem>,
    /// "If accepted" (open proposals only): every cell whose result would change.
    pub effects: Vec<Effect>,
}

fn description(c: &Cell) -> Option<String> {
    c.attrs["description"].as_str().map(str::to_owned)
}

/// The card for `p`. `after` is the what-if evaluation for an open proposal.
pub fn view(
    p: &Proposal,
    cells: &[Cell],
    revs: &HashMap<String, u64>,
    before: &HashMap<String, CellResult>,
    after: Option<&HashMap<String, CellResult>>,
) -> ProposalView {
    let open = p.status == ProposalStatus::Open;
    let stale: Vec<String> = if open { stale_ids(p, cells, revs) } else { Vec::new() };
    let by_id: HashMap<&str, &Cell> = cells.iter().map(|c| (c.id.as_str(), c)).collect();
    let changes = p
        .changes
        .iter()
        .map(|c| {
            let cur = by_id.get(c.cell_id()).copied();
            match c {
                ProposedChange::Insert { after, cell } => ChangeItem {
                    op: "insert",
                    id: cell.id.clone(),
                    kind: cell.kind.as_str().into(),
                    after: after.clone(),
                    before: None,
                    source: Some(cell.source.clone()),
                    description_before: None,
                    description: cell.attrs["description"].as_str().map(str::to_owned),
                    stale: after.as_ref().is_some_and(|a| stale.contains(a)),
                },
                ProposedChange::Modify { id, source, attrs, .. } => ChangeItem {
                    op: "modify",
                    id: id.clone(),
                    kind: cur.map(|c| c.kind.as_str().to_owned()).unwrap_or_default(),
                    after: None,
                    before: cur.map(|c| c.source.clone()),
                    source: source.clone(),
                    description_before: cur.and_then(description),
                    description: attrs.as_ref().and_then(|a| a["description"].as_str().map(str::to_owned)),
                    stale: stale.contains(id),
                },
                ProposedChange::Delete { id, .. } => ChangeItem {
                    op: "delete",
                    id: id.clone(),
                    kind: cur.map(|c| c.kind.as_str().to_owned()).unwrap_or_default(),
                    after: None,
                    before: cur.map(|c| c.source.clone()),
                    source: None,
                    description_before: cur.and_then(description),
                    description: None,
                    stale: stale.contains(id),
                },
            }
        })
        .collect();
    let mut effects = Vec::new();
    if let Some(after) = after {
        let changed: Vec<&str> = p.changes.iter().map(ProposedChange::cell_id).collect();
        let mut ids: Vec<&String> = before.keys().chain(after.keys()).collect();
        ids.sort();
        ids.dedup();
        // Document order: latest cells first, then new ones.
        let order: HashMap<&str, usize> = cells.iter().enumerate().map(|(i, c)| (c.id.as_str(), i)).collect();
        ids.sort_by_key(|id| order.get(id.as_str()).copied().unwrap_or(usize::MAX));
        for id in ids {
            let (b, a) = (before.get(id), after.get(id));
            let differs = match (b, a) {
                (Some(b), Some(a)) => {
                    b.text != a.text || b.error != a.error || b.check != a.check || b.pending != a.pending
                }
                _ => true,
            };
            if differs || changed.contains(&id.as_str()) {
                let kind = by_id
                    .get(id.as_str())
                    .map(|c| c.kind.as_str().to_owned())
                    .or_else(|| {
                        p.changes.iter().find_map(|c| match c {
                            ProposedChange::Insert { cell, .. } if &cell.id == id => Some(cell.kind.as_str().to_owned()),
                            _ => None,
                        })
                    })
                    .unwrap_or_default();
                effects.push(Effect { id: id.clone(), kind, before: b.cloned(), after: a.cloned() });
            }
        }
    }
    ProposalView {
        id: p.id.clone(),
        status: p.status,
        proposed_by: p.proposed_by.clone(),
        message: p.message.clone(),
        base_version: p.base_version,
        created_at_ms: p.created_at_us / 1000,
        decided_by: p.decided_by.clone(),
        decided_at_ms: p.decided_at_us.map(|t| t / 1000),
        decision_note: p.decision_note.clone(),
        accepted_version: p.accepted_version,
        stale: !stale.is_empty(),
        changes,
        effects,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_match_the_editor() {
        // Same cases as app/src/lib/editor/ids.test.ts.
        let k = key_between(None, None).unwrap();
        assert_eq!(k, "9");
        let mut keys = vec![k];
        for _ in 0..300 {
            let next = key_between(Some(keys.last().unwrap()), None).unwrap();
            keys.push(next);
        }
        for _ in 0..100 {
            let first = key_between(None, Some(&keys[0])).unwrap();
            keys.insert(0, first);
        }
        for _ in 0..100 {
            let mid = key_between(Some(&keys[0]), Some(&keys[1])).unwrap();
            keys.insert(1, mid);
        }
        assert!(keys.windows(2).all(|w| w[0] < w[1]));
        for (a, b) in [("a0", "a1"), ("a0", "a0V"), ("0", "1"), ("!", "!0"), ("a", "b")] {
            let k = key_between(Some(a), Some(b)).unwrap();
            assert!(a < k.as_str() && k.as_str() < b, "{a} < {k} < {b}");
        }
        assert!(key_between(Some("b"), Some("a")).is_err());
        assert!(key_between(None, Some("!")).is_err());
    }

    #[test]
    fn prose_from_markdown() {
        let h: Value = serde_json::from_str(&prose_source("## Loads")).unwrap();
        assert_eq!(h["type"], "heading");
        assert_eq!(h["attrs"]["level"], 2);
        assert_eq!(h["content"][0]["text"], "Loads");
        let p: Value = serde_json::from_str(&prose_source("one\ntwo")).unwrap();
        assert_eq!(p["type"], "paragraph");
        assert_eq!(p["content"][1]["type"], "hard_break");
        let hash: Value = serde_json::from_str(&prose_source("#3 is a bolt")).unwrap();
        assert_eq!(hash["type"], "paragraph");
    }
}
