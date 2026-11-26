// SPDX-License-Identifier: GPL-3.0-or-later
// The Log viewer's model (ADR-0029 §3): rows from the live records or the log file, the level,
// category and text filters, search matches, problems, and the file's line format.
import type { LogRecord } from "./api";

export type Level = LogRecord["level"];
/** The level filter: Errors; Warnings and errors; Info (and above); All (Debug included). */
export type LevelFilter = "errors" | "warnings" | "info" | "all";

export const LEVEL_FILTERS: { value: LevelFilter; label: string }[] = [
  { value: "errors", label: "Errors" },
  { value: "warnings", label: "Warnings and errors" },
  { value: "info", label: "Info" },
  { value: "all", label: "All" },
];

const RANK: Record<Level, number> = { error: 0, warn: 1, info: 2, debug: 3 };
const MAX_RANK: Record<LevelFilter, number> = { errors: 0, warnings: 1, info: 2, all: 3 };
const LABEL: Record<Level, string> = { error: "ERROR", warn: "WARN", info: "INFO", debug: "DEBUG" };

/** One line of the viewer: a record, or (from the file) a session marker or stray line. */
export interface LogRow {
  key: string;
  /** Null for a session marker or a line that isn't a record. */
  level: Level | null;
  at: string;
  category: string;
  message: string;
}

export function rowOf(r: LogRecord): LogRow {
  return { key: `r${r.seq}`, level: r.level, at: r.at, category: r.category, message: r.message };
}

export function passesLevel(level: Level | null, filter: LevelFilter): boolean {
  return level === null || RANK[level] <= MAX_RANK[filter];
}

export interface Filter {
  level: LevelFilter;
  /** Categories to show; null or empty: all of them. */
  categories: ReadonlySet<string> | null;
}

/** The rows the filters let through (markers always pass). Search doesn't hide rows; it marks them. */
export function filterRows(rows: readonly LogRow[], f: Filter): LogRow[] {
  const cats = f.categories && f.categories.size ? f.categories : null;
  return rows.filter((r) => passesLevel(r.level, f.level) && (r.level === null || !cats || cats.has(r.category)));
}

/** The categories seen, sorted. */
export function categoriesOf(rows: readonly LogRow[]): string[] {
  const s = new Set<string>();
  for (const r of rows) if (r.level !== null && r.category) s.add(r.category);
  return [...s].sort();
}

/** Errors and warnings among `rows`. */
export function countProblems(rows: readonly LogRow[]): { errors: number; warnings: number } {
  let errors = 0, warnings = 0;
  for (const r of rows) {
    if (r.level === "error") errors++;
    else if (r.level === "warn") warnings++;
  }
  return { errors, warnings };
}

/** A piece of text, marked if it matches the search. */
export interface Segment {
  text: string;
  hit: boolean;
}

/** `text` split at the case-insensitive matches of `query`. */
export function segments(text: string, query: string): Segment[] {
  const q = query.toLowerCase();
  if (!q) return [{ text, hit: false }];
  const lower = text.toLowerCase();
  const out: Segment[] = [];
  let i = 0;
  for (;;) {
    const j = lower.indexOf(q, i);
    if (j < 0) break;
    if (j > i) out.push({ text: text.slice(i, j), hit: false });
    out.push({ text: text.slice(j, j + q.length), hit: true });
    i = j + q.length;
  }
  if (i < text.length || !out.length) out.push({ text: text.slice(i), hit: false });
  return out;
}

/** Whether a row matches the search (in its category or message). */
export function rowMatches(r: LogRow, query: string): boolean {
  if (!query) return false;
  const q = query.toLowerCase();
  return r.category.toLowerCase().includes(q) || r.message.toLowerCase().includes(q);
}

/** Indices (into `rows`) of the rows matching the search. */
export function matchIndices(rows: readonly LogRow[], query: string): number[] {
  const out: number[] = [];
  rows.forEach((r, i) => rowMatches(r, query) && out.push(i));
  return out;
}

/** Indices of the errors and warnings. */
export function problemIndices(rows: readonly LogRow[]): number[] {
  const out: number[] = [];
  rows.forEach((r, i) => (r.level === "error" || r.level === "warn") && out.push(i));
  return out;
}

/** The next (dir 1) or previous (−1) of `indices` after `current` (−1: none yet), wrapping
 *  around; −1 if there are none. */
export function step(indices: readonly number[], current: number, dir: 1 | -1): number {
  if (!indices.length) return -1;
  if (dir === 1) return indices.find((i) => i > current) ?? indices[0];
  for (let k = indices.length - 1; k >= 0; k--) if (indices[k] < current) return indices[k];
  return indices[indices.length - 1];
}

/** A row as the log file writes it: `at LEVEL category message`, continuation lines indented. */
export function formatRow(r: LogRow): string {
  if (r.level === null) return r.message;
  const msg = r.message.replace(/\n/g, "\n  ");
  return `${r.at} ${LABEL[r.level].padEnd(5)} ${r.category.padEnd(9)} ${msg}`;
}

export function formatRows(rows: readonly LogRow[]): string {
  return rows.map(formatRow).join("\n") + (rows.length ? "\n" : "");
}

const LINE = /^(\d{4}-\d\d-\d\dT[\d:.]+) (ERROR|WARN|INFO|DEBUG) +(\S+)(?: +(.*))?$/;
const LEVELS: Record<string, Level> = { ERROR: "error", WARN: "warn", INFO: "info", DEBUG: "debug" };

/** The log file's text as rows: records (with their continuation lines), session markers and
 *  any other line as it is. */
export function parseFile(text: string): LogRow[] {
  const rows: LogRow[] = [];
  const lines = text.split("\n");
  if (lines.length && lines[lines.length - 1] === "") lines.pop();
  lines.forEach((line, n) => {
    const m = LINE.exec(line);
    if (m) {
      rows.push({ key: `f${n}`, level: LEVELS[m[2]], at: m[1], category: m[3], message: m[4] ?? "" });
      return;
    }
    const prev = rows[rows.length - 1];
    if (line.startsWith("  ") && prev && prev.level !== null) {
      prev.message += "\n" + line.slice(2);
      return;
    }
    rows.push({ key: `f${n}`, level: null, at: "", category: "", message: line });
  });
  return rows;
}

/** The time of day of a record's `at` (`10:07:03.406`). */
export function timeOf(at: string): string {
  const t = at.indexOf("T");
  return t < 0 ? at : at.slice(t + 1);
}

/** Keep the newest `cap` items. */
export function capped<T>(items: T[], cap: number): T[] {
  return items.length > cap ? items.slice(items.length - cap) : items;
}
