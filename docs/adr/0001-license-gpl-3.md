# ADR-0001: License: GPL-3.0-or-later

**Status:** Accepted
**Date:** 2026-11-18
**Deciders:** acmeanvil

## Context

The repository started under MIT. The strongest open-source math engines we might use are GPL:
- **Giac**, a fast CAS (C++, with a WASM build);
- **GNU Octave**, MATLAB-compatible numerics;
- **libqalculate**, a large unit and constant database;
- **Maxima** and **SageMath**.

Under MIT we could use them only as separately installed processes, never linked in or bundled. Our planned permissive dependencies are all GPL-3 compatible:
- Typst, delta-rs, wasmtime (Apache-2.0)
- SymPy, NumPy, Polars (BSD/MIT)
- MathLive, Hurmet (MIT)

Apache-2.0 is compatible with GPLv3 but **not with GPLv2**.

## Decision

License Cruncha as **GPL-3.0-or-later**. The full text is in `LICENSE`, and the "or later" notice is in `README.md`. New source files carry `SPDX-License-Identifier: GPL-3.0-or-later`.

## Options Considered

### Option A: Keep MIT
| Dimension | Assessment |
| --- | --- |
| Complexity | High: every GPL engine needs an out-of-process protocol and must be installed separately |
| Cost | Low legal friction for downstream users |
| Scalability | n/a |
| Team familiarity | High |

**Pros:** maximum adoption; anyone can embed Cruncha in a closed product.
**Cons:** no in-process Giac, Octave or qalculate. Competitors can close-source forks.

### Option B: LGPL-3.0-or-later
**Pros:** changes to Cruncha stay open, but proprietary apps can link it.
**Cons:** linking a GPL engine still makes the combined work GPL, so it doesn't solve the engine problem. It is the most confusing option for contributors.

### Option C: GPL-3.0-or-later (chosen)
**Pros:** GPL engines can be linked or bundled; all derivatives stay open source.
**Cons:** closed-source plugins and proprietary embedding are impossible without relicensing.

### Option D: AGPL-3.0-or-later
**Pros:** also covers hosted SaaS forks.
**Cons:** it deters some corporate users, and we have no hosted product yet. We can revisit this if we build one.

## Trade-off Analysis

Engine availability (C) outweighs maximum downstream freedom (A). B gives neither benefit fully. The SaaS loophole (D) doesn't apply until there is a hosted service.

## Consequences

- **Easier:** shipping Giac in-process, including in the WASM web build; bundling Octave and qalculate data.
- **Harder:** commercial dual licensing, unless we keep the right to relicense contributions.
- **Revisit:** AGPL if a hosted Cruncha appears.

## Action Items

1. [x] Replace `LICENSE` with GPL-3.0; add the notice to the README.
2. [ ] Decide on a contribution policy (DCO only, or a CLA that keeps the right to relicense) **before accepting the first outside PR**.
3. [ ] Add `cargo-deny` license checks to CI. Deny GPL-2.0-only, proprietary and non-commercial licenses.
4. [ ] Verify the exact license wording ("or later"?) of Maxima, Giac, Octave and libqalculate before bundling any of them.
