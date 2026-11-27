// SPDX-License-Identifier: GPL-3.0-or-later
import { describe, expect, it } from "vitest";
import { keys } from "./keys";

describe("keys", () => {
  it("leaves Mac labels alone on a Mac", () => {
    expect(keys("Functions (⌘⇧F)", true)).toBe("Functions (⌘⇧F)");
  });
  it("writes them out elsewhere", () => {
    expect(keys("Functions (⌘⇧F)", false)).toBe("Functions (Ctrl+Shift+F)");
    expect(keys("The debug log (⌘⌥L)", false)).toBe("The debug log (Ctrl+Alt+L)");
    expect(keys("Move block up (⌥↑)", false)).toBe("Move block up (Alt+↑)");
    expect(keys("Search (↵ next, ⇧↵ previous)", false)).toBe("Search (↵ next, Shift+Enter previous)");
    expect(keys("Page break (⌘⇧↵, or type \\pagebreak)", false)).toBe("Page break (Ctrl+Shift+Enter, or type \\pagebreak)");
    expect(keys("Settings (⌘,)", false)).toBe("Settings (Ctrl+,)");
    expect(keys("Python cell (⌘⌥P); not run yet", false)).toBe("Python cell (Ctrl+Alt+P); not run yet");
  });
});
