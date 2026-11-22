// SPDX-License-Identifier: GPL-3.0-or-later
// The History tab's model: versions grouped by snapshot. A snapshot is a "major" version
// ("Rev A"); each edit after it is a minor one ("Rev A · 3") until the next snapshot. Edits
// before the first snapshot are "Draft · n". Maintenance versions (squashes, prune markers)
// get no number. When trimming removed part of a group's range, its edits can't be counted, so
// they keep only their version number.
import type { RefView, VersionView } from "./api";

export const DRAFT = "Draft";

export interface Entry {
  version: VersionView;
  /** "Rev A · 3", "Rev A" (the snapshot's own version), "Draft · 12"; null for maintenance or
   *  when the count isn't known. */
  label: string | null;
  /** Snapshots taken at this version. */
  tags: string[];
}

export interface Group {
  /** The snapshot that starts the group (null for the drafts before the first one). */
  snapshot: RefView | null;
  name: string;
  /** Newest first. */
  entries: Entry[];
  /** Edits (non-maintenance versions) in the group, not counting the snapshot's own version. */
  edits: number;
  /** Is the group's range complete (nothing before it was trimmed)? */
  complete: boolean;
  /** Newest and oldest timestamps of its versions (ms), if any are available. */
  newest: number | null;
  oldest: number | null;
}

const isEdit = (v: VersionView) => v.kind !== "maintenance";

/** Group `history` (any order) by `refs`. Newest group first; entries newest first. Versions
 *  older than `earliest` are gone; a snapshot kept through trimming still starts its group. */
export function groupHistory(history: VersionView[], refs: RefView[], earliest: number): Group[] {
  const versions = [...history].sort((a, b) => a.version - b.version);
  // One snapshot names each version where any were taken: the first one taken there.
  const byVersion = new Map<number, RefView[]>();
  for (const r of [...refs].sort((a, b) => a.createdAtMs - b.createdAtMs || a.name.localeCompare(b.name))) {
    const at = byVersion.get(r.version);
    if (at) at.push(r);
    else byVersion.set(r.version, [r]);
  }
  const bases = [...byVersion.keys()].sort((a, b) => a - b);
  const groups: Group[] = [];
  const start = (snapshot: RefView | null, from: number): Group => ({
    snapshot,
    name: snapshot?.name ?? DRAFT,
    entries: [],
    edits: 0,
    // The drafts start at version 0; a snapshot's group at its own version.
    complete: from >= earliest,
    newest: null,
    oldest: null,
  });
  let g = start(null, 0);
  let next = 0;
  for (const v of versions) {
    while (next < bases.length && bases[next] < v.version) {
      groups.push(g);
      g = start(byVersion.get(bases[next])![0], bases[next]);
      next++;
    }
    const tags = (byVersion.get(v.version) ?? []).map((r) => r.name);
    let label: string | null = null;
    if (isEdit(v) && v.prunedBefore == null) {
      g.edits++;
      if (g.complete) label = `${g.name} · ${g.edits}`;
    }
    // The version a snapshot was taken at is that snapshot ("Rev A"), shown at the head of the
    // next group, and still the last edit of this one.
    g.entries.push({ version: v, label, tags });
    g.newest = Math.max(g.newest ?? 0, v.timestampMs);
    g.oldest = Math.min(g.oldest ?? Infinity, v.timestampMs);
  }
  groups.push(g);
  // Snapshots after the last available version (or with nothing after them).
  for (; next < bases.length; next++) groups.push(start(byVersion.get(bases[next])![0], bases[next]));
  for (const x of groups) x.entries.reverse();
  // Drop an empty draft group (every version is after a snapshot, or the drafts were trimmed).
  return groups.filter((x) => x.snapshot || x.entries.length).reverse();
}

/** Split `groups` (newest first, from [`groupHistory`]) into those the History tab shows and
 *  the archived ones (ADR-0023): groups whose own versions are all before `earliest`. A
 *  group's own versions end before the next snapshot, so a group is archived when the next
 *  newer group's snapshot is at or before `earliest`. The newest group never is. */
export function splitArchived(groups: Group[], earliest: number): { shown: Group[]; archived: Group[] } {
  const shown: Group[] = [];
  const archived: Group[] = [];
  groups.forEach((g, i) => {
    const newer = i > 0 ? groups[i - 1].snapshot : null;
    if (earliest > 0 && newer && earliest >= newer.version) archived.push(g);
    else shown.push(g);
  });
  return { shown, archived };
}

/** The archive cutoff for archiving `shown[k]` and every older group: the version of the
 *  snapshot that starts the next newer group. Null for the newest group. */
export function archiveCutoff(shown: Group[], k: number): number | null {
  return k > 0 ? (shown[k - 1].snapshot?.version ?? null) : null;
}

/** The label for `version`: "Rev A" at a snapshot, "Rev A · 3" after one, "Draft · 2" before
 *  any; null when it can't be numbered. */
export function labelOf(groups: Group[], version: number): string | null {
  for (const g of groups) {
    if (g.snapshot?.version === version) return g.name;
    const e = g.entries.find((x) => x.version.version === version);
    if (e) return e.tags[0] ?? e.label;
  }
  return null;
}

/** Does `e` match the search text (label, message, actor, version or tag)? */
export function entryMatches(e: Entry, query: string): boolean {
  const q = query.trim().toLowerCase().replace(/^v(?=\d)/, "");
  if (!q) return true;
  const v = e.version;
  return (
    String(v.version) === q ||
    (e.label ?? "").toLowerCase().includes(q) ||
    v.message.toLowerCase().includes(q) ||
    v.actor.toLowerCase().includes(q) ||
    e.tags.some((t) => t.toLowerCase().includes(q))
  );
}
