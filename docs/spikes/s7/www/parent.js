window.webkit.messageHandlers.log.postMessage("start origin=" + location.origin);
const log = (s) => window.webkit.messageHandlers.log.postMessage(typeof s === "string" ? s : JSON.stringify(s));
const base = location.origin + "/";
let code;
function frame(kind) {
  const f = document.createElement("iframe");
  f.setAttribute("sandbox", "allow-scripts");
  f.style.cssText = "width:700px;height:450px;border:0";
  if (kind === "src") f.src = "frame.html";
  else if (kind === "scheme") f.src = "crunchafig://localhost/frame.html";
  else if (kind === "scheme-eval") f.src = "crunchafig://localhost/frame2.html";
  else f.srcdoc = `<!doctype html><html><head><meta charset="utf-8"></head><body style="margin:0"><div id="gd"></div>` +
    `<script src="${base}frame.js"></script><script src="${base}plotly.min.js"></script><script src="${base}boot.js"></script></body></html>`;
  const queue = [];
  const waiters = [];
  window.addEventListener("message", (ev) => {
    if (ev.source !== f.contentWindow) return;
    const m = ev.data;
    if (m.op === "csp" || m.op === "error") return log({ frame: kind, ...m });
    const w = waiters.shift();
    if (w) w(m); else queue.push(m);
  });
  const next = () => new Promise((r) => (queue.length ? r(queue.shift()) : waiters.push(r)));
  const send = (m) => { f.contentWindow.postMessage(m, "*"); return next(); };
  const t0 = performance.now();
  document.getElementById("frames").append(f);
  let ready;
  if (kind === "scheme-eval") {
    ready = (async () => {
      const tf = performance.now();
      code ??= await (await fetch("plotly.min.js")).text();
      const fetchMs = performance.now() - tf;
      await new Promise((r) => f.addEventListener("load", r, { once: true }));
      f.contentWindow.postMessage({ op: "load", code }, "*");
      const m = await next();
      return { ...m, fetchMs, loadMs: performance.now() - t0 };
    })();
  } else ready = next().then((m) => ({ ...m, loadMs: performance.now() - t0 }));
  return { f, send, ready, remove: () => f.remove() };
}
function line(n) {
  const x = Array.from({ length: n }, (_, i) => i / 10);
  return { data: [{ type: "scatter", mode: "lines", x, y: x.map((v) => v * (20 - v)), name: "M", uid: "c1-0" }],
           layout: { title: { text: "Bending moment" }, xaxis: { title: { text: "x (ft)" } }, yaxis: { title: { text: "M (kip·ft)" } }, font: { family: "Helvetica" } } };
}
function big(type, n) {
  let s = 1; const rnd = () => ((s = (s * 16807) % 2147483647) / 2147483647);
  const x = Array.from({ length: n }, rnd), y = x.map((v) => v + rnd() * 0.1);
  return { data: [{ type, mode: "markers", x, y, uid: "c1-0", marker: { size: 3 } }], layout: {} };
}
(async () => {
  for (const kind of ["scheme-eval"]) {
    const a = frame(kind);
    const r = await Promise.race([a.ready, new Promise((r) => setTimeout(() => r({ op: "noready" }), 15000))]);
    log({ frame: kind, step: "ready", ...r });
    if (r.op !== "ready" && r.op !== "loaded") { a.remove(); continue; }
    log({ frame: kind, step: "probe", ...(await a.send({ op: "probe" })) });
    let t = performance.now();
    for (let i = 0; i < 100; i++) await a.send({ op: "ping", n: i });
    log({ frame: kind, step: "ping x100", avgMs: (performance.now() - t) / 100 });
    log({ frame: kind, step: "line 200", ...(await a.send({ op: "render", spec: line(200) })) });
    const s1 = await a.send({ op: "svg" });
    const s2 = await a.send({ op: "svg" });
    const b = frame(kind); await b.ready;
    await b.send({ op: "render", spec: line(200) });
    const s3 = await b.send({ op: "svg" });
    b.remove();
    log({ frame: kind, step: "svg", ms: s1.ms, bytes: s1.svg?.length, sameFrame: s1.svg === s2.svg, newFrame: s1.svg === s3.svg, failed: s1.message,
          randomIds: (s1.svg?.match(/id="[^"]*"/g) || []).slice(0, 6) });
    log("SVG1" + s1.svg + "\nSVG2" + s2.svg + "\nSVG3" + s3.svg);
    log({ frame: kind, step: "scatter 100k", ...(await a.send({ op: "render", spec: big("scatter", 100000) })) });
    log({ frame: kind, step: "svg-scatter check", ...(await a.send({ op: "glcheck" })) });
    log({ frame: kind, step: "scattergl 100k", ...(await a.send({ op: "render", spec: big("scattergl", 100000) })) });
    log({ frame: kind, step: "gl check", ...(await a.send({ op: "glcheck" })) });
    const g = await a.send({ op: "svg" });
    log({ frame: kind, step: "gl svg", bytes: g.svg.length, images: (g.svg.match(/<image/g) || []).length, png: g.svg.includes("data:image/png") });
    a.remove();
  }
  log("DONE");
})().catch((e) => log("DONE error " + e));
