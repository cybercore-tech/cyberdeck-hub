# Module: Memory Diagnostics

## Overview
The Memory Diagnostics module (`memory::execute`) provides real-time introspection into system RAM. It captures both physical hardware details and OS-level memory management statistics.

## Artifacts Generated
- `memory.md`: The primary dashboard report.
- `dimm/dimm_raw.md`: Physical DIMM information via `dmidecode`.
- `usage/usage.md`: Current RAM/Swap utilization status.
- `topology/topology.md`: CPU/Memory topology mapping.
- `raw/raw.md`: Comprehensive system memory dumps.

## External Dependencies
- `dmidecode`: Required for physical memory slot identification.
- `free`, `vmstat`, `swapon`: Standard Linux procps-ng utilities.
- `lspci`, `lsusb`: Used for peripheral bus mapping.

## API Signature
```rust
pub async fn execute(dir: &str) -> std::io::Result<String>
```
Arguments: dir (string) - The base directory to initialize the diagnostic tree.

Returns: Result containing the path to the primary memory.md file on success.

---
### Module: Battery & Power Diagnostics
- **Path**: `src/modules/battery/mod.rs`
- **Function**: `pub async fn execute(dir: &str) -> std::io::Result<String>`
- **Description**: Inspects hardware power controllers. Dynamically detects AC adapters and batteries (BATx).
- **Data Points**:
    - `capacity` (Percentage)
    - `power_now` (Current instantaneous power draw, critical for AMD/Intel/NVIDIA telemetry)
    - `energy_now` (Current energy state)
    - `voltage_now` (System voltage)
- **Dependencies**: Linux `sysfs` (Root privileges may be required to see some specific `hwmon` paths, but standard `power_supply` nodes are generally user-readable).

---
### Module: BIOS & System Intelligence
- **Path**: `src/modules/bios/mod.rs`
- **Function**: `pub async fn execute(dir: &str) -> std::io::Result<String>`
- **Description**: Performs high-level hardware enumeration. This module acts as the "Master Diagnostics" suite, probing deep kernel parameters, firmware tables, and active CPU power management state.
- **Artifacts Generated**:
    - `bios.md`: Main aggregate dashboard.
    - `cpu/cpu.md`: Processor architecture and scaling.
    - `motherboard/bios.md`: Firmware information.
    - `power/power.md`: Frequency scaling, governors, and driver state.
- **External Dependencies**:
    - `dmidecode`: Requires elevated privileges (`sudo` wrapper context).
    - `lscpu`, `lspci`, `free`: Standard Linux utility suite.
    - `cpupower`: Optional, for deep CPU frequency state introspection.
    
---
### Module: CPU Diagnostic
- **Path**: `src/modules/cpu/mod.rs`
- **Function**: `pub async fn execute(dir: &str) -> std::io::Result<String>`
- **Description**: High-fidelity CPU telemetry. Maps core topology, monitors package temperatures via `hwmon`, and tracks power state via `powercap` (RAPL).
- **Key Metrics**:
    - `scaling_governor`: Identify if CPU is in `powersave` vs `performance` mode.
    - `temp_input`: Hardware-level thermal probe data.
    - `energy_uj`: Real-time energy consumption (RAPL).
    
---
### Module: Disk Diagnostics
- **Path**: `src/modules/disks/mod.rs`
- **Function**: `pub async fn execute(dir: &str) -> std::io::Result<String>`
- **Description**: Probes physical block devices for health, geometry, and mount status. 
- **Advanced Features**:
    - **Btrfs Integration**: Automatically detects Btrfs filesystems to report subvolume lists and true space usage.
    - **S.M.A.R.T. Audit**: Performs hardware health checks on all block devices.
- **Dependencies**: `lsblk` (standard), `smartctl` (smartmontools), `btrfs-progs` (for Btrfs features).

---
### Module: Network Intelligence
- **Path**: `src/modules/network/mod.rs`
- **Function**: `pub async fn execute(dir: &str) -> std::io::Result<String>`
- **Description**: Centralized network stack monitoring.
- **Key Capabilities**:
    - **Interface Diagnostics**: Maps all active NICs, their operational state, and link speeds.
    - **Driver Conflict Auditing**: Automatically parses `lsmod` to detect Realtek driver collisions (r8125 vs r8169).
    - **Network Visibility**: Maps routing tables, active sockets (`ss`), and DNS resolver status.
    - **Hardware Mapping**: Links software interfaces to physical bus/drivers.
    
---
### Module: Cooling & Thermal Diagnostic
- **Path**: `src/modules/fan/mod.rs`
- **Function**: `pub async fn execute(dir: &str) -> std::io::Result<String>`
- **Description**: Real-time cooling telemetry and thermal zone monitoring.
- **Key Capabilities**:
    - **Thermal Zone Tracking**: Maps all system thermal zones to current temperatures.
    - **Fan/PWM Matrix**: Correlates fan RPM with PWM duty cycle percentage, providing a complete view of cooling performance.
    - **PWM Control Mode Detection**: Identifies whether fans are running in `Manual` or `Automatic` modes, allowing for rapid debugging of thermal throttling or fan control conflicts.
    
