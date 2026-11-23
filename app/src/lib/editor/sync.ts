// SPDX-License-Identifier: GPL-3.0-or-later
// Saving: diff the editor's blocks against the committed cells and commit the difference as
// one version. A submitted calc commits at once; other edits after an idle pause (design 02
// §5.3: commit on submit or idle, never per keystroke).
import type { Node } from "prosemirror-model";
import { api, type Cell, type CellResult, type Change, type CommitView } from "../api";
import { isBlock, nodeToContent, signature, type CellContent } from "./cells";

export const IDLE_MS = 1500;

export type SaveStatus = "saved" | "pending" | "saving" | "error";

export interface SaveState {
  status: SaveStatus;
  error: string | null;
}

const KIND_LABEL: Record<string, string> = {
  prose: "text",
  calc: "calc",
  python: "Python",
  rust: "Rust",
  tex: "TeX",
};

function isEmpty(c: CellContent) {
  if (c.kind === "prose") {
    try {
      const j = JSON.parse(c.source);
      return !j.content || j.content.length === 0;
    } catch {
      return false;
    }
  }
  return c.source.trim() === "";
}

function short(s: string, n = 48) {
  const one = s.replace(/\s+/g, " ").trim();
  return one.length > n ? one.slice(0, n - 1) + "…" : one;
}

/** A history message for a change set, e.g. "edit calc M = w L^2/8" or "edit 3 cells". */
export function describe(adds: CellContent[], edits: CellContent[], deletes: Cell[]): string {
  const total = adds.length + edits.length + deletes.length;
  const label = (k: string) => KIND_LABEL[k] ?? k;
  if (total === 1) {
    if (adds.length) return `add ${label(adds[0].kind)}${adds[0].kind === "calc" ? ` ${short(adds[0].source)}` : ""}`;
    if (edits.length) return `edit ${label(edits[0].kind)}${edits[0].kind === "calc" ? ` ${short(edits[0].source)}` : ""}`;
    return `delete ${label(deletes[0].kind)}${deletes[0].kind === "calc" ? ` ${short(deletes[0].source)}` : ""}`;
  }
  const parts: string[] = [];
  if (adds.length) parts.push(`add ${adds.length}`);
  if (edits.length) parts.push(`edit ${edits.length}`);
  if (deletes.length) parts.push(`delete ${deletes.length}`);
  return `${parts.join(", ")} cells`;
}

export class DocSync {
  /** Committed cells shown in the editor, by id. */
  private committed = new Map<string, Cell>();
  private sigs = new Map<string, string>();
  private timer: ReturnType<typeof setTimeout> | undefined;
  private chain: Promise<void> = Promise.resolve();
  private dirty = false;
  state: SaveState = { status: "saved", error: null };

  constructor(
    private getDoc: () => Node,
    private onCommitted: (v: CommitView, sources: Map<string, string>) => void,
    private onState: (s: SaveState) => void,
    /** The document these edits belong to (ADR-0024), so a save never lands in another tab's. */
    private docId: () => string | null = () => null,
  ) {}

  /** The committed source of each cell (results are for these). */
  sources(): Map<string, string> {
    return new Map([...this.committed.values()].map((c) => [c.id, c.source]));
  }

  /** Start from these committed cells (the editor shows exactly these). */
  load(cells: Cell[]) {
    clearTimeout(this.timer);
    this.committed = new Map(cells.filter(isBlock).map((c) => [c.id, c]));
    this.sigs = new Map([...this.committed.values()].map((c) => [c.id, signature(c)]));
    this.dirty = false;
    this.set({ status: "saved", error: null });
  }

  private set(s: SaveState) {
    this.state = s;
    this.onState(s);
  }

  /** The editor changed: commit after an idle pause. */
  changed() {
    this.dirty = true;
    if (this.state.status !== "saving") this.set({ status: "pending", error: this.state.error });
    clearTimeout(this.timer);
    this.timer = setTimeout(() => void this.flush(), IDLE_MS);
  }

  /** Commit now (a submitted calc, Mod-S, switching versions, closing). */
  flush(): Promise<void> {
    clearTimeout(this.timer);
    this.chain = this.chain.then(() => this.commitOnce());
    return this.chain;
  }

  private diff() {
    const doc = this.getDoc();
    const seen = new Set<string>();
    const adds: CellContent[] = [];
    const edits: CellContent[] = [];
    const sources = new Map<string, string>();
    doc.forEach((node) => {
      const c = nodeToContent(node);
      if (!c.id || !c.order_key) return; // not yet given an identity
      seen.add(c.id);
      sources.set(c.id, c.source);
      const old = this.sigs.get(c.id);
      if (old === undefined) {
        if (!isEmpty(c)) adds.push(c); // a new, empty block isn't worth a version yet
      } else if (old !== signature(c)) {
        edits.push(c);
      }
    });
    const deletes = [...this.committed.values()].filter((c) => !seen.has(c.id));
    return { adds, edits, deletes, sources };
  }

  private toCell(c: CellContent): Cell {
    const prev = this.committed.get(c.id);
    const { from, ...content } = c;
    return {
      parent_id: null,
      defines: [],
      reads: [],
      // A new cell pasted from a document records where it came from (ADR-0024).
      provenance: from ? { state: "imported", source: from } : { state: "human" },
      modified_by: "",
      modified_at_us: 0,
      ...prev,
      ...content,
    };
  }

  private async commitOnce() {
    if (!this.dirty && this.state.status !== "error") return;
    this.dirty = false;
    const { adds, edits, deletes, sources } = this.diff();
    if (adds.length + edits.length + deletes.length === 0) {
      if (!this.dirty) this.set({ status: "saved", error: null });
      return;
    }
    const cells = [...adds, ...edits].map((c) => this.toCell(c));
    const changes: Change[] = [
      ...cells.map((cell) => ({ op: "upsert" as const, cell })),
      ...deletes.map((c) => ({ op: "delete" as const, id: c.id })),
    ];
    this.set({ status: "saving", error: null });
    try {
      const v = await api.commit(changes, describe(adds, edits, deletes), this.docId() ?? undefined);
      for (const c of cells) {
        this.committed.set(c.id, c);
        this.sigs.set(c.id, signature(c));
      }
      for (const c of deletes) {
        this.committed.delete(c.id);
        this.sigs.delete(c.id);
      }
      this.onCommitted(v, sources);
      this.set({ status: this.dirty ? "pending" : "saved", error: null });
    } catch (e) {
      this.dirty = true;
      this.set({ status: "error", error: String(e) });
    }
  }
}

export type { CellResult };
