// SPDX-License-Identifier: GPL-3.0-or-later
// The debug log's live records in the page (ADR-0029 §3): loaded once from the ring buffer,
// then appended from `log` events, kept from startup so the Log tab and the status bar's
// unseen-errors dot share them.
import { listen } from "@tauri-apps/api/event";
import { api, type LogRecord } from "./api";
import { capped } from "./logView";

export const FEED_CAP = 5000;

class LogFeed {
  /** The records shown (after Clear view, only newer ones). */
  records = $state.raw<LogRecord[]>([]);
  /** The newest error's seq, and the newest one the user has seen in the Log tab. */
  lastError = $state(0);
  seenError = $state(0);
  #lastSeq = 0;
  #started = false;

  get unseenErrors(): boolean {
    return this.lastError > this.seenError;
  }

  #add(rs: LogRecord[]) {
    const fresh = rs.filter((r) => r.seq > this.#lastSeq);
    if (!fresh.length) return;
    this.#lastSeq = fresh[fresh.length - 1].seq;
    for (const r of fresh) if (r.level === "error") this.lastError = r.seq;
    this.records = capped([...this.records, ...fresh], FEED_CAP);
  }

  /** Start once: listen first, then load, so nothing between the two is lost. */
  async start() {
    if (this.#started) return;
    this.#started = true;
    const early: LogRecord[] = [];
    let loaded = false;
    try {
      // Records arrive in bursts (one per backend command): add them once per frame, so a
      // burst costs one update of the list rather than one each.
      let queued: LogRecord[] = [];
      let frame = 0;
      await listen<LogRecord>("log", (e) => {
        if (!loaded) return void early.push(e.payload);
        queued.push(e.payload);
        if (!frame)
          frame = requestAnimationFrame(() => {
            frame = 0;
            const batch = queued;
            queued = [];
            this.#add(batch);
          });
      });
      this.#add(await api.logRecords(0));
    } catch {
      /* no backend (tests) */
    }
    loaded = true;
    this.#add(early.sort((a, b) => a.seq - b.seq));
  }

  markSeen() {
    if (this.seenError !== this.lastError) this.seenError = this.lastError;
  }

  /** Clear view: the page's list only (the file and the ring buffer are kept). */
  clear() {
    this.records = [];
  }
}

export const logFeed = new LogFeed();
