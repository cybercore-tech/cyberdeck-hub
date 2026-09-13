//! Plain string-built HTML — no template engine dependency, matching the
//! minimal-deps approach used across the rest of this ecosystem (same
//! convention as Dockspace's `views.rs`).

use crate::hub::{Checked, Health, Probe};

pub fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

pub fn layout(active_theme: &str, theme_options: &str, nav_active: &str, body: &str) -> String {
    let nav = |href: &str, label: &str| {
        let active = if nav_active == label { " active" } else { "" };
        format!(r#"<a href="{href}" class="btn-ghost{active}">{label}</a>"#)
    };
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>CYBERDECK</title>
<link rel="stylesheet" href="/vendor/tokens.css">
<style id="theme-vars"></style>
<link rel="stylesheet" href="/static/ui.css">
<script src="/static/htmx.min.js" defer></script>
</head>
<body>
<div class="bg-scanlines" aria-hidden="true"></div>
<div class="bg-glow" aria-hidden="true"></div>
<header class="topbar">
  <span class="brand">CYBERDECK<span class="brand-accent">//</span></span>
  <div class="topbar-right">
    <nav class="topbar-nav">
      {nav_hub}
      {nav_scans}
      {nav_reports}
    </nav>
    <select id="theme-picker" onchange="applyTheme(this.value)">
      {theme_options}
    </select>
  </div>
</header>
<main>
{body}
</main>
<script>
function applyTheme(name) {{
  fetch('/api/cybergrid/css/' + name)
    .then(r => r.text())
    .then(css => {{ document.getElementById('theme-vars').textContent = css; }});
  localStorage.setItem('cyberdeck-theme', name);
}}
window.addEventListener('DOMContentLoaded', () => {{
  const saved = localStorage.getItem('cyberdeck-theme') || '{active_theme}';
  document.getElementById('theme-picker').value = saved;
  applyTheme(saved);
}});
</script>
</body>
</html>"#,
        nav_hub = nav("/", "Hub"),
        nav_scans = nav("/scans", "Scans"),
        nav_reports = nav("/reports", "Reports"),
    )
}

fn status_class(h: Health) -> &'static str {
    match h {
        Health::Up => "status-running",
        Health::Down => "status-stopped",
        Health::Unknown => "status-unknown",
    }
}

fn status_label(probe: &Probe, h: Health) -> &'static str {
    match (probe, h) {
        (Probe::Port(_), Health::Up) => "ONLINE",
        (Probe::Port(_), Health::Down) => "OFFLINE",
        (Probe::SystemUnit(_) | Probe::UserUnit(_), Health::Up) => "ACTIVE",
        (Probe::SystemUnit(_) | Probe::UserUnit(_), Health::Down) => "INACTIVE",
        (Probe::Binary(_), Health::Up) => "INSTALLED",
        (Probe::Binary(_), Health::Down) => "MISSING",
        (_, Health::Unknown) => "UNKNOWN",
    }
}

