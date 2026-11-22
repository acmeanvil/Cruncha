# ADR-0020: Figure frames are served from their own scheme, with their own CSP; the renderer arrives by `postMessage`

**Status:** Proposed
**Date:** 2026-11-21
**Deciders:** acmeanvil
**Refines:** ADR-0017 §3

## Context

ADR-0017 draws each figure in an `<iframe sandbox="allow-scripts" srcdoc=…>` under "its own CSP". Spike S7 ([write-up](../spikes/S7-figures.md)) found two problems with that in WKWebView:

- **A `srcdoc` frame can't have its own CSP.** It inherits the app page's policy, and a policy can only be tightened.
- **plotly.js's `scattergl` needs `'unsafe-eval'`:** regl compiles shaders with `Function`.
  - Under the app's CSP it draws nothing, and raises no error.
  - `scattergl` is the only way to show large scatters at interactive speed: 0.2 s for 100k points, against 2.0 s for SVG `scatter`.

Allowing eval in the app page would weaken the page that holds the Tauri IPC. A sandboxed frame loaded with `src` from the app's own scheme can't load scripts at all, because WebKit doesn't match `'self'` for an opaque-origin document.

## Decision

1. **A `cruncha-figure` URI scheme**, registered by the app (`register_uri_scheme_protocol`).
   - It serves one page: the frame host, a small HTML file and script compiled into the binary.
   - The response's CSP is `default-src 'none'; script-src <figure origin> 'unsafe-eval'; style-src 'unsafe-inline'; img-src data: blob:`.
   - The figure origin is `cruncha-figure://localhost` on macOS and Linux, and `http://cruncha-figure.localhost` on Windows.
   - It serves nothing else, and nothing from the document.
2. **The frame stays sandboxed** (`sandbox="allow-scripts"`, no `allow-same-origin`), so its origin is opaque. The spike confirmed it can't:
   - read the parent, Tauri's IPC or storage;
   - reach the network, including the app's own scheme.

   The app page's CSP gains only `frame-src` for the figure origin.
3. **The renderer arrives by `postMessage`.**
   - The app fetches plotly.js (a pinned npm dependency in its own bundle) once, lazily.
   - It posts the code to each new frame; the frame evaluates it only from `event.source === parent`, then accepts specs.
   - Loading costs the same as a `<script>` tag (≈ 300 ms). The Rust build doesn't depend on `node_modules`, and the renderer's version is the npm lock's.
4. **Everything else in ADR-0017 §3 stands:**
   - specs are sanitized in the app before they are posted;
   - the parent accepts only messages from the frame it created, and only of the expected shapes;
   - the registry maps a MIME type to a renderer.

## Options Considered

### Option A: `srcdoc` under the app's CSP, no `scattergl` (ADR-0017 as written)
**Pros:** no new scheme.
**Cons:** large scatters are ten times slower (they'd be converted to SVG `scatter`), or don't draw.

### Option B: allow `'unsafe-eval'` in the app's CSP
**Pros:** least code.
**Cons:** eval becomes available in the page that can call Tauri commands; this is the weakening ADR-0017 exists to avoid.

### Option C: own scheme, own CSP, renderer served by the scheme from bundled resources
**Pros:** a plain `<script>` tag.
**Cons:**
- the app's resources must include `node_modules/plotly.js-dist-min`, so building the Rust crate needs `npm install`;
- two copies of the renderer (the bundle and the resources).

### Option D: own scheme, own CSP, renderer by `postMessage` (chosen)
| Dimension | Assessment |
| --- | --- |
| Complexity | Low: one protocol handler, one load message |
| Cost | Nothing measurable over a `<script>` tag |
| Security | Eval only inside an opaque-origin frame with no network, storage or IPC |

## Consequences

- **Easier:** `scattergl` and anything else that compiles code at run time; adding a renderer is still an npm package and a registry entry.
- **Harder:**
  - a second scheme to keep restricted;
  - the frame's load protocol (`load` before `render`) to test.
- **Revisit:** if WebKit ever matches `'self'` for sandboxed frames, the scheme could serve the renderer itself.

## Action Items

1. [x] App: the `cruncha-figure` protocol (frame page, CSP header), `frame-src` in the app CSP.
2. [x] Front end: fetch the renderer once, post it to each frame; tests for the message checks.
