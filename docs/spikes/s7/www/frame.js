window.__csp = [];
document.addEventListener("securitypolicyviolation", (e) => {
  parent.postMessage({ op: "csp", directive: e.violatedDirective, blocked: e.blockedURI, sample: e.sample }, "*");
});
window.addEventListener("error", (e) => parent.postMessage({ op: "error", message: String(e.message) }, "*"));
