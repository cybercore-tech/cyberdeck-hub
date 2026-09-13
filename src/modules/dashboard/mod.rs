//! # CYBERDECK: Live Dashboard
//!
//! Visualizes current system metrics via CLI output and generates
//! a persistent state report in the dashboard directory.
//!
//! ## Implementation Notes
//! - **Signature**: Matches the `execute(state, params)` project standard.
//! - **Dual-Output**: Prints live status to stdout and saves diagnostic state to dashboard.md.

//-NOTE: Live Dashboard (/src/modules/dashboard/mod.rs)
//- Use these new "tags" for code blocks and notes.
//- Tag reference in build.rs
//- Files are saved to /snippets/{code, notes} in markdown (.md) format.
//-END


use std::fs;
use std::process::Command;
use crate::types::CyberdeckState;

/// Executes the live dashboard diagnostic sweep.
pub async fn execute(_state: &CyberdeckState, params: &str) -> Result<String, String> {
    let dir = params;
    let base_f = format!("{}/dashboard.md", dir);
    let mut report = String::from("# 👁 CYBERDECK: LIVE DASHBOARD\n\n");

    let run_cmd = |cmd: &str, args: &[&str]| -> String {
        Command::new(cmd)
        .args(args)
        .output()
        .map(|out| String::from_utf8_lossy(&out.stdout).to_string())
        .unwrap_or_else(|_| "Unavailable".to_string())
    };

    // Live Console Output
    println!("====================================");
    println!("     👁 CYBERDECK DASHBOARD");
    println!("====================================");

    // 1. System Status
    let uptime = run_cmd("uptime", &[]);
    println!("🧠 SYSTEM STATUS: {}", uptime.trim());
    report.push_str(&format!("## 🧠 System Status\n`{}`\n\n", uptime.trim()));

    // 2. CPU
    let lscpu_out = run_cmd("lscpu", &[]);
    let model = lscpu_out.lines()
    .find(|l| l.contains("Model name"))
    .unwrap_or("Unknown CPU")
    .trim();
    println!("🔥 CPU: {}", model);
    report.push_str(&format!("## 🔥 CPU\n{}\n\n", model));

    // 3. Storage
    println!("💾 DISKS:");
    let lsblk = run_cmd("lsblk", &["-d", "-o", "NAME,SIZE,MODEL"]);
    print!("{}", lsblk);
    report.push_str(&format!("## 💾 Disks\n```text\n{}```\n\n", lsblk));

    // 4. Network
    println!("🌐 NETWORK:");
    let ip = run_cmd("ip", &["-br", "a"]);
    print!("{}", ip);
    report.push_str(&format!("## 🌐 Network\n```text\n{}```\n\n", ip));

    // 5. Power
    println!("🔋 POWER:");
    let power = run_cmd("upower", &["-e"]);
    if !power.is_empty() {
        let first_dev = power.lines().next().unwrap_or("");
        let info = run_cmd("upower", &["-i", first_dev]);
        println!("{}", info);
        report.push_str(&format!("## 🔋 Power\n```text\n{}```\n\n", info));
    }

    println!("====================================");
    println!("LIVE MODE ACTIVE");
    println!("====================================");

    fs::write(&base_f, report).map_err(|e| e.to_string())?;
    Ok(base_f)
}
