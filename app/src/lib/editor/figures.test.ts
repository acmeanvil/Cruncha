// SPDX-License-Identifier: GPL-3.0-or-later
// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from "vitest";
import { FigureFrame, PLOTLY, checkMessage, drawForReport, sanitizePlotly } from "./figures";
import { sameOutputs } from "./codeView";

describe("sanitizePlotly", () => {
  it("drops config, links and non-embedded images, keeps the data", () => {
    const s = sanitizePlotly({
      data: [{ type: "scatter", x: [1, 2], y: { dtype: "f8", bdata: "AAAA" }, name: 'M <a href="https://evil.example">click</a>', hovertemplate: "javascript:alert(1)" }],
      layout: {
        title: { text: "<b>Moment</b> <A HREF='x'>here</A>" },
        images: [{ source: "https://evil.example/pixel.png" }, { source: "data:image/png;base64,iVBORw0KGgo=" }, { source: "data:text/html;base64,PGI+" }],
        template: { layout: { images: [{ source: "http://x/y.png" }] } },
      },
      config: { editable: true, showLink: true },
    });
    expect(s).not.toBeNull();
    expect(s).not.toHaveProperty("config");
    const t = s!.data[0] as Record<string, unknown>;
    expect(t.name).toBe("M click");
    expect(t.hovertemplate).toBe("alert(1)");
    expect(t.y).toEqual({ dtype: "f8", bdata: "AAAA" });
    expect((s!.layout.title as { text: string }).text).toBe("<b>Moment</b> here");
    expect(s!.layout.images).toEqual([{ source: "data:image/png;base64,iVBORw0KGgo=" }]);
    expect(((s!.layout.template as { layout: { images: unknown[] } }).layout.images)).toEqual([]);
  });

  it("refuses what isn't a spec", () => {
    expect(sanitizePlotly(null)).toBeNull();
    expect(sanitizePlotly([1])).toBeNull();
    expect(sanitizePlotly({ data: "x" })).toBeNull();
    expect(sanitizePlotly({ data: [], layout: [] })).toBeNull();
    expect(sanitizePlotly({})).toEqual({ data: [], layout: {} });
  });
});

describe("checkMessage", () => {
  it("accepts only the expected shapes", () => {
    expect(checkMessage({ op: "hello" })).toEqual({ op: "hello" });
    expect(checkMessage({ op: "rendered", height: 380 })).toEqual({ op: "rendered", height: 380 });
    expect(checkMessage({ op: "rendered", height: "380" })).toBeNull();
    expect(checkMessage({ op: "rendered", height: 1e9 })).toBeNull();
    expect(checkMessage({ op: "svg", svg: "<svg/>" })).toEqual({ op: "svg", svg: "<svg/>" });
    expect(checkMessage({ op: "svg", svg: "<html><script>" })).toBeNull();
    expect(checkMessage({ op: "invoke", cmd: "commit" })).toBeNull();
    expect(checkMessage("hello")).toBeNull();
    expect(checkMessage({ op: "failed", message: "x".repeat(10_000) })).toEqual({ op: "failed", message: "x".repeat(500) });
  });
});

describe("FigureFrame", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
    document.body.replaceChildren();
  });

  function from(frame: FigureFrame, data: unknown, source: Window | null = frame.iframe.contentWindow) {
    window.dispatchEvent(new MessageEvent("message", { data, source }));
  }

  it("is a sandboxed frame on the figure scheme", () => {
    const f = new FigureFrame(document.body);
    expect(f.iframe.getAttribute("sandbox")).toBe("allow-scripts");
    expect(f.iframe.src).toBe("cruncha-figure://localhost/frame.html");
    f.destroy();
    expect(document.querySelector("iframe")).toBeNull();
  });

  it("loads the renderer, then draws; other windows and bad messages are ignored", async () => {
    vi.stubGlobal("fetch", vi.fn(async () => new Response("self.Plotly = {}")));
    const f = new FigureFrame(document.body);
    const sent: unknown[] = [];
    vi.spyOn(f.iframe.contentWindow!, "postMessage").mockImplementation((m: unknown) => void sent.push(m));
    const drawn = f.render({ data: [], layout: {} });
    from(f, { op: "hello" }, window); // not the frame
    from(f, { op: "hello" });
    await vi.waitFor(() => expect(sent).toHaveLength(1));
    expect(sent[0]).toEqual({ op: "load", code: "self.Plotly = {}" });
    from(f, { op: "loaded", version: "4.2.0" });
    await vi.waitFor(() => expect(sent).toHaveLength(2));
    expect(sent[1]).toMatchObject({ op: "render", spec: { data: [], layout: {} } });
    from(f, { op: "rendered", height: "tall" }); // malformed: ignored
    from(f, { op: "rendered", height: 300 });
    await expect(drawn).resolves.toBe(300);
    f.destroy();
  });

  it("report figures: drawn offscreen and handed to the backend; failures counted", async () => {
    vi.stubGlobal("fetch", vi.fn(async () => new Response("")));
    const put = vi.fn(async () => {});
    const jobs = [
      { key: "a".repeat(64), mime: PLOTLY, spec: '{"data":[],"layout":{}}', font: "Helvetica" },
      { key: "b".repeat(64), mime: "application/vnd.vegalite.v5+json", spec: "{}", font: "Helvetica" },
    ];
    const done = drawForReport(jobs, put);
    const f = await vi.waitFor(() => {
      const el = document.querySelector("iframe");
      if (!el) throw new Error("no frame yet");
      return el;
    });
    const sent: { op: string; id?: string }[] = [];
    vi.spyOn(f.contentWindow!, "postMessage").mockImplementation((m: unknown) => void sent.push(m as { op: string }));
    const reply = (data: unknown) => window.dispatchEvent(new MessageEvent("message", { data, source: f.contentWindow }));
    reply({ op: "hello" });
    await vi.waitFor(() => expect(sent.map((m) => m.op)).toEqual(["load"]));
    reply({ op: "loaded", version: "4.2.0" });
    await vi.waitFor(() => expect(sent.map((m) => m.op)).toEqual(["load", "static"]));
    expect(sent[1].id).toBe("faaaaaaaaaaaa");
    reply({ op: "svg", svg: "<svg/>" });
    await expect(done).resolves.toBe(1); // the Vega-Lite one can't be drawn yet
    expect(put).toHaveBeenCalledWith("a".repeat(64), "<svg/>");
    expect(document.querySelector("iframe")).toBeNull();
  });
});

describe("sameOutputs", () => {
  it("compares figures by spec, so an unchanged figure keeps its frame", () => {
    const fig = { kind: "figure" as const, mime: PLOTLY, spec: "{}", summary: "s", ranges: [], staticSrc: null };
    expect(sameOutputs([fig], [{ ...fig }])).toBe(true);
    expect(sameOutputs([fig], [{ ...fig, spec: '{"data":[]}' }])).toBe(false);
    expect(sameOutputs([{ kind: "text", text: "a" }], [{ kind: "text", text: "a" }])).toBe(true);
    expect(sameOutputs([], [fig])).toBe(false);
  });
});
