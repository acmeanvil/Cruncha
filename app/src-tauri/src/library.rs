// SPDX-License-Identifier: GPL-3.0-or-later
//! The template libraries (ADR-0014 §4, §5): the built-in one bundled with the app
//! (read-only) and the user's own, a folder of `.cruncha.md` files (`<app data>/library/`
//! unless Settings points elsewhere). Files are read afresh on every listing, so edits made
//! outside Cruncha show up the next time the library is opened.
//!
//! An entry's key is `builtin:<path>` or `user:<path>`, its path relative to its library.
//! User files are named after their title (renaming a snippet renames its file); the `id`
//! never changes, so provenance in documents still points to it. Deleting moves a file to
//! `.trash/` in the user's library, until the trash is emptied. Built-in templates can only
//! be hidden (by id, in the preferences).

use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path, PathBuf};

use cruncha_templates::{Cell, Entry, Manifest, Template};
use serde::{Deserialize, Serialize};

type Result<T> = std::result::Result<T, String>;

pub const TRASH: &str = ".trash";

/// The two folders.
#[derive(Debug, Clone)]
pub struct Library {
    pub builtin: PathBuf,
    pub user: PathBuf,
}

/// A template as the Templates tab lists it.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateSummary {
    pub key: String,
    /// `builtin` or `user`.
    pub library: &'static str,
    pub path: String,
    pub id: String,
    pub version: String,
    pub title: String,
    pub summary: Option<String>,
    pub category: Vec<String>,
    pub tags: Vec<String>,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    /// Hidden by the user (built-in templates only).
    pub hidden: bool,
    /// Why the file doesn't read (the other fields are then empty but `title`, its name).
    pub error: Option<String>,
}

/// A worked example's outcome, run here with the calc engine.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TestView {
    pub name: String,
    pub source: Option<String>,
    pub passed: bool,
    /// Why it wasn't run here (code cells).
    pub skipped: Option<String>,
    pub lines: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceView {
    pub work: String,
    pub edition: Option<String>,
    pub locator: Option<String>,
    pub url: Option<String>,
}

/// A template's cell as the editor makes a block of it.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CellView {
    pub kind: String,
    pub source: String,
    /// `{ description }` when it has one.
    #[serde(default)]
    pub attrs: serde_json::Value,
}

/// Everything the preview and the insert dialog need.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateDetail {
    #[serde(flatten)]
    pub summary: TemplateSummary,
    pub from: Option<String>,
    pub license: Option<String>,
    pub references: Vec<ReferenceView>,
    pub tests: Vec<TestView>,
    /// What validation found wrong (empty: valid).
    pub problems: Vec<String>,
    pub cells: Vec<CellView>,
}

/// A snippet's title, summary, category and tags (Save as snippet, Edit details).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Details {
    pub title: String,
    pub summary: Option<String>,
    pub category: Vec<String>,
    pub tags: Vec<String>,
}

/// A file in the trash.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrashedView {
    /// Its file name in `.trash/`.
    pub name: String,
    pub title: String,
    pub id: Option<String>,
}

/// A template's cell for the editor.
pub fn cell_view(c: &Cell) -> CellView {
    let attrs = |d: &Option<String>| match d.as_deref().map(str::trim) {
        Some(d) if !d.is_empty() => serde_json::json!({ "description": d }),
        _ => serde_json::json!({}),
    };
    match c {
        Cell::Prose { source } => CellView { kind: "prose".into(), source: source.clone(), attrs: serde_json::json!({}) },
        Cell::Calc { source, description } => CellView { kind: "calc".into(), source: source.clone(), attrs: attrs(description) },
        Cell::Code { lang, source, description } => CellView {
            kind: match lang.as_str() {
                "rust" => "rust",
                "tex" => "tex",
                _ => "python",
            }
            .into(),
            source: source.clone(),
            attrs: attrs(description),
        },
    }
}

/// An editor cell as a template's.
fn template_cell(c: &CellView) -> Result<Cell> {
    let description = c.attrs.get("description").and_then(|d| d.as_str()).map(str::trim).filter(|d| !d.is_empty()).map(str::to_owned);
    Ok(match c.kind.as_str() {
        "prose" => Cell::Prose { source: c.source.clone() },
        "calc" => Cell::Calc { source: c.source.clone(), description },
        "python" | "rust" | "tex" => Cell::Code { lang: c.kind.clone(), source: c.source.clone(), description },
        k => return Err(format!("a {k} cell can't go in a snippet")),
    })
}

