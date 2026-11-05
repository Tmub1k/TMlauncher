# Milestone 4 Core Engine Architecture & Implementation Handoff Report

## 1. Observation

Direct code and environment observations:

1. **Process Management & Session Watchdog**:
   - In `src-tauri/src/core/runner.rs` (lines 45-68, 71-117, 148-185):
     `ProcessRunner` manages active game sessions via `sessions: RwLock<HashMap<GameId, ActiveSession>>`.
     `register_launch` (lines 71-117) receives `(game_id, title, pid, details, cover_url)` and emits `runner://session`.
     `spawn_watchdog` (lines 148-156) runs a 5-second interval loop calling `tick_sessions()`.
     `is_process_alive(pid)` (lines 188-213) checks process vitality using `OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid)` and `GetExitCodeProcess(handle, &mut exit_code)`.
     When PID exits or is stopped, `stop_session` (lines 120-134) removes the session from the active map and clears Discord presence when the map is empty.

2. **Cargo Dependencies & Win32 Features**:
   - In `src-tauri/Cargo.toml` (lines 48-70):
     `zip = { version = "2.2", default-features = false, features = ["deflate"] }` is present.
     `chrono = { version = "0.4", features = ["serde"] }`, `dirs = "5"`, `sha2 = "0.10"`, `hex = "0.4"` are present.
     `windows-sys = { version = "0.59", features = ["Win32_Foundation", "Win32_Security", "Win32_System_Threading", "Win32_Storage_FileSystem"] }` is present.
     `Win32_System_Threading` already provides `GetCurrentProcess`, `SetPriorityClass`, `IDLE_PRIORITY_CLASS`, and `NORMAL_PRIORITY_CLASS`.
     `cargo check --manifest-path src-tauri/Cargo.toml` executes with exit code 0 in 0.53s.

3. **Existing Save & Cloud Patterns**:
   - In `src-tauri/src/platforms/steam/mod.rs` (lines 683-698):
     `SteamPlatform::sync_cloud` already performs snapshotting into `%LOCALAPPDATA%\OmniLauncher\cloud_backups\{app_id}\{timestamp}` by copying folders recursively, returning `CloudSyncResult { snapshot_id, synced: true, ... }`.
   - In `src-tauri/src/commands/emulators.rs` (lines 303-363):
     `safe_extract_archive` implements extraction with path containment (`entry.enclosed_name().ok_or_else("Zip-Slip vulnerability detected")`) and rollback on failure.

4. **Steam Cache & App Data Layout**:
   - In `src-tauri/src/platforms/steam/mod.rs` (lines 736-746, 750-790):
     `locate_steam_root()` resolves Steam install path from Registry (`HKCU\Software\Valve\Steam\SteamPath` and `HKLM\SOFTWARE\WOW6432Node\Valve\Steam\InstallPath`).
     `vdf_parser::parse_library_folders_file` resolves all Steam library root paths (`<steam_root>\steamapps\libraryfolders.vdf`).
     Each library contains `steamapps/appmanifest_<appid>.acf`, `steamapps/shadercache/<appid>`, and `steamapps/compatdata/<appid>`.

5. **Existing Command Registration & Entry Points**:
   - In `src-tauri/src/lib.rs` (lines 121-186):
     All Tauri IPC commands are declared in `tauri::generate_handler![...]`.
     Tauri state manages `ConfigStore`, `reqwest::Client`, `PlatformRegistry`, `DiscordRpcService`, `MediaCapture`, and `ProcessRunner`.

---

## 2. Logic Chain

1. **Zero-Footprint Game Mode (Feature 19)**:
   - *Premise (Observation 1 & 2)*: When a game is active, launcher background resource usage must be minimized to 0% CPU and low RAM. `runner.rs` is already notified synchronously when games launch and when all sessions terminate.
   - *Inference*: By adding a `GameModeService` integrated into `ProcessRunner::register_launch` and `ProcessRunner::stop_session`, OmniLauncher can automatically drop process priority to `IDLE_PRIORITY_CLASS` (via Win32 `SetPriorityClass`) and immediately trim working set physical RAM to pagefile/standby list via `SetProcessWorkingSetSize(GetCurrentProcess(), usize::MAX, usize::MAX)`.
   - *Inference*: Emitting a Tauri event `gamemode://status` enables the Svelte frontend to pause rendering animations, halt background polling, and transition into minimal resource mode. When the final game PID exits in `runner.rs`, priority is restored to `NORMAL_PRIORITY_CLASS`, working set is unconstrained, and `gamemode://status { active: false }` wakes up the UI.

