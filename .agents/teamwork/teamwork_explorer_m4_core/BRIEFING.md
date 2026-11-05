# BRIEFING — 2026-11-05T05:01:00Z

## Mission
Investigate existing backend files in src-tauri/src/ and plan the implementation for Zero-Footprint Game Mode, Save-State Time Machine, Shader Cache Reclaimer, and Offline Vault.

## 🔒 My Identity
- Archetype: explorer
- Roles: explorer, analyst, architect
- Working directory: c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\teamwork_explorer_m4_core
- Original parent: bed11d0d-d77d-423c-a811-8e81f371aec2
- Milestone: Milestone 4 Core Engine

## 🔒 Key Constraints
- Read-only investigation — do NOT implement
- Scope: Zero-Footprint Game Mode (Feature 19), Save-State Time Machine (Feature 20), Shader Cache & Disk Reclaimer (Feature 23), Proactive Offline Vault (Feature 28)
- Target architecture: Rust Tauri backend (src-tauri)
- Deliverables: Comprehensive handoff report with data structures, Tauri command signatures, Win32/cross-platform specifics, error handling, and test plan
- Output: handoff.md and send_message notification to parent

## Current Parent
- Conversation ID: bed11d0d-d77d-423c-a811-8e81f371aec2
- Updated: not yet

## Investigation State
- **Explored paths**:
  - `src-tauri/Cargo.toml`, `src-tauri/src/lib.rs`, `src-tauri/src/error.rs`
  - `src-tauri/src/core/runner.rs`, `src-tauri/src/core/config.rs`, `src-tauri/src/core/platform.rs`
  - `src-tauri/src/commands/library.rs`, `src-tauri/src/commands/platforms.rs`, `src-tauri/src/commands/emulators.rs`
  - `src-tauri/src/platforms/steam/mod.rs`
- **Key findings**:
  - `runner.rs` already manages active game sessions via `ProcessRunner` and contains a 5s watchdog loop monitoring PID exit.
  - Zero-Footprint mode can hook directly into `register_launch` and `stop_session` in `runner.rs`.
  - Win32 API functions `SetPriorityClass` and `SetProcessWorkingSetSize` provide process priority lowering and instant working set trimming to standby/paging memory.
  - `zip` crate (v2.2) is already in `Cargo.toml` and battle-tested in `commands/emulators.rs` with safe extraction & path containment checks against Zip-Slip.
  - Steam cloud backup basics already exist in `SteamPlatform::sync_cloud` copying files to `%LOCALAPPDATA%\OmniLauncher\cloud_backups\{app_id}\{ts}`. Save-State Time Machine will expand this into a universal, multi-store, versioned zip vault.
  - Steam shader caches (`steamapps/shadercache/<appid>`) and compatdata prefixes (`compatdata/<appid>`) can be accurately classified as orphaned by cross-referencing with installed `appmanifest_<appid>.acf` entries across all Steam library folders.
  - GPU shader caches (`%LOCALAPPDATA%\D3DSCache`, `%LOCALAPPDATA%\NVIDIA\DXCache`, `%LOCALAPPDATA%\AMD\DxCache`) can be scanned and pruned with per-file error handling for locked files.
  - Offline Vault can mirror library manifest, cached credentials, and artwork assets locally to `%LOCALAPPDATA%\OmniLauncher\vault\`, calculating an Offline Readiness Score and supporting direct executable launches.
- **Unexplored areas**: None for M4 backend core systems scope.

## Key Decisions Made
- Architecture planned around 4 modular backend domains in `src-tauri/src/core/` and their respective Tauri IPC command surfaces in `src-tauri/src/commands/`:
  1. `core/game_mode.rs` + `commands/game_mode.rs` (Zero-Footprint Game Mode)
  2. `core/save_vault.rs` + `commands/save_vault.rs` (Save-State Time Machine)
  3. `core/shader_reclaimer.rs` + `commands/storage.rs` (Shader Cache & Disk Reclaimer)
  4. `core/offline_vault.rs` + `commands/offline.rs` (Proactive Offline Vault)
- Hook automated pre/post launch backups and memory trimming directly into `ProcessRunner` lifecycle methods.

## Artifact Index
- c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\teamwork_explorer_m4_core\DISPATCH.md — Task assignment
- c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\teamwork_explorer_m4_core\BRIEFING.md — Working memory
- c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\teamwork_explorer_m4_core\progress.md — Liveness heartbeat
- c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\teamwork_explorer_m4_core\handoff.md — Final investigation report
