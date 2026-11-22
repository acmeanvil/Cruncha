# Spike S7: interactive figures in the app's web view

*Date: 2026-11-21. Code: `docs/spikes/s7/` (a Swift host and the test pages), `crates/cruncha-report/tests/figures.rs`. Machine: Intel i7-1068NG7 MacBook, macOS 26, WebKit from the OS, plotly.js 4.2.0 (`plotly.js-dist-min`, MIT).*

ADR-0017 asked whether plotly.js can draw figures inside a sandboxed iframe in Tauri's macOS web view (WKWebView) under a strict CSP, and whether its SVG export can go into a deterministic PDF.

## Method

- **Host** (`s7/host.swift`): a real `WKWebView` in an on-screen, borderless window. Pages are served from a custom URL scheme, as Tauri serves `tauri://localhost`, with a `Content-Security-Policy` header on HTML. A second scheme stands for a figure-only origin. The page reports results through a script message handler.

  Run it with `swiftc -O host.swift -o host && ./host <www dir> "<app CSP>" index.html "<frame CSP>"`. Copy `plotly.min.js` into the www directory first.
- **Pages** (`s7/www/`):
  - The parent creates a sandboxed iframe (`sandbox="allow-scripts"`, opaque origin) in one of four ways:
    - `src` pointing at the app's own scheme;
    - `srcdoc`;
    - `src` on the figure scheme, loading plotly.js by `<script>`;
    - `src` on the figure scheme, with plotly.js fetched by the parent and handed over by `postMessage`.
  - It then probes what the frame can reach, times a `postMessage` round trip, and draws:
    - a 200-point line;
    - 100k-point `scatter` and `scattergl` plots.
  - It exports SVG three times: twice from one frame, once from a fresh frame.
  - It checks that `scattergl` actually drew by exporting PNG and counting coloured pixels.
- **Report:** the normalized SVG of the line plot, placed by the calc-package template through `image()`. It was rendered twice and the PDFs compared.

## Results

| Frame | Loads | Eval in frame | `scattergl` draws |
| --- | --- | --- | --- |
| `src` on the app's scheme | **no**: under WebKit, a sandboxed document doesn't match `'self'`, so its scripts are refused | n/a | n/a |
| `srcdoc` | yes, 351 ms (it inherits the app's CSP, where `'self'` is the app) | blocked | **no**: it draws nothing, with no error |
| figure scheme, `<script>` | yes, 296 ms | allowed (its own CSP) | yes |
| figure scheme, by `postMessage` | yes, 297 ms (fetch 28 ms, eval 244 ms) | allowed | yes |

In every variant, the frame's origin is `null`, and these are blocked: reading `parent.document`, `parent.__TAURI_INTERNALS__` and `localStorage` (`SecurityError`); `fetch` to `https://example.com` and to the app's own scheme (CSP `connect-src`). The frame has no `__TAURI_INTERNALS__` of its own.

| Measure | Value |
| --- | --- |
| `postMessage` round trip | 0.06–0.09 ms |
| 200-point line, first draw | 66–123 ms |
| 100k-point `scatter` (SVG) | 1.96–2.26 s |
| 100k-point `scattergl` | 0.16–0.24 s |
| SVG export (200-point line) | 18–36 ms, 7.5 KB |
| SVG determinism, raw | **no**: each plot gets a random 6-hex id (`defs-cfc216`, `clipcfc216xy`…) |
| SVG determinism, that id replaced | **yes**: byte-identical in one frame and across fresh frames |
| Report with the SVG, rendered twice | byte-identical PDFs; the figure is vector, with labels in the report's fonts |

## Conclusions

1. **`srcdoc` can't draw `scattergl`.** It needs `'unsafe-eval'`: regl compiles its shaders with `Function`. A `srcdoc` frame inherits the app page's CSP, and a policy can only be tightened, never loosened. So allowing eval there would mean allowing it in the page that holds the Tauri IPC.
2. **Figures are served from their own scheme** (`cruncha-figure://`), with a CSP of their own: `default-src 'none'; script-src <figure origin> 'unsafe-eval'; style-src 'unsafe-inline'; img-src data: blob:`.
   - Eval then runs only in an opaque-origin frame with no network, no storage, no IPC and no access to the parent.
   - The app page's CSP gains only `frame-src` for that scheme.
3. **The renderer arrives by `postMessage`.** The scheme serves one small page (from the binary). The app fetches plotly.js from its own bundle, once, and the frame evaluates it.
   - It costs the same as a `<script>` tag.
   - The Rust build doesn't depend on `node_modules`.
   - The renderer's version is the npm lock's.
4. **Deterministic SVG** needs the plot id fixed after export: the renderer replaces Plotly's random id with one derived from the cell and output index. Trace `uid`s are set by the worker as ADR-0017 planned.
5. **Large scatters should use `scattergl`:** ten times faster to draw. Its SVG export embeds the WebGL layer as a PNG `<image>`, so the report's copy of it is raster, at the export size.

These change how ADR-0017 §3 delivers the renderer; see ADR-0020.
