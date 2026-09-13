//! # CYBERDECK: Hardware Intelligence Module
//!
//! Provides a deep-dive inventory of the system's physical components.
//!
//! ## Implementation Notes
//! - **DMI/BIOS Audit**: Extracts motherboard, chassis, and BIOS details via `dmidecode`.
//! - **Bus Scanning**: Catalogs PCI/USB peripherals.
//! - **Graceful Degradation**: Handles missing diagnostic tools (lshw/dmidecode) by providing fallback info.

//-NOTE: Hardware Intelligence Module (/src/modules/hardware/mod.rs)
//- Use these new "tags" for code blocks and notes.
//- Tag reference in build.rs
//- Files are saved to /snippets/{code, notes} in markdown (.md) format.
//-END

use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::types::CyberdeckState;

/// Executes the hardware intelligence diagnostic suite.
pub async fn execute(_state: &CyberdeckState, params: &str) -> Result<String, String> {
    let dir = params;
    let base_f = format!("{}/hardware.md", dir);
    let timestamp = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map_err(|e| e.to_string())?
    .as_secs();

    // Helper: Shell execution with result capture
    let run_cmd = |cmd: &str, args: &[&str]| -> String {
        Command::new(cmd)
        .args(args)
        .output()
        .map(|out| String::from_utf8_lossy(&out.stdout).to_string())
        .unwrap_or_else(|_| "Unavailable".to_string())
    };

    let mut report = format!("<div style='background:#6a0dad;color:white;padding:6px;'>🖥️ CYBERDECK: HARDWARE CORE REPORT</div>\n\nTimestamp: {}\n\n", timestamp);

    // 1. DMI / System Board
    report.push_str("## 🏛️ System Board & BIOS\n```text\n");
    let dmi = run_cmd("dmidecode", &["-t", "system,baseboard,bios"]);
    report.push_str(if dmi.contains("Permission denied") { "Access denied (run as root for full info)\n" } else { &dmi });
    report.push_str("```\n");

    // 2. CPU Profile
    report.push_str("\n## 🧠 Processor (CPU)\n```text\n");
    report.push_str(&run_cmd("lscpu", &[]));
    report.push_str("```\n");

    // 3. PCI/USB Bus Mapping
    report.push_str("\n## 🧩 PCI & USB Peripherals\n");
    report.push_str("### PCI\n```text\n");
    report.push_str(&run_cmd("lspci", &[]));
    report.push_str("```\n### USB\n```text\n");
    report.push_str(&run_cmd("lsusb", &[]));
    report.push_str("```\n");

    // 4. Hardware Tree (lshw fallback)
    report.push_str("\n## ⚙️ Hardware Tree\n```text\n");
    let lshw = run_cmd("lshw", &["-short"]);
    report.push_str(if lshw.is_empty() || lshw.contains("not found") { "lshw not installed or permission denied" } else { &lshw });
    report.push_str("\n```\n");

    fs::write(&base_f, report).map_err(|e| e.to_string())?;
    Ok(base_f)
}