2. **Save-State Time Machine (Feature 20)**:
   - *Premise (Observation 1, 2 & 3)*: Gamers need pre/post game session backup and version rollback. `zip` 2.2 is available, and `safe_extract_archive` in `emulators.rs` provides a secure, Zip-Slip-resistant extraction blueprint.
   - *Inference*: A dedicated `SaveVaultService` in `src-tauri/src/core/save_vault.rs` can manage save game discovery across Steam userdata, `%USERPROFILE%\Saved Games`, `%APPDATA%`, `%LOCALAPPDATA%`, and custom paths.
   - *Inference*: Automatic hooks in `ProcessRunner::register_launch` can create a `PreLaunch` snapshot, while `ProcessRunner::stop_session` creates a `PostLaunch` snapshot. Manual snapshots can be taken at any time from the UI.
   - *Inference*: To guarantee zero risk of data loss during rollback, `restore_snapshot` must execute an atomic `PreRestore` safety backup of the active save directory before writing archive contents to disk. A pinning mechanism ensures user-favored snapshots are never pruned by the retention limit.

3. **Shader Cache & Disk Reclaimer (Feature 23)**:
   - *Premise (Observation 4)*: Uninstalled games leave orphaned shader caches in Steam library folders (`steamapps/shadercache/<appid>` and `compatdata/<appid>`). GPU drivers leave gigabytes of caches in `%LOCALAPPDATA%\D3DSCache`, `%LOCALAPPDATA%\NVIDIA\DXCache`, and `%LOCALAPPDATA%\AMD\DxCache`.
   - *Inference*: By querying `libraryfolders.vdf` for all library roots and checking which `appmanifest_<appid>.acf` files exist, `ShaderReclaimerService` can deterministically flag any `<appid>` cache missing an active manifest as `OrphanedSteamShader` (100% safe to reclaim).
   - *Inference*: For GPU caches, files can be filtered by age (e.g., stale shaders untouched for >30 days). File deletions must use non-aborting loops: if a file has an OS sharing violation (in-use by DirectX/driver), it is skipped and counted, allowing remaining files to be cleaned safely. Active games in `runner.rs` must have their caches shielded from deletion.

