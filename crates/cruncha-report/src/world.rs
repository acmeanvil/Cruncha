// SPDX-License-Identifier: GPL-3.0-or-later

//! A Typst `World` that holds everything in memory: the template, the report as JSON, and its
//! images. There is no file system, network or package access, so evaluating a report's
//! content can't reach anything outside it. "Today" is the version's commit date, so the same
//! version always gives the same PDF.

use std::collections::HashMap;
use std::sync::OnceLock;

use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Datetime, Duration};
use typst::syntax::{FileId, RootedPath, Source, VirtualPath, VirtualRoot};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, LibraryExt, World};
use typst_kit::fonts::FontStore;

/// Fonts: Typst's bundled ones plus the system's, scanned once per process.
pub(crate) fn fonts() -> &'static FontStore {
    static FONTS: OnceLock<FontStore> = OnceLock::new();
    FONTS.get_or_init(|| {
        let mut store = FontStore::new();
        store.extend(typst_kit::fonts::system());
        store.extend(typst_kit::fonts::embedded());
        store
    })
}

fn library() -> &'static LazyHash<Library> {
    static LIB: OnceLock<LazyHash<Library>> = OnceLock::new();
    LIB.get_or_init(|| LazyHash::new(Library::default()))
}

/// The id of a file at `path` in the report's (only) root.
fn file_id(path: &str) -> FileId {
    let vpath = VirtualPath::new(path).unwrap_or_else(|_| VirtualPath::new("/invalid").unwrap());
    RootedPath::new(VirtualRoot::Project, vpath).intern()
}

fn not_found(id: FileId) -> FileError {
    FileError::NotFound(id.vpath().get_without_slash().into())
}

pub(crate) struct ReportWorld {
    main: FileId,
    sources: HashMap<FileId, Source>,
    files: HashMap<FileId, Bytes>,
    today: Option<Datetime>,
}

impl ReportWorld {
    /// `main` is the template; `files` are served at their paths (`/report.json`, `/img/1.png`).
    pub(crate) fn new(main: &str, files: Vec<(String, Vec<u8>)>, today: Option<(i32, u8, u8)>) -> Self {
        let id = file_id("/main.typ");
        let mut sources = HashMap::new();
        sources.insert(id, Source::new(id, main.to_owned()));
        let files = files
            .into_iter()
            .map(|(path, bytes)| (file_id(&path), Bytes::new(bytes)))
            .collect();
        ReportWorld {
            main: id,
            sources,
            files,
            today: today.and_then(|(y, m, d)| Datetime::from_ymd(y, m, d)),
        }
    }
}

impl World for ReportWorld {
    fn library(&self) -> &LazyHash<Library> {
        library()
    }

    fn book(&self) -> &LazyHash<FontBook> {
        fonts().book()
    }

    fn main(&self) -> FileId {
        self.main
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        self.sources
            .get(&id)
            .cloned()
            .ok_or_else(|| not_found(id))
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        if let Some(b) = self.files.get(&id) {
            return Ok(b.clone());
        }
        if let Some(s) = self.sources.get(&id) {
            return Ok(Bytes::from_string(s.text().to_owned()));
        }
        Err(not_found(id))
    }

    fn font(&self, index: usize) -> Option<Font> {
        fonts().font(index)
    }

    fn today(&self, _offset: Option<Duration>) -> Option<Datetime> {
        self.today
    }
}
