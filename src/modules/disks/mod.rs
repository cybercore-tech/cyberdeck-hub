//! # CYBERDECK: Disk Diagnostics Module
//!
//! Provides granular insight into storage media, mountpoints, and filesystem health.
//!
//! ## Implementation Notes
//! - **Categorization**: Dynamically sorts drives into `nvme/`, `ssd/`, `hdd/`, and `usb/`.
//! - **Btrfs Support**: Detects Btrfs partitions to provide volume usage and snapshot diagnostics.
//! - **S.M.A.R.T.**: Probes hardware health via `smartctl`. (Requires root/sudo privileges).
//! - **Mountpoints**: Automatically maps and tracks `/media` and `/run/media` for external drive visibility.

//-NOTE: Disk Diagnostics Module (/src/modules/disks/mod.rs)
//- Use these new "tags" for code blocks and notes.
//- Tag reference in build.rs
//- Files are saved to /snippets/{code, notes} in markdown (.md) format.
//-END


use std::fs::{self, OpenOptions};
use std::process::Command;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::types::CyberdeckState;

/// Executes the full disk diagnostic suite.
///
/// Scans block devices, identifies partition/mount configurations, and performs
/// deep-dive health checks on detected hardware.
pub async fn execute(_state: &CyberdeckState, params: &str) -> Result<String, String> {
    let dir = params;

    // Ensure the base directory exists
    fs::create_dir_all(dir).map_err(|e| e.to_string())?;

    // 1. Structural Setup
    let layout = ["nvme", "ssd", "hdd", "usb", "media", "raw"];
    for folder in layout {
        fs::create_dir_all(format!("{}/{}", dir, folder)).map_err(|e| e.to_string())?;
    }

    let base_f = format!("{}/disks.md", dir);
    let timestamp = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map_err(|e| e.to_string())?
    .as_secs();

    // Helper: Shell execution
    let run_cmd = |cmd: &str, args: &[&str]| -> String {
        Command::new(cmd)
        .args(args)
        .output()
        .map(|out| String::from_utf8_lossy(&out.stdout).to_string())
        .unwrap_or_else(|_| format!("{} not available", cmd))
    };

    // Initialize report
    let mut file = OpenOptions::new()
    .create(true)
    .write(true)
    .truncate(true)
    .open(&base_f)
    .map_err(|e| e.to_string())?;

    writeln!(file, "<div style='background:#6a0dad;color:white;padding:6px;'>💾 CYBERDECK: DISK INTELLIGENCE SYSTEM</div>\n\nTimestamp: {}\n", timestamp)
    .map_err(|e| e.to_string())?;

    // 2. Scan Block Devices
    let lsblk_raw = run_cmd("lsblk", &["-dn", "-o", "NAME,TYPE"]);
    let devices: Vec<String> = lsblk_raw.lines()
    .filter(|l| l.contains("disk"))
    .filter_map(|l| l.split_whitespace().next().map(|s| s.to_string()))
    .collect();

    // 3. Process each device
    for dev in devices {
        let dev_path = format!("/dev/{}", dev);
        let model = fs::read_to_string(format!("/sys/block/{}/device/model", dev)).unwrap_or_else(|_| "Unknown".to_string());
        let is_rotational = fs::read_to_string(format!("/sys/block/{}/queue/rotational", dev)).unwrap_or_else(|_| "0".to_string()).trim() == "1";
        let is_removable = fs::read_to_string(format!("/sys/block/{}/removable", dev)).unwrap_or_else(|_| "0".to_string()).trim() == "1";

        // Categorize
        let category = if dev.starts_with("nvme") { "nvme" }
        else if is_removable { "usb" }
        else if is_rotational { "hdd" }
        else { "ssd" };

        let out_file = format!("{}/{}/{}.md", dir, category, dev);
        let mut report = format!("# 💾 {} Drive: {}\n\n- **Model:** {}\n- **Category:** {}\n",
                                 category.to_uppercase(), dev, model.trim(), category);

        // Partition/Mount details
        let mount_info = run_cmd("lsblk", &["-o", "NAME,SIZE,TYPE,FSTYPE,MOUNTPOINT", &dev_path]);
        report.push_str(&format!("\n## 📊 Layout\n```text\n{}\n```\n", mount_info));

        // Btrfs Specific Diagnostics
        if mount_info.contains("btrfs") {
            report.push_str("\n## 🌳 BTRFS Diagnostics\n");
            let mountpoint = run_cmd("lsblk", &["-n", "-o", "MOUNTPOINT", &dev_path]).lines().next().unwrap_or("").to_string();
            if !mountpoint.is_empty() {
                report.push_str(&format!("- **Mount:** `{}`\n", mountpoint));
                report.push_str(&format!("- **Usage:**\n```text\n{}\n```\n", run_cmd("btrfs", &["filesystem", "usage", &mountpoint])));
                report.push_str(&format!("- **Subvolumes:**\n```text\n{}\n```\n", run_cmd("btrfs", &["subvolume", "list", &mountpoint])));
            }
        }

        // SMART Health Matrix
        report.push_str("\n## 🧠 S.M.A.R.T. Health\n```text\n");
        report.push_str(&run_cmd("smartctl", &["-H", &dev_path]));
        report.push_str("\n```\n");

        fs::write(&out_file, report).map_err(|e| e.to_string())?;
        writeln!(file, "- [{}]({}) - {} ({})\n", dev, out_file, model.trim(), category).map_err(|e| e.to_string())?;
    }

    Ok(base_f)
}
