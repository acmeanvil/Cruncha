// SPDX-License-Identifier: GPL-3.0-or-later
//! Cruncha's document container: a directory of Delta Lake tables that is committed on every
//! change (no "save"), keeps full history (time travel, named versions, rollback), and stays
//! readable by any Delta/Parquet tool (Polars, DuckDB, Spark).
//!
//! See `docs/design/02-document-model.md` §5.

mod cell;
mod error;
mod schema;
mod store;

pub use cell::{Cell, CellDiff, CellKind, Change, Provenance};
pub use error::{Error, Result};
pub use schema::Ref;
pub use store::{
    CommitKind, CompactStats, FORMAT, FORMAT_VERSION, Manifest, Store, VersionInfo, diff_cells,
};
