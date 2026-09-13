//! # CYBERDECK Build File (build.rs)
use regex::Regex;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

struct SnippetRule {
    regex: Regex,
    target_folder: String,
}

// Helper: Converts strings to web-safe slugs
fn to_slug(name: &str) -> String {
    name.to_lowercase()
        .replace(|c: char| !c.is_alphanumeric() && c != '_', "_")
        .replace("__", "_")
}

// Helper: Formats "tech-heavy" filenames into "Readable Titles"
fn clean_title(name: &str) -> String {
    // 1. Remove leading numbers and underscores (e.g., "01_code" -> "code")
    let cleaned = name.trim_start_matches(|c: char| c.is_digit(10) || c == '_');

    // 2. Strip out the "tech" noise
    let cleaned = cleaned
        .replace("_src_modules_", " ")
        .replace("_mod_rs", "")
        .replace(".rs", "")
        .replace("_", " ");

    // 3. Title Case conversion
    cleaned
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<String>>()
        .join(" ")
}

fn create_rule(tag: &str, folder: &str) -> SnippetRule {
    let pattern = format!(r"(?s)//-{}:\s*(.*?)\r?\n(.*?)\s*//-END", tag);
    SnippetRule {
        regex: Regex::new(&pattern).unwrap(),
        target_folder: folder.to_string(),
    }
}

fn main() {
    let rules = vec![
        create_rule("CODE", "docs/src/snippets/code"),
        create_rule("NOTE", "docs/src/snippets/notes"),
        create_rule("TODO", "docs/src/tasks/todo"),
        create_rule("SEC", "docs/src/security"),
        create_rule("REF", "docs/src/references"),
        create_rule("CSS", "docs/src/code/css"),
        create_rule("SASS", "docs/src/code/sass"),
        create_rule("JS", "docs/src/code/js"),
        create_rule("RUST", "docs/src/code/rust"),
        create_rule("LUA", "docs/src/code/lua"),
        create_rule("JSON", "docs/src/code/json"),
        create_rule("UI", "docs/src/design/ui"),
        create_rule("STYLE", "docs/src/design/style"),
        create_rule("ROUTES", "docs/src/pipeline/routes"),
        create_rule("PORTS", "docs/src/pipeline/ports"),
        create_rule("NEW", "docs/src/features/new"),
        create_rule("ADD", "docs/src/features/add"),
        create_rule("REMOVE", "docs/src/features/removed"),
        create_rule("VEC", "docs/src/rules/vectors"),
        create_rule("MODS", "docs/src/system/mods"),
        create_rule("CRATES", "docs/src/crates"),
    ];

    // 1. CLEAN START
    for rule in &rules {
        if Path::new(&rule.target_folder).exists() {
            fs::remove_dir_all(&rule.target_folder).ok();
        }
        fs::create_dir_all(&rule.target_folder).expect("Failed to create folder");
    }

    // 2. EXTRACT
    for rule in rules {
        for entry in WalkDir::new("src").into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                let content = fs::read_to_string(path).unwrap_or_default();
                for cap in rule.regex.captures_iter(&content) {
                    let raw_name = cap[1].trim();
                    let snippet_content = cap[2].trim();
                    let safe_name = to_slug(raw_name);
                    let file_path = format!("{}/{}.md", rule.target_folder, safe_name);
                    let markdown = format!(
                        "## Snippet: {}\n\n```rust\n{}\n```",
                        raw_name, snippet_content
                    );
                    fs::write(&file_path, markdown).expect("Failed to write snippet");
                }
            }
        }
    }

    // 3. GENERATE SUMMARY
    generate_summary();

    println!("cargo:rerun-if-changed=src/");
}

fn generate_summary() {
    let summary_path = "docs/src/SUMMARY.md";
    let mut summary = String::from("# Summary\n\n- [Introduction](README.md)\n");
    let mut processed_files = HashSet::new();

    let categories = vec![
        ("snippets/code", "Code Snippets"),
        ("snippets/notes", "System Notes"),
        ("pipeline/ports", "Ports"),
        ("pipeline/routes", "Routes"),
        ("security", "Security"),
        ("tasks/todo", "Tasks & To-Do"),
        ("features", "Features"),
        ("features/add", "Add Features"),
        ("features/new", "New Features"),
        ("features/remove", "Remove Features"),
        ("references", "References"),
        ("rules/vector", "Vectors"),
        ("crates", "Crates"),
        ("system/mods", "Mods"),
        ("modules", "Modules"),
        ("api", "API"),
    ];

    for (dir, header) in categories {
        let dir_path = format!("docs/src/{}", dir);
        if !Path::new(&dir_path).exists() {
            continue;
        }

        summary.push_str(&format!("\n# {}\n", header));

        let mut files: Vec<PathBuf> = WalkDir::new(&dir_path)
            .max_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("md"))
            .map(|e| e.path().to_path_buf())
            .collect();

        files.sort_by_key(|p| p.file_name().unwrap().to_str().unwrap().to_lowercase());

        for file in files {
            let relative = file.strip_prefix("docs/src/").unwrap().to_str().unwrap();
            let stem = file.file_stem().unwrap().to_str().unwrap();
            let title = clean_title(stem); // <--- INTEGRATED HERE

            summary.push_str(&format!("- [{}]( {})\n", title, relative));
            processed_files.insert(file);
        }
    }

    // Orphaned files catch-all
    let mut orphans: Vec<PathBuf> = WalkDir::new("docs/src")
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("md"))
        .filter(|e| !processed_files.contains(e.path()))
        .filter(|e| {
            let name = e.file_name().to_str().unwrap();
            name != "SUMMARY.md" && name != "README.md"
        })
        .map(|e| e.path().to_path_buf())
        .collect();

    if !orphans.is_empty() {
        summary.push_str("\n# Miscellaneous\n");
        orphans.sort_by_key(|p| p.file_name().unwrap().to_str().unwrap().to_lowercase());
        for file in orphans {
            let relative = file.strip_prefix("docs/src/").unwrap().to_str().unwrap();
            let stem = file.file_stem().unwrap().to_str().unwrap();
            let title = clean_title(stem); // <--- INTEGRATED HERE

            summary.push_str(&format!("- [{}]( {})\n", title, relative));
        }
    }

    fs::write(summary_path, summary).expect("Failed to write SUMMARY.md");
}
