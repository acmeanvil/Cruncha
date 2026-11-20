// SPDX-License-Identifier: GPL-3.0-or-later
//! Cruncha's desktop shell (ADR-0008): a Tauri backend that holds the open document's
//! `Store`, with a web UI in `app/src`. The skeleton views documents read-only (cells at any
//! version, history, tags) and has the history-retention settings (ADR-0011).

mod engine;
mod session;

use std::path::PathBuf;

use cruncha_store::{Cell, Retention};
use std::collections::HashMap;

use engine::CellResult;
use session::{
    At, ChangeView, CommitView, DocInfo, Prefs, PreviewView, PruneView, Session, VersionView,
};
use tauri::{Manager, RunEvent, State};
use tokio::sync::Mutex;

type Shared = Mutex<Session>;
type Result<T> = session::Result<T>;

#[tauri::command]
async fn open_document(path: PathBuf, s: State<'_, Shared>) -> Result<DocInfo> {
    s.lock().await.open(&path).await
}

#[tauri::command]
async fn create_document(path: PathBuf, s: State<'_, Shared>) -> Result<DocInfo> {
    s.lock().await.create(&path).await
}

#[tauri::command]
async fn commit(
    changes: Vec<ChangeView>,
    message: String,
    s: State<'_, Shared>,
) -> Result<CommitView> {
    s.lock().await.commit(changes, &message).await
}

#[tauri::command]
async fn results(at: At, s: State<'_, Shared>) -> Result<HashMap<String, CellResult>> {
    s.lock().await.results(&at).await
}

#[tauri::command]
async fn snapshot(name: String, note: String, s: State<'_, Shared>) -> Result<DocInfo> {
    s.lock().await.snapshot(&name, &note).await
}

#[tauri::command]
async fn restore(at: At, s: State<'_, Shared>) -> Result<CommitView> {
    s.lock().await.restore(&at).await
}

#[tauri::command]
async fn preview_calc(source: String, s: State<'_, Shared>) -> Result<CellResult> {
    s.lock().await.preview_calc(&source)
}

#[tauri::command]
async fn create_sample_document(path: PathBuf, s: State<'_, Shared>) -> Result<DocInfo> {
    s.lock().await.create_sample(&path).await
}

#[tauri::command]
async fn close_document(s: State<'_, Shared>) -> Result<Option<PruneView>> {
    Ok(s.lock().await.close().await)
}

#[tauri::command]
async fn document_info(s: State<'_, Shared>) -> Result<Option<DocInfo>> {
    let s = s.lock().await;
    if !s.is_open() {
        return Ok(None);
    }
    s.info().await.map(Some)
}

#[tauri::command]
async fn cells(version: Option<u64>, s: State<'_, Shared>) -> Result<Vec<Cell>> {
    s.lock().await.cells(version).await
}

#[tauri::command]
async fn cells_at_ref(name: String, s: State<'_, Shared>) -> Result<Vec<Cell>> {
    s.lock().await.cells_at_ref(&name).await
}

#[tauri::command]
async fn history(s: State<'_, Shared>) -> Result<Vec<VersionView>> {
    s.lock().await.history().await
}

#[tauri::command]
async fn get_prefs(s: State<'_, Shared>) -> Result<Prefs> {
    Ok(s.lock().await.prefs().clone())
}

#[tauri::command]
async fn forget_recent(path: String, s: State<'_, Shared>) -> Result<()> {
    s.lock().await.forget_recent(&path)
}

#[tauri::command]
async fn set_prefs(mut prefs: Prefs, s: State<'_, Shared>) -> Result<()> {
    let mut s = s.lock().await;
    // The recent list is the backend's; the front end's copy may be stale.
    prefs.recent = s.prefs().recent.clone();
    s.set_prefs(prefs)
}

#[tauri::command]
async fn set_document_retention(
    policy: Option<Retention>,
    s: State<'_, Shared>,
) -> Result<DocInfo> {
    s.lock().await.set_document_retention(policy).await
}

#[tauri::command]
async fn retention_preview(
    policy: Option<Retention>,
    s: State<'_, Shared>,
) -> Result<PreviewView> {
    s.lock().await.retention_preview(policy).await
}

#[tauri::command]
async fn prune_now(s: State<'_, Shared>) -> Result<Option<PruneView>> {
    s.lock().await.prune_now().await
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let prefs = app.path().app_config_dir()?.join("preferences.json");
            app.manage::<Shared>(Mutex::new(Session::new(Some(prefs))));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            open_document,
            create_document,
            create_sample_document,
            commit,
            results,
            preview_calc,
            snapshot,
            restore,
            close_document,
            document_info,
            cells,
            cells_at_ref,
            history,
            get_prefs,
            set_prefs,
            forget_recent,
            set_document_retention,
            retention_preview,
            prune_now,
        ])
        .build(tauri::generate_context!())
        .expect("failed to start Cruncha")
        .run(|app, event| {
            if let RunEvent::Exit = event {
                // Release the document (and apply its retention policy if this session edited
                // it) before the process ends.
                let s = app.state::<Shared>();
                tauri::async_runtime::block_on(async {
                    s.lock().await.close().await;
                });
            }
        });
}
