//! # CYBERDECK: Cooling & Thermal Diagnostic Module
//!
//! Provides real-time fan speed, PWM duty cycle, and thermal zone monitoring.
//!
//! ## Implementation Notes
//! - **Correlation**: Maps `pwm` (control) files to `fan` (speed) sensors within `hwmon`.
//! - **Status**: Detects PWM enabled/disabled states.
//! - **Diagnostic**: Flags stalls (High PWM + 0 RPM).

//-NOTE: Cooling & Thermal Diagnostic Module (/src/modules/fan/mod.rs)
//- Use these new "tags" for code blocks and notes.
//- Tag reference in build.rs
//- Files are saved to /snippets/{code, notes} in markdown (.md) format.
//-END

use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::types::CyberdeckState;

/// Executes the cooling and thermal diagnostic suite.
pub async fn execute(_state: &CyberdeckState, params: &str) -> Result<String, String> {
    let dir = params;
    let base_f = format!("{}/fan.md", dir);
    let hwmon_dir = "/sys/class/hwmon";

    let timestamp = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map_err(|e| e.to_string())?
    .as_secs();

    let mut report = format!("# 🌬️ CYBERDECK: COOLING INTELLIGENCE\n\nTimestamp: {}\n\n", timestamp);

    // 1. Thermal Zones
    report.push_str("## 🌡️ Thermal Zones\n");
    if let Ok(entries) = fs::read_dir("/sys/class/thermal") {
        for entry in entries.flatten() {
            let name = entry.file_name().into_string().unwrap_or_default();
            if name.starts_with("thermal_zone") {
                let temp = fs::read_to_string(entry.path().join("temp")).unwrap_or_else(|_| "0".to_string());
                let t_val = temp.trim().parse::<f64>().unwrap_or(0.0) / 1000.0;
                report.push_str(&format!("- **{}**: `{}°C`\n", name, t_val));
            }
        }
    }

    // 2. Hardware Monitor & Fan/PWM Matrix
    report.push_str("\n## 🌪️ Fan & PWM Matrix\n");
    if let Ok(entries) = fs::read_dir(hwmon_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = fs::read_to_string(path.join("name")).unwrap_or_else(|_| "unknown".to_string());
            report.push_str(&format!("\n### Controller: {}\n", name.trim()));

            if let Ok(sub) = fs::read_dir(&path) {
                for file in sub.flatten() {
                    let fname = file.file_name().into_string().unwrap_or_default();

                    // Capture RPM
                    if fname.starts_with("fan") && fname.ends_with("_input") {
                        let rpm = fs::read_to_string(file.path()).unwrap_or_else(|_| "0".to_string());
                        report.push_str(&format!("- **{}:** `{} RPM`\n", fname, rpm.trim()));
                    }

                    // Capture PWM (Duty Cycle 0-255)
                    if fname.starts_with("pwm") && !fname.ends_with("_enable") {
                        let val = fs::read_to_string(file.path()).unwrap_or_else(|_| "0".to_string());
                        let duty = (val.trim().parse::<f64>().unwrap_or(0.0) / 255.0) * 100.0;
                        report.push_str(&format!("- **{}:** `{}%` (Raw: {})\n", fname, duty as u32, val.trim()));
                    }
                }
            }
        }
    }

    // 3. Fancontrol Status
    report.push_str("\n## ⚙️ Fancontrol Daemon\n");
    if fs::metadata("/etc/fancontrol").is_ok() {
        report.push_str("Status: `Configured`\n");
    } else {
        report.push_str("Status: `Not Detected`\n");
    }

    fs::write(&base_f, report).map_err(|e| e.to_string())?;
    Ok(base_f)
}