fn tool_card(c: &Checked) -> String {
    let t = c.tool;
    let cls = status_class(c.health);
    let label = status_label(&t.probe, c.health);

    let mut actions = String::new();
    if let Some(url) = t.open_url {
        actions.push_str(&format!(
            r#"<a class="btn-link" href="{url}" target="_blank" rel="noopener">Open</a>"#
        ));
    }
    if let Some(path) = t.wiki_path {
        actions.push_str(&format!(
            r#"<a class="btn-link" href="http://127.0.0.1:3000/{path}" target="_blank" rel="noopener">Wiki</a>"#
        ));
    }
    if actions.is_empty() {
        actions.push_str(r#"<span class="muted" style="font-size:0.75rem;">no wiki page yet</span>"#);
    }

    format!(
        r#"<div class="card">
  <div class="card-head">
    <span class="card-title">{name}</span>
    <span class="badge {cls}">{label}</span>
  </div>
  <p class="card-desc">{desc}</p>
  <p class="card-meta"><code>{path}</code></p>
  <div class="card-actions">{actions}</div>
</div>"#,
        name = t.name,
        desc = escape(t.description),
        path = t.repo_path,
    )
}

/// The category sections alone — used both for the initial page render and
/// as the htmx partial swapped into `#hub-grid` every 10s.
pub fn hub_sections(checked: &[Checked]) -> String {
    // Preserve TOOLS' declared order for category-of-first-appearance,
    // rather than alphabetizing — the registry is already grouped
    // logically (hub apps, then daemons, network, security, ...).
    let mut categories: Vec<&str> = Vec::new();
    for c in checked {
        if !categories.contains(&c.tool.category) {
            categories.push(c.tool.category);
        }
    }

    categories
        .iter()
        .map(|cat| {
            let cards: String = checked
                .iter()
                .filter(|c| &c.tool.category == cat)
                .map(tool_card)
                .collect();
            let count = checked.iter().filter(|c| &c.tool.category == cat).count();
            format!(
                r#"<section class="cat-section">
  <div class="cat-head"><h2>{cat}</h2><span class="cat-count">{count}</span></div>
  <div class="grid">{cards}</div>
</section>"#
            )
        })
        .collect()
}

pub fn dashboard(checked: &[Checked]) -> String {
    let up = checked.iter().filter(|c| c.health == Health::Up).count();
    let total = checked.len();
    let sections = hub_sections(checked);

    format!(
        r#"<div class="dash-head">
  <h1>Tool Fleet</h1>
  <p class="muted">{up} / {total} reporting up — auto-refreshes every 10s.</p>
</div>
<div id="hub-grid" hx-get="/partial/hub" hx-trigger="every 10s" hx-swap="innerHTML">
{sections}
</div>"#
    )
}

/// (dispatcher command, card title, one-line description, id for DOM/JS,
/// output filename under `diagnostics/`).
pub const SCAN_MODULES: &[(&str, &str, &str, &str, &str)] = &[
    ("RunCpuModule", "CPU", "Model, core count, frequency, load.", "cpu", "cpu/cpu.md"),
    ("RunHardwareModule", "Hardware Core", "Full hardware inventory sweep.", "hw", "hardware.md"),
    ("RunPowerModule", "Power Draw", "AC/battery state, wattage via upower.", "pwr", "power.md"),
    ("RunMemoryModule", "Memory Banks", "RAM total/used, per-module DIMM info.", "mem", "memory.md"),
    ("RunStorageAiModule", "Storage AI", "Heuristic storage health pass.", "stg", "storage_ai.md"),
    ("RunThermalModule", "Thermal Array", "Sensor temps, raw + parsed.", "thm", "thermal_ai.md"),
    ("RunNetworkModule", "Network Nodes", "Interfaces, addresses, link state.", "net", "network.md"),
    ("RunFanModule", "Fan Velocity", "Fan RPM / PWM readouts.", "fan", "fan.md"),
    ("RunAudioModule", "Audio Sinks", "PipeWire graph + sink/source summary.", "aud", "audio.md"),
    ("RunBiosModule", "BIOS Layers", "Firmware, board, CPU, chipset, kernel, power sub-reports.", "bios", "bios.md"),
    ("RunDisksModule", "Disk Topology", "Block devices, sizes, per-device detail.", "disk", "disks.md"),
    ("RunBatteryModule", "Battery Health", "Capacity, cycle count, condition.", "bat", "battery/battery.md"),
    ("RunServicesModule", "System Daemons", "systemd unit inventory.", "svc", "services.md"),
    ("RunMotherboardModule", "Mainboard Bus", "Board model, vendor, revision.", "mobo", "motherboard/motherboard.md"),
];

fn scan_card(cmd: &str, title: &str, desc: &str, id: &str, filename: &str, exists: bool) -> String {
    let view = if exists {
        format!(r#"<a class="btn-link" href="/reports/{filename}">View Output</a>"#)
    } else {
        r#"<span class="muted" style="font-size:0.75rem;">not run yet</span>"#.to_string()
    };
    format!(
        r#"<div class="card">
  <div class="card-head">
    <span class="card-title">{title}</span>
    <span class="badge {badge_cls}" id="badge-{id}">{badge_label}</span>
  </div>
  <p class="card-desc">{desc}</p>
  <p class="card-meta"><code>diagnostics/{filename}</code></p>
  <div class="card-actions">
    <button id="btn-{id}" onclick="runScan('{cmd}', '{id}', '{filename}')">Run</button>
    <span id="view-{id}">{view}</span>
  </div>
</div>"#,
        badge_cls = if exists { "status-running" } else { "status-stopped" },
        badge_label = if exists { "HAS OUTPUT" } else { "NO OUTPUT" },
    )
}

pub fn scans_page() -> String {
    let cards: String = SCAN_MODULES
        .iter()
        .map(|(cmd, title, desc, id, filename)| {
            let exists = std::path::Path::new("diagnostics").join(filename).is_file();
            scan_card(cmd, title, desc, id, filename, exists)
        })
        .collect();

    format!(
        r#"<div class="dash-head">
  <h1>Diagnostic Scans</h1>
  <p class="muted">Each scan shells out, prints live output server-side, and writes a markdown report to <code>diagnostics/</code> — see <a class="btn-ghost" style="padding:0.1rem 0.4rem;" href="/reports">Reports</a>.</p>
</div>
<div class="grid">{cards}</div>

<section class="cat-section">
  <div class="cat-head"><h2>Files</h2></div>
  <div class="window">
    <div class="titlebar"><span class="dot dot-a"></span><span class="dot dot-b"></span><span class="dot dot-c"></span><span class="titlebar-text">diagnostics/ file viewer</span></div>
    <div class="window-body">
      <div class="card-actions" style="margin-bottom:0.75rem;">
        <button onclick="loadFileList()">Refresh File List</button>
      </div>
      <table class="res-table"><tbody id="file-list-body">
        <tr><td><div class="empty-state">click "Refresh File List" to scan diagnostics/</div></td></tr>
      </tbody></table>
    </div>
  </div>
</section>

<section class="cat-section">
  <div class="cat-head"><h2>Archive</h2></div>
  <div class="window">
    <div class="titlebar"><span class="dot dot-a"></span><span class="dot dot-b"></span><span class="dot dot-c"></span><span class="titlebar-text">compress diagnostics/ output</span></div>
    <div class="window-body">
      <label class="field-label" for="archive-format">Format</label>
      <select id="archive-format" class="text-input" style="max-width:220px;margin-bottom:0.75rem;">
        <option value="zip">.ZIP (standard)</option>
        <option value="tar">.TAR (uncompressed)</option>
        <option value="gzip">.TAR.GZ (compressed)</option>
        <option value="7z">.7Z (high compression)</option>
      </select>
      <div class="card-actions">
        <button id="btn-archive" onclick="runArchive()">Compress</button>
      </div>
      <p id="archive-result" class="muted" style="margin-top:0.6rem;font-size:0.8rem;"></p>
    </div>
  </div>
</section>

<script>
function runScan(cmd, id, filename) {{
  const btn = document.getElementById('btn-' + id);
  const original = btn.innerText;
  btn.innerText = 'running…'; btn.disabled = true;
  fetch('/cyberdeck/api/command', {{
    method: 'POST',
    headers: {{'Content-Type': 'application/json'}},
    body: JSON.stringify({{ [cmd]: './diagnostics' }})
  }}).then(r => {{
    if (!r.ok) throw new Error('scan failed');
    return r.json();
  }}).then(() => {{
    btn.innerText = 'done ✓';
    document.getElementById('badge-' + id).textContent = 'HAS OUTPUT';
    document.getElementById('badge-' + id).className = 'badge status-running';
    document.getElementById('view-' + id).innerHTML =
      '<a class="btn-link" href="/reports/' + filename + '">View Output</a>';
    setTimeout(() => {{ btn.innerText = original; btn.disabled = false; }}, 1200);
  }}).catch(() => {{
    btn.innerText = 'error'; btn.disabled = false;
  }});
}}

function loadFileList() {{
  const body = document.getElementById('file-list-body');
  body.innerHTML = '<tr><td>loading…</td></tr>';
  fetch('/cyberdeck/api/list').then(r => r.json()).then(files => {{
    if (!files.length) {{ body.innerHTML = '<tr><td><div class="empty-state">diagnostics/ is empty</div></td></tr>'; return; }}
    body.innerHTML = files.map(f =>
      '<tr><td><code>' + f + '</code></td><td>' +
      (f.endsWith('.md') ? '<a class="btn-link" href="/reports/' + f + '">View</a>' : '<span class="muted">binary</span>') +
      '</td></tr>'
    ).join('');
  }}).catch(() => {{ body.innerHTML = '<tr><td>failed to list files</td></tr>'; }});
}}

function runArchive() {{
  const fmt = document.getElementById('archive-format').value;
  const btn = document.getElementById('btn-archive');
  const result = document.getElementById('archive-result');
  btn.innerText = 'compressing…'; btn.disabled = true;
  fetch('/cyberdeck/api/command', {{
    method: 'POST',
    headers: {{'Content-Type': 'application/json'}},
    body: JSON.stringify({{ 'ArchiveFiles': fmt }})
  }}).then(r => r.json()).then(msg => {{
    result.textContent = msg;
    btn.innerText = 'Compress'; btn.disabled = false;
  }}).catch(() => {{
    result.textContent = 'archive failed';
    btn.innerText = 'Compress'; btn.disabled = false;
  }});
}}
</script>"#
    )
}

pub fn reports_page(entries: &[(String, u64, String)]) -> String {
    let rows: String = if entries.is_empty() {
        r#"<tr><td colspan="3"><div class="empty-state">No reports yet — run a scan first.</div></td></tr>"#
            .to_string()
    } else {
        entries
            .iter()
            .map(|(name, size, modified)| {
                format!(
                    r#"<tr>
  <td><a class="btn-link" href="/reports/{name}">{name}</a></td>
  <td>{size} bytes</td>
  <td class="muted">{modified}</td>
</tr>"#
                )
            })
            .collect()
    };

    format!(
        r#"<div class="dash-head">
  <h1>Reports</h1>
  <p class="muted">Markdown output from every scan, written to <code>diagnostics/</code>. For
  full rendering (wikilinks, search, nicer typography) see the dedicated
  <a class="btn-ghost" style="padding:0.1rem 0.4rem;" href="http://127.0.0.1:8766" target="_blank" rel="noopener">Diagnostic Reports deck</a> — this list is the raw quick view.</p>
</div>
<table class="res-table">
  <thead><tr><th>File</th><th>Size</th><th>Modified</th></tr></thead>
  <tbody>{rows}</tbody>
</table>"#
    )
}

pub fn report_view(name: &str, content: &str) -> String {
    format!(
        r#"<div class="dash-head">
  <h1>{name}</h1>
  <p><a class="btn-ghost" href="/reports">&larr; back to reports</a></p>
</div>
<div class="log-view">{}</div>"#,
        escape(content)
    )
}
