//! # CYBERDECK Types (src/types.rs)
//!
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum CyberdeckModule {
    HardwareAudit,
    NetworkDiagnostics,
    StorageAnalysis,
    AudioDiagnostics,
    BiosDiagnostics,
    DisksDiagnostics,
    FanDiagnostics,
    MemoryDiagnostics,
    PowerDiagnostics,
    StorageAiDiagnostics,
    ThermalDiagnostics,
    FullScan,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum CyberdeckCommand {
    InitDisplay,
    SetStealth(bool),
    GenerateReport,

    // Ensure all these have (String) if you are passing data
    RunAudioModule(String),
    RunBatteryModule(String),
    RunBiosModule(String),
    RunCpuModule(String),
    RunDashboardModule(String),
    RunDisksModule(String),
    RunFanModule(String),
    RunHardwareModule(String),
    RunMemoryModule(String),
    RunMotherboardModule(String),
    RunNetworkModule(String),
    RunPowerModule(String),
    RunServicesModule(String),
    RunStorageAiModule(String),
    RunThermalModule(String),

    ArchiveFiles(String),

    Unknown(String),
}

#[derive(Debug, Serialize, Clone)]
pub struct CyberdeckState {
    pub display_active: bool,
    pub active_modules: Vec<CyberdeckModule>,
    pub stealth_mode: bool,
    pub reports_generated: u32,
    pub execution_log: Vec<String>,
}

pub type SharedCyberdeckState = Arc<Mutex<CyberdeckState>>;
