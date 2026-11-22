const gd = document.getElementById("gd");
window.addEventListener("message", async (ev) => {
  if (ev.source !== parent) return;
  const m = ev.data;
  try {
    if (m.op === "ping") return parent.postMessage({ op: "pong", n: m.n }, "*");
    if (m.op === "render") {
      const t = performance.now();
      await Plotly.newPlot(gd, m.spec.data, m.spec.layout, { staticPlot: false, displaylogo: false });
      return parent.postMessage({ op: "rendered", ms: performance.now() - t }, "*");
    }
    if (m.op === "svg") {
      const t = performance.now();
      const url = await Plotly.toImage(gd, { format: "svg", width: 640, height: 400 });
      const svg = decodeURIComponent(url.slice(url.indexOf(",") + 1));
      return parent.postMessage({ op: "svg", svg, ms: performance.now() - t }, "*");
    }
    if (m.op === "glcheck") {
      const url = await Plotly.toImage(gd, { format: "png", width: 400, height: 300 });
      const img = new Image(); img.src = url; await img.decode();
      const c = document.createElement("canvas"); c.width = 400; c.height = 300;
      const x = c.getContext("2d"); x.drawImage(img, 0, 0);
      const d = x.getImageData(0, 0, 400, 300).data; let dark = 0;
      for (let i = 0; i < d.length; i += 4) if (d[i] < 200 && d[i + 2] > d[i]) dark++;
      return parent.postMessage({ op: "gl", bluePixels: dark }, "*");
    }
    if (m.op === "glcheck-old") {
      const cs = [...document.querySelectorAll("canvas")];
      const info = cs.map((c) => { const g = c.getContext("webgl") || c.getContext("webgl2"); if (!g) return "no-ctx";
        const px = new Uint8Array(4 * c.width * c.height); g.readPixels(0, 0, c.width, c.height, g.RGBA, g.UNSIGNED_BYTE, px);
        let n = 0; for (let i = 3; i < px.length; i += 4) if (px[i]) n++; return c.width + "x" + c.height + ":" + n; });
      return parent.postMessage({ op: "gl", canvases: info, traces: gd._fullData && gd._fullData.map((t) => t.type) }, "*");
    }
    if (m.op === "probe") {
      const r = {};
      try { r.parentDoc = !!parent.document.body; } catch (e) { r.parentDoc = "blocked: " + e.name; }
      try { r.tauri = typeof parent.__TAURI_INTERNALS__; } catch (e) { r.tauri = "blocked: " + e.name; }
      r.ownTauri = typeof window.__TAURI_INTERNALS__;
      try { localStorage.setItem("x", "1"); r.storage = "allowed"; } catch (e) { r.storage = "blocked: " + e.name; }
      try { await fetch("https://example.com/"); r.fetchRemote = "allowed"; } catch (e) { r.fetchRemote = "blocked: " + e.name; }
      try { await fetch("cruncha://localhost/index.html"); r.fetchSelf = "allowed"; } catch (e) { r.fetchSelf = "blocked: " + e.name; }
      try { new Function("return 1")(); r.eval = "allowed"; } catch (e) { r.eval = "blocked: " + e.name; }
      r.origin = self.origin;
      return parent.postMessage({ op: "probe", r }, "*");
    }
  } catch (e) {
    parent.postMessage({ op: "failed", for: m.op, message: String(e && e.message || e) }, "*");
  }
});
if (typeof Plotly === "object") parent.postMessage({ op: "ready", plotly: typeof Plotly === "object" ? Plotly.version : "missing" }, "*");
