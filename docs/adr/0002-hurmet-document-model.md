# ADR-0002: Hurmet as the basis of the document and calc model (rewritten in Rust)

**Status:** Accepted
**Date:** 2026-11-18
**Deciders:** acmeanvil

## Context

Cruncha combines Mathcad-style readable, unit-aware calculations with Jupyter-style code blocks. Of the open-source tools surveyed ([01-landscape-survey](../research/01-landscape-survey.md)), **Hurmet** (MIT, JS, ProseMirror) has the best document model for engineering calc packages:
- rich prose with calc cells inline and as blocks;
- statements of the form `name = expr = ?? unit`;
- display selectors (`?` full with a "blue echo" of substituted values, `%`, `@`, `!`);
- exact rationals and an 8-dimension unit system;
- data frames with a units row;
- `@test` assertions;
- headless batch mode;
- results saved in the file so non-Hurmet readers (and AI) can see them.

A source review (design 02 §1) also found structural weaknesses:
- evaluation follows document order with name shadowing;
- renaming a variable doesn't invalidate its readers (stale results);
- the intermediate form is string-encoded RPN;
- run-time state lives in ProseMirror node attrs, and cells have no stable IDs;
- pagination measures the browser DOM;
- the Markdown dialect uses delimiters that are hard to type.

## Decision

Adopt Hurmet's **document model, calc syntax, display selectors, unit data and test corpus** as Cruncha's starting point, and **re-implement the engine in Rust**. We do not fork the JS codebase. Fix the weaknesses listed above in the re-implementation (design 02 §1 has the keep/change/replace table). Cruncha will import Hurmet `.md` files. Hurmet's MIT copyright notice is kept wherever its code or data is ported.

## Options Considered

### Option A: Fork Hurmet (JS) and extend
| Dimension | Assessment |
| --- | --- |
| Complexity | Low at first, high later (a JS engine core conflicts with the Rust/Python goals) |
| Cost | Fastest demo |
| Scalability | Single-threaded JS; no path to Python/Rust kernels in-process |
| Team familiarity | Medium |

**Pros:** working app on day one.
**Cons:** we would inherit the evaluation-order and staleness problems and the string RPN. The core would be in the wrong language.

### Option B: Use Hurmet's model, rewrite in Rust (chosen)
**Pros:** a sound engine (graph, typed AST, stable IDs) that can be shared by desktop, web (WASM), CLI and the Python package. Hurmet's ~450 golden tests become our syntax conformance suite.
**Cons:** large up-front effort; the parser is about 2,600 lines of subtle grammar.

### Option C: Design a new syntax (Calcpad-like or Mathcad-like)
**Pros:** no compatibility burden.
**Cons:** throws away a proven, reviewer-friendly notation and its test corpus.

## Trade-off Analysis

Option A is faster now and slower forever. Option B costs a parser port but unlocks every other goal: Python/Rust cells, a reactive graph, and Delta storage.

## Consequences

- **Easier:** reviewable calc output (blue echo), a familiar notation for Hurmet users, conformance testing.
- **Harder:** compatibility edge cases (implicit multiplication, unit-vs-variable spacing).
- **Revisit:** where we deliberately differ from Hurmet (ADR-0004 names, ADR-0007 units). Document those differences in import notes.

## Action Items

1. [x] Spike S1: port the lexer and parser against Hurmet's 217 calc golden tests.
2. [ ] Port the unit table, fixing the defects found in review (℉ gauge, `Å` exponent arity), and merge it with Numbat's units.
3. [ ] Hurmet `.md` importer in `cruncha-store`.
