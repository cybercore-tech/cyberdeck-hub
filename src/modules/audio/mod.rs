//! # Audio Diagnostics Module
//!
//! This module provides deep introspection capabilities for the system's audio stack,
//! specifically targeting PipeWire, WirePlumber, and PulseAudio compatibility layers.

//-NOTE: Audio Diagnostics Module (/src/modules/audio/mod.rs)
//- Use these new "tags" for code blocks and notes.
//- Tag reference in build.rs
//- Files are saved to /snippets/{code, notes} in markdown (.md) format.
//-END

use crate::types::CyberdeckState;
use crate::modules::utils::write_header; // Assuming your helper is here
use std::fs::{self, OpenOptions, File};
use std::io::Write;
use std::process::Command;

/// Executes the full audio diagnostic suite and generates a structured report.
pub async fn execute(_state: &CyberdeckState, dir: &str) -> Result<String, String> {
    let raw_dir = format!("{}/raw", dir);
    let parsed_dir = format!("{}/parsed", dir);
    let graph_dir = format!("{}/graph", dir);

    // 1. Initialize Directories
    fs::create_dir_all(&raw_dir).map_err(|e| e.to_string())?;
    fs::create_dir_all(&parsed_dir).map_err(|e| e.to_string())?;
    fs::create_dir_all(&graph_dir).map_err(|e| e.to_string())?;

    let base_f = format!("{}/audio.md", dir);
    let raw_f = format!("{}/audio_raw.md", raw_dir);
    let graph_f = format!("{}/pipewire_graph.md", graph_dir);
    let parsed_f = format!("{}/audio_summary.md", parsed_dir);

    // 2. Prime the file with the Cyberdeck Header
    {
        let mut file = File::create(&base_f).map_err(|e| e.to_string())?;
        write_header(&mut file, "ACTIVE").map_err(|e| e.to_string())?;
    }

    // Helper: Appends content to a specific file
    let write_to = |path: &str, content: &str| -> Result<(), String> {
        let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| e.to_string())?;
        file.write_all(content.as_bytes())
        .map_err(|e| e.to_string())?;
        Ok(())
    };

    // 3. Write System Overview
    write_to(&base_f, "\n# 🎧 PIPEWIRE AUDIO INTELLIGENCE\n\n## 📊 SYSTEM OVERVIEW\n- Stack: PipeWire + WirePlumber + Pulse compatibility\n")?;

    // Helper: Executes system shell commands
    let run_cmd = |args: &[&str]| -> String {
        if args.is_empty() {
            return String::new();
        }
        Command::new(args[0])
        .args(&args[1..])
        .output()
        .map(|out| String::from_utf8_lossy(&out.stdout).to_string())
        .unwrap_or_else(|_| format!("{} not available\n", args[0]))
    };

    // 4. Pipewire Services
    write_to(&base_f, "\n## ⚙️ PIPEWIRE SERVICES\n")?;
    for service in &["pipewire", "wireplumber", "pipewire-pulse"] {
        let out = Command::new("systemctl")
        .args(&["--user", "status", service])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();
        write_to(&base_f, &out)?;
    }

    // 5. PulseAudio Layer
    write_to(&base_f, "\n## 🔊 PULSE COMPAT LAYER\n")?;
    let pactl_info = run_cmd(&["pactl", "info"]);
    write_to(&base_f, &pactl_info)?;
    write_to(&base_f, &run_cmd(&["pactl", "list", "short", "sinks"]))?;
    write_to(&base_f, &run_cmd(&["pactl", "list", "short", "sources"]))?;

    // 6. Pipewire Graph
    write_to(&base_f, "\n## 🔗 PIPEWIRE NODE GRAPH\n")?;
    let nodes = run_cmd(&["pw-cli", "ls", "Node"]);
    let links = run_cmd(&["pw-cli", "ls", "Link"]);

    fs::write(&graph_f, format!("{}{}", nodes, links)).map_err(|e| e.to_string())?;
    write_to(&base_f, &format!("Graph exported to: {}\n", graph_f))?;

    // 7. Hardware & Integration
    write_to(&base_f, "\n## 🎛️ AUDIO HARDWARE\n")?;
    let lspci_audio = Command::new("sh")
    .args(&["-c", "lspci | grep -i audio"])
    .output()
    .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
    .unwrap_or_default();
    let lsusb_audio = Command::new("sh")
    .args(&["-c", "lsusb | grep -i audio"])
    .output()
    .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
    .unwrap_or_default();
    write_to(&base_f, &lspci_audio)?;
    write_to(&base_f, &lsusb_audio)?;

    // 8. Final Reporting
    fs::write(
        &raw_f,
        format!(
            "{}{}{}{}{}",
            run_cmd(&["lscpu"]),
                run_cmd(&["lspci"]),
                run_cmd(&["lsusb"]),
                pactl_info,
                nodes
        ),
    )
    .map_err(|e| e.to_string())?;

    let mut pf = fs::File::create(&parsed_f).map_err(|e| e.to_string())?;
    pf.write_all("# 🎧 AUDIO SUMMARY\n\n## 🔊 Active Sinks\n".as_bytes())
    .map_err(|e| e.to_string())?;
    pf.write_all(run_cmd(&["pactl", "list", "short", "sinks"]).as_bytes())
    .map_err(|e| e.to_string())?;
    pf.write_all("\n## 🎤 Active Sources\n".as_bytes())
    .map_err(|e| e.to_string())?;
    pf.write_all(run_cmd(&["pactl", "list", "short", "sources"]).as_bytes())
    .map_err(|e| e.to_string())?;

    write_to(&base_f, "\n## 🧠 AUDIO HEALTH\n")?;
    for svc in &["pipewire", "wireplumber"] {
        let is_active = Command::new("systemctl")
        .args(&["--user", "is-active", svc])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

        let status_msg = if is_active {
            format!("✔ {}: running\n", svc)
        } else {
            format!("❌ {}: not running\n", svc)
        };
        write_to(&base_f, &status_msg)?;
    }

    Ok(base_f)
}
