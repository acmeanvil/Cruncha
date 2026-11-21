# Architecture Decision Records

Each significant, hard-to-reverse decision gets one numbered ADR. ADRs are immutable once **Accepted**. To change a decision, write a new ADR that supersedes the old one and update the old one's status line only.

| ADR | Title | Status |
| --- | --- | --- |
| [0001](0001-license-gpl-3.md) | License: GPL-3.0-or-later | Accepted |
| [0002](0002-hurmet-document-model.md) | Hurmet as the basis of the document and calc model (rewritten in Rust) | Accepted |
| [0003](0003-hybrid-delta-lake-storage.md) | Hybrid Delta Lake container as the storage format | Accepted |
| [0004](0004-reactive-graph-unique-names.md) | Reactive dependency graph with one definition per name | Accepted |
| [0005](0005-python-out-of-process-worker.md) | Python cells run in an out-of-process worker | Accepted |
| [0006](0006-rust-cells-wasm.md) | Rust cells compile to WASM and run in wasmtime | Proposed |
| [0007](0007-always-unit-aware.md) | Calculations are always unit-aware | Accepted |
| [0008](0008-app-shell-tauri-web.md) | App shell: Tauri with a web UI (not GPUI) | Accepted |
| [0009](0009-agent-interface-acp-mcp.md) | Agent interface: ACP client, Cruncha MCP server, built-in agent from looking_glass | Proposed |
| [0010](0010-doc-table-append-only-log.md) | The `doc` table is an append-only revision log with periodic squash (refines 0003) | Accepted |
| [0011](0011-user-configurable-history-retention.md) | User-configurable history retention: saves, size and activity-relative limits; dormant documents are never pruned | Accepted |
| [0012](0012-ui-framework-svelte.md) | UI framework for the app shell: Svelte 5 with Vite and TypeScript (resolves 0008 item 2) | Accepted |
| [0013](0013-out-of-line-cell-content.md) | Large cell sources are stored out of line in content-addressed blobs (refines 0010, 0011; ground for deletion vectors) | Accepted |
| 0014 | *Reserved:* calculation templates (to be written after S6 and the agent, results, Rust-cell and polish work) | — |
| [0015](0015-reports-embedded-typst.md) | Reports are typeset by an embedded Typst, from the evaluated document | Proposed |
| [0016](0016-agent-proposals-table.md) | Agent proposals live in a `proposals/` table, applied by a human's commit (supersedes design 02 D3) | Accepted |
| [0017](0017-interactive-figures.md) | Interactive figures are JSON specs keyed by MIME type, drawn by sandboxed renderers (refines 0005) | Proposed |
| [0018](0018-results-table.md) | Code-cell results are kept in a `results/` table, keyed by what produced them | Accepted |
| [0019](0019-results-from-other-environments.md) | Stored results from another Python environment are found by an environment-free hash (refines 0018) | Proposed |

**Status values:** Proposed → Accepted | Rejected; Accepted → Deprecated | Superseded by NNNN.

Background reading: [landscape survey](../research/01-landscape-survey.md), [design 02: document, execution and storage model](../design/02-document-model.md).

## Template

```markdown
# ADR-NNNN: Title

**Status:** Proposed
**Date:** YYYY-MM-DD
**Deciders:** …

## Context
## Decision
## Options Considered
### Option A: …
| Dimension | Assessment |
| --- | --- |
| Complexity | |
| Cost | |
| Scalability | |
| Team familiarity | |

**Pros:** … **Cons:** …
## Trade-off Analysis
## Consequences
## Action Items
```
