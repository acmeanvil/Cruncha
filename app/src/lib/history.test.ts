// SPDX-License-Identifier: GPL-3.0-or-later
import { describe, expect, it } from "vitest";
import type { CommitKind, RefView, VersionView } from "./api";
import { DRAFT, archiveCutoff, entryMatches, groupHistory, labelOf, splitArchived } from "./history";

const v = (version: number, kind: CommitKind = "edit", message = ""): VersionView => ({
  version, timestampMs: 1000 * version, kind, actor: "nate", message, restoredFrom: null, prunedBefore: null,
});
const ref = (name: string, version: number, createdAtMs = version * 1000 + 1): RefView => ({
  name, version, message: "", actor: "nate", createdAtMs, pinned: false,
});

describe("groupHistory", () => {
  const history = [v(0, "create"), v(1), v(2), v(3, "maintenance"), v(4), v(5), v(6)];

  it("numbers edits within each snapshot, newest group first", () => {
    const g = groupHistory(history, [ref("Rev A", 2), ref("Rev B", 5)], 0);
    expect(g.map((x) => x.name)).toEqual(["Rev B", "Rev A", "Draft"]);
    expect(g[0].entries.map((e) => e.label)).toEqual(["Rev B · 1"]);
    expect(g[1].entries.map((e) => [e.version.version, e.label])).toEqual([
      [5, "Rev A · 2"], [4, "Rev A · 1"], [3, null],
    ]);
    expect(g[2].entries.map((e) => e.label)).toEqual(["Draft · 3", "Draft · 2", "Draft · 1"]);
    expect(g[2].entries[0].tags).toEqual(["Rev A"]);
    expect(g[1].edits).toBe(2);
  });

  it("labels a snapshot's own version with the snapshot", () => {
    const g = groupHistory(history, [ref("Rev A", 2)], 0);
    expect(labelOf(g, 2)).toBe("Rev A");
    expect(labelOf(g, 6)).toBe("Rev A · 3");
    expect(labelOf(g, 1)).toBe("Draft · 2");
    expect(labelOf(g, 3)).toBeNull();
  });

  it("has only drafts without snapshots", () => {
    const g = groupHistory(history, [], 0);
    expect(g).toHaveLength(1);
    expect(g[0].edits).toBe(6);
  });

  it("names a version with two snapshots by the first taken", () => {
    const g = groupHistory(history, [ref("Checked", 4, 9000), ref("Issued", 4, 8000)], 0);
    expect(g[0].name).toBe("Issued");
    expect(labelOf(g, 4)).toBe("Issued");
    expect(g[1].entries.find((e) => e.version.version === 4)!.tags).toEqual(["Issued", "Checked"]);
  });

  it("keeps a snapshot with nothing after it", () => {
    const g = groupHistory(history, [ref("Latest", 6)], 0);
    expect(g[0]).toMatchObject({ name: "Latest", edits: 0, entries: [] });
  });

  it("doesn't number groups whose start was trimmed", () => {
    const kept = history.filter((x) => x.version >= 4);
    const g = groupHistory(kept, [ref("Rev A", 2), ref("Rev B", 5)], 4);
    expect(g.map((x) => [x.name, x.complete])).toEqual([["Rev B", true], ["Rev A", false]]);
    expect(g[1].entries.map((e) => e.label)).toEqual([null, null]);
    expect(labelOf(g, 6)).toBe("Rev B · 1");
  });

  it("searches labels, messages and versions", () => {
    const g = groupHistory([v(0, "create"), v(1, "edit", "Beam span")], [ref("Rev A", 0)], 0);
    const e = g[0].entries[0];
    expect(entryMatches(e, "rev a · 1")).toBe(true);
    expect(entryMatches(e, "span")).toBe(true);
    expect(entryMatches(e, "v1")).toBe(true);
    expect(entryMatches(e, "v2")).toBe(false);
  });
});

it("archived groups leave the tab; the newest never does (ADR-0023)", () => {
  // Draft 0–2, Rev A at 3, Rev B at 6, Rev C at 9; versions to 11.
  const all = Array.from({ length: 12 }, (_, i) => v(i, i === 0 ? "create" : "edit"));
  const refs = [ref("Rev A", 3), ref("Rev B", 6), ref("Rev C", 9)];
  const names = (gs: { name: string }[]) => gs.map((g) => g.name);

  // Nothing archived: every group shows; archiving from Rev A down ends at Rev B's version.
  const g0 = groupHistory(all, refs, 0);
  const s0 = splitArchived(g0, 0);
  expect(names(s0.shown)).toEqual(["Rev C", "Rev B", "Rev A", DRAFT]);
  expect(s0.archived).toEqual([]);
  expect(archiveCutoff(s0.shown, 2)).toBe(6);
  expect(archiveCutoff(s0.shown, 0)).toBeNull();

  // Archived before Rev B (earliest 6): Draft and Rev A go, though Rev A's group listed v6.
  const kept = all.filter((x) => x.version >= 6);
  const s1 = splitArchived(groupHistory(kept, refs.map((r) => ({ ...r, pinned: r.version < 6 })), 6), 6);
  expect(names(s1.shown)).toEqual(["Rev C", "Rev B"]);
  expect(names(s1.archived)).toEqual(["Rev A"]);

  // Trimmed by a policy in the middle of a group (earliest 4): nothing is archived.
  const s2 = splitArchived(groupHistory(all.filter((x) => x.version >= 4), refs, 4), 4);
  expect(names(s2.shown)).toEqual(["Rev C", "Rev B", "Rev A"]);
  expect(s2.archived).toEqual([]);
});
