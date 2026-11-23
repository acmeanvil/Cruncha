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
  /** The office's report style. */
  reportStyle?: ReportStyle;
  /** Agents: default, per-agent model and connection, provider accounts (keys are in the Keychain). */
  agents?: AgentSettings;
  /** Ids of documents trusted to run Python (kept by the backend), and where each was last opened. */
  trusted?: string[];
  trustedPaths?: Record<string, string>;
  /** The open tabs (paths, in order) and the active one, reopened at launch (kept by the backend). */
  openTabs?: string[];
  activeTab?: string | null;
}

export interface KnownModel {
  value: string;
  name: string;
  group: string | null;
}

export interface AgentPref {
  model: string | null;
  connection: string | null;
  knownModels: KnownModel[];
}

/** A provider account. Its key is in the Keychain, never in the app's files or here. */
export interface Connection {
  id: string;
  provider: string;
  name: string;
  baseUrl: string;
}

export interface AgentSettings {
  defaultAgent: string | null;
  agents: Record<string, AgentPref>;
  connections: Connection[];
}

export const NO_AGENT_SETTINGS: AgentSettings = { defaultAgent: null, agents: {}, connections: [] };
export const NO_AGENT_PREF: AgentPref = { model: null, connection: null, knownModels: [] };

export interface ProviderView {
  id: string;
  label: string;
  /** The API key's environment variable; empty when no key is needed (local servers). */
  keyVar: string;
}

export type KeySource = "environment" | "keychain" | "missing" | "notneeded";

export interface PythonEnv {
  available: boolean;
  interpreter: string | null;
  label: string;
  base: string;
}

/** A document's own settings, kept in its meta cell (versioned). */
export interface DocumentSettings {
  report: ReportSettings;
  /** This document's type sizes; null: the app's. */
  textSizes: TextSizes | null;
}

/** The office's look for reports (Preferences; the same for every document here). */
export interface ReportStyle {
  company: string;
  /** A PNG, JPEG or SVG file for the page header. */
  logoPath: string | null;
  /** `#RRGGBB` */
  accent: string;
  paper: "us-letter" | "a4";
}

export const DEFAULT_REPORT_STYLE: ReportStyle = { company: "", logoPath: null, accent: "#1f4e79", paper: "us-letter" };

/** The cover page's title block (kept in the document, with history). */
export interface TitleBlock {
  title: string;
  subject: string;
  project: string;
  projectNumber: string;
  client: string;
  preparedBy: string;
  checkedBy: string;
  approvedBy: string;
}

export interface ReportOptions {
  showCode: boolean;
  checkSummary: boolean;
  tableOfContents: boolean;
}

export interface ReportSettings {
  title: TitleBlock;
  options: ReportOptions;
}

/** An agent the panel can start (ACP; Claude Code, Codex, …). */
export interface AgentDef {
  id: string;
  name: string;
  command: string;
  args: string[];
  note: string;
}

export type ToolOutput =
  | { Text: string }
  | { Diff: { path: string; old_text: string | null; new_text: string } }
  | { Terminal: string };

export type AgentEntry =
  | { type: "user"; text: string }
  | { type: "agent"; text: string }
  | { type: "thought"; text: string }
  | { type: "tool"; id: string; title: string; kind: string; status: "pending" | "running" | "done" | "failed"; output: ToolOutput[] }
  | { type: "notice"; text: string; error: boolean };

export interface PermissionView {
  id: number;
  toolCallId: string;
  title: string;
  options: { id: string; label: string; kind: "allowOnce" | "allowAlways" | "rejectOnce" | "rejectAlways" }[];
}

export interface ConfigOption {
  id: string;
  name: string;
  category: "Model" | "Mode" | "Other";
  current: string;
  choices: { value: string; name: string; description: string | null; group: string | null }[];
}

