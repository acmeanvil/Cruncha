# ADR-0004: Reactive dependency graph with one definition per name

**Status:** Accepted
**Date:** 2026-11-18
**Deciders:** acmeanvil

## Context

Hurmet evaluates calc cells in document order against a flat namespace in which later definitions shadow earlier ones. This causes two problems:
- Renaming a definition leaves stale results in the cells that read the old name.
- Python and Rust cells (ADR-0005, ADR-0006) must take part in the same recalculation. Jupyter's "whatever order you clicked" model is exactly the hidden-state problem we want to avoid.

marimo and Pluto show that a reactive graph works well for notebooks.

## Decision

One graph covers every cell kind:
- **Each name has exactly one defining cell.** A duplicate definition is an error on both cells.
- Cells evaluate in **topological order**. Using a name above its definition is allowed but gets a lint badge, because calc packages should read top to bottom.
- Cycles are errors.
- Invalidation compares a cell's `defines` before and after an edit.
- Results are memoized by `hash(source, inputs, engine version)`.
- The scheduler is async, with cancellation and timeouts.

## Options Considered

### Option A: Document order with shadowing (Hurmet, Mathcad)
**Pros:** familiar to Mathcad users; lets you reuse `M` for several checks.
**Cons:** order-dependent results; staleness bugs; hard to combine with code cells.

### Option B: Reactive graph with unique names (proposed)
**Pros:** deterministic; marimo-proven; cells can be moved freely.
**Cons:** Mathcad habits of reusing names produce errors.

### Option C: Reactive graph with shadowing resolved by position
**Pros:** keeps Mathcad habits.
**Cons:** complex semantics; edits far away silently change bindings.

## Trade-off Analysis

B is the only option whose results don't depend on where cells sit. The cost of reused names can be eased later with **section scopes**: a heading can declare that names defined inside it are local to it.

## Consequences

- Hurmet imports must rename or scope duplicate definitions. The importer reports these.
- Python cells need static analysis of the names they define and read (ADR-0005).

## Action Items

1. [ ] Spike S2: a graph over mixed calc and Python cells, with rename invalidation and cancellation.
2. [ ] Decide on section scopes after user testing.