/// A file name from a title: `Load combination (ASCE 7)` → `load-combination-asce-7`.
pub fn slug(title: &str) -> String {
    let mut s = String::new();
    for c in title.chars() {
        if c.is_ascii_alphanumeric() {
            s.push(c.to_ascii_lowercase());
        } else if !s.ends_with('-') && !s.is_empty() {
            s.push('-');
        }
    }
    let s: String = s.trim_end_matches('-').chars().take(48).collect();
    let s = s.trim_end_matches('-').to_owned();
    if s.is_empty() { "snippet".into() } else { s }
}

fn bump(version: &str, minor: bool) -> String {
    let p: Vec<u64> = version.split('.').map(|x| x.parse().unwrap_or(0)).collect();
    let (a, b, c) = (p.first().copied().unwrap_or(1), p.get(1).copied().unwrap_or(0), p.get(2).copied().unwrap_or(0));
    if minor { format!("{a}.{}.0", b + 1) } else { format!("{a}.{b}.{}", c + 1) }
}

/// Write `text` to `path` through a temporary file, so a crash never leaves half a file.
fn write_atomic(path: &Path, text: &str) -> Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    let tmp = path.with_extension("md.tmp");
    fs::write(&tmp, text).map_err(|e| format!("{}: {e}", tmp.display()))?;
    fs::rename(&tmp, path).map_err(|e| format!("{}: {e}", path.display()))
}

/// A relative path that stays inside its folder.
fn inside(root: &Path, rel: &str) -> Result<PathBuf> {
    let p = Path::new(rel);
    if rel.is_empty() || !p.components().all(|c| matches!(c, Component::Normal(_))) {
        return Err(format!("not a library path: {rel}"));
    }
    Ok(root.join(p))
}

impl Library {
    fn summary(&self, library: &'static str, e: &Entry, hidden: &BTreeSet<String>) -> TemplateSummary {
        let key = format!("{library}:{}", e.path);
        match &e.template {
            Ok(t) => {
                let m = &t.manifest;
                TemplateSummary {
                    key,
                    library,
                    path: e.path.clone(),
                    id: m.id.clone(),
                    version: m.version.clone(),
                    title: m.title.clone(),
                    summary: m.summary.clone(),
                    category: m.category.clone(),
                    tags: m.tags.clone(),
                    inputs: m.inputs.clone(),
                    outputs: m.outputs.clone(),
                    hidden: library == "builtin" && hidden.contains(&m.id),
                    error: None,
                }
            }
            Err(err) => TemplateSummary {
                key,
                library,
                path: e.path.clone(),
                id: String::new(),
                version: String::new(),
                title: e.path.rsplit('/').next().unwrap_or(&e.path).trim_end_matches(".cruncha.md").to_owned(),
                summary: None,
                category: Vec::new(),
                tags: Vec::new(),
                inputs: Vec::new(),
                outputs: Vec::new(),
                hidden: false,
                error: Some(err.clone()),
            },
        }
    }

    /// Both libraries: the user's first, then the built-in one, each sorted by path.
    pub fn list(&self, hidden: &BTreeSet<String>) -> Vec<TemplateSummary> {
        let mut out: Vec<TemplateSummary> =
            cruncha_templates::load_dir(&self.user).iter().map(|e| self.summary("user", e, hidden)).collect();
        out.extend(cruncha_templates::load_dir(&self.builtin).iter().map(|e| self.summary("builtin", e, hidden)));
        out
    }

    /// The file a key names.
    fn file(&self, key: &str) -> Result<(&'static str, PathBuf, String)> {
        let (lib, rel) = key.split_once(':').ok_or_else(|| format!("not a template key: {key}"))?;
        let (lib, root): (&'static str, &Path) = match lib {
            "builtin" => ("builtin", &self.builtin),
            "user" => ("user", &self.user),
            _ => return Err(format!("not a template key: {key}")),
        };
        Ok((lib, inside(root, rel)?, rel.to_owned()))
    }

