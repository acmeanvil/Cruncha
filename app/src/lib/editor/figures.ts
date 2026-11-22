// SPDX-License-Identifier: GPL-3.0-or-later
// Interactive figures (ADR-0017, ADR-0020). A figure's spec is JSON data from a Python cell;
// it is drawn by a renderer (plotly.js) inside a sandboxed iframe served from the app's
// `cruncha-figure` scheme: an opaque origin with no network, storage or IPC. This page:
// - sanitizes each spec before it crosses (defense in depth on top of the sandbox);
// - hands the frame the renderer's code (fetched once from the app's own bundle);
// - accepts only messages from the frame it created, of the expected shapes.
import plotlyUrl from "plotly.js-dist-min/plotly.min.js?url";

export const PLOTLY = "application/vnd.plotly.v1+json";
export const VEGALITE = "application/vnd.vegalite.v5+json";
const MAX_SVG = 16 << 20;
const TIMEOUT_MS = 30_000;

type Json = null | boolean | number | string | Json[] | { [k: string]: Json };
export interface PlotlySpec {
  data: Json[];
  layout: { [k: string]: Json };
  frames?: Json[];
}

const IMAGE_DATA = /^data:image\/(png|jpeg|gif|webp);base64,/i;

/** Plotly's text markup allows `<a href>`; drop links (and anything scripted) from strings. */
function cleanText(s: string): string {
  if (!s.includes("<") && !/javascript:/i.test(s)) return s;
  return s
    .replace(/<a\b[^>]*>/gi, "")
    .replace(/<\/a\s*>/gi, "")
    .replace(/javascript:/gi, "");
}

function clean(v: Json, key = ""): Json {
  if (typeof v === "string") return cleanText(v);
  if (Array.isArray(v)) {
    if (key === "images") {
      // layout.images: only embedded images, never a URL to fetch.
      return v.filter((im) => !!im && typeof im === "object" && !Array.isArray(im) && typeof im.source === "string" && IMAGE_DATA.test(im.source)).map((im) => clean(im));
    }
    return v.map((x) => clean(x));
  }
  if (v && typeof v === "object") {
    const out: { [k: string]: Json } = {};
    for (const [k, x] of Object.entries(v)) out[k] = clean(x, k);
    return out;
  }
  return v;
}

/** A Plotly spec ready to draw, or null when it isn't one. Its `config` is dropped (the frame
 * uses Cruncha's), link markup is removed from text, and layout images must be data URIs. */
export function sanitizePlotly(spec: unknown): PlotlySpec | null {
  if (!spec || typeof spec !== "object" || Array.isArray(spec)) return null;
  const s = spec as { [k: string]: Json };
  const data = s.data ?? [];
  const layout = s.layout ?? {};
  if (!Array.isArray(data) || typeof layout !== "object" || layout === null || Array.isArray(layout)) return null;
  const out: PlotlySpec = { data: clean(data) as Json[], layout: clean(layout) as { [k: string]: Json } };
  if (Array.isArray(s.frames)) out.frames = clean(s.frames) as Json[];
  return out;
}

/** Messages a frame may send. Anything else is ignored. */
export type FrameMsg =
  | { op: "hello" }
  | { op: "loaded"; version: string }
  | { op: "rendered"; height: number }
  | { op: "svg"; svg: string }
  | { op: "failed"; message: string };

export function checkMessage(d: unknown): FrameMsg | null {
  if (!d || typeof d !== "object") return null;
  const m = d as Record<string, unknown>;
  switch (m.op) {
    case "hello":
      return { op: "hello" };
    case "loaded":
      return typeof m.version === "string" ? { op: "loaded", version: m.version.slice(0, 40) } : null;
    case "rendered":
      return typeof m.height === "number" && Number.isFinite(m.height) && m.height > 0 && m.height < 5000 ? { op: "rendered", height: m.height } : null;
    case "svg":
      return typeof m.svg === "string" && m.svg.length <= MAX_SVG && m.svg.trimStart().startsWith("<svg") ? { op: "svg", svg: m.svg } : null;
    case "failed":
      return typeof m.message === "string" ? { op: "failed", message: m.message.slice(0, 500) } : null;
    default:
      return null;
  }
}

/** Where the frame page is served (Tauri's custom-scheme URL differs on Windows). */
export function frameUrl(): string {
  return /Windows/.test(navigator.userAgent) ? "http://cruncha-figure.localhost/frame.html" : "cruncha-figure://localhost/frame.html";
}

/** The figure types this app can draw. */
export function canDraw(mime: string): boolean {
  return mime === PLOTLY;
}

let plotlyCode: Promise<string> | null = null;
function rendererCode(): Promise<string> {
  plotlyCode ??= fetch(plotlyUrl).then((r) => {
    if (!r.ok) throw new Error(`couldn't load the figure renderer (${r.status})`);
    return r.text();
  });
  plotlyCode.catch(() => (plotlyCode = null));
  return plotlyCode;
}

export function theme(): "light" | "dark" {
  return typeof matchMedia === "function" && matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
}

/** One sandboxed figure frame and its request/response protocol. Requests are serialized. */
export class FigureFrame {
  readonly iframe: HTMLIFrameElement;
  private waiting: ((m: FrameMsg) => void) | null = null;
  private early: FrameMsg[] = [];
  private chain: Promise<unknown> = Promise.resolve();
  private loaded: Promise<void> | null = null;
  private listener = (ev: MessageEvent) => {
    if (ev.source !== this.iframe.contentWindow) return;
    const m = checkMessage(ev.data);
    if (!m) return;
    const w = this.waiting;
    if (w) {
      this.waiting = null;
      w(m);
    } else this.early.push(m);
  };

