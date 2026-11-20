// SPDX-License-Identifier: GPL-3.0-or-later
// Cell ids (ULIDs, like the store's) and fractional order keys.

const CROCKFORD = "0123456789ABCDEFGHJKMNPQRSTVWXYZ";

/** A new ULID: 48-bit millisecond time plus 80 random bits, Crockford base32. */
export function ulid(now = Date.now()): string {
  let time = "";
  let t = now;
  for (let i = 0; i < 10; i++) {
    time = CROCKFORD[t % 32] + time;
    t = Math.floor(t / 32);
  }
  const rand = new Uint8Array(16);
  crypto.getRandomValues(rand);
  let tail = "";
  for (let i = 0; i < 16; i++) tail += CROCKFORD[rand[i] % 32];
  return time + tail;
}

// Generated keys use '1'..'z'. Existing keys may contain any printable ASCII, and the store
// sorts them by byte order, which is what JavaScript's `<` does for ASCII.
const GEN_LO = 49; // '1'
const GEN_HI = 122; // 'z'
const MIN_CHAR = 33; // '!'

/**
 * A key strictly between `a` and `b` in byte order (`null`: unbounded). Keys never need
 * renumbering: there is always room between two different keys.
 */
export function keyBetween(a: string | null, b: string | null): string {
  const lower = a ?? "";
  let upper = b;
  if (upper !== null && !(lower < upper)) {
    throw new Error(`keyBetween: ${JSON.stringify(a)} is not below ${JSON.stringify(b)}`);
  }
  let out = "";
  for (let i = 0; ; i++) {
    const ca = i < lower.length ? lower.charCodeAt(i) : -1; // -1: `lower` is used up
    const cb = upper !== null && i < upper.length ? upper.charCodeAt(i) : 1000; // no bound
    const lo = Math.max(ca + 1, GEN_LO);
    const hi = Math.min(cb - 1, GEN_HI);
    if (lo <= hi) {
      // Appending at the end grows slowly: pick a character near the low end.
      const pick = upper === null ? Math.min(lo + 8, hi) : Math.floor((lo + hi) / 2);
      return out + String.fromCharCode(pick);
    }
    if (ca >= 0) {
      out += lower[i];
      if (ca < cb) upper = null; // `out` is now below `upper` whatever follows
    } else if (cb - 1 >= MIN_CHAR) {
      out += String.fromCharCode(cb - 1); // below `upper` at this position
      upper = null;
    } else {
      // Only keys made entirely of the lowest character leave no room below them.
      if (i + 1 >= upper!.length) throw new Error(`keyBetween: no key below ${JSON.stringify(b)}`);
      out += upper![i];
    }
  }
}
