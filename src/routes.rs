//! # CYBERDECK Routes
//!
//! The HTTP interface for the CYBERDECK system: the tool hub, the
//! diagnostic-scan dispatcher, and the reports viewer.

use axum::{
    extract::{Path as AxPath, State},
    response::{Html, IntoResponse},
    Json,
};

use crate::dispatcher;
use crate::hub;
use crate::types::{CyberdeckCommand, SharedCyberdeckState};
use crate::views;

use serde::Deserialize;

/// Wraps a body in the full page shell, pulling the active CYBERGRID theme
/// name + the full theme list for the picker dropdown — same helper as
/// Dockspace's `page()`.
fn page(nav_active: &str, body: String) -> Html<String> {
    let schema = cybercore::schema::load();
    let active = schema.active.clone();
    let options: String = schema
        .theme_names()
        .map(|n| {
            let selected = if n == active { " selected" } else { "" };
            format!("<option value=\"{n}\"{selected}>{n}</option>")
        })
        .collect();
    Html(views::layout(&active, &options, nav_active, &body))
}

pub async fn get_index_page() -> Html<String> {
    let checked = hub::check_all().await;
    page("Hub", views::dashboard(&checked))
}

pub async fn partial_hub() -> Html<String> {
    let checked = hub::check_all().await;
    Html(views::hub_sections(&checked))
}

pub async fn scans_view() -> Html<String> {
    page("Scans", views::scans_page())
}

pub async fn reports_view() -> Html<String> {
    let mut entries = Vec::new();
    collect_md_files(std::path::Path::new("diagnostics"), "", &mut entries);
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    page("Reports", views::reports_page(&entries))
}

/// Several modules nest their report a level or two deep (`cpu/cpu.md`,
/// `motherboard/bios.md`, per-device/per-interface files, ...) — walk the
/// whole tree rather than assuming everything sits flat in `diagnostics/`.
/// `prefix` is the path built up so far, relative to `diagnostics/`.
fn collect_md_files(dir: &std::path::Path, prefix: &str, out: &mut Vec<(String, u64, String)>) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    for entry in rd.filter_map(|e| e.ok()) {
        let Ok(meta) = entry.metadata() else { continue };
        let name = entry.file_name().to_string_lossy().into_owned();
        let rel = if prefix.is_empty() {
            name.clone()
        } else {
            format!("{prefix}/{name}")
        };
        if meta.is_dir() {
            collect_md_files(&entry.path(), &rel, out);
        } else if name.ends_with(".md") {
            let modified = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| format!("{}s ago", now_secs().saturating_sub(d.as_secs())))
                .unwrap_or_else(|| "unknown".to_string());
            out.push((rel, meta.len(), modified));
        }
    }
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// `None` on any traversal attempt — every path segment must be a plain
/// name, no `..`, no empty segment from a stray `//`.
fn safe_diagnostics_path(rel: &str) -> Option<std::path::PathBuf> {
    if rel.split('/').any(|seg| seg == ".." || seg.is_empty()) {
        return None;
    }
    Some(std::path::Path::new("diagnostics").join(rel))
}

pub async fn report_view(AxPath(rel_path): AxPath<String>) -> Html<String> {
    // Several modules write into a sub-directory (cpu/cpu.md,
    // battery/battery.md, per-device/per-iface files, ...) — this route is
    // a wildcard (`/reports/*path`) so those work too.
    let Some(path) = safe_diagnostics_path(&rel_path) else {
        return page("Reports", "<p>invalid report path</p>".to_string());
    };
    let content = std::fs::read_to_string(&path).unwrap_or_else(|_| "not found".to_string());
    page("Reports", views::report_view(&rel_path, &content))
}

/// The same content as `report_view`, rendered without page chrome — for
/// the popup viewer's `fetch()` (see `openViewer()` in `views::layout`).
pub async fn report_fragment(AxPath(rel_path): AxPath<String>) -> Html<String> {
    let Some(path) = safe_diagnostics_path(&rel_path) else {
        return Html("<p>invalid report path</p>".to_string());
    };
    let content = std::fs::read_to_string(&path).unwrap_or_else(|_| "not found".to_string());
    Html(views::report_fragment(&content))
}

pub async fn get_cyberdeck_state(
    State(state): State<SharedCyberdeckState>,
) -> Json<crate::types::CyberdeckState> {
    let s = state.lock().await;
    Json(s.clone())
}

pub async fn post_cyberdeck_command(
    State(state): State<SharedCyberdeckState>,
    Json(cmd): Json<CyberdeckCommand>,
) -> Json<String> {
    dispatcher::execute_cyberdeck_command(cmd, &state).await;
    // Every dispatcher arm pushes its own result (a report path, or an
    // "Error: ..." line) onto execution_log before returning — surface that
    // instead of a canned string so the scans/archive UI can show what
    // actually happened.
    let s = state.lock().await;
    let msg = s
        .execution_log
        .last()
        .cloned()
        .unwrap_or_else(|| "Instruction pipeline advanced successfully.".to_string());
    Json(msg)
}

#[derive(Deserialize)]
pub struct DeckAction {
    pub action: String,
}

pub async fn post_cyberdeck_action(Json(payload): Json<DeckAction>) -> Json<String> {
    match payload.action.as_str() {
        "open" => {
            let _ = open::that("./diagnostics");
            Json("System: Opening output...".to_string())
        }
        "archive" => {
            let _ = std::process::Command::new("zip")
                .args(["-r", "output_archive.zip", "./diagnostics"])
                .output();
            Json("System: Archive created.".to_string())
        }
        _ => Json("System: Unknown command.".to_string()),
    }
}

/// Every file under `diagnostics/`, recursively, as paths relative to it
/// (`cpu/cpu.md`, `interfaces/wlp2s0.md`, ...) — a flat, non-recursive
/// listing used to hide everything written into a module's own
/// subdirectory, which is most of them.
fn collect_all_files(dir: &std::path::Path, prefix: &str, out: &mut Vec<String>) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    let mut entries: Vec<_> = rd.filter_map(|e| e.ok()).collect();
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        let Ok(meta) = entry.metadata() else { continue };
        let name = entry.file_name().to_string_lossy().into_owned();
        let rel = if prefix.is_empty() { name.clone() } else { format!("{prefix}/{name}") };
        if meta.is_dir() {
            collect_all_files(&entry.path(), &rel, out);
        } else {
            out.push(rel);
        }
    }
}

pub async fn list_diagnostics() -> Json<Vec<String>> {
    let mut paths = Vec::new();
    collect_all_files(std::path::Path::new("diagnostics"), "", &mut paths);
    Json(paths)
}

pub async fn tokens_css() -> impl IntoResponse {
    (
        [(
            axum::http::header::CONTENT_TYPE,
            cybercore::tokens::CSS_CONTENT_TYPE,
        )],
        cybercore::tokens::CSS,
    )
}
