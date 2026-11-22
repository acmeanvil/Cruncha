// SPDX-License-Identifier: GPL-3.0-or-later
import { expect, test } from "vitest";
import type { CatalogUnit, UnitCatalog } from "./api";
import { findUnits, formatNumber, sizeText, unitInsertion } from "./unitRef";

const u = (name: string, quantity: string, siFactor: number, more: Partial<CatalogUnit> = {}): CatalogUnit => ({
  name, aliases: [], quantity, siFactor, gauge: 0, prefixes: null, currency: null, ...more,
});

const cat: UnitCatalog = {
  quantities: [
    { name: "Length", si: "m" }, { name: "Force", si: "N" }, { name: "Pressure and stress", si: "Pa" },
    { name: "Temperature", si: "K" }, { name: "Currency", si: "currency" },
  ],
  prefixes: [
    { symbol: "M", word: "mega", factor: 1e6 }, { symbol: "k", word: "kilo", factor: 1e3 },
    { symbol: "m", word: "milli", factor: 1e-3 },
  ],
  units: [
    u("m", "Length", 1, { aliases: ["meter", "metre"], prefixes: "symbol" }),
    u("ft", "Length", 0.3048, { aliases: ["foot", "feet"] }),
    u("N", "Force", 1, { aliases: ["newton"], prefixes: "symbol" }),
    u("kip", "Force", 4448.2216152605),
    u("Pa", "Pressure and stress", 1, { aliases: ["pascal"], prefixes: "symbol" }),
    u("°F", "Temperature", 5 / 9, { aliases: ["Fahrenheit"], gauge: 459.67 }),
    u("USD", "Currency", 1, { aliases: ["$"], currency: "USD" }),
  ],
};

test("search finds names and aliases, exact matches first, by quantity", () => {
  expect(findUnits(cat, "feet", null)[0].unit.name).toBe("ft");
  expect(findUnits(cat, "", "Force").map((h) => h.unit.name)).toEqual(["N", "kip"]);
  expect(findUnits(cat, "force", null).map((h) => h.unit.name)).toEqual(["N", "kip"]);
  expect(findUnits(cat, "m", null)[0].unit.name).toBe("m");
});

test("a prefixed query finds the unit with its prefix, case-sensitively", () => {
  const kn = findUnits(cat, "kN", null)[0];
  expect([kn.unit.name, kn.prefix]).toEqual(["N", "k"]);
  expect(unitInsertion(kn)).toBe("'kN'");
  const mpa = findUnits(cat, "MPa", null)[0];
  expect([mpa.unit.name, mpa.prefix, sizeText(cat, mpa)]).toEqual(["Pa", "M", "1 MPa = 10⁶ Pa"]);
  expect(findUnits(cat, "mPa", null)[0].prefix).toBe("m");
  // kip takes no prefix: "kkip" finds nothing prefixed.
  expect(findUnits(cat, "kkip", null)).toEqual([]);
  expect(unitInsertion({ unit: cat.units[2], prefix: "kilo" })).toBe("'kilonewton'");
});

test("sizes in SI, offset scales and currencies", () => {
  expect(sizeText(cat, { unit: cat.units[1], prefix: "" })).toBe("1 ft = 0.3048 m");
  expect(sizeText(cat, { unit: cat.units[3], prefix: "" })).toBe("1 kip = 4448.22 N");
  expect(sizeText(cat, { unit: cat.units[5], prefix: "" })).toMatch(/offset scale: \(x \+ 459\.67\) × 0\.555556 K/);
  expect(sizeText(cat, { unit: cat.units[6], prefix: "" })).toBe("currency (USD)");
});

test("numbers: 6 significant digits, powers of ten outside 1e-4 … 1e6", () => {
  expect(formatNumber(0.3048)).toBe("0.3048");
  expect(formatNumber(1e-6)).toBe("10⁻⁶");
  expect(formatNumber(1609344)).toBe("1.60934 × 10⁶");
  expect(formatNumber(6894.757293168)).toBe("6894.76");
});
