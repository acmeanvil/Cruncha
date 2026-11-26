// SPDX-License-Identifier: GPL-3.0-or-later
import { describe, expect, it } from "vitest";
import type { LogRecord } from "./api";
import {
  capped,
  categoriesOf,
  countProblems,
  filterRows,
  formatRow,
  formatRows,
  matchIndices,
  parseFile,
  problemIndices,
  rowOf,
  segments,
  step,
  timeOf,
} from "./logView";

const rec = (seq: number, level: LogRecord["level"], category: string, message: string): LogRecord => ({
  seq, at: `2026-11-26T10:07:0${seq}.406`, level, category, message,
});

const rows = [
  rec(1, "info", "ui.action", "Insert calc"),
  rec(2, "debug", "cmd", "commit 12 ms"),
  rec(3, "warn", "ui.stall", "the window was held 800 ms"),
  rec(4, "error", "app", "panic at session.rs\nbacktrace line"),
  rec(5, "info", "cmd.slow", "export_report 2400 ms"),
].map(rowOf);

describe("filters", () => {
  it("filters by level", () => {
    const at = (level: "errors" | "warnings" | "info" | "all") =>
      filterRows(rows, { level, categories: null }).map((r) => r.key);
    expect(at("errors")).toEqual(["r4"]);
    expect(at("warnings")).toEqual(["r3", "r4"]);
    expect(at("info")).toEqual(["r1", "r3", "r4", "r5"]);
    expect(at("all")).toEqual(["r1", "r2", "r3", "r4", "r5"]);
  });

  it("filters by category; none chosen shows all", () => {
    expect(filterRows(rows, { level: "all", categories: new Set(["cmd", "app"]) }).map((r) => r.key)).toEqual(["r2", "r4"]);
    expect(filterRows(rows, { level: "all", categories: new Set() }).length).toBe(5);
  });

  it("lists categories and counts problems", () => {
    expect(categoriesOf(rows)).toEqual(["app", "cmd", "cmd.slow", "ui.action", "ui.stall"]);
    expect(countProblems(rows)).toEqual({ errors: 1, warnings: 1 });
  });

  it("keeps session markers through every filter", () => {
    const file = parseFile("=== Cruncha 0.1 started ===\n2026-11-26T10:07:03.406 DEBUG cmd       commit\n");
    expect(filterRows(file, { level: "errors", categories: new Set(["x"]) }).map((r) => r.message)).toEqual([
      "=== Cruncha 0.1 started ===",
    ]);
  });
});

describe("search", () => {
  it("splits text at case-insensitive matches", () => {
    expect(segments("Held 800 ms, held again", "HELD")).toEqual([
      { text: "Held", hit: true },
      { text: " 800 ms, ", hit: false },
      { text: "held", hit: true },
      { text: " again", hit: false },
    ]);
    expect(segments("abc", "")).toEqual([{ text: "abc", hit: false }]);
    expect(segments("abc", "abc")).toEqual([{ text: "abc", hit: true }]);
    expect(segments("", "x")).toEqual([{ text: "", hit: false }]);
  });

  it("finds rows by message or category", () => {
    expect(matchIndices(rows, "stall")).toEqual([2]);
    expect(matchIndices(rows, "MS")).toEqual([1, 2, 4]);
    expect(matchIndices(rows, "")).toEqual([]);
  });

  it("steps to the next and previous match, wrapping", () => {
    const m = [1, 4, 7];
    expect(step(m, -1, 1)).toBe(1);
    expect(step(m, 1, 1)).toBe(4);
    expect(step(m, 7, 1)).toBe(1);
    expect(step(m, -1, -1)).toBe(7);
    expect(step(m, 4, -1)).toBe(1);
    expect(step(m, 1, -1)).toBe(7);
    expect(step([], 3, 1)).toBe(-1);
  });

  it("finds problems", () => {
    expect(problemIndices(rows)).toEqual([2, 3]);
  });
});

describe("the file's line format", () => {
  it("formats rows as the backend writes them", () => {
    expect(formatRow(rows[2])).toBe("2026-11-26T10:07:03.406 WARN  ui.stall  the window was held 800 ms");
    expect(formatRow(rows[3])).toBe("2026-11-26T10:07:04.406 ERROR app       panic at session.rs\n  backtrace line");
    expect(formatRows([])).toBe("");
    expect(formatRows(rows.slice(0, 1))).toBe("2026-11-26T10:07:01.406 INFO  ui.action Insert calc\n");
  });

  it("parses the file back, with continuation lines and markers", () => {
    const text = [
      "=== Cruncha 0.1.0 started 2026-11-26T10:00:00 ===",
      formatRow(rows[3]),
      formatRow(rows[0]),
      "=== stopped ===",
      "",
    ].join("\n");
    const parsed = parseFile(text);
    expect(parsed.map((r) => [r.level, r.category, r.message])).toEqual([
      [null, "", "=== Cruncha 0.1.0 started 2026-11-26T10:00:00 ==="],
      ["error", "app", "panic at session.rs\nbacktrace line"],
      ["info", "ui.action", "Insert calc"],
      [null, "", "=== stopped ==="],
    ]);
    expect(formatRows(parsed)).toBe(text);
  });

  it("shows the time of day and keeps the newest", () => {
    expect(timeOf("2026-11-26T10:07:03.406")).toBe("10:07:03.406");
    expect(capped([1, 2, 3, 4], 2)).toEqual([3, 4]);
    expect(capped([1], 2)).toEqual([1]);
  });
});
