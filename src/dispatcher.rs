//! # CYBERDECK Dispatcher (src/dispatcher.rs)
//!
//! The command execution engine for the CYBERDECK system.
//! This module handles the interpretation and routing of system commands,
//! managing state updates and asynchronous diagnostic/module execution.
//!
#![warn(missing_docs)]

use crate::types::{SharedCyberdeckState, CyberdeckCommand};
use std::time::{SystemTime, UNIX_EPOCH};
use std::process::Command; // Added for shell execution

use crate::modules::{
    audio, battery, bios, cpu, dashboard, disks, fan, hardware,
    memory, motherboard, network, power, services, storage_ai, thermal_ai
};

//-NOTE: Dispatcher [Execution Engine] (src/dispatcher.rs)
//- Use these new "tags" for code blocks and notes.
//- Tag reference in build.rs
//- Files are saved to /snippets/{code, notes} in markdown (.md) format.
//-END

pub async fn execute_cyberdeck_command(cmd: CyberdeckCommand, state: &SharedCyberdeckState) {
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

    match cmd {
        CyberdeckCommand::InitDisplay => {
            //-CODE: Init_Display
            let mut s = state.lock().await;
            s.display_active = true;
            s.execution_log.push(format!("[{}] Display system activated.", timestamp));
            //-END
        }

        CyberdeckCommand::SetStealth(mode) => {
            //-CODE: Set_Stealth_Mode
            let mut s = state.lock().await;
            s.stealth_mode = mode;
            s.execution_log.push(format!("[{}] Stealth configuration set to: {}", timestamp, mode));
            //-END
        }
        // Generate report
        CyberdeckCommand::GenerateReport => {
            //-CODE: Generate_Report_Logic
            let mut s = state.lock().await;
            s.reports_generated += 1;
            // Extract the value into a local variable first (this is an immutable copy)
            let current_report_num = s.reports_generated;
            // Now pass that local variable to the formatter.
            // s is now free to be borrowed mutably by log_event.

            s.execution_log.push(format!("[{}] System Report #{} generated.", timestamp, current_report_num));
            //-END
        }

        //-NEW: NEW ARCHIVE LOGIC
        //- Recently added features [compression]
        //- Options: zip, gzip, tar, 7zip
        //- Output Folder: /diagnostics (change dir in Execute system command below)
        //-END

        //-NOTE: Archive files in various formats; zip, tar, gzip, 7z.
        //- Can now save diagnostic the output to a compressed folder
        //-END
        CyberdeckCommand::ArchiveFiles(format) => {
            //-CODE: Archive_Files
            let fmt = format.to_lowercase();

            //-NOTE: Ensure output directory exists.
            if let Err(e) = std::fs::create_dir_all("output") {
                //-END
                tracing::error!("Failed to create output directory: {}", e);
                return;
            }
            //-END

            //-CODE: Generate_date-stamped_filename
            let now = chrono::Local::now();
            let date_str = now.format("%Y-%m-%d").to_string();

            let file_ext = if fmt == "gzip" { "tar.gz" } else { &fmt };
            let filename = format!("cyberdeck_archive_{}.{}", date_str, file_ext);
            let output_path = format!("output/{}", filename);

            let mut s = state.lock().await;
            tracing::info!("Archiving to: {}", output_path);
            //-END

            //-NOTE: Execute system command.
            let result = match fmt.as_str() {
                "zip" => Command::new("zip").args(["-r", &output_path, "./diagnostics"]).status(),
                "tar" => Command::new("tar").args(["-cvf", &output_path, "./diagnostics"]).status(),
                "gzip" => Command::new("tar").args(["-czvf", &output_path, "./diagnostics"]).status(),
                "7z" => Command::new("7z").args(["a", &output_path, "./diagnostics"]).status(),
                _ => {
                    tracing::error!("Unsupported format requested: {}", fmt);
                    s.execution_log.push(format!("[{}] Error: Unsupported format '{}'", timestamp, fmt));
                    return;
                }
            };
            //-END

            //-NOTE: Log results
            match result {
                Ok(status) if status.success() => {
                    tracing::info!("Archive successful: {}", filename);
                    s.execution_log.push(format!("[{}] Archive created: {}", timestamp, filename));
                }
                _ => {
                    tracing::error!("Compression command failed.");
                    s.execution_log.push(format!("[{}] Critical: Compression failed.", timestamp));
                }
            }
            //-END
        }

        CyberdeckCommand::RunAudioModule(p) => run_module(state, &p, |params| {
            let params = params.to_string(); let state_arc = state.clone();
            async move { audio::execute(&*state_arc.lock().await, &params).await.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e)) }
        }).await,

        CyberdeckCommand::RunBatteryModule(p) => run_module(state, &p, |params| {
            let params = params.to_string(); let state_arc = state.clone();
            async move { battery::execute(&*state_arc.lock().await, &params).await.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e)) }
        }).await,

        CyberdeckCommand::RunBiosModule(p) => run_module(state, &p, |params| {
            let params = params.to_string(); let state_arc = state.clone();
            async move { bios::execute(&*state_arc.lock().await, &params).await.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e)) }
        }).await,

        CyberdeckCommand::RunCpuModule(p) => run_module(state, &p, |params| {
            let params = params.to_string(); let state_arc = state.clone();
            async move { cpu::execute(&*state_arc.lock().await, &params).await.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e)) }
        }).await,

        CyberdeckCommand::RunDashboardModule(p) => run_module(state, &p, |params| {
            let params = params.to_string(); let state_arc = state.clone();
            async move { dashboard::execute(&*state_arc.lock().await, &params).await.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e)) }
        }).await,

        CyberdeckCommand::RunDisksModule(p) => run_module(state, &p, |params| {
            let params = params.to_string(); let state_arc = state.clone();
            async move { disks::execute(&*state_arc.lock().await, &params).await.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e)) }
        }).await,

        CyberdeckCommand::RunFanModule(p) => run_module(state, &p, |params| {
            let params = params.to_string(); let state_arc = state.clone();
            async move { fan::execute(&*state_arc.lock().await, &params).await.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e)) }
        }).await,

        CyberdeckCommand::RunHardwareModule(p) => run_module(state, &p, |params| {
            let params = params.to_string(); let state_arc = state.clone();
            async move { hardware::execute(&*state_arc.lock().await, &params).await.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e)) }
        }).await,

        CyberdeckCommand::RunMemoryModule(p) => run_module(state, &p, |params| {
            let params = params.to_string(); let state_arc = state.clone();
            async move { memory::execute(&*state_arc.lock().await, &params).await.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e)) }
        }).await,

        CyberdeckCommand::RunMotherboardModule(p) => run_module(state, &p, |params| {
            let params = params.to_string(); let state_arc = state.clone();
            async move { motherboard::execute(&*state_arc.lock().await, &params).await.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e)) }
        }).await,

        CyberdeckCommand::RunNetworkModule(p) => run_module(state, &p, |params| {
            let params = params.to_string(); let state_arc = state.clone();
            async move { network::execute(&*state_arc.lock().await, &params).await.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e)) }
        }).await,

        CyberdeckCommand::RunPowerModule(p) => run_module(state, &p, |params| {
            let params = params.to_string(); let state_arc = state.clone();
            async move { power::execute(&*state_arc.lock().await, &params).await.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e)) }
        }).await,

        CyberdeckCommand::RunServicesModule(p) => run_module(state, &p, |params| {
            let params = params.to_string(); let state_arc = state.clone();
            async move { services::execute(&*state_arc.lock().await, &params).await.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e)) }
        }).await,

        CyberdeckCommand::RunStorageAiModule(p) => run_module(state, &p, |params| {
            let params = params.to_string(); let state_arc = state.clone();
            async move { storage_ai::execute(&*state_arc.lock().await, &params).await.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e)) }
        }).await,

        CyberdeckCommand::RunThermalModule(p) => run_module(state, &p, |params| {
            let params = params.to_string(); let state_arc = state.clone();
            async move { thermal_ai::execute(&*state_arc.lock().await, &params).await.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e)) }
        }).await,

        // Print warning if not implemented
        _ => println!("Command not implemented"),
    }
}

async fn run_module<F, Fut>(state: &SharedCyberdeckState, p: &str, exec_fn: F)
where F: Fn(&str) -> Fut, Fut: std::future::Future<Output = std::io::Result<String>> {
    match exec_fn(p).await {
        Ok(res) => state.lock().await.execution_log.push(res),
        Err(e) => state.lock().await.execution_log.push(format!("Error: {}", e)),
    }
}
