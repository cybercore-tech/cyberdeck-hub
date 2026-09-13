# Changelog - CYBERDECK

## [0.1.0-alpha.5] - 2026-07-13
### Added
- Dynamic theme selector with real-time CSS variable injection
- "Default (CYBERDECK)" reset functionality for UI restoration
- Automated API theme discovery

### Fixed
- CSS loading hierarchy issues
- Key mismatch between JSON configuration and CSS variables

# Changelog
All notable changes to the CYBERDECK system will be documented in this file.

## [0.55] - 2026-07-14
## [0.1.0-alpha.11] - 2026-07-14
### Added
* **Archive System Command:** Integrated `ArchiveFiles` to the backend dispatcher. This allows the system to trigger `zip`, `tar`, `gzip`, or `7z` compression on the `diagnostics/` directory.
* **Observability Stack:** Added `tracing` and `tracing-subscriber` to replace standard print macros. This provides structured async logging, crucial for debugging the asynchronous nature of the CYBERDECK backend.
* **Chrono Integration:** Added `chrono` for generating UTC-compliant timestamps, enabling automated, unique filenames for backups.
* **Frontend Frameworks:** Added **Alpine.js** and **HeadlessUI** (see *Notes*).

### Changed
* **Output Management:** Redirected all generated archive files to a dedicated `output/` directory to prevent root directory clutter.
* **Filename Convention:** Archives now use a `archive_YYYY-MM-DD.ext` naming pattern to prevent file overwriting and maintain a clear audit trail.

### Fixed
* **Deserialization Error:** Resolved the "unknown variant" system error by updating the `CyberdeckCommand` enum and serialization logic to correctly map frontend JSON requests to backend variants.

---

### Notes: Frontend Architecture

#### Why Alpine.js?
We added **Alpine.js** to handle the frontend interactivity because it provides a reactive, "sprinkle-on" approach that pairs perfectly with the CYBERDECK backend.
* **Lightweight:** It keeps the CYBERDECK interface snappy without the massive overhead of a heavy SPA framework.
* **Direct Interaction:** It allows us to bind the "Execute" buttons directly to the `fetch` API calls in the HTML markup, reducing the complexity of the JavaScript files.

#### Why HeadlessUI?
We integrated **HeadlessUI** to ensure the interface is accessible and robust.
* **Accessibility:** It handles the complex "behind-the-scenes" state (like focus management for modals or the state of a dropdown) that is notoriously difficult to write from scratch.
* **Unstyled Components:** Because CYBERDECK requires a specific aesthetic, HeadlessUI gives us the *logic* (the "headless" part) but leaves the *styling* entirely up to us, ensuring the UI remains perfectly in character.

## [1.0.0] - 2026-07-14

### Added
- **Archive System:** Fully implemented backend compression (zip, tar, gzip, 7z) for diagnostic data.
- **Observability:** Integrated `tracing` and `tracing-subscriber` for robust asynchronous logging.
- **Timestamping:** Implemented `chrono` for automated, unique archival filenames (`archive_YYYY-MM-DD.ext`).
- **Frontend:** Integrated Alpine.js and HeadlessUI to handle UI interactivity and state management.

### Changed
- **Stable Milestone:** Promoted version from `alpha` to `stable`. The CYBERDECK core is now considered feature-complete and production-ready.
- **Output Management:** Redirected all generated archive assets to the `./output/` directory.
- **Project Scope:** Official move from alpha testing phase to v1.0.0 stable release.

### Fixed
- **Serialization:** Resolved critical `serde` deserialization issues (unknown variant errors) in the command dispatcher.
- **Code Hygiene:** Removed all unused imports and dead code across `dispatcher.rs`, `routes.rs`, and audio modules.
- **Build Integrity:** Resolved all compiler warnings; project now compiles with zero warnings/dead code.

---

### Notes: The Road to Stable
This release marks the transition of CYBERDECK from an experimental alpha to a stable, reliable system. The codebase has been audited for memory safety, logging transparency, and architectural cleanliness. The system is now ready for deployment and package management integration (AUR/Nix/etc.).

---
## [1.1.0] - 2026-07-15
### Added
- **mdBook Infrastructure:** Initialized `docs/` directory for system documentation.
- **Auto-Summary Generator:** Integrated `build.rs` script to dynamically generate `SUMMARY.md` from code snippets.
- **Custom Theming:** Added CSS support for project styling and readability.

### Changed
- **Build Workflow:** Automated documentation pipeline triggered on every `cargo build`.
- **Documentation Structure:** Standardized file naming and organizational structure for better maintainability.

### Fixed
- **Configuration:** Resolved `book.toml` deserialization errors regarding CSS and chapter numbering.
- **Build Pathing:** Cleaned up technical noise from generated filenames.
