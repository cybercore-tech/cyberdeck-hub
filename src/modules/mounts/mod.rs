//! # CYBERDECK: Mounted Filesystems
//!
//! What's mounted where (`findmnt`, tree view) and how full each one is
//! (`df -hT`) — distinct from `disks.md`, which covers block devices, not
//! what's actually mounted on top of them.

use crate::types::CyberdeckState;
use std::fs;
use std::process::Command;

fn run(cmd: &str, args: &[&str]) -> String {
    Command::new(cmd)
        .args(args)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| format!("{cmd} not available"))
}

pub async fn execute(_state: &CyberdeckState, params: &str) -> Result<String, String> {
    let dir = params;
    let base_f = format!("{}/mounts.md", dir);
    let timestamp = crate::modules::utils::now_human();

    let findmnt = run("findmnt", &[]);
    let df = run("df", &["-hT"]);

    let report = format!(
        "# Mounted Filesystems\n\n\
        _Generated {timestamp}_\n\n\
        ## Mount tree\n\n```text\n{findmnt}\n```\n\n\
        ## Usage\n\n```text\n{df}\n```\n"
    );

    fs::write(&base_f, report).map_err(|e| e.to_string())?;
    Ok(base_f)
}
