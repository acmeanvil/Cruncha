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
  /** Recently opened documents, most recent first (kept by the backend). */
  recent?: string[];
  /** Editor type sizes in CSS pixels (the backend fills in defaults). */
  textSizes?: TextSizes;
}

/** Editor type sizes in CSS pixels. Headings scale with `text` (H1 2×, H2 1.5×, …). */
export interface TextSizes {
  text: number;
  calc: number;
  code: number;
}

export const DEFAULT_TEXT_SIZES: TextSizes = { text: 15, calc: 15, code: 13 };

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

/** What a calc cell shows (app/src-tauri/src/engine.rs). */
export interface CellResult {
  /** The display line (echo and result) as TeX; empty if it never got to evaluation. */
  tex: string;
  error: string | null;
  /** The plain-text result line. */
  text?: string;
  /** Names the cell defines. */
  defines?: string[];
  /** Names the cell reads. */
  reads?: string[];
  /** The value of each name it defined, as evaluated. */
  values?: ValueView[];
  /** Lints: the cell runs, but something deserves a look. */
  warnings?: string[];
  /** Waiting for a Python cell upstream. */
  pending?: boolean;
  /** Python cells: run state and output. */
  python?: PyView | null;
}

export type PyState =
  | "no_python" | "past" | "untrusted" | "edited" | "blocked"
  | "queued" | "running" | "done" | "error" | "interrupted";

export interface PyError {
  type: string;
  message: string;
  line: number | null;
  traceback: string;
}

export type OutputView =
  | { kind: "image"; src: string; width: number; height: number }
  | { kind: "table"; columns: string[]; rows: (string | null)[][]; totalRows: number }
  | { kind: "text"; text: string };

export interface PyView {
  state: PyState;
  note: string | null;
  ms: number | null;
  stdout: string;
  stderr: string;
  outputs: OutputView[];
  error: PyError | null;
}

export interface PythonStatus {
  available: boolean;
  interpreter: string | null;
  trusted: boolean;
  hasPython: boolean;
}

/** `python-setup` events. */
export type SetupProgress =
  | { step: "installingUv" } | { step: "installingPython" } | { step: "installingPackages" }
  | { step: "done"; python: string };

/** `results` events: fresh results of the latest version, computed in the background. */
export interface ResultsEvent {
  docId: string;
  version: number;
  results: Record<string, CellResult>;
}

/** One variable's value (engine.rs `ValueView`). */
export interface ValueView {
  name: string;
  /** The value as TeX, as a later calc's echo shows it. */
  tex: string;
  /** "number", "matrix", "dataframe", "text", …, or "python" (calcs can't use it) */
  kind: string;
  /** Values from Python: what the object is. */
  summary?: string | null;
  /** Vectors, matrices, frames: their size ("3 × 4 matrix"), shown while collapsed. */
  shape?: string | null;
}

export type Change = { op: "upsert"; cell: Cell } | { op: "delete"; id: string };

export interface CommitView {
  info: DocInfo;
  results: Record<string, CellResult>;
}

/** A state of the document for `results`: the latest, a version, or a named version. */
export interface At {
  version?: number | null;
  ref?: string | null;
}

export const api = {
  openDocument: (path: string) => invoke<DocInfo>("open_document", { path }),
  createDocument: (path: string) => invoke<DocInfo>("create_document", { path }),
  commit: (changes: Change[], message: string) => invoke<CommitView>("commit", { changes, message }),
  results: (at: At) => invoke<Record<string, CellResult>>("results", { at }),
  previewCalc: (source: string) => invoke<CellResult>("preview_calc", { source }),
  /** Name the latest version as a known-good snapshot. */
  snapshot: (name: string, note: string) => invoke<DocInfo>("snapshot", { name, note }),
  /** Make a version or snapshot the latest again (a new version). */
  restore: (at: At) => invoke<CommitView>("restore", { at }),
  pythonStatus: () => invoke<PythonStatus | null>("python_status"),
  /** Set up the Python environment (progress as `python-setup` events). */
  setupPython: () => invoke<Record<string, CellResult>>("setup_python"),
  trustDocument: () => invoke<Record<string, CellResult>>("trust_document"),
  /** Run these Python cells (all if omitted), even if nothing changed. */
  runPython: (ids?: string[]) => invoke<Record<string, CellResult>>("run_python", { ids: ids ?? null }),
  stopPython: () => invoke<void>("stop_python"),
  restartPython: () => invoke<Record<string, CellResult>>("restart_python"),
  createSampleDocument: (path: string) => invoke<DocInfo>("create_sample_document", { path }),
  closeDocument: () => invoke<PruneResult | null>("close_document"),
  documentInfo: () => invoke<DocInfo | null>("document_info"),
  cells: (version: number | null) => invoke<Cell[]>("cells", { version }),
  cellsAtRef: (name: string) => invoke<Cell[]>("cells_at_ref", { name }),
  history: () => invoke<VersionView[]>("history"),
  getPrefs: () => invoke<Prefs>("get_prefs"),
  setPrefs: (prefs: Prefs) => invoke<void>("set_prefs", { prefs }),
  forgetRecent: (path: string) => invoke<void>("forget_recent", { path }),
  setDocumentRetention: (policy: Retention | null) =>
    invoke<DocInfo>("set_document_retention", { policy }),
  retentionPreview: (policy: Retention | null) => invoke<Preview>("retention_preview", { policy }),
  pruneNow: () => invoke<PruneResult | null>("prune_now"),
};
