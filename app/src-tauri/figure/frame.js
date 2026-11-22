// SPDX-License-Identifier: GPL-3.0-or-later
// A figure frame (ADR-0017, ADR-0020): served by the app's cruncha-figure scheme into a
// sandboxed iframe (opaque origin, no network, no storage, no IPC). It takes messages only
// from the page that embeds it: `load` (the renderer's code), `render` (draw a spec here),
// `static` (an SVG of a spec at a fixed size, for reports). Specs arrive sanitized.
"use strict";
(function () {
  const gd = document.getElementById("gd");
  const reply = (m) => parent.postMessage(m, "*");
  let ready = false;
  // Plotly gives each plot a random id that ends up in the SVG; replace it so the same spec
  // always gives the same SVG.
  function stableSvg(svg, id) {
    const m = /id="defs-([0-9a-z]+)"/.exec(svg);
    return m ? svg.split(m[1]).join(id) : svg;
  }
  function themed(layout, theme) {
    const l = Object.assign({}, layout);
    l.autosize = true;
    delete l.width;
    if (theme === "dark") {
      l.paper_bgcolor = "rgba(0,0,0,0)";
      l.plot_bgcolor = "rgba(0,0,0,0)";
      l.font = Object.assign({}, l.font, { color: "#d7dade" });
      for (const k of Object.keys(l)) {
        if (/^[xy]axis\d*$/.test(k)) l[k] = Object.assign({ gridcolor: "#3a3f45", zerolinecolor: "#555b62", linecolor: "#555b62" }, l[k]);
      }
      if (!Object.keys(l).some((k) => /^[xy]axis\d*$/.test(k))) {
        l.xaxis = { gridcolor: "#3a3f45", zerolinecolor: "#555b62" };
        l.yaxis = { gridcolor: "#3a3f45", zerolinecolor: "#555b62" };
      }
    }
    return l;
  }
  const config = { displaylogo: false, responsive: true, showLink: false, showSendToCloud: false, showEditInChartStudio: false,
                    modeBarButtonsToRemove: ["sendDataToCloud", "toImage"], scrollZoom: false };
  window.addEventListener("message", async (ev) => {
    if (ev.source !== parent) return;
    const m = ev.data || {};
    try {
      if (m.op === "load") {
        if (!ready) {
          new Function(String(m.code)).call(self);
          ready = typeof self.Plotly === "object";
        }
        return reply(ready ? { op: "loaded", version: self.Plotly.version } : { op: "failed", message: "the renderer didn't load" });
      }
      if (!ready) return reply({ op: "failed", message: "no renderer" });
      if (m.op === "render") {
        const layout = themed(m.spec.layout || {}, m.theme);
        const height = layout.height || 380;
        layout.height = height;
        gd.style.height = height + "px";
        await self.Plotly.react(gd, m.spec.data || [], layout, config);
        return reply({ op: "rendered", height });
      }
      if (m.op === "static") {
        const layout = Object.assign({}, m.spec.layout || {});
        if (m.font) layout.font = Object.assign({}, layout.font, { family: m.font });
        const width = layout.width || 640, height = layout.height || 400;
        const url = await self.Plotly.toImage({ data: m.spec.data || [], layout }, { format: "svg", width, height });
        const svg = decodeURIComponent(url.slice(url.indexOf(",") + 1));
        return reply({ op: "svg", svg: stableSvg(svg, String(m.id || "c0")) });
      }
    } catch (e) {
      reply({ op: "failed", message: String((e && e.message) || e) });
    }
  });
  window.addEventListener("resize", () => { if (ready && gd.data) self.Plotly.Plots.resize(gd); });
  reply({ op: "hello" });
})();
