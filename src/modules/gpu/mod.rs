//! # CYBERDECK: GPU Intelligence
//!
//! Every VGA/3D/display PCI device (`lspci -vmm`, vendor-agnostic — works
//! for Intel/AMD integrated and NVIDIA/AMD discrete alike), plus the
//! kernel driver bound to each (`/sys/bus/pci/devices/<addr>/driver`) and,
//! when present, `nvidia-smi` for live utilization/memory/temperature.

use crate::types::CyberdeckState;
use std::fs;
use std::process::Command;

struct PciBlock {
    slot: String,
    class: String,
    vendor: String,
    device: String,
}

fn parse_blocks(vmm: &str) -> Vec<PciBlock> {
    vmm.split("\n\n")
        .filter_map(|block| {
            let mut slot = String::new();
            let mut class = String::new();
            let mut vendor = String::new();
            let mut device = String::new();
            for line in block.lines() {
                let Some((key, val)) = line.split_once(':') else { continue };
                let val = val.trim().to_string();
                match key.trim() {
                    "Slot" => slot = val,
                    "Class" => class = val,
                    "Vendor" => vendor = val,
                    "Device" => device = val,
                    _ => {}
                }
            }
            if slot.is_empty() {
                return None;
            }
            Some(PciBlock { slot, class, vendor, device })
        })
        .filter(|b| {
            let c = b.class.to_lowercase();
            c.contains("vga") || c.contains("3d controller") || c.contains("display controller")
        })
        .collect()
}

fn driver_for(slot: &str) -> String {
    let link = format!("/sys/bus/pci/devices/0000:{slot}/driver");
    fs::read_link(&link)
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        .unwrap_or_else(|| "no driver bound".to_string())
}

pub async fn execute(_state: &CyberdeckState, params: &str) -> Result<String, String> {
    let dir = params;
    let base_f = format!("{}/gpu.md", dir);
    let timestamp = crate::modules::utils::now_human();

    let vmm = Command::new("lspci")
        .arg("-vmm")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();
    let gpus = parse_blocks(&vmm);

    let mut report = format!("# GPU\n\n_Generated {timestamp}_\n\n");

    if gpus.is_empty() {
        report.push_str("No VGA/3D/display PCI device found.\n");
    } else {
        report.push_str("## Devices\n\n| Slot | Class | Vendor | Device | Driver |\n|---|---|---|---|---|\n");
        for g in &gpus {
            report.push_str(&format!(
                "| {} | {} | {} | {} | {} |\n",
                g.slot,
                g.class,
                g.vendor,
                g.device,
                driver_for(&g.slot)
            ));
        }
    }

    // NVIDIA-specific live stats, only when the tool is actually present —
    // most boxes (this one included) are Intel/AMD-only and shouldn't show
    // an "unavailable" error for a vendor tool that was never expected to
    // apply.
    report.push_str("\n## NVIDIA live stats\n\n");
    match Command::new("nvidia-smi")
        .args(["--query-gpu=name,driver_version,utilization.gpu,memory.used,memory.total,temperature.gpu", "--format=csv"])
        .output()
    {
        Ok(out) if out.status.success() => {
            report.push_str("```text\n");
            report.push_str(String::from_utf8_lossy(&out.stdout).trim());
            report.push_str("\n```\n");
        }
        _ => report.push_str("`nvidia-smi` not present — no NVIDIA GPU/driver on this system.\n"),
    }

    fs::write(&base_f, report).map_err(|e| e.to_string())?;
    Ok(base_f)
}