  constructor(host: HTMLElement, opts: { hidden?: boolean; title?: string } = {}) {
    const f = document.createElement("iframe");
    f.setAttribute("sandbox", "allow-scripts");
    f.setAttribute("referrerpolicy", "no-referrer");
    f.title = opts.title ?? "Figure";
    f.className = "figure-frame";
    if (opts.hidden) f.style.cssText = "position:fixed;left:-10000px;top:0;width:700px;height:450px;border:0;visibility:hidden";
    f.src = frameUrl();
    this.iframe = f;
    window.addEventListener("message", this.listener);
    host.append(f);
  }

  private next(): Promise<FrameMsg> {
    const m = this.early.shift();
    if (m) return Promise.resolve(m);
    return new Promise((resolve, reject) => {
      const t = setTimeout(() => {
        this.waiting = null;
        reject(new Error("the figure frame didn't answer"));
      }, TIMEOUT_MS);
      this.waiting = (msg) => {
        clearTimeout(t);
        resolve(msg);
      };
    });
  }

  private request(msg: object, expect: FrameMsg["op"]): Promise<FrameMsg> {
    const run = async () => {
      await this.ready();
      this.iframe.contentWindow?.postMessage(msg, "*");
      const m = await this.next();
      if (m.op === "failed") throw new Error(m.message);
      if (m.op !== expect) throw new Error(`unexpected ${m.op} from the figure frame`);
      return m;
    };
    const p = this.chain.then(run, run);
    this.chain = p.catch(() => undefined);
    return p;
  }

  /** The frame has said hello and has the renderer. */
  ready(): Promise<void> {
    this.loaded ??= (async () => {
      const [code, hello] = await Promise.all([rendererCode(), this.next()]);
      if (hello.op !== "hello") throw new Error("the figure frame didn't start");
      this.iframe.contentWindow?.postMessage({ op: "load", code }, "*");
      const m = await this.next();
      if (m.op !== "loaded") throw new Error(m.op === "failed" ? m.message : "the renderer didn't load");
    })();
    return this.loaded;
  }

  /** Draw `spec` in the frame; resolves with its height in pixels. */
  async render(spec: PlotlySpec): Promise<number> {
    const m = await this.request({ op: "render", spec, theme: theme() }, "rendered");
    return (m as { height: number }).height;
  }

  /** `spec` as SVG at its static size, in `font`, with its plot id set to `id`. */
  async toSvg(spec: PlotlySpec, font: string, id: string): Promise<string> {
    const m = await this.request({ op: "static", spec, font, id }, "svg");
    return (m as { svg: string }).svg;
  }

  destroy() {
    window.removeEventListener("message", this.listener);
    this.iframe.remove();
  }
}

/** A figure output in a cell: a live frame, or its summary when it can't be drawn. */
export function figureElement(o: { mime: string; spec: string; summary: string }): HTMLElement {
  const box = document.createElement("div");
  box.className = "code-plot";
  const caption = document.createElement("div");
  caption.className = "code-plot-summary";
  caption.textContent = o.summary;
  if (!canDraw(o.mime)) {
    caption.textContent = `${o.summary} (this app can't draw ${o.mime} figures yet)`;
    box.append(caption);
    return box;
  }
  let spec: PlotlySpec | null = null;
  try {
    spec = sanitizePlotly(JSON.parse(o.spec));
  } catch {
    spec = null;
  }
  if (!spec) {
    caption.textContent = `${o.summary} (the figure's spec isn't valid)`;
    box.append(caption);
    return box;
  }
  const frame = new FigureFrame(box, { title: o.summary });
  frame.iframe.style.height = `${(typeof spec.layout.height === "number" ? spec.layout.height : 380) + 4}px`;
  box.append(caption);
  frame
    .render(spec)
    .then((h) => (frame.iframe.style.height = `${h + 4}px`))
    .catch((e) => {
      frame.destroy();
      caption.textContent = `${o.summary} (couldn't draw it: ${e instanceof Error ? e.message : e})`;
    });
  // Kept so the cell view can tear the frame down with its output.
  (box as HTMLElement & { figureFrame?: FigureFrame }).figureFrame = frame;
  return box;
}

/** Draw the figures a report needs (offscreen) and hand their SVG to the backend. Returns
 * how many couldn't be drawn; those print as FIGURE NOT RENDERED. */
export async function drawForReport(
  jobs: { key: string; mime: string; spec: string; font: string }[],
  put: (key: string, svg: string) => Promise<void>,
): Promise<number> {
  const drawable = jobs.filter((j) => canDraw(j.mime));
  let failed = jobs.length - drawable.length;
  if (!drawable.length) return failed;
  const frame = new FigureFrame(document.body, { hidden: true, title: "Report figure" });
  try {
    for (const j of drawable) {
      try {
        const spec = sanitizePlotly(JSON.parse(j.spec));
        if (!spec) throw new Error("not a Plotly spec");
        const svg = await frame.toSvg(spec, j.font, `f${j.key.slice(0, 12)}`);
        await put(j.key, svg);
      } catch (e) {
        failed += 1;
        console.warn("cruncha: a report figure wasn't drawn:", e);
      }
    }
  } finally {
    frame.destroy();
  }
  return failed;
}
