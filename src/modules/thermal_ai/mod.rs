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
use crate::types::CyberdeckState;

/// Executes the thermal diagnostic suite.
pub async fn execute(_state: &CyberdeckState, dir: &str) -> Result<String, String> {
    let base_f = format!("{}/thermal_ai.md", dir);
    let raw_f = format!("{}/raw/raw_temps.md", dir);
    let parsed_f = format!("{}/parsed/parsed_temps.md", dir);

    // Ensure directories exist
    fs::create_dir_all(format!("{}/raw", dir)).map_err(|e| e.to_string())?;
    fs::create_dir_all(format!("{}/parsed", dir)).map_err(|e| e.to_string())?;

    let timestamp = crate::modules::utils::now_human();

    let mut report = format!("# 🌡️ CYBERDECK: THERMAL INTELLIGENCE\n\nTimestamp: {}\n\n", timestamp);

    let mut max_temp: f64 = 0.0;
    let mut max_zone = String::new();
    let mut sensor_count = 0;
    let mut raw_data = String::new();
    let mut zone_readings: Vec<(String, f64)> = Vec::new();

    // 1. Attempt Kernel Thermal Zone Discovery — reads each zone's own
    // `type` file for a real sensor name (x86_pkg_temp, acpitz,
    // iwlwifi_1, ...) instead of a meaningless "Zone 1"/"Zone 2" index
    // that told you nothing about which physical sensor it actually was.
    if let Ok(entries) = fs::read_dir("/sys/class/thermal") {
        let mut zones = Vec::new();
        for entry in entries.flatten() {
            let name = entry.file_name().into_string().unwrap_or_default();
            if name.starts_with("thermal_zone") {
                zones.push(entry.path());
            }
        }
        zones.sort();

        if !zones.is_empty() {
            report.push_str("## 📡 RAW SENSOR DATA\n\n| Sensor | Temp |\n|---|---|\n");
            for path in &zones {
                let zone_type = fs::read_to_string(path.join("type"))
                    .map(|s| s.trim().to_string())
                    .unwrap_or_else(|_| "unknown".to_string());
                if let Ok(raw_str) = fs::read_to_string(path.join("temp")) {
                    if let Ok(val) = raw_str.trim().parse::<f64>() {
                        let temp_c = val / 1000.0;
                        sensor_count += 1;
                        if temp_c > max_temp {
                            max_temp = temp_c;
                            max_zone = zone_type.clone();
                        }
                        report.push_str(&format!("| {} | {:.2}°C |\n", zone_type, temp_c));
                        raw_data.push_str(&format!("{}: {:.2}\n", zone_type, temp_c));
                        zone_readings.push((zone_type, temp_c));
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
    report.push_str(&format!("- Peak temperature: {:.2}°C ({})\n", max_temp, if max_zone.is_empty() { "n/a" } else { &max_zone }));
    report.push_str(&format!("- Sensors detected: {}\n", sensor_count));

    // parsed_temps.md used to be just "Peak: X / Sensors: N" — the same
    // per-zone breakdown the main report has, condensed for a quick read.
    let mut parsed = format!(
        "# Parsed Temperatures\n\nPeak: {:.2}°C ({})\nSensors: {}\n\n",
        max_temp, if max_zone.is_empty() { "n/a" } else { &max_zone }, sensor_count
    );
    for (name, temp) in &zone_readings {
        parsed.push_str(&format!("- {}: {:.2}°C\n", name, temp));
    }

    // Write all artifacts
    fs::write(&base_f, report).map_err(|e| e.to_string())?;
    fs::write(&raw_f, raw_data).map_err(|e| e.to_string())?;
    fs::write(&parsed_f, parsed).map_err(|e| e.to_string())?;

    Ok(base_f)
}
