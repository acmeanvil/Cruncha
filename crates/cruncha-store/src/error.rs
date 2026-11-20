// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("delta: {0}")]
    Delta(#[from] deltalake::DeltaTableError),
    #[error("arrow: {0}")]
    Arrow(#[from] deltalake::arrow::error::ArrowError),
    #[error("datafusion: {0}")]
    DataFusion(#[from] deltalake::datafusion::error::DataFusionError),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("{0} is not a Cruncha container")]
    NotAContainer(PathBuf),
    #[error("{0} already exists")]
    AlreadyExists(PathBuf),
    #[error("container is open for writing by another process ({0})")]
    Locked(PathBuf),
    #[error("unsupported container format version {0}")]
    UnsupportedFormat(u32),
    #[error("no such version: {0}")]
    NoSuchVersion(u64),
    #[error("history before version {earliest} has been pruned")]
    Pruned { earliest: u64 },
    #[error("no such ref: {0}")]
    NoSuchRef(String),
    #[error("ref already exists: {0}")]
    RefExists(String),
    #[error("duplicate cell id in one commit: {0}")]
    DuplicateId(String),
    #[error("corrupt container: {0}")]
    Corrupt(String),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