---
### Module: Hardware Intelligence
- **Path**: `src/modules/hardware/mod.rs`
- **Function**: `pub async fn execute(dir: &str) -> std::io::Result<String>`
- **Description**: Comprehensive physical system inventory and bus topology audit.
- **Key Capabilities**:
    - **System Board Audit**: Extracts manufacturer, model, serial, and BIOS versions via DMI decoding.
    - **Peripheral Topology**: Maps all PCI and USB devices currently connected to the system.
    - **Capability Reporting**: Summarizes system-wide hardware components, including architecture, cores, and bus-level details.
    
---
### Module: Memory Intelligence
- **Path**: `src/modules/memory/mod.rs`
- **Function**: `pub async fn execute(dir: &str) -> std::io::Result<String>`
- **Description**: High-fidelity memory diagnostics and OS-level tuning oversight.
- **Key Capabilities**:
    - **Physical Inventory**: Automatically identifies DDR generation, module speed, and manufacturer via DMI decoding.
    - **Swap/Pressure Monitoring**: Real-time tracking of swap usage and OS-level "swappiness" configurations.
    - **Tuning Oversight**: Monitors system performance profiles (`tuned`) to ensure memory latency is optimized for the workload.
    - **Topology**: Maps NUMA node structures for multi-socket or high-performance memory configurations.
    
---
### Module: Power & Performance Intelligence
- **Path**: `src/modules/power/mod.rs`
- **Function**: `pub async fn execute(dir: &str) -> std::io::Result<String>`
- **Description**: Monitors system CPU scaling, power governors, and battery/AC status.
- **Key Capabilities**:
    - **Single-Pass Topology**: Efficiently maps all CPU cores, their active governor, and current frequency in one read pass.
    - **Driver Auditing**: Detects active CPU scaling drivers (`amd_pstate`, `intel_pstate`, `acpi`) for thermal/power debugging.
    - **Energy Reporting**: Automatically hooks into `upower` to report current charge, battery health, or power source (AC/DC).
    
---
### Module: Motherboard Intelligence
- **Path**: `src/modules/motherboard/mod.rs`
- **Function**: `pub async fn execute(dir: &str) -> std::io::Result<String>`
- **Description**: Extracts deep-level hardware identity and UEFI/BIOS configuration.
- **Key Capabilities**:
    - **SMBIOS/DMI Auditing**: Pulls manufacturer, product model, and serial number directly from the hardware controller.
    - **Identity Summarization**: Parses raw DMI output into a human-readable "Quick Identity" header.
    - **Permissions Handling**: Gracefully handles non-root execution environments to prevent log pollution.
    
---
### Module: Storage Intelligence (AI)
- **Path**: `src/modules/storage_ai/mod.rs`
- **Function**: `pub async fn execute(dir: &str) -> std::io::Result<String>`
- **Description**: High-fidelity storage diagnostic suite for the CYBERDECK project, including LUKS encryption auditing and SMART health predictions.
- **Key Capabilities**:
    - **Block Topology**: Maps physical disks to filesystems and mount points.
    - **Predictive Health**: Parses hardware SMART registers to identify drive degradation.
    - **Encryption Visibility**: Identifies active LUKS/Mapper devices for security compliance.
    
---
### Module: Thermal Intelligence
- **Path**: `src/modules/thermal_ai/mod.rs`
- **Function**: `pub async fn execute(dir: &str) -> std::io::Result<String>`
- **Description**: Monitors system temperatures through kernel sysfs and hardware-level sensor tools.
- **Key Capabilities**:
    - **Thermal Topology**: Automatically maps thermal zones from the kernel.
    - **Heuristic Fallback**: Intelligent fallback to `lm-sensors` if the system provides no standard sysfs nodes.
    - **Telemetry Parsing**: Separates raw heat logs from parsed peak-temperature summaries.
    
---
### Module: Service Intelligence
- **Path**: `src/modules/services/mod.rs`
- **Function**: `pub async fn execute(dir: &str) -> std::io::Result<String>`
- **Description**: Monitors and categorizes systemd services for CYBERDECK operational readiness.
- **Key Capabilities**:
    - **Heuristic Classification**: Automatically tags services as Web, DB, Infrastructure, or Security based on naming patterns.
    - **Operational Snapshot**: Maps running units into a clean Markdown table for quick auditing.
    - **Graceful Failure**: Handles non-systemd environments without crashing the diagnostic sequence.
    
---
### Module: Live Dashboard
- **Path**: `src/modules/dashboard/mod.rs`
- **Function**: `pub async fn execute(dir: &str) -> std::io::Result<String>`
- **Description**: The visual interface for the CYBERDECK system.
- **Key Capabilities**:
    - **Synchronous Output**: Prints real-time system state to the console for live monitoring.
    - **Persistent Logging**: Writes a snapshot of the dashboard state to a markdown file, ensuring it is included in the Orchestrator's final report.
    
---

