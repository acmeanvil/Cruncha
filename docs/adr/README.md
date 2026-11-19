# Architecture Decision Records

Each significant, hard-to-reverse decision gets one numbered ADR. ADRs are immutable once **Accepted**. To change a decision, write a new ADR that supersedes the old one and update the old one's status line only.

| ADR | Title | Status |
| --- | --- | --- |
| [0001](0001-license-gpl-3.md) | License: GPL-3.0-or-later | Accepted |
| [0002](0002-hurmet-document-model.md) | Hurmet as the basis of the document and calc model (rewritten in Rust) | Accepted |
| [0003](0003-hybrid-delta-lake-storage.md) | Hybrid Delta Lake container as the storage format | Accepted |
| [0004](0004-reactive-graph-unique-names.md) | Reactive dependency graph with one definition per name | Proposed |
| [0005](0005-python-out-of-process-worker.md) | Python cells run in an out-of-process worker | Proposed |
| [0006](0006-rust-cells-wasm.md) | Rust cells compile to WASM and run in wasmtime | Proposed |
| [0007](0007-always-unit-aware.md) | Calculations are always unit-aware | Proposed |
| [0008](0008-app-shell-tauri-web.md) | App shell: Tauri with a web UI (not GPUI) | Accepted |
| [0009](0009-agent-interface-acp-mcp.md) | Agent interface: ACP client, Cruncha MCP server, built-in agent from looking_glass | Proposed |

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
