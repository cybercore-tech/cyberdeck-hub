//! # CYBERDECK: Thermal Intelligence (AI Mode)
//!
//! Monitors system thermal zones and provides a fallback to lm-sensors
//! for detailed hardware temperature auditing.
//!
//! ## Implementation Notes
//! - **Direct Kernel Access**: Reads raw thermal data from `/sys/class/thermal`.
//! - **Predictive Analytics**: Tracks peak temperatures across all detected zones.
//! - **Fallback**: Automatically defaults to `sensors` CLI if sysfs nodes are unavailable.

//-NOTE: Thermal Intelligence (AI Mode) (/src/modules/thermal_ai/mod.rs)
//- Use these new "tags" for code blocks and notes.
//- Tag reference in build.rs
//- Files are saved to /snippets/{code, notes} in markdown (.md) format.
//-END

use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::types::CyberdeckState;

/// Executes the thermal diagnostic suite.
pub async fn execute(_state: &CyberdeckState, dir: &str) -> Result<String, String> {
    let base_f = format!("{}/thermal_ai.md", dir);
    let raw_f = format!("{}/raw/raw_temps.md", dir);
    let parsed_f = format!("{}/parsed/parsed_temps.md", dir);

    // Ensure directories exist
    fs::create_dir_all(format!("{}/raw", dir)).map_err(|e| e.to_string())?;
    fs::create_dir_all(format!("{}/parsed", dir)).map_err(|e| e.to_string())?;

    let timestamp = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map_err(|e| e.to_string())?
    .as_secs();

    let mut report = format!("# 🌡️ CYBERDECK: THERMAL INTELLIGENCE\n\nTimestamp: {}\n\n", timestamp);

    let mut max_temp: f64 = 0.0;
    let mut sensor_count = 0;
    let mut raw_data = String::new();

    // 1. Attempt Kernel Thermal Zone Discovery
    if let Ok(entries) = fs::read_dir("/sys/class/thermal") {
        let mut zones = Vec::new();
        for entry in entries.flatten() {
            let name = entry.file_name().into_string().unwrap_or_default();
            if name.starts_with("thermal_zone") {
                zones.push(entry.path().join("temp"));
            }
        }

        if !zones.is_empty() {
            report.push_str("## 📡 RAW SENSOR DATA\n");
            for (index, path) in zones.iter().enumerate() {
                if let Ok(raw_str) = fs::read_to_string(path) {
                    if let Ok(val) = raw_str.trim().parse::<f64>() {
                        let temp_c = val / 1000.0;
                        sensor_count += 1;
                        max_temp = max_temp.max(temp_c);

                        let line = format!("Zone {}: {:.2}°C\n", index + 1, temp_c);
                        report.push_str(&line);
                        raw_data.push_str(&format!("{:.2}\n", temp_c));
                    }
                }
            }
        }
    }

    // 2. Fallback to lm-sensors if sysfs is empty
    if sensor_count == 0 {
        if let Ok(out) = Command::new("sensors").output() {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            report.push_str("## 🧪 LM-SENSORS OUTPUT\n```text\n");
            report.push_str(&stdout);
            report.push_str("```\n");
            raw_data = stdout;
        } else {
            report.push_str("## ⚠️ Warning\nNo thermal sensors detected via sysfs or lm-sensors.\n");
        }
    }

    // 3. Generate Summaries
    report.push_str("\n## 🧠 THERMAL SUMMARY\n");
    report.push_str(&format!("- Peak temperature: {:.2}°C\n", max_temp));
    report.push_str(&format!("- Sensors detected: {}\n", sensor_count));

    // Write all artifacts
    fs::write(&base_f, report).map_err(|e| e.to_string())?;
    fs::write(&raw_f, raw_data).map_err(|e| e.to_string())?;
    fs::write(&parsed_f, format!("Peak: {:.2}°C\nSensors: {}\n", max_temp, sensor_count)).map_err(|e| e.to_string())?;

    Ok(base_f)
}
