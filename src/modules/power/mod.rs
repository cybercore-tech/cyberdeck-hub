//! # CYBERDECK: Power & Performance Intelligence
//!
//! Monitors CPU scaling governors, P-State drivers, and system power states.
//!
//! ## Implementation Notes
//! - **CPU Mapping**: Single-pass traversal of `/sys/devices/system/cpu` for efficiency.
//! - **Driver Detection**: Automatically differentiates between P-State and ACPI scaling drivers.
//! - **Energy Aware**: Integrates with `upower` for battery and power source state.

//-NOTE: Power & Performance Intelligence (/src/modules/power/mod.rs)
//- Use these new "tags" for code blocks and notes.
//- Tag reference in build.rs
//- Files are saved to /snippets/{code, notes} in markdown (.md) format.
//-END

use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::types::CyberdeckState;

/// Executes the power/performance diagnostic suite.
pub async fn execute(_state: &CyberdeckState, params: &str) -> Result<String, String> {
    let dir = params;
    let base_f = format!("{}/power.md", dir);

    let timestamp = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map_err(|e| e.to_string())?
    .as_secs();

    let mut report = format!("# 🔋 POWER & PERFORMANCE\n\nTimestamp: {}\n\n", timestamp);

    // 1. CPU Scaling & Governors (Single-pass collection)
    report.push_str("## ⚡ CPU Scaling State\n");
    if let Ok(entries) = fs::read_dir("/sys/devices/system/cpu") {
        let mut core_data = Vec::new();
        for entry in entries.flatten() {
            let name = entry.file_name().into_string().unwrap_or_default();
            if name.starts_with("cpu") && name.chars().nth(3).map_or(false, |c| c.is_ascii_digit()) {
                let id = &name[3..];
                // Read Governor
                let gov = fs::read_to_string(entry.path().join("cpufreq/scaling_governor"))
                .unwrap_or_else(|_| "N/A".to_string());
                // Read Frequency
                let freq = fs::read_to_string(entry.path().join("cpufreq/scaling_cur_freq"))
                .unwrap_or_else(|_| "0".to_string());
                let mhz = freq.trim().parse::<f64>().unwrap_or(0.0) / 1000.0;

                core_data.push(format!("- **Core {}**: Gov: `{}` | Freq: `{} MHz`", id, gov.trim(), mhz));
            }
        }
        report.push_str(&core_data.join("\n"));
        report.push_str("\n");
    }

    // 2. Driver & P-State Audit
    report.push_str("\n## ⚙️ Driver Intelligence\n");
    let driver = fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/scaling_driver")
    .unwrap_or_else(|_| "Unknown".to_string());
    report.push_str(&format!("- **Scaling Driver**: `{}`\n", driver.trim()));

    if fs::metadata("/sys/devices/system/cpu/amd_pstate/status").is_ok() {
        let pstate = fs::read_to_string("/sys/devices/system/cpu/amd_pstate/status")
        .unwrap_or_default();
        report.push_str(&format!("- **AMD P-State**: `{}`\n", pstate.trim()));
    }

    // 3. Energy Status
    report.push_str("\n## 🔋 Energy Source\n");
    let upower = Command::new("upower").args(["-e"]).output().ok();
    if let Some(out) = upower {
        let dev_list = String::from_utf8_lossy(&out.stdout);
        if let Some(first_dev) = dev_list.lines().next() {
            let info = Command::new("upower").args(["-i", first_dev.trim()]).output().ok();
            if let Some(info_out) = info {
                report.push_str("```text\n");
                report.push_str(&String::from_utf8_lossy(&info_out.stdout));
                report.push_str("```\n");
            }
        }
    } else {
        report.push_str("- `upower` not installed.\n");
    }

    fs::write(&base_f, report).map_err(|e| e.to_string())?;
    Ok(base_f)
}
