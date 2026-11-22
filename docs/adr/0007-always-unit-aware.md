# ADR-0007: Calculations are always unit-aware

**Status:** Accepted
**Date:** 2026-11-18
**Deciders:** acmeanvil

## Context

In Hurmet, a doubled display selector (`??`) switches on unit-aware evaluation. With `L = 3 ft`:
- `2 L = ?` gives **6**, the plain number as typed;
- `2 L = ??` gives **1.8288 m**.

So the same variable gives different numbers depending on the selector. Mathcad, EngineeringPaper and Numbat are always unit-aware.

## Decision

All Cruncha calculations are unit-aware. Selectors control **display only** (echo and result visibility). When importing Hurmet files, `??` is treated as `?`, and `?` statements whose result would change are flagged in the import report.

## Consequences

- **Easier:** one value per variable, and Python/Rust marshalling is unambiguous.
- **Harder:** some Hurmet documents that relied on plain numbers need fixing, and the import report lists them.
- Unitless functions (such as Hurmet's column-interaction module) need explicit `unitless(x, "in")`-style conversion.

## Action Items

1. [ ] Implement it in `cruncha-eval`; add the import-report rule to the Hurmet importer. *(Review 2026-11-22: partial. The engine always evaluates unit-aware (`UnitAware::Always`, app `engine.rs`); the import rule waits for the Hurmet importer (ADR-0002 item 3).)*
