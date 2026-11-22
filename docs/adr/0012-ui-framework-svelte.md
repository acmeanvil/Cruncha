# ADR-0012: UI framework for the app shell: Svelte 5 with Vite and TypeScript

**Status:** Accepted (2026-11-19)
**Date:** 2026-11-19
**Deciders:** acmeanvil
**Resolves:** ADR-0008 action item 2 (Svelte, Solid or React for the non-editor UI).

## Context

ADR-0008 puts the UI in a Tauri webview. The document editor itself will be ProseMirror with Temml and MathLive, which are framework-agnostic: each owns its DOM and is mounted into a host element. The framework only drives the **rest** of the UI:
- the toolbar, panels, history list and dialogs (Preferences, document settings, confirmations);
- the agent panel (ADR-0009): streaming Markdown, tool cards, inline diffs, permission prompts;
- result tables, data grids and plot frames around the editor.

Constraints:
- every dependency must be GPL-3 compatible;
- the backend is Rust, and the owner's front-end familiarity is medium (ADR-0008);
- the same UI should later ship as a pure web build, with the engine in WASM.

The skeleton (ADR-0008 item 1) needed a choice in order to exist, so it uses Svelte. This ADR asks for it to be confirmed or swapped while the UI is still about 1,000 lines.

## Decision

Use **Svelte 5** (runes) with **Vite** and **TypeScript**, in `app/`. All three are MIT or Apache-2.0.

- No SvelteKit: Tauri serves a single-page app, and routing isn't needed.
- No component library yet: plain CSS with light and dark tokens (`app/src/app.css`).
- ProseMirror, Temml and MathLive are wrapped as Svelte components that own a host element.

## Options Considered

### Option A: Svelte 5 (chosen for the skeleton)
| Dimension | Assessment |
| --- | --- |
| Complexity | Low: components are close to HTML; runes make reactivity explicit |
| Cost | Small runtime (the skeleton's JS bundle is about 60 kB, 22 kB gzipped) |
| Scalability | Fine-grained updates suit long, streaming lists (agent transcript, history) |
| Team familiarity | Medium; the learning curve is the gentlest of the three |

**Pros:** little boilerplate; scoped CSS built in; Tauri's templates support it first-class.
**Cons:** smaller ecosystem than React. The runes API is new (late 2024), so older examples online are out of date.

### Option B: React
**Pros:** largest ecosystem: ready-made data grids, ProseMirror bindings and agent-UI kits; the most hiring familiarity.
**Cons:** more boilerplate; re-render performance needs care for streaming transcripts; a bigger runtime.

### Option C: Solid
**Pros:** fine-grained reactivity like Svelte, with JSX.
**Cons:** the smallest ecosystem of the three, for little gain over Svelte here.

## Trade-off Analysis

The hard UI parts (editor, math, plots, PDF) are framework-independent, so the framework's ecosystem matters less than usual. The deciding factors are the amount of code to write and maintain around them, and smooth streaming updates in the agent panel. Svelte wins on both. React would win if we wanted a heavy off-the-shelf data grid or agent-UI kit. Even then, most grids, such as AG Grid Community (MIT) and Glide (MIT), ship framework-free cores.

## Consequences

- CI needs Node: `npm ci`, `npm run check` (svelte-check) and `npm run build` in `app/`.
- Switching later means rewriting the components, but not the editor integrations or the backend commands (`app/src/lib/api.ts` is plain TypeScript).

## Action Items

1. [x] Skeleton in `app/` (Svelte 5, Vite, TypeScript) with a typed command layer (`src/lib/api.ts`).
2. [x] Owner: accepted 2026-11-19.
3. [x] Add the front-end check and build to CI when CI exists (2026-11-21).
