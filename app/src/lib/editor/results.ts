// SPDX-License-Identifier: GPL-3.0-or-later
// What each calc shows: results from the backend, keyed by cell id, plus live previews of
// sources being typed. Node views subscribe and re-render when either changes.
import { api, type CellResult } from "../api";

type Listener = () => void;

export class Results {
  private byId = new Map<string, CellResult>();
  /** The source each result was computed for, so a stale result isn't shown for new text. */
  private sourceOf = new Map<string, string>();
  private previews = new Map<string, CellResult>();
  private pending = new Map<string, Promise<CellResult>>();
  private listeners = new Set<Listener>();

  subscribe(f: Listener): () => void {
    this.listeners.add(f);
    return () => this.listeners.delete(f);
  }

  private emit() {
    for (const f of this.listeners) f();
  }

  /** Replace all results (after a commit or when viewing another version). */
  set(results: Record<string, CellResult>, sources: Map<string, string>) {
    this.byId = new Map(Object.entries(results));
    this.sourceOf = new Map(sources);
    this.emit();
  }

  /** The result to show for cell `id` with `source`: the committed result if it is for this
   *  source, otherwise a preview (fetched on demand; listeners fire when it arrives). */
  get(id: string | null, source: string): CellResult | undefined {
    if (id && this.sourceOf.get(id) === source && this.byId.has(id)) return this.byId.get(id);
    const p = this.previews.get(source);
    if (p) return p;
    void this.preview(source).then(() => this.emit());
    return undefined;
  }

  /** The committed cell `id`'s last result. */
  committed(id: string): CellResult | undefined {
    return this.byId.get(id);
  }

  /** Names the committed cell `id` defines (from its last evaluation). */
  definesOf(id: string): string[] | undefined {
    return this.byId.get(id)?.defines;
  }

  /** A preview of `source` as if it were committed now. */
  preview(source: string): Promise<CellResult> {
    const hit = this.previews.get(source);
    if (hit) return Promise.resolve(hit);
    let p = this.pending.get(source);
    if (!p) {
      p = api
        .previewCalc(source)
        .catch((e) => ({ tex: "", error: String(e) }))
        .then((r) => {
          if (this.previews.size > 500) this.previews.clear();
          this.previews.set(source, r);
          this.pending.delete(source);
          return r;
        });
      this.pending.set(source, p);
    }
    return p;
  }
}
