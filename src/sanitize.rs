//! Auto-repairs garbled markdown before it's ever rendered — a safety net
//! behind `modules::utils::code_block()` for reports that already exist on
//! disk (written before that fix) or from a module nobody's touched yet.
//!
//! The one real failure mode seen in practice: a fallback string with no
//! trailing newline (e.g. `"Unavailable"` when a tool like `dmidecode`
//! isn't installed) landing directly against a closing ` ``` `, so the
//! fence marker isn't on its own line and never actually closes. Every
//! heading/fence after it then gets swallowed as literal text inside the
//! still-open code block — one missing newline breaks the rest of the
//! document. Runs at render time only; never rewrites the file on disk,
//! so a scan's real generated output is untouched either way.

/// A line that (once trimmed) is nothing but a fence marker: ` ``` ` or
/// ` ```lang `. Anything else containing "```" has real content glued
/// against it and needs splitting.
fn is_fence_marker(trimmed: &str) -> bool {
    trimmed.starts_with("```") && trimmed[3..].chars().all(|c| c.is_ascii_alphanumeric())
}

/// Returns the repaired markdown, and whether a repair actually happened
/// (surfaced as an "auto-fixed" badge in the Reports list).
pub fn sanitize(content: &str) -> (String, bool) {
    let mut changed = false;
    let mut lines: Vec<String> = Vec::new();

    for line in content.lines() {
        if !is_fence_marker(line.trim()) {
            if let Some(idx) = line.find("```") {
                // A glued line — content before the fence marker (the
                // "Unavailable```" case), the fence marker with trailing
                // junk after it, or both. Split so the fence lands alone.
                let (prefix, fence_and_rest) = line.split_at(idx);
                if !prefix.trim().is_empty() {
                    lines.push(prefix.trim_end().to_string());
                    lines.push(fence_and_rest.to_string());
                    changed = true;
                    continue;
                }
            }
        }
        lines.push(line.to_string());
    }

    // Second pass: a fence line is any line whose trimmed start begins
    // with ``` — track open/close state; if still open at EOF, close it
    // rather than let every report after this one in a directory listing
    // stay invisible to normal markdown parsing.
    let mut in_fence = false;
    for line in &lines {
        if line.trim_start().starts_with("```") {
            in_fence = !in_fence;
        }
    }
    if in_fence {
        lines.push("```".to_string());
        changed = true;
    }

    (lines.join("\n"), changed)
}

/// Cheap check for the Reports listing — did this file's raw content need
/// a repair? Used to show an "auto-fixed" badge without needing the
/// caller to separately track `sanitize()`'s changed flag.
pub fn needs_repair(content: &str) -> bool {
    sanitize(content).1
}

#[cfg(test)]
mod tests {
    use super::sanitize;

    #[test]
    fn leaves_well_formed_markdown_untouched() {
        let src = "# Title\n\n```text\nfine\n```\n\nmore text\n";
        let (out, changed) = sanitize(src);
        assert!(!changed);
        assert_eq!(out, src.trim_end_matches('\n'));
    }

    #[test]
    fn splits_a_glued_closing_fence_and_reports_a_change() {
        let src = "## Section\n```text\nUnavailable```\n\n## Next\n```text\nreal output\n```\n";
        let (out, changed) = sanitize(src);
        assert!(changed);
        // The next heading must survive as a real heading, not get eaten
        // by an unclosed code block.
        assert!(out.contains("## Next"));
        assert!(!out.contains("Unavailable```"));
    }

    #[test]
    fn closes_a_fence_left_open_at_eof() {
        let src = "```text\nsome output with no closing fence\n";
        let (out, changed) = sanitize(src);
        assert!(changed);
        assert_eq!(out.matches("```").count(), 2);
    }
}
