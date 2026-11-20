# ADR-0008: App shell: Tauri with a web UI (not GPUI)

**Status:** Accepted (2026-11-18)
**Date:** 2026-11-18
**Deciders:** acmeanvil

## Context

Cruncha's main window needs:
- rich-text prose editing (ProseMirror-class), with inline calc cells;
- math rendering (MathML/TeX) and optional 2D math input (MathLive);
- editable tables and data grids, interactive plots, and PDF preview of Typst reports;
- IME and accessibility.

`looking_glass` (same owner, MIT) is a polished agent front end built on **GPUI** (Zed's GPU UI framework, Apache-2.0). It is the baseline for Cruncha's AI interface (ADR-0009), so a GPUI shell was considered. Review of GPUI 0.2.2 (see the looking_glass review notes in ADR-0009) found it lacks:
- a rich-text editor component (looking_glass's multi-line input alone is about 1,000 lines);
- math layout and 2D math input;
- a table editor, plotting, and a PDF viewer;
- a webview;
- accessibility support.

## Decision

Build the app on **Tauri**:
- The Rust backend hosts `cruncha-store`, the engine, the kernels, the MCP server and the agent runtime.
- The **web UI** provides ProseMirror, Temml/MathML, MathLive, plots and the PDF preview.

The agent panel is rebuilt in the web UI, following looking_glass's interaction design (ADR-0009).

## Options Considered

### Option A: Tauri + web (proposed)
| Dimension | Assessment |
| --- | --- |
| Complexity | Medium: two languages (Rust plus TS) and an IPC bridge |
| Cost | Mature editors and math libraries available off the shelf |
| Scalability | The webview handles documents of hundreds of cells; heavy compute stays in Rust |
| Team familiarity | Rust high; web front end medium |

**Pros:** every hard UI component already exists; the same UI ships as a pure web build with the engine in WASM.
**Cons:** not GPU-native; webview quirks differ per OS.

### Option B: GPUI native (like looking_glass)
**Pros:** very fast text and layout; one language; reuses looking_glass components directly.
**Cons:** we'd build a rich-text editor, math layout and editing, tables, plots and a PDF viewer from scratch. That is months of high-risk work. No accessibility. Zed's editor is GPL and lives outside GPUI.

### Option C: GPUI window that embeds a webview for the document
**Cons:** unsupported in GPUI; focus, IME, z-order and scrolling would cross two toolkits. Fragile.

### Option D (interim): looking_glass as a separate agent console next to the Cruncha app
**Pros:** works as soon as `cruncha-mcp` exists. It's a good bridge until the web agent panel lands.

## Trade-off Analysis

What makes looking_glass feel more modern than a terminal is its **interaction design**:
- streaming Markdown and tool cards;
- inline diffs before the permission prompt;
- Allow / Always / Reject;
- a plan panel, model menus, and a usage pill.

None of that depends on GPU rendering, and all of it can be reproduced in web UI. The document editor, though, *does* depend on web-only libraries.

## Consequences

- The front-end stack (TS, ProseMirror, Temml, MathLive) becomes a project dependency, so we need a front-end build in CI.
- looking_glass's GPUI view code is used as a UX spec, not as code.

## Action Items

1. [x] Tauri skeleton whose backend opens a `cruncha-store` container, with a read-only document view.
2. [x] Decide the front-end framework for the non-editor UI (Svelte, Solid or React) as part of that skeleton: Svelte 5 (ADR-0012).
