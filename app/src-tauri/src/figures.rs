// SPDX-License-Identifier: GPL-3.0-or-later
//! Static renderings of interactive figures, for reports (ADR-0017 §4).
//!
//! A report never evaluates and the backend has no renderer: the app's figure frame draws
//! each figure's SVG (with the random plot id replaced) and hands it here. Renderings are
//! cached on this computer by [`static_key`], over everything that changes the drawing: the
//! MIME type, the spec, the renderer's version and the report's fonts. So the same version
//! prints the same PDF, and a figure is drawn once.

use std::path::PathBuf;

use sha2::{Digest, Sha256};

use crate::engine::{CellResult, OutputView};

/// The pinned plotly.js (`app/package.json`); part of the cache key.
pub const PLOTLY_JS: &str = "4.2.0";

/// Largest SVG accepted from the renderer.
pub const MAX_SVG: usize = 16 << 20;

/// What the report's figures are drawn with: the house style's fonts.
pub fn report_fonts() -> Vec<String> {
    cruncha_report::HouseStyle::default().fonts
}

fn renderer_version(mime: &str) -> &'static str {
    match mime {
        "application/vnd.plotly.v1+json" => PLOTLY_JS,
        _ => "none",
    }
}

/// SHA-256 (hex) over a length-prefixed encoding of what determines a figure's SVG.
pub fn static_key(mime: &str, spec: &str, fonts: &[String]) -> String {
    let mut h = Sha256::new();
    let mut field = |b: &[u8]| {
        h.update((b.len() as u64).to_be_bytes());
        h.update(b);
    };
    field(b"cruncha-figure/1");
    field(mime.as_bytes());
    field(renderer_version(mime).as_bytes());
    field(fonts.join("\u{1f}").as_bytes());
    field(spec.as_bytes());
    hex(&h.finalize())
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

/// A figure the report needs drawn.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FigureJob {
    pub key: String,
    pub mime: String,
    pub spec: String,
    /// Fonts to draw it with (the report's).
    pub font: String,
}

/// Renderings on disk: `<dir>/<key>.svg`.
#[derive(Debug, Clone)]
pub struct Cache {
    dir: PathBuf,
}

impl Cache {
    pub fn new(dir: PathBuf) -> Cache {
        Cache { dir }
    }

    fn path(&self, key: &str) -> Option<PathBuf> {
        (key.len() == 64 && key.bytes().all(|b| b.is_ascii_hexdigit())).then(|| self.dir.join(format!("{key}.svg")))
    }

    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        std::fs::read(self.path(key)?).ok()
    }

    /// Keep a rendering. It must be an SVG document without script; it is written whole
    /// (temp file, then rename), so a reader never sees half of one.
    pub fn put(&self, key: &str, svg: &str) -> Result<(), String> {
        let path = self.path(key).ok_or("not a figure key")?;
        check_svg(svg)?;
        std::fs::create_dir_all(&self.dir).map_err(|e| e.to_string())?;
        let tmp = path.with_extension("svg.tmp");
        std::fs::write(&tmp, svg).map_err(|e| e.to_string())?;
        std::fs::rename(&tmp, &path).map_err(|e| e.to_string())
    }
}

/// What a static rendering may be: an SVG document, bounded, with no script or external
/// references (Typst ignores both, but nothing else should rely on that).
pub fn check_svg(svg: &str) -> Result<(), String> {
    if svg.len() > MAX_SVG {
        return Err("the figure's SVG is over 16 MB".into());
    }
    if !svg.trim_start().starts_with("<svg") {
        return Err("not an SVG document".into());
    }
    let lower = svg.to_ascii_lowercase();
    for bad in ["<script", "javascript:", "<foreignobject", "href=\"http", "href='http"] {
        if lower.contains(bad) {
            return Err(format!("the figure's SVG contains {bad}"));
        }
    }
    Ok(())
}

