//! The tool registry — every app built this cycle, one entry each, with a
//! cheap live health probe (TCP port / systemd unit / binary presence) and
//! the "quick options" a card offers (open, wiki page, repo path). This is
//! what makes CYBERDECK the central hub rather than just a diagnostics UI.

use std::path::Path;
use std::time::Duration;
use tokio::process::Command;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Health {
    Up,
    Down,
    Unknown,
}

#[derive(Clone, Copy)]
pub enum Probe {
    /// A loopback web service — TCP-connect to 127.0.0.1:PORT.
    Port(u16),
    /// `systemctl is-active <unit>` (system scope).
    SystemUnit(&'static str),
    /// `systemctl --user is-active <unit>`.
    UserUnit(&'static str),
    /// A CLI-only tool: just confirm the release binary was actually built.
    Binary(&'static str),
}

pub struct Tool {
    pub name: &'static str,
    pub category: &'static str,
    pub description: &'static str,
    pub repo_path: &'static str,
    pub probe: Probe,
    /// Set for web services — the card's primary "Open" action.
    pub open_url: Option<&'static str>,
    /// Path under the Cybercore Wiki (served at :3000) for this tool's page,
    /// e.g. `daemons/ghostport.html`. `None` for the 6 not-yet-written pages.
    pub wiki_path: Option<&'static str>,
}

pub const TOOLS: &[Tool] = &[
    // ---- Web hub / desktop apps -------------------------------------------------
    Tool {
        name: "Dockspace",
        category: "Hub & Docker",
        description: "Compose stack fleet manager — start/stop, logs, resource pruning.",
        repo_path: "~/.sysops/dockspace",
        probe: Probe::Port(7070),
        open_url: Some("http://127.0.0.1:7070"),
        wiki_path: None,
    },
    Tool {
        name: "SubgridSec Deck",
        category: "Hub & Docker",
        description: "Back-office web app — roadmap tracking, theme picker.",
        repo_path: "~/.sysops/subgridsec-deck",
        probe: Probe::Port(8880),
        open_url: Some("http://127.0.0.1:8880"),
        wiki_path: None,
    },
    Tool {
        name: "Cyberdesk",
        category: "Hub & Docker",
        description: "Darknotes vault portal/editor — markdown notes, wikilinks, snippets.",
        repo_path: "~/.sysops/cyberdesk",
        probe: Probe::Port(8765),
        open_url: Some("http://127.0.0.1:8765"),
        wiki_path: None,
    },
    Tool {
        name: "Darkbox SysTools",
        category: "Hub & Docker",
        description: "Live system control console — config editor, root shell, package updates. Privileged, token-gated.",
        repo_path: "~/.sysops/darkbox_systools",
        probe: Probe::Port(9200),
        open_url: Some("http://127.0.0.1:9200"),
        wiki_path: None,
    },
    Tool {
        name: "Cybercore Wiki",
        category: "Hub & Docker",
        description: "Reference docs for every authored app — mdBook, live-reloading.",
        repo_path: "~/Wiki",
        probe: Probe::Port(3000),
        open_url: Some("http://127.0.0.1:3000"),
        wiki_path: None,
    },
    Tool {
        name: "Diagnostic Reports",
        category: "Hub & Docker",
        description: "Cyberdesk instance rendering this hub's own scan output as a proper markdown deck.",
        repo_path: "~/.sysops/cyberdeck/diagnostics",
        probe: Probe::Port(8766),
        open_url: Some("http://127.0.0.1:8766"),
        wiki_path: None,
    },
    // ---- Daemons ------------------------------------------------------------
    Tool {
        name: "WraithFlow",
        category: "Daemons",
        description: "TCP proxy / traffic analyzer.",
        repo_path: "~/tools/daemons/wraithflow",
        probe: Probe::SystemUnit("wraithflow"),
        open_url: None,
        wiki_path: Some("daemons/wraithflow.html"),
    },
    Tool {
        name: "VortexWall",
        category: "Daemons",
        description: "Active-blackholing firewall (nftables).",
        repo_path: "~/tools/daemons/vortexwall",
        probe: Probe::SystemUnit("vortexwall"),
        open_url: None,
        wiki_path: Some("daemons/vortexwall.html"),
    },
    Tool {
        name: "GhostPort",
        category: "Daemons",
        description: "Encrypted NAT-traversing port forwarder (Noise KK).",
        repo_path: "~/tools/daemons/ghostport",
        probe: Probe::SystemUnit("ghostport"),
        open_url: None,
        wiki_path: Some("daemons/ghostport.html"),
    },
    Tool {
        name: "ApexDaemon",
        category: "Daemons",
        description: "Plugin-style background automation (theme-sync, fleet-health, security, vault-backup).",
        repo_path: "~/tools/daemons/apexdaemon",
        probe: Probe::UserUnit("apexdaemon"),
        open_url: None,
        wiki_path: None,
    },
    // ---- Network --------------------------------------------------------------
    Tool {
        name: "AetherScope",
        category: "Network Tools",
        description: "libpcap packet-capture CLI.",
        repo_path: "~/tools/network/aetherscope",
        probe: Probe::Binary("/home/raven/.cargo-target/release/aetherscope"),
        open_url: None,
        wiki_path: Some("network/aetherscope.html"),
    },
    Tool {
        name: "SentryGrid",
        category: "Network Tools",
        description: "Network exposure auditor — correlates ss/ufw/docker.",
        repo_path: "~/tools/network/sentrygrid",
        probe: Probe::Binary("/home/raven/.cargo-target/release/sentrygrid"),
        open_url: None,
        wiki_path: Some("network/sentrygrid.html"),
    },
    // ---- Security ---------------------------------------------------------
    Tool {
        name: "SigilWard",
        category: "Security Tools",
        description: "File-integrity monitor (AIDE/Tripwire category).",
        repo_path: "~/tools/security/sigilward",
        probe: Probe::Binary("/home/raven/.cargo-target/release/sigilward"),
        open_url: None,
        wiki_path: Some("security/sigilward.html"),
    },
    Tool {
        name: "Chronicle",
        category: "Security Tools",
        description: "Git activity digest, logged to darknotes.",
        repo_path: "~/tools/security/chronicle",
        probe: Probe::Binary("/home/raven/.cargo-target/release/chronicle"),
        open_url: None,
        wiki_path: Some("security/chronicle.html"),
    },
    Tool {
        name: "Undertow",
        category: "Security Tools",
        description: "Rootkit / compromise indicator scanner (detection-only).",
        repo_path: "~/tools/security/undertow",
        probe: Probe::Binary("/home/raven/.cargo-target/release/undertow"),
        open_url: None,
        wiki_path: Some("security/undertow.html"),
    },
    // ---- Crypto -------------------------------------------------------------
    Tool {
        name: "Keysmith",
        category: "Crypto Tools",
        description: "Password / passphrase / hash generator.",
        repo_path: "~/tools/crypto/keysmith",
        probe: Probe::Binary("/home/raven/.cargo-target/release/keysmith"),
        open_url: None,
        wiki_path: Some("crypto/keysmith.html"),
    },
    Tool {
        name: "CyberVault",
        category: "Crypto Tools",
        description: "Encrypted secrets vault — Argon2id + ChaCha20-Poly1305.",
        repo_path: "~/tools/crypto/cybervault",
        probe: Probe::Binary("/home/raven/.cargo-target/release/cybervault"),
        open_url: None,
        wiki_path: Some("crypto/cybervault.html"),
    },
    // ---- Pentest ------------------------------------------------------------
    Tool {
        name: "RoninSuite",
        category: "Pentest Toolkits",
        description: "Portable TUI pentest toolkit — scope-enforced, tiered CVSS reports.",
        repo_path: "~/Toolkits/RoninSuite",
        probe: Probe::Binary("/home/raven/.local/bin/ronin"),
        open_url: None,
        wiki_path: Some("pentest/roninsuite.html"),
    },
    // ---- Framework ----------------------------------------------------------
    Tool {
        name: "cybercore",
        category: "Shared Framework",
        description: "The CYBERGRID palette + design tokens every tool above shares.",
        repo_path: "~/.sysops/cybercore",
        probe: Probe::Binary("/home/raven/.sysops/cybercore/Cargo.toml"),
        open_url: None,
        wiki_path: Some("framework/cybercore.html"),
    },
];

pub async fn check(probe: &Probe) -> Health {
    match probe {
        Probe::Port(port) => {
            let addr = format!("127.0.0.1:{port}");
            match tokio::time::timeout(
                Duration::from_millis(300),
                tokio::net::TcpStream::connect(&addr),
            )
            .await
            {
                Ok(Ok(_)) => Health::Up,
                Ok(Err(_)) => Health::Down,
                Err(_) => Health::Unknown,
            }
        }
        Probe::SystemUnit(unit) => systemctl_is_active(&["is-active", unit]).await,
        Probe::UserUnit(unit) => systemctl_is_active(&["--user", "is-active", unit]).await,
        Probe::Binary(path) => {
            if Path::new(path).exists() {
                Health::Up
            } else {
                Health::Down
            }
        }
    }
}

async fn systemctl_is_active(args: &[&str]) -> Health {
    match tokio::time::timeout(
        Duration::from_millis(800),
        Command::new("systemctl").args(args).output(),
    )
    .await
    {
        Ok(Ok(out)) => {
            let text = String::from_utf8_lossy(&out.stdout);
            if text.trim() == "active" {
                Health::Up
            } else {
                Health::Down
            }
        }
        _ => Health::Unknown,
    }
}

pub struct Checked {
    pub tool: &'static Tool,
    pub health: Health,
}

/// Probes every tool concurrently — sequential `.await`s here would let a
/// slow systemctl call (or a port nobody's listening on) stall the whole
/// page load; a dozen tools is cheap in parallel and bounds latency to the
/// single slowest probe.
pub async fn check_all() -> Vec<Checked> {
    let futures = TOOLS.iter().map(|tool| async move {
        let health = check(&tool.probe).await;
        Checked { tool, health }
    });
    futures_util::future::join_all(futures).await
}
