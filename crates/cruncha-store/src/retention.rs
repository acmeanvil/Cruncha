// SPDX-License-Identifier: GPL-3.0-or-later
//! History retention (ADR-0011): the per-document policy, the commit-log reader it needs, and
//! the cutoff computation.
//!
//! Time never prunes on its own. The activity window counts back from the document's latest
//! *edit*, never from today, and [`crate::Store::prune`] only acts after new edits. Delta's own
//! wall-clock retention stays disabled on every table.

use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::error::{Error, Result};
use crate::store::{CommitKind, META_AT, META_BLOB_BYTES, META_KIND, META_PRUNED_BEFORE};

/// Default floor: the newest versions kept whatever the limits say.
pub const DEFAULT_FLOOR: u64 = 100;
/// The floor can be lowered, but never below this.
pub const MIN_FLOOR: u64 = 10;

const DAY_MS: i64 = 86_400_000;

fn default_floor() -> u64 {
    DEFAULT_FLOOR
}

/// A history-retention policy. With no limits set it means **keep everything** (the default).
/// When several limits are set, the strictest wins.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Retention {
    /// Keep at most the last N versions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub saves: Option<u64>,
    /// Keep history within this many bytes (`doc/` data and log; pins excluded).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<u64>,
    /// Keep versions made within this many days *before the latest edit* (not before today).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub activity_days: Option<u32>,
    /// The newest `floor` versions are always kept (clamped to at least [`MIN_FLOOR`]).
    #[serde(default = "default_floor")]
    pub floor: u64,
}

impl Default for Retention {
    fn default() -> Self {
        Self::keep_everything()
    }
}

impl Retention {
    pub fn keep_everything() -> Self {
        Retention {
            saves: None,
            size_bytes: None,
            activity_days: None,
            floor: DEFAULT_FLOOR,
        }
    }

    pub fn is_keep_everything(&self) -> bool {
        self.saves.is_none() && self.size_bytes.is_none() && self.activity_days.is_none()
    }

    /// The effective floor.
    pub fn floor(&self) -> u64 {
        self.floor.max(MIN_FLOOR)
    }
}

impl std::fmt::Display for Retention {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_keep_everything() {
            return f.write_str("keep everything");
        }
        let mut parts = Vec::new();
        if let Some(n) = self.saves {
            parts.push(format!("last {n} saves"));
        }
        if let Some(b) = self.size_bytes {
            parts.push(format!("up to {:.1} MB", b as f64 / 1e6));
        }
        if let Some(d) = self.activity_days {
            parts.push(format!("{d} days of activity"));
        }
        write!(f, "{}; newest {} always kept", parts.join(", "), self.floor())
    }
}

/// One Delta commit of `doc/`, read straight from `_delta_log/NNN.json`.
#[derive(Debug, Clone)]
pub(crate) struct LogEntry {
    pub version: u64,
    /// Delta's `commitInfo.timestamp` (the real clock at commit time).
    pub delta_ts_ms: i64,
    /// Cruncha's commit time (`cruncha.at`), falling back to Delta's.
    pub at_ms: i64,
    pub kind: CommitKind,
    /// The `commitInfo` object, including Cruncha's metadata keys.
    pub info: Map<String, Value>,
    /// Size of the log entry itself.
    pub log_bytes: u64,
    /// Bytes of data files and new blobs the commit added.
    pub added_bytes: u64,
    /// Bytes of data files the commit removed (when the log records their size).
    pub removed_bytes: u64,
}

impl LogEntry {
    pub fn is_edit(&self) -> bool {
        matches!(self.kind, CommitKind::Edit | CommitKind::Restore)
    }

    pub fn pruned_before(&self) -> Option<u64> {
        self.info.get(META_PRUNED_BEFORE).and_then(Value::as_u64)
    }
}

fn parse_entry(version: u64, bytes: &[u8]) -> Result<LogEntry> {
    let mut info = Map::new();
    let (mut added, mut removed) = (0u64, 0u64);
    for line in bytes.split(|b| *b == b'\n').filter(|l| !l.is_empty()) {
        let action: Value = serde_json::from_slice(line)?;
        if let Some(ci) = action.get("commitInfo").and_then(Value::as_object) {
            info = ci.clone();
        } else if let Some(add) = action.get("add") {
            added += add.get("size").and_then(Value::as_u64).unwrap_or(0);
        } else if let Some(rm) = action.get("remove") {
            removed += rm.get("size").and_then(Value::as_u64).unwrap_or(0);
        }
    }
    added += info.get(META_BLOB_BYTES).and_then(Value::as_u64).unwrap_or(0);
    let delta_ts_ms = info.get("timestamp").and_then(Value::as_i64).unwrap_or(0);
    let at_ms = info
        .get(META_AT)
        .and_then(Value::as_i64)
        .map_or(delta_ts_ms, |us| us.div_euclid(1000));
    let kind = info
        .get(META_KIND)
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or(match info.get("operation").and_then(Value::as_str) {
            Some("CREATE TABLE") => CommitKind::Create,
            _ => CommitKind::Maintenance,
        });
    Ok(LogEntry {
        version,
        delta_ts_ms,
        at_ms,
        kind,
        info,
        log_bytes: bytes.len() as u64,
        added_bytes: added,
        removed_bytes: removed,
    })
}