/// The figures in `results` without a static rendering (their cell gave none and none is
/// cached), deduplicated by key.
pub fn missing(results: &std::collections::HashMap<String, CellResult>, cache: &Cache) -> Vec<FigureJob> {
    let fonts = report_fonts();
    let font = fonts.join(", ");
    let mut seen = std::collections::HashSet::new();
    let mut ids: Vec<&String> = results.keys().collect();
    ids.sort();
    let mut jobs = Vec::new();
    for id in ids {
        let Some(py) = &results[id].python else { continue };
        for o in &py.outputs {
            let OutputView::Figure { mime, spec, static_src: None, .. } = o else { continue };
            let key = static_key(mime, spec, &fonts);
            if cache.get(&key).is_none() && seen.insert(key.clone()) {
                jobs.push(FigureJob { key, mime: mime.clone(), spec: spec.clone(), font: font.clone() });
            }
        }
    }
    jobs
}

/// The figure frame's CSP (ADR-0020): its own scripts and eval (plotly's WebGL traces compile
/// shaders with `Function`), inline styles and data images; nothing else, no network.
pub const FRAME_CSP: &str = "default-src 'none'; script-src cruncha-figure://localhost http://cruncha-figure.localhost 'unsafe-eval'; \
     style-src 'unsafe-inline'; img-src data: blob:; font-src data:";

/// What the `cruncha-figure` scheme serves: the frame page and its script, from the binary.
/// Returns (status, content type, body).
pub fn frame_file(path: &str) -> (u16, &'static str, &'static [u8]) {
    match path {
        "/frame.html" => (200, "text/html; charset=utf-8", include_bytes!("../figure/frame.html")),
        "/frame.js" => (200, "text/javascript; charset=utf-8", include_bytes!("../figure/frame.js")),
        _ => (404, "text/plain", b"not found"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_pinned_renderer_is_the_app_s() {
        let pkg: serde_json::Value =
            serde_json::from_str(include_str!("../../package.json")).expect("package.json");
        assert_eq!(pkg["dependencies"]["plotly.js-dist-min"], PLOTLY_JS, "bump PLOTLY_JS with the npm pin");
    }

    #[test]
    fn keys_cover_spec_mime_renderer_and_fonts() {
        let f = vec!["Helvetica".to_string()];
        let k = static_key("application/vnd.plotly.v1+json", "{}", &f);
        assert_eq!(k.len(), 64);
        assert_eq!(k, static_key("application/vnd.plotly.v1+json", "{}", &f));
        assert_ne!(k, static_key("application/vnd.plotly.v1+json", "{ }", &f));
        assert_ne!(k, static_key("application/vnd.vegalite.v5+json", "{}", &f));
        assert_ne!(k, static_key("application/vnd.plotly.v1+json", "{}", &["Arial".to_string()]));
    }

    #[test]
    fn the_scheme_serves_only_the_frame() {
        assert_eq!(frame_file("/frame.html").0, 200);
        assert_eq!(frame_file("/frame.js").0, 200);
        for p in ["/", "/../Cargo.toml", "/plotly.min.js", "/frame.html?x"] {
            assert_eq!(frame_file(p).0, 404, "{p}");
        }
        assert!(!FRAME_CSP.contains("connect-src") && FRAME_CSP.starts_with("default-src 'none'"));
    }

    #[test]
    fn only_plain_svg_is_kept() {
        let dir = tempfile::tempdir().unwrap();
        let c = Cache::new(dir.path().join("figures"));
        let k = "a".repeat(64);
        assert!(c.put(&k, "<svg xmlns='http://www.w3.org/2000/svg'><script>x</script></svg>").is_err());
        assert!(c.put(&k, "<html></html>").is_err());
        assert!(c.put("../../etc", "<svg/>").is_err(), "keys are hashes, never paths");
        c.put(&k, "<svg xmlns='http://www.w3.org/2000/svg'/>").unwrap();
        assert_eq!(c.get(&k).unwrap(), b"<svg xmlns='http://www.w3.org/2000/svg'/>");
    }
}
