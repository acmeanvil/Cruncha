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
  /** `@test` calcs that evaluated: whether the check holds. */
  check?: boolean | null;
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
  reportSettings: () => invoke<ReportSettings>("report_settings"),
  setReportSettings: (settings: ReportSettings) => invoke<CommitView>("set_report_settings", { settings }),
  exportReport: (path: string) => invoke<ExportView>("export_report", { path }),
  openPath: (path: string) => invoke<void>("open_path", { path }),
  agentServerStatus: () => invoke<AgentServerView | null>("agent_server_status"),
  agentServerStart: () => invoke<AgentServerView>("agent_server_start"),
  agentServerStop: () => invoke<void>("agent_server_stop"),
  agentCatalog: () => invoke<AgentDef[]>("agent_catalog"),
  agentPanelView: () => invoke<PanelView>("agent_panel_view"),
  agentStart: (agentId: string) => invoke<PanelView>("agent_start", { agentId }),
  agentPrompt: (text: string) => invoke<PanelView>("agent_prompt", { text }),
  agentCancel: () => invoke<PanelView>("agent_cancel"),
  agentStop: () => invoke<PanelView>("agent_stop"),
  agentSetOption: (id: string, value: string) => invoke<void>("agent_set_option", { id, value }),
  agentPermission: (id: number, option: string | null) => invoke<PanelView>("agent_permission", { id, option }),
  agentConversations: () => invoke<ConversationSummary[]>("agent_conversations"),
  agentConversation: (id: string) => invoke<{ summary: ConversationSummary; entries: AgentEntry[] }>("agent_conversation", { id }),
  forgetRecent: (path: string) => invoke<void>("forget_recent", { path }),
  forgetTrusted: (id: string) => invoke<Prefs>("forget_trusted", { id }),
  pythonEnv: () => invoke<PythonEnv>("python_env"),
  providers: () => invoke<ProviderView[]>("providers"),
  /** Where each connection's key comes from (reads the Keychain). */
  connectionKeys: () => invoke<Record<string, KeySource>>("connection_keys"),
  connectionAdd: (provider: string, name: string, baseUrl: string, key: string) =>
    invoke<Prefs>("connection_add", { provider, name, baseUrl, key }),
  connectionSetKey: (id: string, key: string) => invoke<void>("connection_set_key", { id, key }),
  connectionRemove: (id: string) => invoke<Prefs>("connection_remove", { id }),
  /** Start an agent briefly to list its models; they are remembered in the prefs returned. */
  agentModels: (agentId: string) => invoke<Prefs>("agent_models", { agentId }),
  documentSettings: () => invoke<DocumentSettings>("document_settings"),
  setDocumentSettings: (settings: DocumentSettings) => invoke<CommitView>("set_document_settings", { settings }),
  setDocumentRetention: (policy: Retention | null) =>
    invoke<DocInfo>("set_document_retention", { policy }),
  retentionPreview: (policy: Retention | null) => invoke<Preview>("retention_preview", { policy }),
  pruneNow: () => invoke<PruneResult | null>("prune_now"),
  /** The open document's proposals, newest first (open ones with "if accepted" results). */
  proposals: () => invoke<ProposalView[]>("proposals"),
  acceptProposal: (id: string, force: boolean) => invoke<CommitView>("accept_proposal", { id, force }),
  rejectProposal: (id: string, note: string) => invoke<ProposalView>("reject_proposal", { id, note }),
};
