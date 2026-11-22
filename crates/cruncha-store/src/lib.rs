// SPDX-License-Identifier: GPL-3.0-or-later
//! Cruncha's document container: a directory of Delta Lake tables that is committed on every
//! change (no "save"), keeps full history (time travel, named versions, rollback), and stays
//! readable by any Delta/Parquet tool (Polars, DuckDB, Spark).
//!
//! See `docs/design/02-document-model.md` §5.

mod blobs;
mod cell;
mod error;
mod logindex;
mod proposals;
mod results;
mod retention;
#[doc(hidden)] // Exposed for spikes and benchmarks; not a stable API.
pub mod schema;
mod store;

pub use blobs::DEFAULT_BLOB_THRESHOLD;
pub use cell::{Cell, CellDiff, CellKind, Change, Provenance};
pub use error::{Error, Result};
pub use proposals::{Proposal, ProposalStatus, ProposedCell, ProposedChange};
pub use results::{MAX_RUN_BYTES, MAX_TEXT_BYTES, PutReport, RunEntry, RunRecord, RunStatus};
pub use retention::{DEFAULT_FLOOR, MIN_FLOOR, Retention};
pub use schema::Ref;
pub use store::{
    CommitKind, CompactStats, FORMAT, FORMAT_VERSION, META_PROPOSAL, Manifest, PruneReport, RetentionPreview, Store,
    VersionInfo, diff_cells,
};
