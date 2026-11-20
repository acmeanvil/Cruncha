// SPDX-License-Identifier: GPL-3.0-or-later
// Typed wrappers around the backend's Tauri commands (app/src-tauri/src/lib.rs).
import { invoke } from "@tauri-apps/api/core";

/** A history-retention policy (ADR-0011). No limits set means "keep everything". */
export interface Retention {
  saves?: number | null;
  size_bytes?: number | null;
  activity_days?: number | null;
  floor: number;
}

export const KEEP_EVERYTHING: Retention = { floor: 100 };

export interface Prefs {
  defaultRetention: Retention;
}

export interface RefView {
  name: string;
  version: number;
  message: string;
  actor: string;
  createdAtMs: number;
  pinned: boolean;
}

export interface DocInfo {
  path: string;
  name: string;
  id: string;
  version: number;
  earliestVersion: number;
  historyBytes: number;
  retention: Retention | null;
  effectiveRetention: Retention;
  refs: RefView[];
}

export type CommitKind = "create" | "edit" | "restore" | "maintenance";

export interface VersionView {
  version: number;
  timestampMs: number;
  kind: CommitKind;
  actor: string;
  message: string;
  restoredFrom: number | null;
  prunedBefore: number | null;
}

export type Provenance =
  | { state: "human" }
  | { state: "ai_proposed"; session: string }
  | { state: "ai_accepted"; session: string; by: string }
  | { state: "imported"; source: string };

export interface Cell {
  id: string;
  kind: string;
  order_key: string;
  parent_id: string | null;
  source: string;
  attrs: unknown;
  defines: string[];
  reads: string[];
  provenance: Provenance;
  modified_by: string;
  modified_at_us: number;
}

/** Which state of the document is on screen. */
export type Selection =
  | { kind: "latest" }
  | { kind: "version"; version: number }
  | { kind: "ref"; name: string };

export interface Preview {
  earliestVersion: number;
  cutoff: number | null;
  cutoffTimestampMs: number | null;
  bytesFreed: number;
  refsToPin: string[];
  waitsForEdit: boolean;
}

export interface PruneResult {
  prunedBefore: number;
  pinned: string[];
  dataFilesDeleted: number;
  logFilesDeleted: number;
}

export const api = {
  openDocument: (path: string) => invoke<DocInfo>("open_document", { path }),
  createSampleDocument: (path: string) => invoke<DocInfo>("create_sample_document", { path }),
  closeDocument: () => invoke<PruneResult | null>("close_document"),
  documentInfo: () => invoke<DocInfo | null>("document_info"),
  cells: (version: number | null) => invoke<Cell[]>("cells", { version }),
  cellsAtRef: (name: string) => invoke<Cell[]>("cells_at_ref", { name }),
  history: () => invoke<VersionView[]>("history"),
  getPrefs: () => invoke<Prefs>("get_prefs"),
  setPrefs: (prefs: Prefs) => invoke<void>("set_prefs", { prefs }),
  setDocumentRetention: (policy: Retention | null) =>
    invoke<DocInfo>("set_document_retention", { policy }),
  retentionPreview: (policy: Retention | null) => invoke<Preview>("retention_preview", { policy }),
  pruneNow: () => invoke<PruneResult | null>("prune_now"),
};
