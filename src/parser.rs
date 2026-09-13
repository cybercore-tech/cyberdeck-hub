//! # CYBERDECK Parser
//!
//! The command translation engine for the CYBERDECK system.
//! This module converts raw input strings from the UI or environment
//! into structured `CyberdeckCommand` variants for the dispatcher.
//!
#![warn(missing_docs)]

use crate::types::CyberdeckCommand;

/// Parses a raw command string into a `CyberdeckCommand` variant.
///
/// This function acts as the bridge between raw user input (from the UI)
/// and the internal system execution logic.
pub fn parse_cyberdeck_script(script: &str) -> Vec<CyberdeckCommand> {
    let mut commands = Vec::new();

    // Split input by newlines to handle multi-line scripts
    for line in script.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        let cmd = match parts[0] {
            "init_display" => CyberdeckCommand::InitDisplay,
            "stealth_on" => CyberdeckCommand::SetStealth(true),
            "stealth_off" => CyberdeckCommand::SetStealth(false),
            "generate_report" => CyberdeckCommand::GenerateReport,

            // --- Diagnostics ---
            "battery" => {
                CyberdeckCommand::RunBatteryModule(parts.get(1).unwrap_or(&"default").to_string())
            }
            "cpu" => CyberdeckCommand::RunCpuModule(parts.get(1).unwrap_or(&"default").to_string()),
            "memory" => {
                CyberdeckCommand::RunMemoryModule(parts.get(1).unwrap_or(&"default").to_string())
            }
            "motherboard" => CyberdeckCommand::RunMotherboardModule(
                parts.get(1).unwrap_or(&"default").to_string(),
            ),
            "power" => {
                CyberdeckCommand::RunPowerModule(parts.get(1).unwrap_or(&"default").to_string())
            }

            // --- Modules ---
            "audio" => {
                CyberdeckCommand::RunAudioModule(parts.get(1).unwrap_or(&"default").to_string())
            }
            "bios" => {
                CyberdeckCommand::RunBiosModule(parts.get(1).unwrap_or(&"default").to_string())
            }
            "disks" => {
                CyberdeckCommand::RunDisksModule(parts.get(1).unwrap_or(&"default").to_string())
            }
            "fan" => CyberdeckCommand::RunFanModule(parts.get(1).unwrap_or(&"default").to_string()),
            "hardware" => {
                CyberdeckCommand::RunHardwareModule(parts.get(1).unwrap_or(&"default").to_string())
            }
            "network" => {
                CyberdeckCommand::RunNetworkModule(parts.get(1).unwrap_or(&"default").to_string())
            }
            "storage_ai" => {
                CyberdeckCommand::RunStorageAiModule(parts.get(1).unwrap_or(&"default").to_string())
            }
            "thermal_ai" => {
                CyberdeckCommand::RunThermalModule(parts.get(1).unwrap_or(&"default").to_string())
            }

            unknown => CyberdeckCommand::Unknown(unknown.to_string()),
        };
        commands.push(cmd);
    }

    commands
}
