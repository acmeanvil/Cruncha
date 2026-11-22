// SPDX-License-Identifier: GPL-3.0-or-later
import { expect, test } from "vitest";
import type { FunctionReference } from "./api";
import reference from "../../../crates/cruncha-eval/reference/functions.json";
import { findEntries, insertion } from "./functionRef";

const ref = reference as unknown as FunctionReference;
const entry = (name: string) => ref.entries.find((e) => e.name === name)!;

test("Insert types the function with the cursor inside, or the symbol", () => {
  expect(insertion(entry("lerp"))).toEqual(["lerp()", 1]);
  expect(insertion(entry("rand"))).toEqual(["rand()", 1]);
  expect(insertion(entry("sqrt"))).toEqual(["√", 0]);
  expect(insertion(entry("π"))).toEqual(["π", 0]);
  expect(insertion(entry("< ≤ > ≥ = ≠"))).toEqual([" ≤ ", 0]);
  // Every entry inserts something.
  for (const e of ref.entries) expect(insertion(e)[0].trim(), e.name).not.toBe("");
});

test("search ranks name matches first and filters by category", () => {
  expect(findEntries(ref.entries, "sin", null)[0].name).toBe("sin");
  expect(findEntries(ref.entries, "degrees", null).some((e) => e.name === "sind")).toBe(true);
  expect(findEntries(ref.entries, "", "Dates").map((e) => e.name)).toEqual(["today", "savedate"]);
  expect(findEntries(ref.entries, "interpolation", null)[0].name).toBe("lerp");
});
