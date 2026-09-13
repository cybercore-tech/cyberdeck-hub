//! # CYBERDECK: Memory Intelligence Module
//!
//! Provides deep-dive metrics on physical RAM status, swap health,
//! and NUMA topology. Ready for DDR5+ reporting.

//-NOTE: Memory Intelligence Module (/src/modules/memory/mod.rs)
//- Use these new "tags" for code blocks and notes.
//- Tag reference in build.rs
//- Files are saved to /snippets/{code, notes} in markdown (.md) format.
//-END

use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::types::CyberdeckState;

/// Executes the memory and topology diagnostic suite.
pub async fn execute(_state: &CyberdeckState, params: &str) -> Result<String, String> {
    let dir = params;
    let base_f = format!("{}/memory.md", dir);

    let timestamp = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map_err(|e| e.to_string())?
    .as_secs();

    let mut report = format!("<div style='background:#6a0dad;color:white;padding:6px;'>🧠 CYBERDECK: MEMORY & TOPOLOGY CORE</div>\n\nTimestamp: {}\n\n", timestamp);

    // 1. Memory Usage
    report.push_str("## 📊 Usage Statistics\n```text\n");
    let free = Command::new("free")
    .args(["-h"])
    .output()
    .map_err(|e| e.to_string())?;
    report.push_str(&String::from_utf8_lossy(&free.stdout));
    report.push_str("```\n");

    // 2. Physical DIMM Inventory
    // Note: dmidecode usually requires root privileges.
    report.push_str("\n## 🧩 DIMM Inventory\n");
    let dmi_out = Command::new("sudo")
    .args(["dmidecode", "-t", "memory"])
    .output()
    .map_err(|e| e.to_string())?;

    let dmi = String::from_utf8_lossy(&dmi_out.stdout);

    for line in dmi.lines() {
        if line.contains("Size:") || line.contains("Speed:") || line.contains("Type:") || line.contains("Part Number:") {
            report.push_str(&format!("- `{}`\n", line.trim()));
        }
    }

    // 3. Swap & NUMA
    report.push_str("\n## 🧬 Swap & NUMA Nodes\n");
    report.push_str("- **Swappiness Setting**: ");
    let swappiness = fs::read_to_string("/proc/sys/vm/swappiness")
    .unwrap_or_else(|_| "Unknown".to_string());
    report.push_str(&swappiness);

    report.push_str("\n- **Active Swap Partition**:\n```text\n");
    let swapon = Command::new("swapon")
    .args(["--show"])
    .output()
    .map_err(|e| e.to_string())?;
    report.push_str(&String::from_utf8_lossy(&swapon.stdout));
    report.push_str("```\n");

    // 4. Performance Profile
    report.push_str("\n## ⚙️ Performance Tuning Status\n");
    if let Ok(profile) = Command::new("tuned-adm").arg("active").output() {
        report.push_str(&format!("- **Active Profile**: {}", String::from_utf8_lossy(&profile.stdout)));
    } else {
        report.push_str("- **Tuned Daemon**: Not installed (Optional: `apt install tuned`)");
    }

    fs::write(&base_f, report).map_err(|e| e.to_string())?;
    Ok(base_f)
}
