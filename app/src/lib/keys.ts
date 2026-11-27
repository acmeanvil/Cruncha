// SPDX-License-Identifier: GPL-3.0-or-later
// Shortcut labels for this platform: the UI writes them the Mac way (⌘⇧F), and on Windows
// and Linux they read Ctrl+Shift+F. The shortcuts themselves already take Ctrl for ⌘.

export const isMac: boolean =
  typeof navigator !== "undefined" && /Mac|iPhone|iPad/.test(navigator.platform || navigator.userAgent || "");

const MODS: [string, string][] = [
  ["⌃", "Ctrl"],
  ["⌘", "Ctrl"],
  ["⌥", "Alt"],
  ["⇧", "Shift"],
];
const KEYS: Record<string, string> = { "↵": "Enter", "⌫": "Backspace", "⎋": "Esc" };

/** `s` with every Mac shortcut (⌘⇧F, ⌥↑, ⇧↵) written for this platform (`mac`: as is). */
export function keys(s: string, mac: boolean = isMac): string {
  if (mac) return s;
  return s.replace(/([⌃⌘⌥⇧]+)(\S)/g, (_, mods: string, key: string) => {
    const names = [...new Set(MODS.filter(([m]) => mods.includes(m)).map(([, n]) => n))];
    return [...names, KEYS[key] ?? key.toUpperCase()].join("+");
  });
}
