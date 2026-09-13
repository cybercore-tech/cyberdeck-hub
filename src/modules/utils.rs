//! # CYBERDECK: Utils
//! System utilites module.

//-NOTE: Utils (/src/modules/utils.rs)
//- Keep all notes current for utils development purposes.
//- Tag reference in build.rs
//- New "tags" can be added at any time to [build.rs]. Document all new tags.
//-END

use std::fs::{self, File};
use std::io::Write;

/// Local wall-clock time, e.g. `2026-09-12 20:29 PDT` — every module's
/// report header used to print a raw `SystemTime::now().as_secs()` unix
/// timestamp; this is the human-readable replacement.
pub fn now_human() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M %Z").to_string()
}

/// Read one `/sys/class/dmi/id/<field>` value — vendor/model/BIOS info
/// without needing root. A few fields (serials, `product_uuid`) really are
/// root-only; this reports that honestly instead of coming back blank the
/// way a failed unattended `sudo dmidecode` silently did before.
pub fn dmi_field(field: &str) -> String {
    fs::read_to_string(format!("/sys/class/dmi/id/{field}"))
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "restricted (root only) or unavailable".to_string())
}

// A single clean generated-at line — replaces the old ASCII-art
// "CYBERDECK INTERNAL DIAGNOSTIC SYSTEM / SYS_ID: <hex unix time>" block,
// which read like internal jargon with no real information in it.
pub fn write_header(file: &mut File, status: &str) -> std::io::Result<()> {
    writeln!(file, "_Generated {} — status: {}_\n", now_human(), status)
}

// THE NEW HELPER: Standardizes initialization
#[allow(dead_code)]
pub fn init_diagnostic_file(dir: &str, filename: &str) -> Result<File, String> {
    fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let path = format!("{}/{}", dir, filename);
    let mut file = File::create(&path).map_err(|e| e.to_string())?;

    // Auto-inject header
    write_header(&mut file, "ACTIVE").map_err(|e| e.to_string())?;

    Ok(file)
}
