//! # CYBERDECK: Network & Connectivity Intelligence
//!
//! Unified module for probing system-wide networking. Handles interface telemetry,
//! routing tables, DNS resolution, and driver/hardware auditing.

//-NOTE: Network & Connectivity Intelligence (/src/modules/network/mod.rs)
//- Use these new "tags" for code blocks and notes.
//- Tag reference in build.rs
//- Files are saved to /snippets/{code, notes} in markdown (.md) format.
//-END

use std::fs::{self, OpenOptions};
use std::process::Command;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::types::CyberdeckState;

/// Executes the comprehensive network diagnostic sweep.
pub async fn execute(_state: &CyberdeckState, params: &str) -> Result<String, String> {
    let dir = params;

    // 1. Structural Setup
    let subdirs = ["interfaces", "routes", "dns", "sockets", "stats", "raw"];
    for sub in subdirs {
        fs::create_dir_all(format!("{}/{}", dir, sub)).map_err(|e| e.to_string())?;
    }

    let base_f = format!("{}/network.md", dir);
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

    // Initialize Global Report
    let mut file = OpenOptions::new()
    .create(true)
    .write(true)
    .truncate(true)
    .open(&base_f)
    .map_err(|e| e.to_string())?;

    writeln!(file, "<div style='background:#6a0dad;color:white;padding:6px;'>🌐 NETWORK & CONNECTIVITY INTELLIGENCE</div>\n\nTimestamp: {}\n", timestamp)
    .map_err(|e| e.to_string())?;

    // 2. Global Network Stack
    file.write_all("\n## 📦 GLOBAL NETWORK STACK\n".as_bytes()).map_err(|e| e.to_string())?;
    writeln!(file, "- **IP Addr:**\n```\n{}\n```", run_cmd("ip", &["a"])).map_err(|e| e.to_string())?;
    writeln!(file, "- **Routes:**\n```\n{}\n```", run_cmd("ip", &["route"])).map_err(|e| e.to_string())?;

    // 3. Interface Enumeration
    file.write_all("\n## 🔌 INTERFACES & HARDWARE\n".as_bytes()).map_err(|e| e.to_string())?;
    if let Ok(net_entries) = fs::read_dir("/sys/class/net") {
        for entry in net_entries.flatten() {
            let iface = entry.file_name().into_string().unwrap_or_default();
            if iface.is_empty() { continue; }

            // Create per-interface report
            let iface_report = format!("{}/interfaces/{}.md", dir, iface);
            let mut iface_file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&iface_report)
            .map_err(|e| e.to_string())?;

            writeln!(iface_file, "# 🔌 INTERFACE: {}\n", iface).map_err(|e| e.to_string())?;
            writeln!(iface_file, "## 📡 CONFIG\n```\n{}\n```", run_cmd("ip", &["addr", "show", &iface])).map_err(|e| e.to_string())?;

            // Gather metrics from sysfs
            if let Ok(state) = fs::read_to_string(entry.path().join("operstate")) {
                writeln!(iface_file, "\n## ⚙️ STATE: {}", state.trim()).map_err(|e| e.to_string())?;
            }
            if let Ok(speed) = fs::read_to_string(entry.path().join("speed")) {
                writeln!(iface_file, "\n## 🚀 SPEED: {} Mbps", speed.trim()).map_err(|e| e.to_string())?;
            }

            // Driver Diagnostics
            if let Ok(target) = fs::read_link(entry.path().join("device/driver")) {
                if let Some(drv) = target.file_name().and_then(|n| n.to_str()) {
                    writeln!(iface_file, "\n## 🧩 DRIVER: `{}`", drv).map_err(|e| e.to_string())?;
                }
            }

            // Ethtool
            let ethtool = run_cmd("ethtool", &[&iface]);
            if !ethtool.contains("not available") {
                writeln!(iface_file, "\n## 🔗 ETH-TOOL\n```\n{}\n```", ethtool).map_err(|e| e.to_string())?;
            }

            // Cross-reference in global report
            writeln!(file, "- [{}](interfaces/{}.md)", iface, iface).map_err(|e| e.to_string())?;
        }
    }

    // 4. DNS & Sockets
    file.write_all("\n## 🌍 DNS RESOLUTION\n".as_bytes()).map_err(|e| e.to_string())?;
    writeln!(file, "```\n{}\n```", fs::read_to_string("/etc/resolv.conf").unwrap_or_else(|_| "Unavailable".to_string())).map_err(|e| e.to_string())?;

    file.write_all("\n## 🔥 ACTIVE SOCKETS\n".as_bytes()).map_err(|e| e.to_string())?;
    writeln!(file, "```\n{}\n```", run_cmd("ss", &["-tulpn"])).map_err(|e| e.to_string())?;

    // 5. Driver Conflicts
    file.write_all("\n## ⚠️ DRIVER CONFLICT AUDIT\n".as_bytes()).map_err(|e| e.to_string())?;
    let lsmod = run_cmd("sh", &["-c", "lsmod | grep -E 'r8125|r8169'"]);
    if lsmod.contains("r8125") && lsmod.contains("r8169") {
        writeln!(file, "❌ **WARNING**: Conflict detected. Both `r8125` and `r8169` are loaded.").map_err(|e| e.to_string())?;
    } else {
        writeln!(file, "✅ Driver environment clean.").map_err(|e| e.to_string())?;
    }

    Ok(base_f)
}
