//! # CYBERDECK: CPU Diagnostic Module
//!
//! Provides granular telemetry for processor architecture, topology, thermal
//! states, and power consumption (RAPL).
//!
//! ## Implementation Notes
//! - **Topology**: Probes `/sys/devices/system/cpu/` to map logical cores.
//! - **Thermal**: Scans `/sys/class/hwmon/` for package/core temperatures.
//! - **RAPL (Power)**: Attempts to read energy counters from `/sys/class/powercap/`.
//! - **Scaling**: Monitors frequency governors and scaling limits.

//-NOTE: CPU Diagnostic Module (/src/modules/cpu/mod.rs)
//- Use these new "tags" for code blocks and notes.
//- Tag reference in build.rs
//- Files are saved to /snippets/{code, notes} in markdown (.md) format.
//-END

use std::fs;
use std::process::Command;
use crate::types::CyberdeckState;

/// Executes the CPU diagnostic sweep.
///
/// Generates a structured multi-file report covering architecture,
/// thermal performance, and power state.
pub async fn execute(_state: &CyberdeckState, params: &str) -> Result<String, String> {
    let dir = params;
    let cpu_dir = format!("{}/cpu", dir);
    fs::create_dir_all(&cpu_dir).map_err(|e| e.to_string())?;

    let report_path = format!("{}/cpu.md", cpu_dir);
    let mut report = String::from("# 🧠 CYBERDECK: CPU TELEMETRY\n\n");

    // Helper: Shell execution
    let run_cmd = |cmd: &str, args: &[&str]| -> String {
        Command::new(cmd)
        .args(args)
        .output()
        .map(|out| String::from_utf8_lossy(&out.stdout).to_string())
        .unwrap_or_else(|_| format!("{} not found", cmd))
    };

    // 1. Topology & Architecture
    report.push_str("## 🧩 Topology & Architecture\n```text\n");
    report.push_str(&run_cmd("lscpu", &[]));
    report.push_str("\n```\n");

    // 2. Frequency & Governor Status
    report.push_str("## ⚡ Frequency Scaling & Governors\n");
    if let Ok(cpu0) = fs::read_dir("/sys/devices/system/cpu/cpu0/cpufreq") {
        for entry in cpu0.flatten() {
            let name = entry.file_name().into_string().unwrap_or_default();
            if let Ok(val) = fs::read_to_string(entry.path()) {
                report.push_str(&format!("- **{}:** `{}`\n", name, val.trim()));
            }
        }
    }

    // 3. Thermal Monitoring (Hardware Monitors)
    report.push_str("\n## 🌡️ Thermal Metrics\n");
    if let Ok(hwmon) = fs::read_dir("/sys/class/hwmon") {
        for entry in hwmon.flatten() {
            let path = entry.path();
            let name = fs::read_to_string(path.join("name")).unwrap_or_else(|_| "unknown".to_string());
            report.push_str(&format!("### Device: {}\n", name.trim()));

            // Look for temp inputs
            for i in 0..5 {
                let temp_path = path.join(format!("temp{}_input", i));
                if let Ok(temp) = fs::read_to_string(temp_path) {
                    let temp_c = temp.trim().parse::<f64>().unwrap_or(0.0) / 1000.0;
                    report.push_str(&format!("- Temp {}: {:.2}°C\n", i, temp_c));
                }
            }
        }
    }

    // 4. Power (RAPL - Running Average Power Limit)
    report.push_str("\n## 🔋 Power & Energy (RAPL)\n");
    let rapl_paths = ["/sys/class/powercap/intel-rapl", "/sys/class/powercap/amd_rapl"];
    for rapl_root in rapl_paths {
        if let Ok(entries) = fs::read_dir(rapl_root) {
            for entry in entries.flatten() {
                let name = entry.file_name().into_string().unwrap_or_default();
                if let Ok(energy) = fs::read_to_string(entry.path().join("energy_uj")) {
                    report.push_str(&format!("- **{}:** {} µJ\n", name, energy.trim()));
                }
            }
        }
    }

    fs::write(&report_path, report).map_err(|e| e.to_string())?;
    Ok(report_path)
}
