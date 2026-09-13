//! # CYBERDECK: Storage Intelligence (AI Mode)
//!
//! Deep-dive block device analysis, SMART telemetry, and filesystem allocation.
//!
//! ## Implementation Notes
//! - **Auto-Discovery**: Recursively maps block devices.
//! - **Predictive Analytics**: Hooks into `smartctl` for hardware health.
//! - **Layered Reporting**: Separates raw logs from human-readable intelligence.

//-NOTE: Storage Intelligence (AI Mode) (/src/modules/storage_ai/mod.rs)
//- Use these new "tags" for code blocks and notes.
//- Tag reference in build.rs
//- Files are saved to /snippets/{code, notes} in markdown (.md) format.
//-END

use crate::types::CyberdeckState;
use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

/// Executes the storage diagnostic logic.
pub async fn execute(_state: &CyberdeckState, dir: &str) -> Result<String, String> {
    // Note: We use 'dir' as the path, and 'state' is available for future
    // global config access if needed.

    let base_f = format!("{}/storage_ai.md", dir);
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs();

    // 1. Initialize Subdirectory Tree
    let subdirs = [
        "nvme",
        "ssd",
        "hdd",
        "us",
        ".as_bytes()raw",
        "filesystems",
        "smart",
    ];
    for sd in subdirs {
        fs::create_dir_all(format!("{}/{}", dir, sd)).map_err(|e| e.to_string())?;
    }

    let mut report = format!(
        "# 💾 CYBERDECK: STORAGE INTELLIGENCE (AI MODE)\n\nTimestamp: {}\n\n",
        timestamp
    );

    // Helpers
    let run_cmd = |cmd: &str, args: &[&str]| -> String {
        Command::new(cmd)
            .args(args)
            .output()
            .map(|out| String::from_utf8_lossy(&out.stdout).to_string())
            .unwrap_or_else(|_| "Command failed".to_string())
    };

    // 2. Block Device Mapping
    report.push_str("## 🧩 BLOCK DEVICE MAP\n```text\n");
    report.push_str(&run_cmd(
        "lsblk",
        &["-o", "NAME,SIZE,MODEL,TYPE,MOUNTPOINT,FSTYPE"],
    ));
    report.push_str("```\n");

    // 3. File System Usage
    report.push_str("\n## 📊 FILESYSTEM ALLOCATION\n```text\n");
    report.push_str(&run_cmd("df", &["-h"]));
    report.push_str("```\n");

    // 4. Device Classification & SMART Intelligence
    report.push_str("\n## 🧠 DEVICE CLASSIFICATION & HEALTH\n");
    if let Ok(entries) = fs::read_dir("/sys/block") {
        for entry in entries.flatten() {
            let dev = entry.file_name().into_string().unwrap_or_default();
            if dev.starts_with("loop") || dev.starts_with("ram") {
                continue;
            }

            let path = entry.path();
            let rota = fs::read_to_string(path.join("queue/rotational"))
                .unwrap_or_else(|_| "1".to_string());
            let dev_type = if rota.trim() == "0" {
                "SSD/NVMe"
            } else {
                "HDD"
            };

            report.push_str(&format!("- **{} ({})**\n", dev, dev_type));

            // SMART Check
            let smart_out = run_cmd("smartctl", &["-H", &format!("/dev/{}", dev)]);
            let status = if smart_out.contains("PASSED") {
                "✅ HEALTHY"
            } else {
                "⚠️ CHECK REQUIRED"
            };
            report.push_str(&format!("  - Status: {}\n", status));
        }
    }

    // 5. Encryption Layer Audit
    report.push_str("\n## 🔐 ENCRYPTION LAYER (LUKS)\n");
    report.push_str(&run_cmd("lsblk", &["-f"]));

    fs::write(&base_f, report).map_err(|e| e.to_string())?;
    Ok(base_f)
}
