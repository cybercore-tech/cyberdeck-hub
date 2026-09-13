//! CYBERGRID palette bridge — same pattern as dockspace's `src/cybergrid.rs`.
//! Exposes the shared `cybercore` crate's named colour themes over HTTP so
//! the hub can list and apply them. Replaces the old bespoke
//! `static/themes/sub-cyber/*.json` picker entirely.
//!
//! - `GET /api/cybergrid/themes`      — every theme + its 11 colour roles
//! - `GET /api/cybergrid/css/:name`   — a ready `:root{ --bg:#… }` block

use axum::{extract::Path, http::header, response::IntoResponse, Json};
use serde_json::{json, Value};
use std::collections::BTreeMap;

/// `family name -> slugs`, read straight off cybercore's real
/// `schema/themes/<family>/<slug>.json` folder layout (path dep, so this
/// is a stable relative path at compile time via `CARGO_MANIFEST_DIR`).
/// Not derivable from the slug alone: most families share a `<family>-
/// <word>` naming convention, but the `default` family is a grab-bag of
/// well-known theme names (`dracula`, `nord`, ...) plus a curated
/// re-export of a few themes that live under other families too — a
/// slug-prefix heuristic would misfile or duplicate those.
pub fn theme_families() -> BTreeMap<String, Vec<String>> {
    let themes_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../cybercore/schema/themes");
    let mut families = BTreeMap::new();
    let Ok(rd) = std::fs::read_dir(themes_dir) else { return families };
    for family_entry in rd.filter_map(|e| e.ok()) {
        if !family_entry.path().is_dir() {
            continue;
        }
        let family = family_entry.file_name().to_string_lossy().into_owned();
        let mut slugs: Vec<String> = std::fs::read_dir(family_entry.path())
            .map(|rd| {
                rd.filter_map(|e| e.ok())
                    .filter_map(|e| {
                        let name = e.file_name().to_string_lossy().into_owned();
                        name.strip_suffix(".json").map(str::to_string)
                    })
                    .collect()
            })
            .unwrap_or_default();
        slugs.sort();
        families.insert(family, slugs);
    }
    families
}

fn role_map(name: &str, p: &cybercore::schema::Palette) -> Value {
    json!({
        "name": name,
        "bg": p.bg,
        "white": p.white,
        "acid_green": p.acid_green,
        "hot_pink": p.hot_pink,
        "purple": p.purple,
        "cyan": p.cyan,
        "orange": p.orange,
        "red": p.red,
        "panel": p.panel,
        "line": p.line,
        "muted": p.muted,
    })
}

pub async fn list_themes() -> impl IntoResponse {
    let s = cybercore::schema::load();
    let themes: Vec<Value> = s
        .theme_names()
        .filter_map(|n| s.theme(n).map(|p| role_map(n, p)))
        .collect();
    Json(json!({ "active": s.active, "themes": themes }))
}

pub async fn theme_css(Path(name): Path<String>) -> impl IntoResponse {
    let s = cybercore::schema::load();
    let p = s.theme(&name).unwrap_or_else(|| s.active_theme());
    let css = format!(
        ":root{{\
--bg:#{bg};--fg:#{white};\
--acid:#{acid};--pink:#{pink};--purple:#{purple};--cyan:#{cyan};\
--orange:#{orange};--red:#{red};\
--panel:#{panel};--line:#{line};--muted:#{muted};\
}}\n",
        bg = p.bg,
        white = p.white,
        acid = p.acid_green,
        pink = p.hot_pink,
        purple = p.purple,
        cyan = p.cyan,
        orange = p.orange,
        red = p.red,
        panel = p.panel,
        line = p.line,
        muted = p.muted,
    );
    ([(header::CONTENT_TYPE, "text/css; charset=utf-8")], css)
}
