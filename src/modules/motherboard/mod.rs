//! # CYBERDECK: Motherboard Intelligence
//!
//! Board/BIOS identity via `/sys/class/dmi/id/*`.
//!
//! ## Implementation Notes
//! - **No sudo**: the original version shelled out to `sudo dmidecode`,
//!   which silently produced an empty report — this process (a systemd
//!   service with no TTY/cached credential) can never actually satisfy an
//!   interactive sudo prompt, so the command always failed quietly.
//!   `/sys/class/dmi/id/*` exposes the same vendor/model/BIOS fields
//!   without root; only the serial numbers and product UUID are
//!   root-restricted (rightly so — those are real identifying secrets),
//!   and this reports them as "restricted (root only)" rather than
//!   silently blank.

use crate::modules::utils::dmi_field as dmi;
use crate::types::CyberdeckState;
use std::fs;

fn chassis_type_label(code: &str) -> &'static str {
    match code.trim() {
        "3" => "Desktop",
        "4" => "Low Profile Desktop",
        "6" => "Mini Tower",
        "7" => "Tower",
        "8" => "Portable",
        "9" => "Laptop",
        "10" => "Notebook",
        "13" => "All in One",
        "14" => "Sub Notebook",
        "30" => "Tablet",
        "31" => "Convertible",
        "32" => "Detachable",
        _ => "Unknown",
    }
}

/// Executes motherboard and BIOS diagnostic suite.
pub async fn execute(_state: &CyberdeckState, params: &str) -> Result<String, String> {
    let dir = params;
    let board_dir = format!("{}/motherboard", dir);
    fs::create_dir_all(&board_dir).map_err(|e| e.to_string())?;
    let base_f = format!("{}/motherboard.md", board_dir);

    let timestamp = crate::modules::utils::now_human();
    let chassis_type = dmi("chassis_type");

    let report = format!(
        "# Motherboard & BIOS\n\n\
        _Generated {timestamp}_\n\n\
        ## Quick Identity\n\n\
        | Field | Value |\n|---|---|\n\
        | System vendor | {sys_vendor} |\n\
        | Product name | {product_name} |\n\
        | Chassis type | {chassis_label} (code {chassis_type}) |\n\
        | Chassis vendor | {chassis_vendor} |\n\n\
        ## Baseboard\n\n\
        | Field | Value |\n|---|---|\n\
        | Vendor | {board_vendor} |\n\
        | Model | {board_name} |\n\
        | Version | {board_version} |\n\
        | Serial | {board_serial} |\n\n\
        ## BIOS\n\n\
        | Field | Value |\n|---|---|\n\
        | Vendor | {bios_vendor} |\n\
        | Version | {bios_version} |\n\
        | Release | {bios_release} |\n\
        | Date | {bios_date} |\n\n\
        ## Restricted fields\n\n\
        Product UUID and every serial number are root-only on this system \
        (`/sys/class/dmi/id/*_serial`, `product_uuid`) — genuinely \
        restricted, not a bug in this scan.\n",
        sys_vendor = dmi("sys_vendor"),
        product_name = dmi("product_name"),
        chassis_label = chassis_type_label(&chassis_type),
        chassis_vendor = dmi("chassis_vendor"),
        board_vendor = dmi("board_vendor"),
        board_name = dmi("board_name"),
        board_version = dmi("board_version"),
        board_serial = dmi("board_serial"),
        bios_vendor = dmi("bios_vendor"),
        bios_version = dmi("bios_version"),
        bios_release = dmi("bios_release"),
        bios_date = dmi("bios_date"),
    );

    fs::write(&base_f, report).map_err(|e| e.to_string())?;
    Ok(base_f)
}
