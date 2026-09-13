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
use crate::present;
use crate::types::{CyberdeckCommand, SharedCyberdeckState};
use crate::views;

use serde::{Deserialize, Serialize};

/// Wraps a body in the full page shell, pulling the active CYBERGRID theme
/// name + the full theme list for the picker dropdown — same helper as
/// Dockspace's `page()`.
fn page(nav_active: &str, body: String) -> Html<String> {
    let schema = cybercore::schema::load();
    let active = schema.active.clone();
    let families = crate::cybergrid::theme_families();

    let options: String = families
        .iter()
        .map(|(family, slugs)| {
            let items: String = slugs
                .iter()
                // Only list slugs the compiled schema actually knows about
                // — the on-disk folder and the binary's embedded merge can
                // drift (themes are merged at *compile time*; a new file
                // needs a rebuild before it's real).
                .filter(|slug| schema.theme(slug).is_some())
                .map(|slug| {
                    let selected = if **slug == active { " selected" } else { "" };
                    format!("<option value=\"{slug}\"{selected}>{slug}</option>")
                })
                .collect();
            if items.is_empty() {
                String::new()
            } else {
                format!(r#"<optgroup label="{family}">{items}</optgroup>"#)
            }
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
    entries.sort_by(|a, b| a.path.cmp(&b.path));
    page("Reports", views::reports_page(&entries))
}

/// One row in the Reports table — resolved once here so `views::
/// reports_page` is pure rendering, no filesystem/formatting logic of
/// its own.
pub struct ReportEntry {
    pub path: String,
    pub folder: Option<String>,
    pub title: String,
    pub size_human: String,
    pub modified_human: String,
    pub garbled: bool,
}

/// Several modules nest their report a level or two deep (`cpu/cpu.md`,
/// `motherboard/bios.md`, per-device/per-interface files, ...) — walk the
/// whole tree rather than assuming everything sits flat in `diagnostics/`.
/// `prefix` is the path built up so far, relative to `diagnostics/`.
fn collect_md_files(dir: &std::path::Path, prefix: &str, out: &mut Vec<ReportEntry>) {
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
            let modified_human = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| present::human_ago(now_secs().saturating_sub(d.as_secs())))
                .unwrap_or_else(|| "unknown".to_string());
            let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
            out.push(ReportEntry {
                folder: (!prefix.is_empty()).then(|| prefix.to_string()),
                title: present::extract_title(&content, &name),
                size_human: present::human_bytes(meta.len()),
                modified_human,
                garbled: crate::sanitize::needs_repair(&content),
                path: rel,
            });
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
    let title = present::extract_title(&content, &rel_path);
    page("Reports", views::report_view(&title, &rel_path, &content))
}

/// The same content as `report_view`, rendered without page chrome — for
/// the popup viewer's `fetch()` (see `openViewer()` in `views::layout`).
/// The title rides along as a response header (`X-Report-Title`) rather
/// than wrapping the body in JSON, so the fragment stays plain HTML the
/// viewer can drop straight into the DOM.
pub async fn report_fragment(AxPath(rel_path): AxPath<String>) -> impl IntoResponse {
    let Some(path) = safe_diagnostics_path(&rel_path) else {
        return (
            [("X-Report-Title", "Invalid path".to_string())],
            Html("<p>invalid report path</p>".to_string()),
        );
    };
    let content = std::fs::read_to_string(&path).unwrap_or_else(|_| "not found".to_string());
    let title = present::extract_title(&content, &rel_path);
    (
        [("X-Report-Title", title)],
        Html(views::report_fragment(&content)),
    )
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

#[derive(Serialize)]
pub struct FileEntry {
    pub path: String,
    pub folder: Option<String>,
    pub title: String,
    pub is_md: bool,
}

/// Every file under `diagnostics/`, recursively — a flat, non-recursive
/// listing used to hide everything written into a module's own
/// subdirectory, which is most of them.
fn collect_all_files(dir: &std::path::Path, prefix: &str, out: &mut Vec<FileEntry>) {
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
            let is_md = name.ends_with(".md");
            let title = if is_md {
                let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
                present::extract_title(&content, &name)
            } else {
                present::prettify_filename(&name)
            };
            out.push(FileEntry {
                folder: (!prefix.is_empty()).then(|| prefix.to_string()),
                title,
                is_md,
                path: rel,
            });
        }
    }
}

pub async fn list_diagnostics() -> Json<Vec<FileEntry>> {
    let mut files = Vec::new();
    collect_all_files(std::path::Path::new("diagnostics"), "", &mut files);
    Json(files)
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
