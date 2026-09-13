//! # CYBERDECK: Motherboard Intelligence
//!
//! Handles DMI/SMBIOS extraction for BIOS and baseboard identity.
//!
//! ## Implementation Notes
//! - **DMI Extraction**: Requires root (sudo) for full hardware data.
//! - **Graceful Failure**: If run without sudo, it captures what it can and flags restricted access.

//-NOTE: Motherboard Intelligence (/src/modules/motherboard/mod.rs)
//- Use these new "tags" for code blocks and notes.
//- Tag reference in build.rs
//- Files are saved to /snippets/{code, notes} in markdown (.md) format.
//-END

use std::fs;
use std::process::Command;
use crate::types::CyberdeckState;

/// Executes motherboard and BIOS diagnostic suite.
pub async fn execute(_state: &CyberdeckState, params: &str) -> Result<String, String> {
    let dir = params;
    let board_dir = format!("{}/motherboard", dir);

    // Ensure directory exists, converting IO errors to String for Result
    fs::create_dir_all(&board_dir).map_err(|e| e.to_string())?;

    let base_f = format!("{}/motherboard.md", board_dir);
    let mut report = String::from("# 🧩 CYBERDECK: MOTHERBOARD & BIOS\n\n");

    // Helper: Run commands and capture output
    let run_cmd = |cmd: &str, args: &[&str]| -> String {
        let out = Command::new(cmd).args(args).output();
        match out {
            Ok(o) => String::from_utf8_lossy(&o.stdout).to_string(),
            Err(_) => "Access denied or command unavailable".to_string(),
        }
    };

    // 1. Fetch DMI Data
    let dmi_bios = run_cmd("sudo", &["dmidecode", "-t", "bios"]);
    let dmi_board = run_cmd("sudo", &["dmidecode", "-t", "baseboard"]);

    // 2. Summary Section
    report.push_str("## 📋 Quick Identity\n");
    for line in dmi_board.lines() {
        if line.contains("Manufacturer:") || line.contains("Product Name:") || line.contains("Serial Number:") {
            report.push_str(&format!("- **{}**\n", line.trim()));
        }
    }

    // 3. Detailed Data Section
    report.push_str("\n---\n\n## ⚙️ BIOS Details\n```text\n");
    report.push_str(dmi_bios.trim());
    report.push_str("\n```\n\n## 🧩 Baseboard Details\n```text\n");
    report.push_str(dmi_board.trim());
    report.push_str("\n```\n");

    // Final file write, error mapping to String
    fs::write(&base_f, report).map_err(|e| e.to_string())?;

    Ok(base_f)
}
