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

use crate::types::CyberdeckState;

/// Executes the hardware intelligence diagnostic suite.
pub async fn execute(_state: &CyberdeckState, params: &str) -> Result<String, String> {
    let dir = params;
    let base_f = format!("{}/hardware.md", dir);
    let timestamp = crate::modules::utils::now_human();

    // Helper: Shell execution with result capture
    let run_cmd = |cmd: &str, args: &[&str]| -> String {
        Command::new(cmd)
        .args(args)
        .output()
        .map(|out| String::from_utf8_lossy(&out.stdout).to_string())
        .unwrap_or_else(|_| "Unavailable".to_string())
    };

    let mut report = format!("# Hardware Core Report\n\n_Generated {}_\n\n", timestamp);

    // 1. DMI / System Board — dmidecode needs root either way; when it's
    // not even installed (true on this box), the old code pushed a bare
    // "Unavailable" straight against the closing fence with no newline
    // between them, which never actually closed the block and swallowed
    // every heading after it as literal text. `code_block()` can't do
    // that regardless of what the command returns.
    report.push_str("## System Board & BIOS\n");
    let dmi = run_cmd("dmidecode", &["-t", "system,baseboard,bios"]);
    let dmi_text = if dmi.contains("Permission denied") {
        "Access denied (run as root for full info)".to_string()
    } else if dmi.trim().is_empty() || dmi == "Unavailable" {
        "dmidecode not installed on this system.".to_string()
    } else {
        dmi
    };
    report.push_str(&crate::modules::utils::code_block("text", &dmi_text));

    // 2. CPU Profile
    report.push_str("\n## Processor (CPU)\n");
    report.push_str(&crate::modules::utils::code_block("text", &run_cmd("lscpu", &[])));

    // 3. PCI/USB Bus Mapping
    report.push_str("\n## PCI & USB Peripherals\n\n### PCI\n");
    report.push_str(&crate::modules::utils::code_block("text", &run_cmd("lspci", &[])));
    report.push_str("\n### USB\n");
    report.push_str(&crate::modules::utils::code_block("text", &run_cmd("lsusb", &[])));

    // 4. Hardware Tree (lshw fallback)
    report.push_str("\n## Hardware Tree\n");
    let lshw = run_cmd("lshw", &["-short"]);
    let lshw_text = if lshw.trim().is_empty() {
        "lshw not installed or permission denied.".to_string()
    } else {
        lshw
    };
    report.push_str(&crate::modules::utils::code_block("text", &lshw_text));

    fs::write(&base_f, report).map_err(|e| e.to_string())?;
    Ok(base_f)
}
