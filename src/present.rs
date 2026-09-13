//! Small presentation-formatting helpers shared by the Reports page and
//! the Scans page's file viewer — human-readable sizes/times, "cooler"
//! titles pulled from a report's own first heading, and a stable
//! per-folder color for the folder chips.

/// "4577" -> "4.5 KB", "315" -> "315 B".
pub fn human_bytes(n: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = n as f64;
    let mut unit = 0;
    while size >= 1024.0 && unit < UNITS.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{n} {}", UNITS[0])
    } else {
        format!("{size:.1} {}", UNITS[unit])
    }
}

/// Seconds-ago -> "just now" / "5m ago" / "3h ago" / "2d ago".
pub fn human_ago(secs_ago: u64) -> String {
    if secs_ago < 60 {
        "just now".to_string()
    } else if secs_ago < 3600 {
        format!("{}m ago", secs_ago / 60)
    } else if secs_ago < 86400 {
        format!("{}h ago", secs_ago / 3600)
    } else {
        format!("{}d ago", secs_ago / 86400)
    }
}

/// "kernel_modules.md" -> "Kernel Modules", "bios-crossref.md" -> "Bios Crossref".
pub fn prettify_filename(name: &str) -> String {
    let stem = name.strip_suffix(".md").unwrap_or(name);
    stem.split(['_', '-'])
        .filter(|w| !w.is_empty())
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                Some(first) => first.to_uppercase().collect::<String>() + c.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Pulls the report's own `# Heading` as its display title — reads
/// nicely as "CPU" or "Motherboard & BIOS" instead of a raw filename.
/// Falls back to a prettified filename when there's no H1 (binary files,
/// truncated/failed reads).
pub fn extract_title(content: &str, filename: &str) -> String {
    content
        .lines()
        .find_map(|l| l.trim_start().strip_prefix("# ").map(str::trim))
        .filter(|t| !t.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| prettify_filename(filename))
}

/// A stable color role for a folder name, so the same folder always gets
/// the same accent — cycles the CYBERGRID palette by a simple string hash
/// rather than declaring an arbitrary folder->color map that would need
/// updating every time a module's output tree changes shape.
pub fn folder_color(folder: &str) -> &'static str {
    const COLORS: &[&str] = &["cyan", "purple", "pink", "orange", "acid", "red"];
    let hash: u32 = folder.bytes().fold(0u32, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u32));
    COLORS[(hash as usize) % COLORS.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bytes_scale_up_through_units() {
        assert_eq!(human_bytes(315), "315 B");
        assert_eq!(human_bytes(4577), "4.5 KB");
        assert_eq!(human_bytes(17369), "17.0 KB");
    }

    #[test]
    fn ago_buckets_by_magnitude() {
        assert_eq!(human_ago(30), "just now");
        assert_eq!(human_ago(120), "2m ago");
        assert_eq!(human_ago(7200), "2h ago");
        assert_eq!(human_ago(172800), "2d ago");
    }

    #[test]
    fn filenames_become_title_case_words() {
        assert_eq!(prettify_filename("kernel_modules.md"), "Kernel Modules");
        assert_eq!(prettify_filename("bios-crossref.md"), "Bios Crossref");
        assert_eq!(prettify_filename("cpu.md"), "Cpu");
    }

    #[test]
    fn title_prefers_the_reports_own_heading() {
        assert_eq!(extract_title("# CPU\n\nbody", "cpu.md"), "CPU");
        assert_eq!(extract_title("no heading here", "kernel_modules.md"), "Kernel Modules");
    }
}
