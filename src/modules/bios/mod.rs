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
use std::time::{SystemTime, UNIX_EPOCH};

use crate::types::CyberdeckState;

/// Executes the full-stack system diagnostic suite.
pub async fn execute(_state: &CyberdeckState, params: &str) -> Result<String, String> {
    let dir = params;

    // 1. Initialize Subdirectory Tree Structure
    let dirs = ["cpu", "chipset", "motherboard", "memory", "kernel", "power"];
    for folder in dirs {
        fs::create_dir_all(format!("{}/{}", dir, folder)).map_err(|e| e.to_string())?;
    }

    let base_f = format!("{}/bios.md", dir);
    let sub_bios_f = format!("{}/motherboard/bios.md", dir);
    let sub_board_f = format!("{}/motherboard/motherboard.md", dir);
    let sub_cpu_f = format!("{}/cpu/cpu.md", dir);
    let sub_chipset_f = format!("{}/chipset/chipset.md", dir);
    let sub_mem_f = format!("{}/memory/memory.md", dir);
    let sub_kernel_f = format!("{}/kernel/kernel.md", dir);
    let sub_power_f = format!("{}/power/power.md", dir);

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

    let timestamp = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map_err(|e| e.to_string())?
    .as_secs();

    // 2. Base Header Configuration
    write_to(&base_f, &format!("⚙️ CYBERDECK: BIOS & SYSTEM INTELLIGENCE\nTimestamp: {}\n\n## 🧠 SYSTEM OVERVIEW\n", timestamp))?;
    write_to(&base_f, &format!("Timestamp: {}\n\n## 🧠 SYSTEM OVERVIEW\n", timestamp))?;

    let uname_all = run_cmd("uname", &["-a"]);
    write_to(&base_f, &uname_all)?;

    // 3. BIOS Records Extraction
    write_to(&base_f, "\n## ⚙️ BIOS\n")?;
    let dmi_bios = run_cmd("sudo", &["dmidecode", "-t", "bios"]);
    write_to(&base_f, &dmi_bios)?;
    overwrite_to(&sub_bios_f, "# ⚙️ BIOS\n")?;
    write_to(&sub_bios_f, &dmi_bios)?;

    // 4. Motherboard Configuration
    write_to(&base_f, "\n## 🧩 MOTHERBOARD\n")?;
    let dmi_board = run_cmd("sudo", &["dmidecode", "-t", "baseboard"]);
    write_to(&base_f, &dmi_board)?;
    overwrite_to(&sub_board_f, "# 🧩 MOTHERBOARD\n")?;
    write_to(&sub_board_f, &dmi_board)?;

    // 5. Processor Diagnostics
    write_to(&base_f, "\n## 🧠 CPU\n")?;
    let lscpu_out = run_cmd("lscpu", &[]);
    write_to(&base_f, &lscpu_out)?;
    overwrite_to(&sub_cpu_f, "# 🧠 CPU\n")?;
    write_to(&sub_cpu_f, &lscpu_out)?;
    if let Ok(cpuinfo) = fs::read_to_string("/proc/cpuinfo") {
        write_to(&sub_cpu_f, &cpuinfo)?;
    }

    // 6. Chipset Hardware Components
    write_to(&base_f, "\n## 🔌 CHIPSET / PCI\n")?;
    let lspci_out = run_cmd("lspci", &[]);
    write_to(&base_f, &lspci_out)?;
    overwrite_to(&sub_chipset_f, "# 🔌 CHIPSET\n")?;
    write_to(&sub_chipset_f, &lspci_out)?;

    // 7. Memory Profile
    write_to(&base_f, "\n## 🧠 MEMORY\n")?;
    let free_out = run_cmd("free", &["-h"]);
    write_to(&base_f, &free_out)?;
    overwrite_to(&sub_mem_f, "# 🧠 MEMORY\n")?;
    write_to(&sub_mem_f, &free_out)?;
    let dmi_mem = run_cmd("sudo", &["dmidecode", "-t", "memory"]);
    write_to(&sub_mem_f, &dmi_mem)?;

    // 8. Kernel Parameters
    write_to(&base_f, "\n## 🧬 KERNEL\n")?;
    let uname_release = run_cmd("uname", &["-r"]);
    write_to(&base_f, &uname_release)?;
    let cmdline = fs::read_to_string("/proc/cmdline").unwrap_or_default();
    write_to(&base_f, &format!("{}\n", cmdline))?;
    overwrite_to(&sub_kernel_f, "# 🧬 KERNEL\n")?;
    write_to(&sub_kernel_f, &uname_all)?;
    write_to(&sub_kernel_f, &format!("{}\n", cmdline))?;

    // 9. Power Profile Mapping
    write_to(&base_f, "\n## 🔋 POWER & CPU CONTROL\n")?;
    overwrite_to(&sub_power_f, "# 🔋 POWER PROFILE\n")?;
    let cpupower_info = run_cmd("cpupower", &["frequency-info"]);
    if cpupower_info.contains("not available") {
        write_to(&base_f, "cpupower not installed\n")?;
    } else {
        write_to(&base_f, "### ⚙️ CPUPOWER INFO\n")?;
        write_to(&base_f, &cpupower_info)?;
        write_to(&sub_power_f, &cpupower_info)?;
    }

    // 10. Live Frequency Matrix
    write_to(&base_f, "\n### 📊 CPU FREQUENCY SNAPSHOT\n")?;
    if let Ok(entries) = fs::read_dir("/sys/devices/system/cpu") {
        for entry in entries.flatten() {
            let name = entry.file_name().into_string().unwrap_or_default();
            if name.starts_with("cpu") && name[3..].chars().all(|c| c.is_ascii_digit()) {
                let freq_path = format!("/sys/devices/system/cpu/{}/cpufreq/scaling_cur_freq", name);
                if let Ok(raw_freq_str) = fs::read_to_string(freq_path) {
                    if let Ok(raw_freq) = raw_freq_str.trim().parse::<f64>() {
                        write_to(&base_f, &format!("{}: {:.2} MHz\n", name, raw_freq / 1000.0))?;
                    }
                }
            }
        }
    }

    Ok(base_f)
}
