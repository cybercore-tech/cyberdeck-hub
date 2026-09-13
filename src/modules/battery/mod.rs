//! # CYBERDECK: Battery & Power Diagnostics Module
//!
//! This module provides battery monitoring and reports.

//-NOTE: Battery & Power Diagnostics Module (/src/modules/battery/mod.rs)
//- Use these new "tags" for code blocks and notes.
//- Tag reference in build.rs
//- Files are saved to /snippets/{code, notes} in markdown (.md) format.
//-END

use std::fs::{self, File};
use std::io::Write; // Needed for the writeln! macro
use crate::types::CyberdeckState;
use crate::modules::utils::write_header; // Assuming your helper is here

/// Executes the battery diagnostic sweep.
pub async fn execute(_state: &CyberdeckState, params: &str) -> Result<String, String> {
    let dir = params;
    let bat_dir = format!("{}/battery", dir);

    fs::create_dir_all(&bat_dir).map_err(|e| e.to_string())?;
    let report_path = format!("{}/battery.md", bat_dir);

    // Create the file handle directly
    let mut file = File::create(&report_path).map_err(|e| e.to_string())?;

    // 1. Inject the Cyberdeck Header
    write_header(&mut file, "ACTIVE").map_err(|e| e.to_string())?;

    // 2. Write the Title
    writeln!(file, "# 🔋 CYBERDECK: SYSTEM POWER & BATTERY TELEMETRY\n").map_err(|e| e.to_string())?;

    let power_supply_root = "/sys/class/power_supply";

    if let Ok(entries) = fs::read_dir(power_supply_root) {
        let mut found_any = false;

        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().into_string().unwrap_or_default();

            if !path.is_dir() { continue; }
            found_any = true;

            writeln!(file, "## ⚡ Controller: `{}`", name).map_err(|e| e.to_string())?;

            let attributes = [
                ("Status", "status"),
                ("Capacity", "capacity"),
                ("Technology", "technology"),
                ("Model", "model_name"),
                ("Manufacturer", "manufacturer"),
                ("Voltage (µV)", "voltage_now"),
                ("Current Energy (µWh)", "energy_now"),
                ("Full Energy (µWh)", "energy_full"),
                ("Power Draw (µW)", "power_now"),
            ];

            for (label, filename) in attributes {
                let attr_path = path.join(filename);
                if let Ok(content) = fs::read_to_string(attr_path) {
                    writeln!(file, "- **{}:** {}", label, content.trim()).map_err(|e| e.to_string())?;
                }
            }
            writeln!(file, "").map_err(|e| e.to_string())?;
        }

        if !found_any {
            writeln!(file, "⚠️ No power controllers found in `/sys/class/power_supply/`.").map_err(|e| e.to_string())?;
        }
    } else {
        writeln!(file, "❌ Access Denied: Cannot read `/sys/class/power_supply/`.").map_err(|e| e.to_string())?;
    }

    Ok(report_path)
}
