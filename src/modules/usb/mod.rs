//! # CYBERDECK: USB Intelligence
//!
//! Every USB device currently enumerated (`lsusb`), parsed into a table —
//! bus/device address, vendor:product ID, and description. `hardware.md`
//! already embeds a raw `lsusb` dump as part of its bus-mapping section;
//! this is the dedicated, focused version for when only USB matters.

use crate::types::CyberdeckState;
use std::fs;
use std::process::Command;

struct UsbDevice {
    bus: String,
    device: String,
    id: String,
    description: String,
}

fn parse_lsusb(out: &str) -> Vec<UsbDevice> {
    // "Bus 001 Device 002: ID 046d:c52f Logitech, Inc. Nano Receiver"
    out.lines()
        .filter_map(|line| {
            let rest = line.strip_prefix("Bus ")?;
            let (bus, rest) = rest.split_once(' ')?;
            let rest = rest.strip_prefix("Device ")?;
            let (device, rest) = rest.split_once(':')?;
            let rest = rest.trim().strip_prefix("ID ")?;
            let (id, description) = rest.split_once(' ')?;
            Some(UsbDevice {
                bus: bus.to_string(),
                device: device.to_string(),
                id: id.to_string(),
                description: description.trim().to_string(),
            })
        })
        .collect()
}

pub async fn execute(_state: &CyberdeckState, params: &str) -> Result<String, String> {
    let dir = params;
    let base_f = format!("{}/usb.md", dir);
    let timestamp = crate::modules::utils::now_human();

    let raw = Command::new("lsusb")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();
    let devices = parse_lsusb(&raw);

    let mut report = format!("# USB Devices\n\n_Generated {timestamp}_\n\n");
    if devices.is_empty() {
        report.push_str("`lsusb` returned nothing — not installed, or no devices enumerated.\n");
    } else {
        report.push_str(&format!("{} device(s) enumerated.\n\n", devices.len()));
        report.push_str("| Bus | Device | Vendor:Product | Description |\n|---|---|---|---|\n");
        for d in &devices {
            report.push_str(&format!("| {} | {} | `{}` | {} |\n", d.bus, d.device, d.id, d.description));
        }
    }

    fs::write(&base_f, report).map_err(|e| e.to_string())?;
    Ok(base_f)
}