    /// A template's file as it is.
    pub fn text(&self, key: &str) -> Result<String> {
        let (_, path, _) = self.file(key)?;
        fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))
    }

    fn read(&self, key: &str) -> Result<(&'static str, PathBuf, Template)> {
        let (lib, path, _) = self.file(key)?;
        let text = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let t = cruncha_templates::parse(&text).map_err(|e| format!("{}: {e}", path.display()))?;
        Ok((lib, path, t))
    }

    /// One template with its cells, references, validation and worked examples (calcs run
    /// here; templates with code cells say their examples are checked where Python is).
    pub fn detail(&self, key: &str, hidden: &BTreeSet<String>) -> Result<TemplateDetail> {
        let (lib, path, rel) = self.file(key)?;
        let text = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let template = cruncha_templates::parse(&text);
        let summary = self.summary(lib, &Entry { path: rel, template: template.clone() }, hidden);
        let t = template.map_err(|e| format!("{}: {e}", path.display()))?;
        let m = &t.manifest;
        let tests = cruncha_templates::run_tests(&t)
            .into_iter()
            .zip(&m.tests)
            .map(|(r, case)| TestView {
                name: r.name,
                source: case.source.clone(),
                passed: r.passed,
                skipped: r.skipped.map(|_| "has code cells: checked by Cruncha's tests, not here".to_owned()),
                lines: r.lines,
            })
            .collect();
        Ok(TemplateDetail {
            summary,
            from: m.from.clone(),
            license: m.license.clone(),
            references: m
                .references
                .iter()
                .map(|r| ReferenceView { work: r.work.clone(), edition: r.edition.clone(), locator: r.locator.clone(), url: r.url.clone() })
                .collect(),
            tests,
            problems: cruncha_templates::validate(&t),
            cells: t.cells.iter().map(cell_view).collect(),
        })
    }

    /// Ids in either library.
    fn ids(&self) -> BTreeSet<String> {
        cruncha_templates::load_dir(&self.user)
            .into_iter()
            .chain(cruncha_templates::load_dir(&self.builtin))
            .filter_map(|e| e.template.ok().map(|t| t.manifest.id))
            .collect()
    }

    /// A new user id from a title: `user.load-combination`, `user.load-combination-2`, …
    fn new_id(&self, title: &str) -> String {
        let ids = self.ids();
        let base = format!("user.{}", slug(title));
        std::iter::once(base.clone())
            .chain((2..).map(|k| format!("{base}-{k}")))
            .find(|id| !ids.contains(id))
            .unwrap()
    }

    /// A free file name in the user's library for `title` (`keep`: the file being renamed,
    /// which may keep its own name).
    fn new_path(&self, title: &str, keep: Option<&Path>) -> PathBuf {
        let base = slug(title);
        std::iter::once(format!("{base}.cruncha.md"))
            .chain((2..).map(|k| format!("{base}-{k}.cruncha.md")))
            .map(|n| self.user.join(n))
            .find(|p| Some(p.as_path()) == keep || !p.exists())
            .unwrap()
    }

    fn user_key(&self, path: &Path) -> String {
        let rel = path.strip_prefix(&self.user).unwrap_or(path);
        format!("user:{}", rel.components().map(|c| c.as_os_str().to_string_lossy()).collect::<Vec<_>>().join("/"))
    }

    fn manifest(&self, details: &Details, id: String, cells: &[Cell]) -> Result<Manifest> {
        let title = details.title.trim();
        if title.is_empty() {
            return Err("a snippet needs a title".into());
        }
        Ok(Manifest {
            format: cruncha_templates::FORMAT,
            id,
            version: "1.0.0".into(),
            title: title.to_owned(),
            summary: details.summary.as_deref().map(str::trim).filter(|s| !s.is_empty()).map(str::to_owned),
            category: details.category.iter().map(|c| c.trim().to_owned()).filter(|c| !c.is_empty()).collect(),
            tags: details.tags.iter().map(|c| c.trim().to_owned()).filter(|c| !c.is_empty()).collect(),
            from: None,
            license: None,
            inputs: cruncha_templates::placeholder_inputs(cells),
            outputs: Vec::new(),
            references: Vec::new(),
            tests: Vec::new(),
        })
    }

    /// Save cells from the editor as a new snippet in the user's library. Its inputs are
    /// the names its calcs define with a placeholder (`_`). Returns its key.
    pub fn save_snippet(&self, details: &Details, cells: &[CellView]) -> Result<String> {
        let cells: Vec<Cell> = cells.iter().map(template_cell).collect::<Result<_>>()?;
        if cells.is_empty() {
            return Err("select the cells to save first".into());
        }
        let manifest = self.manifest(details, self.new_id(&details.title), &cells)?;
        let path = self.new_path(&manifest.title, None);
        write_atomic(&path, &cruncha_templates::write(&Template { manifest, cells })?)?;
        Ok(self.user_key(&path))
    }

    fn user_only(&self, key: &str) -> Result<(PathBuf, Template)> {
        let (lib, path, t) = self.read(key)?;
        if lib != "user" {
            return Err("built-in templates can't be changed: duplicate it into your library first".into());
        }
        Ok((path, t))
    }

    /// Replace a snippet's cells (a new minor version). Inputs follow the placeholders;
    /// the worked examples stay, so the tests say if they no longer hold.
    pub fn update_cells(&self, key: &str, cells: &[CellView]) -> Result<String> {
        let (path, mut t) = self.user_only(key)?;
        let cells: Vec<Cell> = cells.iter().map(template_cell).collect::<Result<_>>()?;
        if cells.is_empty() {
            return Err("select the cells to save first".into());
        }
        t.manifest.inputs = cruncha_templates::placeholder_inputs(&cells);
        t.manifest.version = bump(&t.manifest.version, true);
        t.cells = cells;
        write_atomic(&path, &cruncha_templates::write(&t)?)?;
        Ok(key.to_owned())
    }

    /// Change a snippet's title, summary, category and tags (a new patch version). A new
    /// title renames its file; the id stays. Returns its (new) key.
    pub fn update_details(&self, key: &str, details: &Details) -> Result<String> {
        let (path, mut t) = self.user_only(key)?;
        let m = self.manifest(details, t.manifest.id.clone(), &t.cells)?;
        let renamed = m.title != t.manifest.title;
        t.manifest.title = m.title;
        t.manifest.summary = m.summary;
        t.manifest.category = m.category;
        t.manifest.tags = m.tags;
        t.manifest.version = bump(&t.manifest.version, false);
        let to = if renamed && path.parent() == Some(self.user.as_path()) { self.new_path(&t.manifest.title, Some(&path)) } else { path.clone() };
        write_atomic(&to, &cruncha_templates::write(&t)?)?;
        if to != path {
            fs::remove_file(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        }
        Ok(self.user_key(&to))
    }

    /// Copy a template (built-in or the user's) into the user's library under a new id:
    /// how a built-in template is customised.
    pub fn duplicate(&self, key: &str) -> Result<String> {
        let (_, _, mut t) = self.read(key)?;
        let title = format!("{} (copy)", t.manifest.title);
        let base = t.manifest.id.trim_start_matches("user.").to_owned();
        let ids = self.ids();
        t.manifest.id = std::iter::once(format!("user.{base}"))
            .chain((2..).map(|k| format!("user.{base}-{k}")))
            .find(|id| !ids.contains(id))
            .unwrap();
        t.manifest.title = title;
        t.manifest.version = "1.0.0".into();
        let path = self.new_path(&t.manifest.title, None);
        write_atomic(&path, &cruncha_templates::write(&t)?)?;
        Ok(self.user_key(&path))
    }

    /// Move a snippet to the trash.
    pub fn delete(&self, key: &str) -> Result<()> {
        let (lib, path, _) = self.file(key)?;
        if lib != "user" {
            return Err("built-in templates can't be deleted, only hidden".into());
        }
        let trash = self.user.join(TRASH);
        fs::create_dir_all(&trash).map_err(|e| format!("{}: {e}", trash.display()))?;
        let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        let stem = name.trim_end_matches(".cruncha.md").to_owned();
        let to = std::iter::once(name.clone())
            .chain((2..).map(|k| format!("{stem}-{k}.cruncha.md")))
            .map(|n| trash.join(n))
            .find(|p| !p.exists())
            .unwrap();
        fs::rename(&path, &to).map_err(|e| format!("{}: {e}", path.display()))
    }

    /// What's in the trash.
    pub fn trash(&self) -> Vec<TrashedView> {
        cruncha_templates::load_dir(&self.user.join(TRASH))
            .into_iter()
            .filter(|e| !e.path.contains('/'))
            .map(|e| match e.template {
                Ok(t) => TrashedView { name: e.path, title: t.manifest.title, id: Some(t.manifest.id) },
                Err(_) => TrashedView { title: e.path.trim_end_matches(".cruncha.md").to_owned(), name: e.path, id: None },
            })
            .collect()
    }

    /// Put a trashed snippet back (under a free name). Returns its key.
    pub fn restore(&self, name: &str) -> Result<String> {
        let from = inside(&self.user.join(TRASH), name)?;
        let text = fs::read_to_string(&from).map_err(|e| format!("{}: {e}", from.display()))?;
        let t = cruncha_templates::parse(&text)?;
        if self.ids().contains(&t.manifest.id) {
            return Err(format!("your library already has {}: delete or duplicate that one first", t.manifest.id));
        }
        let to = self.new_path(&t.manifest.title, None);
        fs::rename(&from, &to).map_err(|e| format!("{}: {e}", from.display()))?;
        Ok(self.user_key(&to))
    }

    /// Delete everything in the trash for good.
    pub fn empty_trash(&self) -> Result<()> {
        let trash = self.user.join(TRASH);
        for e in self.trash() {
            let p = trash.join(&e.name);
            fs::remove_file(&p).map_err(|err| format!("{}: {err}", p.display()))?;
        }
        Ok(())
    }

    /// Copy `.cruncha.md` files into the user's library. Each must read as a template, and
    /// its id must be new to both libraries. Returns the new keys.
    pub fn import(&self, files: &[PathBuf]) -> Result<Vec<String>> {
        let mut ids = self.ids();
        let mut read = Vec::new();
        for f in files {
            let text = fs::read_to_string(f).map_err(|e| format!("{}: {e}", f.display()))?;
            let t = cruncha_templates::parse(&text).map_err(|e| format!("{}: {e}", f.display()))?;
            if !ids.insert(t.manifest.id.clone()) {
                return Err(format!("{}: a template with id {} is already in a library", f.display(), t.manifest.id));
            }
            read.push((t, text));
        }
        let mut keys = Vec::new();
        for (t, text) in read {
            let path = self.new_path(&t.manifest.title, None);
            write_atomic(&path, &text)?;
            keys.push(self.user_key(&path));
        }
        Ok(keys)
    }

    /// Copy a template's file to `dest`.
    pub fn export(&self, key: &str, dest: &Path) -> Result<()> {
        let (_, path, _) = self.file(key)?;
        fs::copy(&path, dest).map(|_| ()).map_err(|e| format!("{}: {e}", dest.display()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lib(dir: &Path) -> Library {
        Library { builtin: PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../templates")), user: dir.join("library") }
    }

    fn cells() -> Vec<CellView> {
        vec![
            CellView { kind: "prose".into(), source: r#"{"type":"paragraph","content":[{"type":"text","text":"Factored load"}]}"#.into(), attrs: serde_json::json!({}) },
            CellView { kind: "calc".into(), source: "D = 10 kN_".into(), attrs: serde_json::json!({ "description": "Dead load" }) },
            CellView { kind: "calc".into(), source: "L = 5 kN_".into(), attrs: serde_json::json!({}) },
            CellView { kind: "calc".into(), source: "U = 1.2 D + 1.6 L = ?? kN".into(), attrs: serde_json::json!({}) },
        ]
    }

    #[test]
    fn the_built_in_library_lists_and_previews() {
        let d = tempfile::tempdir().unwrap();
        let l = lib(d.path());
        let all = l.list(&BTreeSet::new());
        assert!(all.len() >= 13 && all.iter().all(|t| t.library == "builtin" && t.error.is_none()));
        let bolts = all.iter().find(|t| t.id == "structural.bolts.shearTension").unwrap();
        let det = l.detail(&bolts.key, &BTreeSet::new()).unwrap();
        assert!(det.problems.is_empty(), "{:?}", det.problems);
        assert!(!det.tests.is_empty() && det.tests.iter().all(|t| t.passed), "{:?}", det.tests);
        assert_eq!(det.cells.iter().find(|c| c.kind == "calc").unwrap().attrs["description"], "Nominal bolt diameter");
        let hidden = BTreeSet::from([bolts.id.clone()]);
        assert!(l.list(&hidden).iter().find(|t| t.id == bolts.id).unwrap().hidden);
        assert!(l.delete(&bolts.key).is_err(), "built-in templates can't be deleted");
        assert!(l.detail("builtin:../Cargo.toml", &hidden).is_err(), "keys stay inside their library");
    }

    #[test]
    fn snippets_save_update_rename_and_go_to_the_trash() {
        let d = tempfile::tempdir().unwrap();
        let l = lib(d.path());
        let details = Details { title: "Factored load (LRFD)".into(), category: vec!["Loads".into()], ..Default::default() };
        let key = l.save_snippet(&details, &cells()).unwrap();
        assert_eq!(key, "user:factored-load-lrfd.cruncha.md");
        let det = l.detail(&key, &BTreeSet::new()).unwrap();
        assert_eq!((det.summary.id.as_str(), det.summary.version.as_str()), ("user.factored-load-lrfd", "1.0.0"));
        assert_eq!(det.summary.inputs, ["D", "L"]);
        assert_eq!(det.cells, cells(), "the cells come back as they were saved");
        assert!(det.problems.is_empty(), "{:?}", det.problems);

        // A second snippet with the same title gets its own id and file.
        let second = l.save_snippet(&details, &cells()[1..2]).unwrap();
        assert_eq!(second, "user:factored-load-lrfd-2.cruncha.md");
        assert_eq!(l.detail(&second, &BTreeSet::new()).unwrap().summary.id, "user.factored-load-lrfd-2");

        // Update the cells: a minor version; edit details: a patch version and a new file name.
        l.update_cells(&key, &cells()[1..]).unwrap();
        assert_eq!(l.detail(&key, &BTreeSet::new()).unwrap().summary.version, "1.1.0");
        let renamed = l.update_details(&key, &Details { title: "Load combination 2".into(), ..details.clone() }).unwrap();
        assert_eq!(renamed, "user:load-combination-2.cruncha.md");
        let det = l.detail(&renamed, &BTreeSet::new()).unwrap();
        assert_eq!((det.summary.id.as_str(), det.summary.version.as_str()), ("user.factored-load-lrfd", "1.1.1"), "the id stays");
        assert!(!l.user.join("factored-load-lrfd.cruncha.md").exists());

        // Delete to the trash, then restore.
        l.delete(&renamed).unwrap();
        assert!(l.detail(&renamed, &BTreeSet::new()).is_err());
        let trash = l.trash();
        assert_eq!(trash.len(), 1);
        assert_eq!(trash[0].title, "Load combination 2");
        assert!(l.list(&BTreeSet::new()).iter().all(|t| t.id != "user.factored-load-lrfd"), "the trash isn't listed");
        let back = l.restore(&trash[0].name).unwrap();
        assert_eq!(l.detail(&back, &BTreeSet::new()).unwrap().summary.id, "user.factored-load-lrfd");
        l.delete(&back).unwrap();
        l.empty_trash().unwrap();
        assert!(l.trash().is_empty());
    }

    #[test]
    fn a_built_in_template_is_customised_by_duplicating_it() {
        let d = tempfile::tempdir().unwrap();
        let l = lib(d.path());
        let src = l.list(&BTreeSet::new()).into_iter().find(|t| t.id == "physics.kinematics.fromTime").unwrap();
        let copy = l.duplicate(&src.key).unwrap();
        let det = l.detail(&copy, &BTreeSet::new()).unwrap();
        assert_eq!(det.summary.id, "user.physics.kinematics.fromTime");
        assert!(det.summary.title.ends_with("(copy)"));
        assert!(det.tests.iter().all(|t| t.passed), "its worked examples come along");
        assert!(l.update_cells(&src.key, &cells()).is_err(), "the built-in one stays as it is");

        // Export it, and importing it again is refused: the id is taken.
        let out = d.path().join("out.cruncha.md");
        l.export(&copy, &out).unwrap();
        assert!(l.import(std::slice::from_ref(&out)).unwrap_err().contains("already in a library"));
        l.delete(&copy).unwrap();
        l.empty_trash().unwrap();
        let keys = l.import(&[out]).unwrap();
        assert_eq!(l.detail(&keys[0], &BTreeSet::new()).unwrap().summary.id, "user.physics.kinematics.fromTime");
    }

    #[test]
    fn titles_make_file_names() {
        assert_eq!(slug("Load combination (ASCE 7-22)"), "load-combination-asce-7-22");
        assert_eq!(slug("σ — ½"), "snippet");
        assert_eq!(bump("1.2.3", true), "1.3.0");
        assert_eq!(bump("1.2.3", false), "1.2.4");
    }
}