export interface PanelView {
  agentId: string | null;
  /** The document the conversation belongs to: the tab it started in (ADR-0024). */
  docId?: string | null;
  agentName: string | null;
  agentVersion: string | null;
  /** An agent process is running (starting, ready or busy). */
  running: boolean;
  ready: boolean;
  busy: boolean;
  entries: AgentEntry[];
  plan: { content: string; status: "Pending" | "Active" | "Done" }[];
  permissions: PermissionView[];
  options: ConfigOption[];
  used: number;
  size: number;
  totals: { model: string | null; input_tokens: number; output_tokens: number; cache_read_tokens: number; requests: number } | null;
  conversationId: string | null;
}

export interface ConversationSummary {
  id: string;
  agent: string;
  startedMs: number;
  updatedMs: number;
  title: string;
  turns: number;
  version: number;
}

/** The MCP server agents on this computer connect to (while allowed). */
export interface AgentServerView {
  url: string;
  token: string;
  /** `claude mcp add …` */
  claudeCommand: string;
  /** An `mcpServers` JSON block for other agents. */
  configJson: string;
}

export interface ExportView {
  path: string;
  pages: number;
  bytes: number;
  ms: number;
  /** Failing checks, errors and cells not calculated. */
  problems: number;
  /** Python was still running when the wait ran out. */
  timedOut: boolean;
  /** Figures printed as FIGURE NOT RENDERED. */
  figuresMissing: number;
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
  /** Stored code-cell results, counted apart from history. */
  resultsBytes: number;
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

/** What archiving the snapshot groups before `cutoff` would do (ADR-0023). */
export interface ArchivePreview {
  fromVersion: number;
  cutoff: number;
  saves: number;
  bytesFreed: number;
  /** Snapshots archived (kept and restorable), oldest first. */
  snapshots: string[];
}

/** One row of the save log: a save that archiving or trimming removed. */
export interface ArchivedSave {
  version: number;
  atMs: number;
  actor: string;
  message: string;
  kind: "create" | "edit" | "restore" | "proposal";
  snapshot: string | null;
  cells: string[];
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
  /** `@test` calcs that evaluated: whether the check holds. */
  check?: boolean | null;
}

export type PyState =
  | "no_python" | "no_rust" | "past" | "untrusted" | "edited" | "blocked"
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
  | { kind: "text"; text: string }
  /** An interactive figure (ADR-0017): a JSON spec, drawn in a sandboxed frame. */
  | { kind: "figure"; mime: string; spec: string; summary: string; ranges: unknown; staticSrc: string | null };

/** A figure a report needs drawn before export (ADR-0017 §4). */
export interface FigureJob {
  key: string;
  mime: string;
  spec: string;
  font: string;
}

export interface PyView {
  state: PyState;
  note: string | null;
  ms: number | null;
  stdout: string;
  stderr: string;
  outputs: OutputView[];
  error: PyError | null;
  /** Read from the document's stored results (ADR-0018), not run this session. */
  stored: StoredFrom | null;
}

export interface StoredFrom {
  /** `user:<name>@<host>`. */
  by: string;
  atUs: number;
  env: string;
  /** Computed on another computer: not reproduced here. */
  elsewhere: boolean;
  /** Computed with another Python environment (ADR-0019): shown because the cell can't run here. */
  otherEnv: boolean;
}

export interface PythonStatus {
  available: boolean;
  interpreter: string | null;
  trusted: boolean;
  hasPython: boolean;
  /** Rust is set up (ADR-0006); the document has Rust cells. */
  rustAvailable: boolean;
  hasRust: boolean;
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
  /** The same: columns and the first and last rows, for the inspector. */
  preview?: Preview | null;
}

/** The calc language's functions, operators and constants (cruncha-eval reference/functions.json). */
export interface FunctionReference {
  categories: string[];
  entries: FunctionEntry[];
}

export interface FunctionEntry {
  name: string;
  category: string;
  signature: string;
  summary: string;
  /** Lines evaluated in turn; the last one's result is `result` (checked by a test). */
  example?: string[];
  result?: string;
  /** Works, but not as the name suggests. */
  caution?: string;
  /** false: parsed but not evaluated yet (`note` says what to use). */
  available?: boolean;
  note?: string;
  /** What Insert types (default: `name()` with the cursor inside). */
  insert?: string;
}

/** The units a calc can use (cruncha_units::catalog, `unit_catalog`). */
export interface UnitCatalog {
  /** In the order the palette groups by; `si` is the unit sizes are shown in. */
  quantities: { name: string; si: string }[];
  /** SI and binary prefixes, largest first; `symbol` is empty when only the word works. */
  prefixes: { symbol: string; word: string; factor: number }[];
  units: CatalogUnit[];
}

export interface CatalogUnit {
  name: string;
  aliases: string[];
  quantity: string;
  /** One of this unit in its quantity's SI unit. */
  siFactor: number;
  /** Non-zero for an offset scale (°C, °F, psig). */
  gauge: number;
  /** SI prefixes as symbols (kN) or words (kilonewton), or none. */
  prefixes: "symbol" | "word" | null;
  /** Needs exchange rates, which Cruncha doesn't have yet. */
  currency: string | null;
}

/** The start and end of a vector, matrix or data frame (engine.rs `Preview`). */
export interface Preview {
  rows: number;
  columns: { name: string; unit: string | null; dtype: "number" | "text"; missing: number }[];
  /** How many of `columns` the rows show (the first ones). */
  shownColumns: number;
  head: string[][];
  /** The last rows, when `head` doesn't reach them. */
  tail: string[][];
  rowNames: boolean;
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

export type ProposalStatus = "open" | "accepted" | "rejected" | "withdrawn";

/** One change of an agent's proposal (ADR-0016). Sources are the cells' own (prose: ProseMirror JSON). */
export interface ProposalChange {
  op: "insert" | "modify" | "delete";
  id: string;
  kind: string;
  /** Inserts: the cell it goes after. */
  after: string | null;
  /** The cell's latest source (modify, delete). */
  before: string | null;
  /** The proposed source (insert, modify). */
  source: string | null;
  descriptionBefore: string | null;
  description: string | null;
  /** The cell changed since the agent read it. */
  stale: boolean;
}

/** A result the proposal would change: now and if accepted. */
export interface ProposalEffect {
  id: string;
  kind: string;
  before: CellResult | null;
  after: CellResult | null;
}

export interface ProposalView {
  id: string;
  status: ProposalStatus;
  proposedBy: string;
  message: string;
  baseVersion: number;
  createdAtMs: number;
  decidedBy: string | null;
  decidedAtMs: number | null;
  decisionNote: string | null;
  acceptedVersion: number | null;
  stale: boolean;
  changes: ProposalChange[];
  effects: ProposalEffect[];
}

/** The active tab's document (ADR-0024): document commands act on it unless they name one.
 *  Calls that must reach a particular document (the editor's saves and runs) pass `doc`. */
let activeDoc: string | null = null;
export function setActiveDocument(id: string | null) {
  activeDoc = id;
}
const DOC_COMMANDS = new Set([
  "commit", "results", "snapshot", "restore", "preview_calc", "close_document", "document_info", "cells",
  "cells_at_ref", "history", "document_settings", "set_document_settings", "set_document_retention",
  "archive_preview", "archive_history", "archived_saves", "retention_preview", "python_status", "setup_python",
  "setup_rust", "trust_document", "run_python", "stop_python", "restart_python", "report_settings",
  "set_report_settings", "export_report", "figures_to_render", "proposals", "accept_proposal", "reject_proposal",
  "prune_now", "clear_results", "paste_clashes", "rename_cells",
]);
function call<T>(cmd: string, args: Record<string, unknown> = {}): Promise<T> {
  return invoke<T>(cmd, DOC_COMMANDS.has(cmd) && !("doc" in args) ? { doc: activeDoc, ...args } : args);
}

/** An open document, as the tab bar shows it. */
export interface OpenDocView {
  id: string;
  name: string;
  path: string;
  version: number;
  active: boolean;
}

export const api = {
  openDocument: (path: string) => call<DocInfo>("open_document", { path }),
  createDocument: (path: string) => call<DocInfo>("create_document", { path }),
  commit: (changes: Change[], message: string, doc?: string) =>
    call<CommitView>("commit", { changes, message, ...(doc ? { doc } : {}) }),
  results: (at: At) => call<Record<string, CellResult>>("results", { at }),
  previewCalc: (source: string, doc?: string) => call<CellResult>("preview_calc", { source, ...(doc ? { doc } : {}) }),
  /** Name the latest version as a known-good snapshot. */
  snapshot: (name: string, note: string) => call<DocInfo>("snapshot", { name, note }),
  /** Make a version or snapshot the latest again (a new version). */
  restore: (at: At) => call<CommitView>("restore", { at }),
  pythonStatus: () => call<PythonStatus | null>("python_status"),
  /** Set up the Python environment (progress as `python-setup` events). */
  setupPython: () => call<Record<string, CellResult>>("setup_python"),
  /** Set up Rust for Rust cells (progress as `rust-setup` events). */
  setupRust: () => call<Record<string, CellResult>>("setup_rust"),
  functionReference: () => call<FunctionReference>("function_reference"),
  unitCatalog: () => call<UnitCatalog>("unit_catalog"),
  trustDocument: () => call<Record<string, CellResult>>("trust_document"),
  /** Run these Python cells (all if omitted), even if nothing changed. */
  runPython: (ids?: string[], doc?: string) =>
    call<Record<string, CellResult>>("run_python", { ids: ids ?? null, ...(doc ? { doc } : {}) }),
  stopPython: (doc?: string) => call<void>("stop_python", doc ? { doc } : {}),
  restartPython: () => call<Record<string, CellResult>>("restart_python"),
  createSampleDocument: (path: string) => call<DocInfo>("create_sample_document", { path }),
  closeDocument: (doc?: string) => call<PruneResult | null>("close_document", doc ? { doc } : {}),
  /** Names pasted cells define that the document already defines, with a free name for each. */
  pasteClashes: (cells: { kind: string; source: string }[], doc?: string) =>
    call<{ name: string; cells: number[]; suggestion: string; others: string[] }[]>("paste_clashes", { cells, ...(doc ? { doc } : {}) }),
  /** The cells' sources with names renamed (old → new), each checked; refused if unsafe. */
  renameCells: (cells: { kind: string; source: string }[], names: Record<string, string>, doc?: string) =>
    call<string[]>("rename_cells", { cells, names, ...(doc ? { doc } : {}) }),
  /** The open documents, in tab order. */
  openDocuments: () => call<OpenDocView[]>("open_documents"),
  /** Switch to a tab. */
  activateDocument: (doc: string) => call<DocInfo>("activate_document", { doc }),
  documentInfo: () => call<DocInfo | null>("document_info"),
  cells: (version: number | null) => call<Cell[]>("cells", { version }),
  cellsAtRef: (name: string) => call<Cell[]>("cells_at_ref", { name }),
  history: () => call<VersionView[]>("history"),
  getPrefs: () => call<Prefs>("get_prefs"),
  setPrefs: (prefs: Prefs) => call<void>("set_prefs", { prefs }),
  reportSettings: () => call<ReportSettings>("report_settings"),
  setReportSettings: (settings: ReportSettings) => call<CommitView>("set_report_settings", { settings }),
  /** The latest version, or (with `at`) a past version or snapshot from its stored results. */
  exportReport: (path: string, at?: At) => call<ExportView>("export_report", { path, at: at ?? null }),
  openPath: (path: string) => call<void>("open_path", { path }),
  /** Open an http, https or mailto link in the system's browser or mail app. */
  openUrl: (url: string) => call<void>("open_url", { url }),
  /** Markdown as cells (cruncha-prose): prose blocks as ProseMirror JSON, fenced calc lines
   *  and python/rust/tex blocks as their cells. */
  markdownCells: (markdown: string) =>
    call<{ kind: string; source: string; attrs: Record<string, unknown> }[]>("markdown_cells", { markdown }),
  agentServerStatus: () => call<AgentServerView | null>("agent_server_status"),
  agentServerStart: () => call<AgentServerView>("agent_server_start"),
  agentServerStop: () => call<void>("agent_server_stop"),
  agentCatalog: () => call<AgentDef[]>("agent_catalog"),
  agentPanelView: () => call<PanelView>("agent_panel_view"),
  agentStart: (agentId: string) => call<PanelView>("agent_start", { agentId }),
  agentPrompt: (text: string) => call<PanelView>("agent_prompt", { text }),
  agentCancel: () => call<PanelView>("agent_cancel"),
  agentStop: () => call<PanelView>("agent_stop"),
  agentSetOption: (id: string, value: string) => call<void>("agent_set_option", { id, value }),
  agentPermission: (id: number, option: string | null) => call<PanelView>("agent_permission", { id, option }),
  agentConversations: () => call<ConversationSummary[]>("agent_conversations"),
  agentConversation: (id: string) => call<{ summary: ConversationSummary; entries: AgentEntry[] }>("agent_conversation", { id }),
  forgetRecent: (path: string) => call<void>("forget_recent", { path }),
  forgetTrusted: (id: string) => call<Prefs>("forget_trusted", { id }),
  pythonEnv: () => call<PythonEnv>("python_env"),
  providers: () => call<ProviderView[]>("providers"),
  /** Where each connection's key comes from (reads the Keychain). */
  connectionKeys: () => call<Record<string, KeySource>>("connection_keys"),
  connectionAdd: (provider: string, name: string, baseUrl: string, key: string) =>
    call<Prefs>("connection_add", { provider, name, baseUrl, key }),
  connectionSetKey: (id: string, key: string) => call<void>("connection_set_key", { id, key }),
  connectionRemove: (id: string) => call<Prefs>("connection_remove", { id }),
  /** Start an agent briefly to list its models; they are remembered in the prefs returned. */
  agentModels: (agentId: string) => call<Prefs>("agent_models", { agentId }),
  documentSettings: () => call<DocumentSettings>("document_settings"),
  setDocumentSettings: (settings: DocumentSettings) => call<CommitView>("set_document_settings", { settings }),
  setDocumentRetention: (policy: Retention | null) =>
    call<DocInfo>("set_document_retention", { policy }),
  retentionPreview: (policy: Retention | null) => call<Preview>("retention_preview", { policy }),
  pruneNow: () => call<PruneResult | null>("prune_now"),
  archivePreview: (cutoff: number) => call<ArchivePreview>("archive_preview", { cutoff }),
  /** Archive every version before `cutoff` (a snapshot's version): irreversible. */
  archiveHistory: (cutoff: number) => call<DocInfo>("archive_history", { cutoff }),
  archivedSaves: (cell: string | null = null) => call<ArchivedSave[]>("archived_saves", { cell }),
  clearResults: () => call<DocInfo>("clear_results"),
  figuresToRender: (at?: At) => call<FigureJob[]>("figures_to_render", { at: at ?? null }),
  putFigure: (key: string, svg: string) => call<void>("put_figure", { key, svg }),
  /** The open document's proposals, newest first (open ones with "if accepted" results). */
  proposals: () => call<ProposalView[]>("proposals"),
  acceptProposal: (id: string, force: boolean) => call<CommitView>("accept_proposal", { id, force }),
  rejectProposal: (id: string, note: string) => call<ProposalView>("reject_proposal", { id, note }),
};
