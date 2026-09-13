//! # CYBERDECK: Service Intelligence Module
//!
//! Audits systemd services, categorizes them by function (Web, DB, Security, Infra),
//! and provides operational status reporting.
//!
//! ## Implementation Notes
//! - **Discovery**: Queries `systemctl` for active services.
//! - **Categorization**: Heuristic-based grouping (Web, DB, Infra, Security).
//! - **Reporting**: Generates a formatted markdown report of running processes.

//-NOTE: Service Intelligence Module (/src/modules/services/mod.rs)
//- Use these new "tags" for code blocks and notes.
//- Tag reference in build.rs
//- Files are saved to /snippets/{code, notes} in markdown (.md) format.
//-END

use std::fs;
use std::process::Command;
use crate::types::CyberdeckState;

#[derive(Debug, PartialEq)]
pub enum ServiceCategory {
    WebServer,
    Infrastructure,
    Database,
    Security,
    Unknown,
}

pub struct ServiceItem {
    pub name: String,
    pub category: ServiceCategory,
    pub is_active: bool,
}

/// Executes the service diagnostic suite.
pub async fn execute(_state: &CyberdeckState, params: &str) -> Result<String, String> {
    let dir = params;
    let base_f = format!("{}/services.md", dir);

    // 1. Discover and categorize
    let services = discover_services();

    // 2. Build report
    let mut report = String::from("# ⚙️ CYBERDECK: ACTIVE SERVICES\n\n");
    report.push_str("| Service Name | Category | Status |\n");
    report.push_str("| :--- | :--- | :--- |\n");

    for svc in services {
        let cat = format!("{:?}", svc.category);
        let status = if svc.is_active { "✅ Active" } else { "❌ Inactive" };
        report.push_str(&format!("| `{}` | {} | {} |\n", svc.name, cat, status));
    }

    // 3. Persist
    fs::write(&base_f, report).map_err(|e| e.to_string())?;
    Ok(base_f)
}

fn discover_services() -> Vec<ServiceItem> {
    let output = Command::new("systemctl")
    .args(["list-units", "--type=service", "--state=running", "--no-pager", "--no-legend"])
    .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            stdout.lines().map(|line| {
                let parts: Vec<&str> = line.split_whitespace().collect();
                let name = parts.first().unwrap_or(&"unknown").to_string();
                ServiceItem {
                    category: classify_service(&name),
                               name,
                               is_active: true,
                }
            }).collect()
        },
        Err(_) => vec![], // Return empty if systemctl fails/not found
    }
}

fn classify_service(name: &str) -> ServiceCategory {
    if name.contains("nginx") || name.contains("httpd") || name.contains("caddy") {
        ServiceCategory::WebServer
    } else if name.contains("postgresql") || name.contains("redis") || name.contains("mariadb") {
        ServiceCategory::Database
    } else if name.contains("sshd") || name.contains("ufw") || name.contains("fail2ban") {
        ServiceCategory::Security
    } else if name.contains("traefik") || name.contains("bind") || name.contains("dnsmasq") {
        ServiceCategory::Infrastructure
    } else {
        ServiceCategory::Unknown
    }
}
