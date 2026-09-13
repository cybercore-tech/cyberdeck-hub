//! # CYBERDECK: Kernel Modules
//!
//! Every loaded kernel module (`lsmod`) — name, resident size, and
//! reference count/dependents — sorted largest-first so memory-hungry
//! modules surface immediately instead of needing a manual sort.

use crate::types::CyberdeckState;
use std::fs;
use std::process::Command;

struct KMod {
    name: String,
    size: u64,
    used_by: String,
}

fn parse_lsmod(out: &str) -> Vec<KMod> {
    // "Module                  Size  Used by\nudp_diag  12288  0\nip6t_REJECT  12288  1 nf_reject_ipv6"
    out.lines()
        .skip(1) // header
        .filter_map(|line| {
            let mut parts = line.split_whitespace();
            let name = parts.next()?.to_string();
            let size = parts.next()?.parse().ok()?;
            let rest: Vec<&str> = parts.collect();
            // First remaining token is the reference count, anything after
            // is the comma-joined list of modules depending on this one.
            let used_by = if rest.len() > 1 {
                format!("{} ({})", rest[0], rest[1..].join(" "))
            } else {
                rest.first().unwrap_or(&"0").to_string()
            };
            Some(KMod { name, size, used_by })
        })
        .collect()
}

pub async fn execute(_state: &CyberdeckState, params: &str) -> Result<String, String> {
    let dir = params;
    let base_f = format!("{}/kernel_modules.md", dir);
    let timestamp = crate::modules::utils::now_human();

    let raw = Command::new("lsmod")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();
    let mut mods = parse_lsmod(&raw);
    mods.sort_by(|a, b| b.size.cmp(&a.size));

    let mut report = format!("# Kernel Modules\n\n_Generated {timestamp}_\n\n");
    if mods.is_empty() {
        report.push_str("`lsmod` returned nothing.\n");
    } else {
        let total_kb: u64 = mods.iter().map(|m| m.size).sum::<u64>() / 1024;
        report.push_str(&format!(
            "{} modules loaded, {} KB resident total. Sorted by size, largest first.\n\n",
            mods.len(),
            total_kb
        ));
        report.push_str("| Module | Size (KB) | Ref count (dependents) |\n|---|---|---|\n");
        for m in &mods {
            report.push_str(&format!("| `{}` | {} | {} |\n", m.name, m.size / 1024, m.used_by));
        }
    }

    fs::write(&base_f, report).map_err(|e| e.to_string())?;
    Ok(base_f)
}