4. **Proactive Offline Vault (Feature 28)**:
   - *Premise (Observation 1, 2 & 4)*: Offline gaming fails when launchers require remote handshakes or fail to display remote artwork and game metadata.
   - *Inference*: An `OfflineVaultService` in `src-tauri/src/core/offline_vault.rs` can maintain an atomic local JSON cache of the full library, copy/cache remote posters to `%LOCALAPPDATA%\OmniLauncher\vault\covers\`, and store direct executable paths and launch parameters.
   - *Inference*: The service calculates an `OfflineReadinessScore` (`(ready_games / installed_games) * 100`) and provides a `launch_offline_game` command that invokes `launch_executable` directly with working directory and arguments, bypassing third-party launcher network requirements.

---

## 3. Caveats

1. **Win32 `SetProcessWorkingSetSize` Behavior**:
   - Calling `SetProcessWorkingSetSize(GetCurrentProcess(), !0, !0)` empties the working set, transferring pages to the standby/paging list. If the user immediately interacts with OmniLauncher while a game is running, Windows will page required memory back in on-demand (soft page faults), causing negligible micro-second latency only on the first input frame.
2. **Third-Party DRM on Protocol Launches**:
   - Games with mandatory online DRM (e.g. Denuvo with periodic online token renewal, or live-service games like Fortnite / Valorant) will still require their respective publisher servers to authenticate. The Offline Vault can reliably launch DRM-free titles, emulators, itch.io games, and Steam games in offline mode, but cannot bypass publisher-enforced server-side DRM.
3. **GPU Driver Cache Locking**:
   - On Windows 11, the `D3DSCache` directory or NVIDIA `DXCache` may have active write locks held by `dwm.exe` or background GPU services. The reclaimer must treat `ERROR_SHARING_VIOLATION` (code 32) as a non-fatal skip rather than an operation failure.
4. **Save Folder Heuristics**:
   - While Steam games with Steam Cloud have exact paths in `remotecache.vdf` or `userdata/<account_id>/<app_id>`, games using non-standard save directories (e.g. registry-only or obscure subfolders) require the user to designate a custom save directory via `GamePropertiesModal`, which the Save Vault then records and tracks.

---

## 4. Conclusion & Technical Implementation Plan

### A. Concrete Data Structures (Rust Models)

#### 1. Zero-Footprint Game Mode
```rust
// In src-tauri/src/core/game_mode.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessPriorityLevel {
    Idle,
    BelowNormal,
    Normal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameModeStatus {
    pub active: bool,
    pub active_sessions_count: usize,
    pub process_priority: ProcessPriorityLevel,
    pub working_set_bytes: u64,
    pub last_trimmed_at: Option<i64>,
    pub games_running: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryTrimResult {
    pub success: bool,
    pub previous_working_set_bytes: u64,
    pub current_working_set_bytes: u64,
    pub bytes_freed: u64,
    pub trimmed_at: i64,
}
```

#### 2. Save-State Time Machine
```rust
// In src-tauri/src/core/save_vault.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SnapshotKind {
    PreLaunch,
    PostLaunch,
    Manual,
    PreRestore,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveSnapshot {
    pub id: String,
    pub game_id: String,
    pub title: String,
    pub label: String,
    pub kind: SnapshotKind,
    pub timestamp: i64,
    pub created_at: String,
    pub file_count: usize,
    pub uncompressed_bytes: u64,
    pub compressed_bytes: u64,
    pub sha256: String,
    pub source_path: String,
    pub archive_path: String,
    pub pinned: bool,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveLocationCandidate {
    pub path: String,
    pub exists: bool,
    pub file_count: usize,
    pub total_size_bytes: u64,
    pub source: String, // "steam_userdata" | "saved_games" | "appdata_local" | "appdata_roaming" | "documents" | "custom"
    pub is_primary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreReport {
    pub success: bool,
    pub snapshot_id: String,
    pub target_path: String,
    pub restored_files_count: usize,
    pub safety_snapshot_id: Option<String>,
    pub restored_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultSummary {
    pub total_snapshots: usize,
    pub total_games: usize,
    pub total_vault_bytes: u64,
    pub oldest_snapshot_timestamp: Option<i64>,
    pub newest_snapshot_timestamp: Option<i64>,
}
```

#### 3. Shader Cache & Disk Reclaimer
```rust
// In src-tauri/src/core/shader_reclaimer.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CacheCategory {
    OrphanedSteamShader,
    OrphanedCompatdata,
    InstalledSteamShader,
    DirectxCache,
    NvidiaCache,
    AmdCache,
    IntelCache,
    LauncherTemp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheItem {
    pub id: String,
    pub title: String,
    pub category: CacheCategory,
    pub path: String,
    pub size_bytes: u64,
    pub file_count: usize,
    pub last_modified: Option<i64>,
    pub is_safe_to_delete: bool,
    pub is_in_use: bool,
    pub app_id: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShaderCacheSummary {
    pub items: Vec<CacheItem>,
    pub total_reclaimable_bytes: u64,
    pub orphaned_bytes: u64,
    pub gpu_cache_bytes: u64,
    pub temp_cache_bytes: u64,
    pub scanned_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanReport {
    pub total_freed_bytes: u64,
    pub cleaned_items_count: usize,
    pub failed_items_count: usize,
    pub skipped_locked_files: usize,
    pub cleaned_at: i64,
    pub details: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanRequest {
    pub item_ids: Vec<String>,
    pub include_gpu_caches: bool,
    pub only_orphans: bool,
}
```

#### 4. Proactive Offline Vault
```rust
// In src-tauri/src/core/offline_vault.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OfflineGameRecord {
    pub game_id: String,
    pub title: String,
    pub platform: String,
    pub install_dir: String,
    pub direct_executable: Option<String>,
    pub launch_args: Vec<String>,
    pub local_cover_path: Option<String>,
    pub is_drm_free_or_offline_capable: bool,
    pub last_synced_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OfflineVaultStatus {
    pub total_cached_games: usize,
    pub offline_ready_games: usize,
    pub readiness_percentage: f32,
    pub cached_covers_count: usize,
    pub vault_size_bytes: u64,
    pub last_vault_sync: Option<i64>,
    pub is_currently_offline: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OfflineSyncReport {
    pub synced_games_count: usize,
    pub new_covers_cached: usize,
    pub vault_bytes_total: u64,
    pub synced_at: i64,
}
```

---

### B. Tauri Command Signatures

| Feature | Tauri Command | Arguments | Return Type | Purpose |
|---|---|---|---|---|
| **Zero-Footprint** | `enter_game_mode` | `pid: Option<u32>` | `Result<GameModeStatus>` | Forces priority drop & memory trim |
| **Zero-Footprint** | `exit_game_mode` | (none) | `Result<GameModeStatus>` | Restores normal priority & wakes UI |
| **Zero-Footprint** | `get_game_mode_status` | (none) | `Result<GameModeStatus>` | Returns current active mode & memory |
| **Zero-Footprint** | `trim_process_memory` | (none) | `Result<MemoryTrimResult>` | Manual RAM flush command |
| **Save Time Machine** | `create_save_snapshot` | `game_id: String, label: Option<String>, kind: Option<SnapshotKind>, custom_path: Option<String>` | `Result<SaveSnapshot>` | Zips active save folder with metadata |
| **Save Time Machine** | `list_save_snapshots` | `game_id: String` | `Result<Vec<SaveSnapshot>>` | Enumerate chronological snapshots |
| **Save Time Machine** | `restore_save_snapshot` | `game_id: String, snapshot_id: String` | `Result<RestoreReport>` | Pre-restore backup + safe unzip |
| **Save Time Machine** | `delete_save_snapshot` | `game_id: String, snapshot_id: String` | `Result<()>` | Delete snapshot archive & manifest entry |
| **Save Time Machine** | `toggle_snapshot_pin` | `game_id: String, snapshot_id: String` | `Result<bool>` | Protects snapshot from max retention purge |
| **Save Time Machine** | `detect_game_save_locations` | `game_id: String` | `Result<Vec<SaveLocationCandidate>>` | Discovers save paths across system |
| **Save Time Machine** | `get_save_vault_summary` | (none) | `Result<VaultSummary>` | Total storage used by snapshots |
| **Save Time Machine** | `export_save_snapshot` | `game_id: String, snapshot_id: String, target_path: String` | `Result<()>` | Exports zip to arbitrary path |
| **Shader Reclaimer** | `scan_shader_caches` | (none) | `Result<ShaderCacheSummary>` | Fast scan of Steam, GPU, temp caches |
| **Shader Reclaimer** | `clean_shader_caches` | `request: CleanRequest` | `Result<CleanReport>` | Prunes caches safely, skipping locked files |
| **Shader Reclaimer** | `get_cache_item_files` | `item_id: String` | `Result<Vec<String>>` | Detailed file breakdown per cache entry |
| **Offline Vault** | `get_offline_vault_status` | (none) | `Result<OfflineVaultStatus>` | Offline score, cache size, sync date |
| **Offline Vault** | `sync_offline_vault` | (none) | `Result<OfflineSyncReport>` | Pre-caches metadata, posters, recipes |
| **Offline Vault** | `get_offline_games` | (none) | `Result<Vec<OfflineGameRecord>>` | Offline catalog view |
| **Offline Vault** | `launch_offline_game` | `game_id: GameId` | `Result<LaunchHandle>` | Direct binary launch bypassing store client |

---

### C. Proposed File Structure Map

```
src-tauri/
├── Cargo.toml                                 # Add Win32_System_ProcessStatus, Win32_System_Memory to windows-sys
├── src/
│   ├── lib.rs                                 # Register new services in state & commands in handler
│   ├── core/
│   │   ├── game_mode.rs                       # [NEW] Win32 priority & SetProcessWorkingSetSize controller
│   │   ├── save_vault.rs                      # [NEW] Save game discovery, zip compressor, manifest, restore
│   │   ├── shader_reclaimer.rs                # [NEW] Shader/compatdata orphan discovery & disk reclaimer
│   │   ├── offline_vault.rs                   # [NEW] Offline library cache, cover mirroring, launch recipes
│   │   ├── runner.rs                          # Update: hooks for auto-game-mode and auto-save snapshots
│   │   └── mod.rs                             # Expose new modules
│   └── commands/
│       ├── game_mode.rs                       # [NEW] IPC handlers for Zero-Footprint mode
│       ├── save_vault.rs                      # [NEW] IPC handlers for Save-State Time Machine
│       ├── storage.rs                         # [NEW] IPC handlers for Shader Reclaimer & disk space
│       ├── offline.rs                         # [NEW] IPC handlers for Proactive Offline Vault
│       └── mod.rs                             # Expose new command modules
```

---

### D. Win32 & Cross-Platform Implementation Strategy

1. **Working Set Trimming**:
   ```rust
   #[cfg(windows)]
   pub fn trim_working_set() -> (u64, u64) {
       use windows_sys::Win32::System::Threading::GetCurrentProcess;
       extern "system" {
           fn SetProcessWorkingSetSize(
               hprocess: windows_sys::Win32::Foundation::HANDLE,
               dwminimumworkingsetsize: usize,
               dwmaximumworkingsetsize: usize,
           ) -> windows_sys::Win32::Foundation::BOOL;
       }
       let before = query_working_set_bytes();
       unsafe {
           SetProcessWorkingSetSize(GetCurrentProcess(), usize::MAX, usize::MAX);
       }
       let after = query_working_set_bytes();
       (before, after)
   }

   #[cfg(not(windows))]
   pub fn trim_working_set() -> (u64, u64) {
       (0, 0)
   }
   ```

2. **Process Priority Adjustment**:
   ```rust
   #[cfg(windows)]
   pub fn set_process_priority(level: ProcessPriorityLevel) {
       use windows_sys::Win32::System::Threading::{
           GetCurrentProcess, SetPriorityClass, BELOW_NORMAL_PRIORITY_CLASS,
           IDLE_PRIORITY_CLASS, NORMAL_PRIORITY_CLASS,
       };
       let prio = match level {
           ProcessPriorityLevel::Idle => IDLE_PRIORITY_CLASS,
           ProcessPriorityLevel::BelowNormal => BELOW_NORMAL_PRIORITY_CLASS,
           ProcessPriorityLevel::Normal => NORMAL_PRIORITY_CLASS,
       };
       unsafe {
           SetPriorityClass(GetCurrentProcess(), prio);
       }
   }

   #[cfg(not(windows))]
   pub fn set_process_priority(_level: ProcessPriorityLevel) {}
   ```

3. **Zip Archive Creation & Rollback Protection**:
   - `zip::ZipWriter` with `zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated)`
   - Writing files to temporary `.zip.tmp` and renaming atomically to `.zip` once completed.
   - SHA-256 calculation computed during compression stream via `sha2::Sha256`.

4. **Locked File Resilience in Shader Cleanup**:
   - When iterating over shader directory files:
     ```rust
     match std::fs::remove_file(&entry_path) {
         Ok(()) => freed_bytes += file_size,
         Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied || e.raw_os_error() == Some(32) => {
             skipped_locked += 1;
             tracing::debug!(path = %entry_path.display(), "File locked by driver/DirectX; skipping");
         }
         Err(e) => {
             failed_count += 1;
             tracing::warn!(path = %entry_path.display(), error = %e, "Failed to remove cache file");
         }
     }
     ```

---

## 5. Verification Method

1. **Compilation Check**:
   - Run `cargo check --manifest-path src-tauri/Cargo.toml` to verify all new dependencies and type signatures compile without warnings or errors.
2. **Unit Tests**:
   - Test `save_vault` snapshot compression, manifest serialization, zip-slip detection, rollback, and retention limits with a synthetic save directory fixture.
   - Test `shader_reclaimer` orphan detection by providing mock Steam library folders with both installed and uninstalled app IDs.
   - Test `offline_vault` readiness score computation with varied mock game catalogs.
3. **Runtime IPC Verification**:
   - Test `enter_game_mode` and `exit_game_mode` via Tauri invoke and observe working set memory in Windows Task Manager (shrinking from ~100MB to <10MB).
   - Test `create_save_snapshot` and verify generated `.zip` in `%LOCALAPPDATA%\OmniLauncher\save_vault\` can be extracted and restored cleanly.
   - Test `scan_shader_caches` to verify DirectX and Steam shader paths are accurately detected and reported in MB.
