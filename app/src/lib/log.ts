// SPDX-License-Identifier: GPL-3.0-or-later
// The page's side of the debug log (ADR-0029 §2): user actions, uncaught errors, rejected
// commands and stalls, sent to the backend's `log_write` in batches (every 500 ms; errors at
// once). Nothing here ever throws, and nothing logs document contents, calc sources or
// values: action names, ids and counts only.
import { invoke } from "@tauri-apps/api/core";

export type LogLevel = "error" | "warn" | "info" | "debug";

export interface PageRecord {
  level: LogLevel;
  category: string;
  message: string;
}

export const FLUSH_MS = 500;
/** Records waiting beyond this are dropped (the backend has stopped answering). */
const MAX_QUEUE = 2000;
/** A message is cut to this many characters. */
const MAX_MESSAGE = 4000;

let queue: PageRecord[] = [];
let timer: ReturnType<typeof setTimeout> | undefined;
let sender: (records: PageRecord[]) => Promise<unknown> = (records) => invoke("log_write", { records });

/** Replace how batches are sent (tests). */
export function setLogSender(f: (records: PageRecord[]) => Promise<unknown>) {
  sender = f;
}

/** Send what's queued now. */
export function flushLog() {
  clearTimeout(timer);
  timer = undefined;
  if (!queue.length) return;
  const batch = queue;
  queue = [];
  try {
    void Promise.resolve(sender(batch)).catch(() => {});
  } catch {
    /* never throw */
  }
}

function push(level: LogLevel, category: string, message: string) {
  try {
    if (message.length > MAX_MESSAGE) message = message.slice(0, MAX_MESSAGE) + "…";
    if (queue.length >= MAX_QUEUE) queue.shift();
    queue.push({ level, category, message });
    if (level === "error") flushLog();
    else if (timer === undefined) timer = setTimeout(flushLog, FLUSH_MS);
  } catch {
    /* never throw */
  }
}

/** The last action the user took, and when (for the stall watch). */
let lastAction: { name: string; at: number } | null = null;
let clock: () => number = () => Date.now();

/** A user action: a menu item, button, shortcut or dialog decision, by name (INFO `ui.action`). */
export function logAction(name: string) {
  lastAction = { name, at: clock() };
  push("info", "ui.action", name);
}

/** The last action and when it happened (tests). */
export function lastLoggedAction() {
  return lastAction;
}

/** An error's message (and stack, when it has one). */
export function describeError(err: unknown): string {
  try {
    if (err instanceof Error) {
      const stack = err.stack && !err.stack.includes(err.message) ? `${err.message}\n${err.stack}` : err.stack;
      return stack || `${err.name}: ${err.message}`;
    }
    if (typeof err === "string") return err;
    return JSON.stringify(err) ?? String(err);
  } catch {
    return String(err);
  }
}

/** An error (ERROR `ui`), sent at once. */
export function logError(where: string, err: unknown) {
  push("error", "ui", `${where}: ${describeError(err)}`);
}

/** A warning in category `cat` (the backend adds `ui.` if it doesn't start with `ui`). */
export function logWarn(cat: string, msg: string) {
  push("warn", cat, msg);
}

/** An informational record in category `cat`. */
export function logInfo(cat: string, msg: string) {
  push("info", cat, msg);
}

export const STALL_TICK_MS = 100;
export const STALL_LATE_MS = 500;
/** An action is named in a stall record if it happened this long before the window was held. */
export const STALL_ACTION_MS = 2000;

/** Watch for the page being held: a 100 ms tick arriving 500 ms or more late is a WARN
 *  `ui.stall` record. Returns a function that stops the watch. */
export function startStallWatch(now: () => number = () => Date.now()): () => void {
  clock = now;
  let last = now();
  // A hidden window's timers are throttled (about once a second): not a stall. A tick after
  // the window was hidden is skipped too.
  const hidden = () => typeof document !== "undefined" && document.hidden;
  let wasHidden = hidden();
  const onVisibility = () => (wasHidden = wasHidden || hidden());
  if (typeof document !== "undefined") document.addEventListener("visibilitychange", onVisibility);
  const id = setInterval(() => {
    try {
      const t = now();
      const held = t - last - STALL_TICK_MS;
      last = t;
      if (wasHidden || hidden()) {
        wasHidden = hidden();
        return;
      }
      if (held < STALL_LATE_MS) return;
      const start = t - held;
      const a = lastAction && lastAction.at >= start - STALL_ACTION_MS ? ` (last action ${lastAction.name})` : "";
      logWarn("ui.stall", `the window was held ${Math.round(held)} ms${a}`);
    } catch {
      /* never throw */
    }
  }, STALL_TICK_MS);
  return () => {
    clearInterval(id);
    if (typeof document !== "undefined") document.removeEventListener("visibilitychange", onVisibility);
  };
}

let installed = false;

/** Log uncaught errors and unhandled rejections, and start the stall watch. Call once. */
export function installPageLogging() {
  if (installed || typeof window === "undefined") return;
  installed = true;
  window.addEventListener("error", (e: ErrorEvent) => {
    const where = e.filename ? `uncaught error at ${e.filename.split("/").pop()}:${e.lineno}:${e.colno}` : "uncaught error";
    logError(where, e.error ?? e.message);
  });
  window.addEventListener("unhandledrejection", (e: PromiseRejectionEvent) => {
    logError("unhandled rejection", e.reason);
  });
  window.addEventListener("beforeunload", () => flushLog());
  startStallWatch();
}
