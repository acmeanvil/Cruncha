// SPDX-License-Identifier: GPL-3.0-or-later
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  FLUSH_MS,
  describeError,
  flushLog,
  lastLoggedAction,
  logAction,
  logError,
  logWarn,
  setLogSender,
  startStallWatch,
  type PageRecord,
} from "./log";

let sent: PageRecord[][];

beforeEach(() => {
  vi.useFakeTimers();
  sent = [];
  setLogSender(async (records) => void sent.push(records));
  flushLog();
  sent = [];
});

afterEach(() => {
  flushLog();
  vi.useRealTimers();
});

describe("batching", () => {
  it("sends actions and warnings together after 500 ms", () => {
    logAction("Insert calc");
    logWarn("ui.cmd", "command x failed: no");
    expect(sent).toEqual([]);
    vi.advanceTimersByTime(FLUSH_MS - 1);
    expect(sent).toEqual([]);
    vi.advanceTimersByTime(1);
    expect(sent).toEqual([
      [
        { level: "info", category: "ui.action", message: "Insert calc" },
        { level: "warn", category: "ui.cmd", message: "command x failed: no" },
      ],
    ]);
    vi.advanceTimersByTime(FLUSH_MS * 3);
    expect(sent.length).toBe(1);
  });

  it("sends an error at once, with what was waiting", () => {
    logAction("Run code");
    logError("uncaught error", new Error("boom"));
    expect(sent.length).toBe(1);
    expect(sent[0].map((r) => r.level)).toEqual(["info", "error"]);
    expect(sent[0][1].category).toBe("ui");
    expect(sent[0][1].message).toContain("uncaught error: ");
    expect(sent[0][1].message).toContain("boom");
  });

  it("never throws, even when sending fails", () => {
    setLogSender(() => {
      throw new Error("no backend");
    });
    expect(() => logError("x", "y")).not.toThrow();
    setLogSender(() => Promise.reject(new Error("rejected")));
    expect(() => logError("x", "y")).not.toThrow();
  });

  it("describes non-errors", () => {
    expect(describeError("plain")).toBe("plain");
    expect(describeError({ a: 1 })).toBe('{"a":1}');
    expect(describeError(undefined)).toBe("undefined");
  });
});

describe("stall watch", () => {
  it("logs a tick that arrives 500 ms late, naming a recent action", () => {
    let now = 10_000;
    const stop = startStallWatch(() => now);
    // Normal ticks: nothing.
    for (let i = 0; i < 5; i++) {
      now += 100;
      vi.advanceTimersByTime(100);
    }
    flushLog();
    expect(sent).toEqual([]);

    logAction("Insert template a.b 1.0.0 (3 cells)");
    sent = [];
    // The window is held for 800 ms: the next tick comes 800 ms late.
    now += 900;
    vi.advanceTimersByTime(100);
    flushLog();
    const stalls = sent.flat().filter((r) => r.category === "ui.stall");
    expect(stalls).toEqual([
      { level: "warn", category: "ui.stall", message: "the window was held 800 ms (last action Insert template a.b 1.0.0 (3 cells))" },
    ]);
    stop();
  });

  it("doesn't name an action from long before, or log a small delay", () => {
    let now = 50_000;
    const stop = startStallWatch(() => now);
    logAction("Old action");
    expect(lastLoggedAction()?.at).toBe(50_000);
    // 3 s of normal ticks, then a 4.9 s hold: the action was more than 2 s before it.
    for (let i = 0; i < 30; i++) {
      now += 100;
      vi.advanceTimersByTime(100);
    }
    now += 5_000;
    vi.advanceTimersByTime(100);
    now += 500; // 400 ms late: not a stall
    vi.advanceTimersByTime(100);
    flushLog();
    const stalls = sent.flat().filter((r) => r.category === "ui.stall");
    expect(stalls.map((r) => r.message)).toEqual(["the window was held 4900 ms"]);
    stop();
  });
});
