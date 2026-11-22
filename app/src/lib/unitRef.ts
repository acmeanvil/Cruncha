// SPDX-License-Identifier: GPL-3.0-or-later
// The Units palette's model (UnitReference.svelte): search, prefixes, sizes in SI, and what
// Insert types. The catalog comes from cruncha_units::catalog (`unit_catalog`).
import type { CatalogUnit, UnitCatalog } from "./api";

const norm = (s: string) => s.toLowerCase().normalize("NFKD");

/** A unit as the list shows it: with a prefix when the search asked for one (`kN`). */
export interface UnitHit {
  unit: CatalogUnit;
  prefix: string;
}

/** Whether `prefix` (a symbol like "k", or a word like "kilo") applies to `u`. */
export function takesPrefix(u: CatalogUnit, prefix: string): boolean {
  if (!prefix) return true;
  if (!u.prefixes) return false;
  return u.prefixes === "word" ? prefix.length > 2 : true;
}

/** Units matching `query` in `quantity`. An exact name or alias comes first, then names that
 *  start with the query, then the rest. A query that is a prefix plus a unit (`kN`, `MPa`,
 *  `kilonewton`) also finds that unit, with the prefix. */
export function findUnits(cat: UnitCatalog, query: string, quantity: string | null): UnitHit[] {
  const raw = query.trim();
  const q = norm(raw);
  const pool = cat.units.filter((u) => !quantity || u.quantity === quantity);
  if (!q) return pool.map((unit) => ({ unit, prefix: "" }));
  const names = (u: CatalogUnit) => [u.name, ...u.aliases];
  const rank = (u: CatalogUnit) => {
    const ns = names(u).map(norm);
    if (ns.includes(q)) return 0;
    if (ns.some((n) => n.startsWith(q))) return 1;
    return 2;
  };
  const hits: UnitHit[] = pool
    .filter((u) => names(u).some((n) => norm(n).includes(q)) || norm(u.quantity).includes(q))
    .sort((a, b) => rank(a) - rank(b))
    .map((unit) => ({ unit, prefix: "" }));
  // Prefixed forms, matched case-sensitively (m is milli, M is mega).
  const prefixed: UnitHit[] = [];
  for (const p of cat.prefixes) {
    for (const [pre, kind] of [[p.symbol, "symbol"], [p.word, "word"]] as const) {
      if (!pre || !raw.startsWith(pre) || raw.length === pre.length) continue;
      const rest = raw.slice(pre.length);
      for (const unit of pool) {
        const ok = kind === "symbol" ? unit.prefixes === "symbol" && unit.name === rest
          : unit.prefixes !== null && names(unit).includes(rest);
        if (ok && !hits.some((h) => h.unit === unit && rank(unit) === 0)) prefixed.push({ unit, prefix: pre });
      }
    }
  }
  return [...prefixed, ...hits];
}

/** The text Insert types: the unit's name, quoted, with the prefix (`'kN'`). A prefix word
 *  goes with the unit's word form (`kilonewton` needs a unit that takes words). */
export function unitInsertion(h: UnitHit): string {
  const { unit, prefix } = h;
  if (!prefix) return `'${unit.name}'`;
  if (prefix.length > 2) {
    const word = [unit.name, ...unit.aliases].find((n) => /^[a-z]{3,}$/.test(n)) ?? unit.name;
    return `'${prefix}${word}'`;
  }
  return `'${prefix}${unit.name}'`;
}

/** A number as the list shows it: 6 significant digits, × 10ⁿ outside 1e-4 … 1e6. */
export function formatNumber(x: number): string {
  if (!Number.isFinite(x)) return String(x);
  if (x === 0) return "0";
  const a = Math.abs(x);
  if (a >= 1e-4 && a < 1e6) return String(Number(x.toPrecision(6)));
  const [m, e] = x.toExponential(5).split("e");
  const mant = String(Number(m));
  const sup = String(Number(e)).replace(/[-0-9]/g, (c) => "⁻⁰¹²³⁴⁵⁶⁷⁸⁹"["-0123456789".indexOf(c)]);
  return `${mant === "1" ? "" : `${mant} × `}10${sup}`;
}

/** "1 ft = 0.3048 m" (with the prefix applied), or why the unit can't be sized. */
export function sizeText(cat: UnitCatalog, h: UnitHit): string {
  const { unit, prefix } = h;
  if (unit.currency) return `currency (${unit.currency})`;
  const si = cat.quantities.find((q) => q.name === unit.quantity)?.si ?? "SI";
  const p = cat.prefixes.find((x) => x.symbol === prefix || x.word === prefix);
  const f = unit.siFactor * (p ? p.factor : 1);
  const shown = unitInsertion(h).slice(1, -1);
  if (unit.gauge) return `${shown} is an offset scale: (x + ${formatNumber(unit.gauge)}) × ${formatNumber(f)} ${si}`;
  return `1 ${shown} = ${formatNumber(f)}${si === "1" ? "" : ` ${si}`}`;
}
