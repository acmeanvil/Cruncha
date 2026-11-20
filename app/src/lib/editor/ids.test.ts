// SPDX-License-Identifier: GPL-3.0-or-later
// Run with `npm test` (vitest).
import { test, assert } from "vitest";
import { keyBetween, ulid } from "./ids";

test("ulid is 26 Crockford characters and sorts by time", () => {
  const a = ulid(1_700_000_000_000);
  const b = ulid(1_700_000_000_001);
  assert.match(a, /^[0-9A-HJKMNP-TV-Z]{26}$/);
  assert.ok(a.slice(0, 10) < b.slice(0, 10));
});

test("keyBetween is strictly between, including awkward existing keys", () => {
  const cases: [string | null, string | null][] = [
    [null, null], ["a0", null], [null, "a0"], ["a0", "a1"], ["b1", "b2"], ["a0", "a00"],
    [null, "0"], [null, "1"], [null, "!0"], [null, "!!a"], ["z", null], ["zz", null], ["a", "b"],
    ["a1", "a1V"], ["01JWORKLOAD000000000000001", "01JWORKLOAD000000000000002"],
  ];
  for (const [a, b] of cases) {
    const k = keyBetween(a, b);
    if (a !== null) assert.ok(a < k, `${a} < ${k}`);
    if (b !== null) assert.ok(k < b, `${k} < ${b}`);
  }
});

test("repeated inserts stay ordered and short", () => {
  // Append 500 times, prepend 200 times, and always insert after the first key.
  const keys: string[] = [keyBetween(null, null)];
  for (let i = 0; i < 500; i++) keys.push(keyBetween(keys[keys.length - 1], null));
  for (let i = 0; i < 200; i++) keys.unshift(keyBetween(null, keys[0]));
  for (let i = 0; i < 200; i++) keys.splice(1, 0, keyBetween(keys[0], keys[1]));
  for (let i = 1; i < keys.length; i++) assert.ok(keys[i - 1] < keys[i], `order at ${i}`);
  assert.ok(Math.max(...keys.map((k) => k.length)) < 60);
  assert.throws(() => keyBetween("b", "a"));
  assert.throws(() => keyBetween(null, "!"), /no key below/);
});
