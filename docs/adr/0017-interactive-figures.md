# ADR-0017: Interactive figures are JSON specs keyed by MIME type, drawn by sandboxed renderers (refines 0005)

**Status:** Proposed
**Date:** 2026-11-21
**Deciders:** acmeanvil

## Context

Python cells show figures today, but only as matplotlib PNGs (`python/cruncha_worker/outputs.py`). Engineers also want plots they can zoom, pan and hover, such as a moment diagram where hovering shows the value at a station. Plotly is the library most of them know. When the agent was asked for "a Plotly graph of the beam moment", it found Plotly wasn't installed and that Cruncha had nowhere to show one.

Rules this has to respect:
- **ADR-0005:** the worker sends structured data (PNG figures, table rows, text), **never HTML**. Nothing a cell outputs may inject markup or script into the app, which can call Tauri commands that write the document.
- **ADR-0015:** a report never evaluates, and the same version gives a byte-identical PDF. A PDF can't be interactive, so every interactive figure also needs a static rendering that is deterministic.
- **The app works offline.** Nothing is loaded from a CDN, and a figure must not be able to fetch anything (no tracking pixels or remote data).
- **Agents** read the document as text (the mirror) and can't see images, so a figure needs a text summary.
- **Design 02 §2** plans a native `plot` cell ("a declarative plot spec over variables → figure"), and Rust cells (ADR-0006) will want figures too. Whatever Python uses should serve them as well.

The libraries already separate *describing* a figure from *drawing* it, and the description is JSON:

| Library (license) | Spec | Jupyter MIME type | Browser renderer (license) |
| --- | --- | --- | --- |
| Plotly (MIT) | `fig.to_plotly_json()`: `data` + `layout` | `application/vnd.plotly.v1+json` | plotly.js (MIT) |
| Altair (BSD-3) | `chart.to_dict()`: Vega-Lite | `application/vnd.vegalite.v5+json` | vega-embed (BSD-3) |
| Bokeh (BSD-3) | `json_item`, but drawn by executing JS | `application/vnd.bokehjs_exec.v0+json` | (script execution, so excluded) |

## Decision

1. **A new output kind, `figure`, carries a JSON spec and its format.** The format is the spec's MIME type, so the plumbing doesn't depend on any one plotting library:

   ```json
   { "kind": "figure", "mime": "application/vnd.plotly.v1+json", "spec": <blob>,
     "summary": "Bending moment (kip·ft) vs x (ft); 1 trace: M", "static": <blob, optional> }
   ```

   - The spec travels as a binary frame, like images, and is limited to **8 MB**. A bigger figure is an error that suggests downsampling, or `scattergl` with fewer points.
   - Only **allowlisted MIME types** are accepted: Plotly v1 and Vega-Lite v5 at first. Anything else, including `text/html`, `application/javascript` and Bokeh's exec type, is dropped with a warning. ADR-0005's "never HTML" rule stands; a JSON spec is data.
2. **Each library gets a small adapter in the worker.**
   - **Plotly:** a figure left as the cell's last expression, or passed to `fig.show()`, is captured. `fig.show()` works through a custom `plotly.io` renderer that the worker registers as the default, so nothing opens a browser.
   - **Altair:** charts are captured the same way through their `to_dict()`.
   - **Any other object** is offered through `_repr_mimebundle_()`, keeping only allowlisted types.
   - **`cruncha.show(obj)`** shows several figures from one cell, as `plt` figures already do.
   - The `cruncha` package registers a `cruncha` Plotly template and makes it the default, so figures match the app's look and the report's fonts. A cell's own template still wins.
   - Pint quantities are unwrapped to magnitudes, and the axis title gets the unit if it has none. Plotly can't take quantities, and this keeps the units visible.
   - matplotlib stays as it is: open figures become PNGs.
3. **Renderers run in a sandboxed iframe, never in the app's own page.**
   - Each figure is drawn in an `<iframe sandbox="allow-scripts" srcdoc=…>`. Its origin is opaque, so it has no Tauri IPC, no access to the editor's DOM, and no same-origin storage.
   - It has its own CSP: `default-src 'none'`, plus the renderer's script, inline styles, and `img-src data: blob:`. **No network.**
   - The spec goes in, and the static rendering comes out, through `postMessage`. The parent accepts only messages from the frame it created, and only of the expected shapes.
   - Renderer bundles (plotly.js, vega-embed) are pinned npm dependencies, shipped with the app and loaded only when a figure of that type first appears. Nothing comes from a CDN.
   - The specs themselves are sanitized before drawing, defense in depth on top of the sandbox:
     - Plotly: `layout.images` sources other than `data:` URIs are removed; links in Plotly's text markup are removed; the spec's `config` is ignored and replaced with Cruncha's.
     - Vega: data `url`s are refused, and the loader has no network.
   - A registry maps a MIME type to `{ load(), render(spec, theme), toStatic(spec) }`. Adding a library means adding an adapter and a renderer; the result, report and mirror plumbing stays the same.
