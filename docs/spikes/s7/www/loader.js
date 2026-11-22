window.addEventListener("message", (ev) => {
  if (ev.source !== parent || ev.data.op !== "load") return;
  const t = performance.now();
  try { new Function(ev.data.code).call(self); } catch (e) { return parent.postMessage({ op: "loaded", error: String(e) }, "*"); }
  parent.postMessage({ op: "loaded", plotly: typeof Plotly === "object" ? Plotly.version : "missing", evalMs: performance.now() - t }, "*");
});
