// SPDX-License-Identifier: GPL-3.0-or-later
import type { Retention } from "./api";

export function bytes(n: number): string {
  if (n < 1e3) return `${n} B`;
  if (n < 1e6) return `${(n / 1e3).toFixed(1)} kB`;
  if (n < 1e9) return `${(n / 1e6).toFixed(1)} MB`;
  return `${(n / 1e9).toFixed(2)} GB`;
}

export function date(ms: number): string {
  return new Date(ms).toLocaleString(undefined, {
    year: "numeric",
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

export function isKeepEverything(p: Retention): boolean {
  return p.saves == null && p.size_bytes == null && p.activity_days == null;
}

export function describeRetention(p: Retention): string {
  if (isKeepEverything(p)) return "Keep everything";
  const parts: string[] = [];
  if (p.saves != null) parts.push(`last ${p.saves.toLocaleString()} saves`);
  if (p.size_bytes != null) parts.push(`up to ${bytes(p.size_bytes)}`);
  if (p.activity_days != null) parts.push(`${p.activity_days} days of activity`);
  return parts.join(", ");
}

/** Plain text of a ProseMirror JSON node (prose cells store their content that way). */
export function proseText(source: string): string {
  try {
    const walk = (n: any): string =>
      typeof n?.text === "string" ? n.text : (n?.content ?? []).map(walk).join("");
    return walk(JSON.parse(source));
  } catch {
    return source;
  }
}