4. **The static rendering for reports is SVG made by the same renderer, cached by content.**
   - The static rendering is the renderer's SVG export (`Plotly.toImage({format: "svg"})`, Vega's `view.toSVG()`). It is made in the same sandbox and drawn with the report's template and fonts.
   - It is cached by `sha256(spec, MIME type, renderer version, report style)`. When it isn't cached, report export asks the app to render the figure offscreen, the way it already waits for running Python.
   - Typst places the SVG as vector graphics, using the report's embedded fonts. `Output::Image` gains an SVG variant.
   - **Determinism:** the adapter assigns trace `uid`s from the cell id and the trace index before rendering (Plotly otherwise makes random `uid`s that reach the SVG's ids). The spike confirms that the same input gives byte-identical SVG.
   - A figure that can't be rendered prints in the report as **FIGURE NOT RENDERED**, with the reason. It is never left out silently.
   - A cell may also provide its own static image (a matplotlib or kaleido PNG) in `static`. If present, the report uses it.
5. **Agents get the summary, not the picture.**
   - The mirror shows `> ⇒ figure (plotly): <summary>`.
   - The adapter builds the summary from the spec: title, axis titles with units, trace names and types, and point counts.
   - `explain` on the cell can return each trace's data range (min and max of x and y).
6. **Where figures live.**
   - A figure is part of a cell's result. Like every result, it doesn't go in `doc/`; it lives in memory now and in the planned `results/` table later.
   - Interaction state (zoom, hidden traces) isn't saved; a re-run redraws from the spec.
   - The `plot` cell, when it comes, will produce the same `figure` output (most likely a Vega-Lite spec built over document variables) and use the same renderers.
7. **Packages.**
   - `plotly` (MIT) is added to `python/requirements.in`. A new lock means a new environment (ADR-0005).
   - Altair waits for demand.
   - `plotly.js-dist-min` (MIT) is added to the app. vega-embed comes with the first Vega-Lite figure.

## Options Considered

### Option A: Plotly only, drawn directly in the app's page
**Pros:** least code; one library to learn.
**Cons:**
- plotly.js would run in the page that holds the Tauri IPC, so a crafted spec (links, images, the text markup subset) is one bug away from the app;
- the next library (Altair, a native `plot` cell) needs its own plumbing.

### Option B: let the worker emit HTML (`fig.to_html()`) into a sandboxed iframe
**Pros:** works with any library that can export HTML, Bokeh included.
**Cons:**
- breaks ADR-0005's output rule;
- each figure carries its own copy of the renderer (about 4 MB for Plotly) unless it is fetched from a CDN, which can't happen offline;
- opaque to reports (no static rendering) and to agents (no summary).

### Option C: static images only (Plotly through kaleido)
**Pros:** no new renderer in the app; reports are trivial.
**Cons:**
- no interactivity, which is the point;
- kaleido v1 needs a Chrome installed on the machine.

### Option D: Cruncha's own chart spec and renderer only
**Pros:** full control; the spec can understand units natively.
**Cons:**
- a large project;
- users can't bring the libraries they know.

It stays the long-term shape of the `plot` cell, which can still target Vega-Lite under this ADR.

### Option E: MIME-typed JSON specs, allowlisted renderers in sandboxed iframes (chosen)
| Dimension | Assessment |
| --- | --- |
| Complexity | Medium: worker adapters, an iframe host with `postMessage`, a renderer registry, SVG in reports |
| Cost | About 3.5 MB of plotly.js in the app bundle, loaded lazily; specs are small except for large scatters |
| Scalability | Bounded by the 8 MB spec cap; large scatters use `scattergl` |
| Team familiarity | Jupyter's MIME-bundle convention; plotly.js and Vega are well documented |

**Pros:**
- interactive plots in the libraries engineers already use;
- no HTML or script crosses from the worker;
- the same figure works in the editor, the report (SVG) and the mirror (summary);
- adding a library is an adapter and a renderer.

**Cons:**
- report export depends on the app's webview for the static rendering, so a future headless `cruncha run` report needs its own renderer (for example vl-convert for Vega-Lite, or kaleido);
- two renderers' sanitizers to maintain.

## Consequences

- **Easier:**
  - interactive plots in Python cells;
  - vector figures in PDFs;
  - agents can propose a plot cell (through ADR-0016 proposals) and describe it from its summary;
  - the future `plot` and Rust cells reuse the same output.
- **Harder:**
  - an iframe host component, and its message protocol, to test;
  - deterministic SVG has to be checked for each renderer version (pinned, and part of the cache key);
  - the app bundle grows.
- **Changes elsewhere:**
  - ADR-0005's output list gains `figure` (JSON specs of allowlisted MIME types); "never HTML" is unchanged;
  - ADR-0015's `Output::Image` gains SVG and FIGURE NOT RENDERED;
  - the `cruncha-code-cells` skill documents Plotly, `cruncha.show` and the limits;
  - the agent instructions stop saying "matplotlib only";
  - design 02 line 117 loses its mention of `_repr_html_`.
- **Revisit:**
  - saving interaction state;
  - Altair and Vega-Lite;
  - a headless renderer for CLI reports;
  - letting a calc cell's values feed a figure without Python (the `plot` cell).

## Action Items

1. [ ] **Spike S7** (macOS WKWebView in Tauri):
   - plotly.js in a `srcdoc` sandboxed iframe under the strict CSP (does it need `unsafe-eval`? does `scattergl` work?);
   - `postMessage` round trip;
   - first-load time;
   - a 100k-point scatter;
   - SVG determinism with fixed `uid`s;
   - Typst placing Plotly's SVG with our fonts.
   Record the numbers here.
2. [ ] Worker: `figure` output, Plotly adapter (last expression, `fig.show()` renderer, template, pint unwrapping, uids, summary), `cruncha.show`, MIME allowlist, 8 MB cap; tests.
3. [ ] Add `plotly` to `requirements.in` and recompile the lock.
4. [ ] App: `FigureFrame.svelte` (iframe host), renderer registry with Plotly, sanitizer, light/dark theme; vitest for the sanitizer and the message checks.
5. [ ] Reports: SVG `Output`, a static-render request from export, the cache, FIGURE NOT RENDERED.
6. [ ] Mirror and `explain`: the figure summary and data ranges.
7. [ ] Update `skills/cruncha-code-cells`, the agent instructions in `agent_panel.rs`, and CLAUDE.md.
