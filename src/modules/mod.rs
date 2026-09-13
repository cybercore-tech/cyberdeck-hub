//! # CYBERDECK: Orchestrator
//!
//! The gateway for the diagnostic suite.
//! Aggregates results from individual modules to generate
//! the master system intelligence report.
//!
//! Modules Included: 11 + 2 AI [13]
//! More modules will be implemented in future releases.
//!
//! ## Implementation Notes
//! - **Registry**: Acts as the single interface for importing all diagnostic sub-systems.
//! - **Execution**: Provides the `execute_all()` function to loop through registered modules.

//-NOTE: Orchestrator (/src/modules/mod.rs)
//- Modules: [11] (AI Included) + [2]
//- Keep all notes current for module development purposes.
//- Tag reference in build.rs
//- New "tags" can be added at any time to [build.rs]. Document all new tags.
//-END

// Declare the utility first
pub mod utils;

// Declare all modules so the Dispatcher can find them
// Main Modules: [11] (AI Included) + [2]
pub mod audio;
pub mod battery;
pub mod bios;
pub mod cpu;
pub mod disks;
pub mod fan;
pub mod hardware;
pub mod memory;
pub mod motherboard;
pub mod network;
pub mod power;

// AI Services / Modules
pub mod storage_ai;
pub mod thermal_ai;

// System Dashboard
pub mod dashboard;

// Services
pub mod services;