/// Read commits `from..=to` of the Delta table at `table_dir`, oldest first. Every one must be
/// present: a gap means the container is damaged.
pub(crate) async fn read_log(table_dir: &Path, from: u64, to: u64) -> Result<Vec<LogEntry>> {
    let log = table_dir.join("_delta_log");
    let mut out = Vec::with_capacity((to + 1).saturating_sub(from) as usize);
    for v in from..=to {
        let bytes = tokio::fs::read(log.join(format!("{v:020}.json")))
            .await
            .map_err(|e| Error::Corrupt(format!("commit {v} of {}: {e}", table_dir.display())))?;
        out.push(parse_entry(v, &bytes)?);
    }
    Ok(out)
}

/// The cutoff version for `policy`: versions before it may be pruned. `entries` are the
/// available commits, oldest first. Returns `None` when the policy keeps everything that's
/// available.
pub(crate) fn cutoff(policy: &Retention, entries: &[LogEntry]) -> Option<u64> {
    let (first, last) = (entries.first()?, entries.last()?);
    let (earliest, latest) = (first.version, last.version);
    let mut limits: Vec<u64> = Vec::new();

    if let Some(n) = policy.saves {
        limits.push((latest + 1).saturating_sub(n.max(1)));
    }
    if let Some(budget) = policy.size_bytes {
        // Newest first: keep versions while the total still fits; the latest is always kept.
        let mut total = 0u64;
        let mut c = latest;
        for e in entries.iter().rev() {
            total += e.log_bytes + e.added_bytes;
            if total > budget && e.version != latest {
                break;
            }
            c = e.version;
        }
        limits.push(c);
    }
    if let Some(days) = policy.activity_days {
        // Measured from the latest edit; a document with no edits in range has no activity
        // to measure against, so this limit keeps everything.
        if let Some(last_edit) = entries
            .iter()
            .rev()
            .find(|e| e.is_edit() || e.kind == CommitKind::Create)
        {
            let from = last_edit.at_ms - i64::from(days) * DAY_MS;
            if let Some(e) = entries.iter().find(|e| e.at_ms >= from) {
                limits.push(e.version);
            }
        }
    }

    let strictest = limits.into_iter().max()?;
    let floor_cap = (latest + 1).saturating_sub(policy.floor());
    let c = strictest.min(floor_cap);
    (c > earliest).then_some(c)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(version: u64, at_ms: i64, kind: CommitKind, bytes: u64) -> LogEntry {
        LogEntry {
            version,
            delta_ts_ms: at_ms,
            at_ms,
            kind,
            info: Map::new(),
            log_bytes: bytes,
            added_bytes: 0,
            removed_bytes: 0,
        }
    }

    /// v0 create, then edits one per hour, each 1 KiB.
    fn hourly(n: u64) -> Vec<LogEntry> {
        (0..n)
            .map(|v| {
                let kind = if v == 0 {
                    CommitKind::Create
                } else {
                    CommitKind::Edit
                };
                entry(v, v as i64 * 3_600_000, kind, 1024)
            })
            .collect()
    }

    fn policy() -> Retention {
        Retention {
            floor: 10,
            ..Retention::keep_everything()
        }
    }

    #[test]
    fn keep_everything_never_cuts() {
        assert_eq!(cutoff(&Retention::keep_everything(), &hourly(5000)), None);
        assert_eq!(cutoff(&policy(), &hourly(5000)), None);
    }

    #[test]
    fn saves_keeps_last_n() {
        let p = Retention {
            saves: Some(30),
            ..policy()
        };
        // versions 0..=199; keep 170..=199
        assert_eq!(cutoff(&p, &hourly(200)), Some(170));
    }

    #[test]
    fn size_keeps_what_fits() {
        let p = Retention {
            size_bytes: Some(50 * 1024),
            ..policy()
        };
        assert_eq!(cutoff(&p, &hourly(200)), Some(150));
    }

    #[test]
    fn activity_counts_back_from_last_edit_not_today() {
        let p = Retention {
            activity_days: Some(1),
            ..policy()
        };
        // 200 hourly versions: the last edit is at hour 199, so hours 175..=199 are in the window.
        assert_eq!(cutoff(&p, &hourly(200)), Some(175));
        // Trailing maintenance commits years later don't move the window.
        let mut e = hourly(200);
        e.push(entry(200, 5 * 365 * DAY_MS, CommitKind::Maintenance, 10));
        assert_eq!(cutoff(&p, &e), Some(175));
    }

    #[test]
    fn strictest_wins_and_floor_caps() {
        let p = Retention {
            saves: Some(100),
            size_bytes: Some(20 * 1024),
            activity_days: Some(2),
            ..policy()
        };
        assert_eq!(cutoff(&p, &hourly(200)), Some(180)); // size is strictest
        let p = Retention {
            saves: Some(1),
            floor: 25,
            ..policy()
        };
        assert_eq!(cutoff(&p, &hourly(200)), Some(175)); // floor keeps 25
        let p = Retention {
            saves: Some(1),
            floor: 0,
            ..policy()
        };
        assert_eq!(cutoff(&p, &hourly(200)), Some(190)); // floor never below 10
    }

    #[test]
    fn nothing_to_do_at_or_before_earliest() {
        let p = Retention {
            saves: Some(30),
            ..policy()
        };
        let e: Vec<_> = hourly(200).into_iter().skip(170).collect();
        assert_eq!(cutoff(&p, &e), None);
        assert_eq!(cutoff(&p, &hourly(20)), None);
    }

    #[test]
    fn policy_json_round_trips_with_default_floor() {
        let p: Retention = serde_json::from_str(r#"{"saves":1000}"#).unwrap();
        assert_eq!(p.saves, Some(1000));
        assert_eq!(p.floor, DEFAULT_FLOOR);
        assert!(Retention::keep_everything().is_keep_everything());
    }
}
