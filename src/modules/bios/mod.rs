//! # CYBERDECK: BIOS & System Intelligence Module
//!
//! The central diagnostic engine for low-level system identification. This module
//! probes motherboard firmware, processor states, and hardware bus mappings to
//! provide a comprehensive system snapshot.
//!
//! ## Implementation Notes
//! - **Privileged Execution**: Requires `sudo` for `dmidecode` to access DMI tables.
//! - **Data Flow**: Organizes outputs into a tree structure (`cpu/`, `memory/`,
//!   `chipset/`, etc.) for granular subsystem analysis.
//! - **Real-time Metrics**: Uses `sysfs` to poll frequency governors and current
//!   core frequencies.

//-NOTE: BIOS & System Intelligence Module (/src/modules/bios/mod.rs)
//- Use these new "tags" for code blocks and notes.
//- Tag reference in build.rs
//- Files are saved to /snippets/{code, notes} in markdown (.md) format.
//-END

use std::fs::{self, OpenOptions};
use std::process::Command;
use std::io::Write;

use crate::types::CyberdeckState;

/// Executes the full-stack system diagnostic suite.
pub async fn execute(_state: &CyberdeckState, params: &str) -> Result<String, String> {
    let dir = params;

    // 1. Initialize Subdirectory Tree Structure
    let dirs = ["cpu", "chipset", "motherboard", "memory", "kernel", "power"];
    for folder in dirs {
        fs::create_dir_all(format!("{}/{}", dir, folder)).map_err(|e| e.to_string())?;
    }

    // Cross-referenced copies under each subsystem's own folder — named
    // `bios-crossref.md`, NOT reusing the dedicated module's own filename
    // (`cpu.md`, `motherboard.md`, `memory.md`, `power.md`): four of these
    // used to collide with the standalone CPU/Motherboard/Memory/Power
    // scans' actual output, so whichever scan ran last silently overwrote
    // the other with a thinner copy.
    let base_f = format!("{}/bios.md", dir);
    let sub_bios_f = format!("{}/motherboard/bios.md", dir);
    let sub_board_f = format!("{}/motherboard/bios-crossref.md", dir);
    let sub_cpu_f = format!("{}/cpu/bios-crossref.md", dir);
    let sub_chipset_f = format!("{}/chipset/chipset.md", dir);
    let sub_mem_f = format!("{}/memory/bios-crossref.md", dir);
    let sub_kernel_f = format!("{}/kernel/kernel.md", dir);
    let sub_power_f = format!("{}/power/bios-crossref.md", dir);

    // Reusable file writing handlers
    let write_to = |path: &str, content: &str| -> Result<(), String> {
        let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| e.to_string())?;
        file.write_all(content.as_bytes()).map_err(|e| e.to_string())?;
        Ok(())
    };

    let overwrite_to = |path: &str, content: &str| -> Result<(), String> {
        let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)
        .map_err(|e| e.to_string())?;
        file.write_all(content.as_bytes()).map_err(|e| e.to_string())?;
        Ok(())
    };

    let run_cmd = |cmd: &str, args: &[&str]| -> String {
        Command::new(cmd)
        .args(args)
        .output()
        .map(|out| String::from_utf8_lossy(&out.stdout).to_string())
        .unwrap_or_else(|_| format!("{} not available\n", cmd))
    };

    let timestamp = crate::modules::utils::now_human();

    // 2. Base Header Configuration
    write_to(&base_f, &format!("# BIOS & System Intelligence\n\n_Generated {}_\n\n## System Overview\n", timestamp))?;

    let uname_all = run_cmd("uname", &["-a"]);
    write_to(&base_f, &uname_all)?;

    // 3. BIOS Records Extraction — `/sys/class/dmi/id/*`, not `sudo
    // dmidecode` (which silently produced nothing: this process has no TTY
    // to ever satisfy an interactive sudo prompt).
    write_to(&base_f, "\n## BIOS\n")?;
    let dmi_bios = format!(
        "- Vendor: {}\n- Version: {}\n- Release: {}\n- Date: {}\n",
        crate::modules::utils::dmi_field("bios_vendor"),
        crate::modules::utils::dmi_field("bios_version"),
        crate::modules::utils::dmi_field("bios_release"),
        crate::modules::utils::dmi_field("bios_date"),
    );
    write_to(&base_f, &dmi_bios)?;
    overwrite_to(&sub_bios_f, "# BIOS\n")?;
    write_to(&sub_bios_f, &dmi_bios)?;

    // 4. Motherboard Configuration
    write_to(&base_f, "\n## Motherboard\n")?;
    let dmi_board = format!(
        "- Vendor: {}\n- Model: {}\n- Version: {}\n",
        crate::modules::utils::dmi_field("board_vendor"),
        crate::modules::utils::dmi_field("board_name"),
        crate::modules::utils::dmi_field("board_version"),
    );
    write_to(&base_f, &dmi_board)?;
    overwrite_to(&sub_board_f, "# Motherboard\n")?;
    write_to(&sub_board_f, &dmi_board)?;

    // 5. Processor Diagnostics
    write_to(&base_f, "\n## 🧠 CPU\n")?;
    let lscpu_out = run_cmd("lscpu", &[]);
    write_to(&base_f, &crate::modules::utils::code_block("text", &lscpu_out))?;
    overwrite_to(&sub_cpu_f, "# 🧠 CPU\n")?;
    write_to(&sub_cpu_f, &crate::modules::utils::code_block("text", &lscpu_out))?;
    if let Ok(cpuinfo) = fs::read_to_string("/proc/cpuinfo") {
        write_to(&sub_cpu_f, &crate::modules::utils::code_block("text", &cpuinfo))?;
    }

    // 6. Chipset Hardware Components
    write_to(&base_f, "\n## 🔌 CHIPSET / PCI\n")?;
    let lspci_out = run_cmd("lspci", &[]);
    write_to(&base_f, &crate::modules::utils::code_block("text", &lspci_out))?;
    overwrite_to(&sub_chipset_f, "# 🔌 CHIPSET\n")?;
    write_to(&sub_chipset_f, &crate::modules::utils::code_block("text", &lspci_out))?;

    // 7. Memory Profile
    write_to(&base_f, "\n## 🧠 MEMORY\n")?;
    let free_out = run_cmd("free", &["-h"]);
    write_to(&base_f, &crate::modules::utils::code_block("text", &free_out))?;
    overwrite_to(&sub_mem_f, "# 🧠 MEMORY\n")?;
    write_to(&sub_mem_f, &crate::modules::utils::code_block("text", &free_out))?;
    write_to(
        &sub_mem_f,
        "\nPer-DIMM detail (`dmidecode -t memory`) needs root — not \
         available to this unattended service; the usage stats above are.\n",
    )?;

    // 8. Kernel Parameters — single-line values, no fence needed.
    write_to(&base_f, "\n## 🧬 KERNEL\n")?;
    let uname_release = run_cmd("uname", &["-r"]);
    write_to(&base_f, &format!("- Release: `{}`\n", uname_release.trim()))?;
    let cmdline = fs::read_to_string("/proc/cmdline").unwrap_or_default();
    write_to(&base_f, &format!("- Cmdline: `{}`\n", cmdline.trim()))?;
    // Uptime (from /proc/uptime, seconds.subseconds — first field only)
    // and taint status (0 = clean; any other value flags something like
    // an out-of-tree or proprietary module loaded) — two real, cheap-to-
    // read signals that were previously just missing entirely.
    if let Ok(uptime_raw) = fs::read_to_string("/proc/uptime") {
        if let Some(secs) = uptime_raw.split_whitespace().next().and_then(|s| s.parse::<f64>().ok()) {
            let days = (secs / 86400.0) as u64;
            let hours = ((secs % 86400.0) / 3600.0) as u64;
            let mins = ((secs % 3600.0) / 60.0) as u64;
            write_to(&base_f, &format!("- Uptime: {days}d {hours}h {mins}m\n"))?;
        }
    }
    let tainted = fs::read_to_string("/proc/sys/kernel/tainted").unwrap_or_default();
    write_to(&base_f, &format!(
        "- Tainted: `{}`{}\n",
        tainted.trim(),
        if tainted.trim() == "0" { " (clean)" } else { " (see `dmesg | grep -i taint` for why)" }
    ))?;
    write_to(&base_f, "- Loaded modules: see the dedicated Kernel Modules scan\n")?;
    overwrite_to(&sub_kernel_f, "# 🧬 KERNEL\n")?;
    write_to(&sub_kernel_f, &crate::modules::utils::code_block("text", &uname_all))?;
    write_to(&sub_kernel_f, &format!("Cmdline: `{}`\n", cmdline.trim()))?;

    // 9. Power Profile Mapping
    write_to(&base_f, "\n## 🔋 POWER & CPU CONTROL\n")?;
    overwrite_to(&sub_power_f, "# 🔋 POWER PROFILE\n")?;
    let cpupower_info = run_cmd("cpupower", &["frequency-info"]);
    if cpupower_info.contains("not available") {
        // cpupower isn't installed on this box — this used to just note
        // that in base_f and leave sub_power_f completely empty (nothing
        // but its own bare heading). The same governor/frequency data
        // cpupower would report is directly readable from sysfs (same
        // source power.rs's own dedicated scan already uses), so read
        // that instead of leaving a real section with nothing in it.
        write_to(&base_f, "`cpupower` not installed — reading governor/frequency from sysfs instead:\n\n")?;
        let mut freq_table = String::from("| Core | Governor | Current MHz |\n|---|---|---|\n");
        if let Ok(entries) = fs::read_dir("/sys/devices/system/cpu") {
            let mut cores: Vec<_> = entries.flatten().collect();
            cores.sort_by_key(|e| e.file_name());
            for entry in cores {
                let name = entry.file_name().into_string().unwrap_or_default();
                if name.starts_with("cpu") && name[3..].chars().all(|c| c.is_ascii_digit()) {
                    let gov = fs::read_to_string(entry.path().join("cpufreq/scaling_governor"))
                        .unwrap_or_else(|_| "n/a".to_string());
                    let freq_khz = fs::read_to_string(entry.path().join("cpufreq/scaling_cur_freq"))
                        .ok()
                        .and_then(|s| s.trim().parse::<f64>().ok());
                    let mhz = freq_khz.map(|f| format!("{:.0}", f / 1000.0)).unwrap_or_else(|| "n/a".to_string());
                    freq_table.push_str(&format!("| {} | {} | {} |\n", name, gov.trim(), mhz));
                }
            }
        }
        write_to(&base_f, &freq_table)?;
        write_to(&sub_power_f, &freq_table)?;
    } else {
        write_to(&base_f, "### ⚙️ CPUPOWER INFO\n")?;
        write_to(&base_f, &crate::modules::utils::code_block("text", &cpupower_info))?;
        write_to(&sub_power_f, &crate::modules::utils::code_block("text", &cpupower_info))?;
    }

    // 10. Live Frequency Matrix — built up line-by-line in the loop below,
    // so it has to be accumulated into one string first and wrapped in a
    // single fence afterward; fencing each line individually would open
    // and close a separate code block per CPU core.
    write_to(&base_f, "\n### 📊 CPU FREQUENCY SNAPSHOT\n")?;
    let mut freq_lines = String::new();
    if let Ok(entries) = fs::read_dir("/sys/devices/system/cpu") {
        for entry in entries.flatten() {
            let name = entry.file_name().into_string().unwrap_or_default();
            if name.starts_with("cpu") && name[3..].chars().all(|c| c.is_ascii_digit()) {
                let freq_path = format!("/sys/devices/system/cpu/{}/cpufreq/scaling_cur_freq", name);
                if let Ok(raw_freq_str) = fs::read_to_string(freq_path) {
                    if let Ok(raw_freq) = raw_freq_str.trim().parse::<f64>() {
                        freq_lines.push_str(&format!("{}: {:.2} MHz\n", name, raw_freq / 1000.0));
                    }
                }
            }
        }
    }
    if !freq_lines.is_empty() {
        write_to(&base_f, &crate::modules::utils::code_block("text", &freq_lines))?;
    }

    Ok(base_f)
}
