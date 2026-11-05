//! Roblox platform connector (`RobloxPlatform`).
//!
//! Capabilities:
//! - Locates the latest installed Roblox version folder in
//!   `%LOCALAPPDATA%\Roblox\Versions\version-*\RobloxPlayerBeta.exe` (or via
//!   `HKCU\Software\Classes\roblox-player\shell\open\command`).
//! - **FastFlags Editor**: reads and atomically writes
//!   `ClientSettings/ClientAppSettings.json` with high-level presets (FPS cap,
//!   graphics backend D3D11 / Vulkan / Metal, Future-is-Bright Phase 3 lighting).
//! - **Multi-Instance Engine**: holds the Windows named mutex
//!   `ROBLOX_singletonMutex` in the launcher process so multiple Roblox client
//!   instances can run simultaneously without killing internal client handles.
//! - **Place Tracker**: parses the newest `%LOCALAPPDATA%\Roblox\logs\*.log` for
//!   the active `placeId` and resolves metadata via the Roblox Games API.

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::Mutex as StdMutex,
    time::SystemTime,
};

use async_trait::async_trait;
use chrono::Utc;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::core::platform::{
    launch_executable, launch_uri, ArtRef, Artwork, Game, GameId, LaunchHandle, PlatformConnector,
    PlatformKind, ProfileSync,
};
use crate::error::{IoResultExt, LauncherError, Result};

pub type FastFlagMap = BTreeMap<String, serde_json::Value>;

// ---------------------------------------------------------------------------
// FastFlag presets
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GraphicsApi {
    Default,
    Direct3D11,
    Vulkan,
    OpenGl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LightingTechnology {
    Default,
    Voxel,
    ShadowMap,
    Future,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct FastFlagPreset {
    /// Target framerate (`DFIntTaskSchedulerTargetFps`). `Some(0)` or `None` leaves default.
    pub target_fps: Option<u32>,
    pub graphics_api: Option<GraphicsApi>,
    pub lighting: Option<LightingTechnology>,
    pub disable_telemetry: bool,
}

impl FastFlagPreset {
    /// Apply this preset on top of an existing `ClientAppSettings.json` map.
    pub fn apply_to(&self, flags: &mut FastFlagMap) {
        if let Some(fps) = self.target_fps {
            if fps > 0 {
                flags.insert("DFIntTaskSchedulerTargetFps".into(), serde_json::json!(fps));
                flags.insert("FFlagTaskSchedulerLimitTargetFpsTo2402".into(), serde_json::json!("False"));
            } else {
                flags.remove("DFIntTaskSchedulerTargetFps");
            }
        }

        if let Some(api) = self.graphics_api {
            for k in [
                "FFlagDebugGraphicsPreferD3D11",
                "FFlagDebugGraphicsPreferVulkan",
                "FFlagDebugGraphicsPreferOpenGL",
                "FFlagDebugGraphicsDisableDirect3D11",
            ] {
                flags.remove(k);
            }
            match api {
                GraphicsApi::Default => {}
                GraphicsApi::Direct3D11 => {
                    flags.insert("FFlagDebugGraphicsPreferD3D11".into(), serde_json::json!(true));
                }
                GraphicsApi::Vulkan => {
                    flags.insert("FFlagDebugGraphicsDisableDirect3D11".into(), serde_json::json!(true));
                    flags.insert("FFlagDebugGraphicsPreferVulkan".into(), serde_json::json!(true));
                }
                GraphicsApi::OpenGl => {
                    flags.insert("FFlagDebugGraphicsDisableDirect3D11".into(), serde_json::json!(true));
                    flags.insert("FFlagDebugGraphicsPreferOpenGL".into(), serde_json::json!(true));
                }
            }
        }

        if let Some(lighting) = self.lighting {
            for k in [
                "FFlagDebugForceFutureIsBrightPhase2",
                "FFlagDebugForceFutureIsBrightPhase3",
            ] {
                flags.remove(k);
            }
            match lighting {
                LightingTechnology::Default | LightingTechnology::Voxel => {}
                LightingTechnology::ShadowMap => {
                    flags.insert(
                        "FFlagDebugForceFutureIsBrightPhase2".into(),
                        serde_json::json!(true),
                    );
                }
                LightingTechnology::Future => {
                    flags.insert(
                        "FFlagDebugForceFutureIsBrightPhase3".into(),
                        serde_json::json!(true),
                    );
                }
            }
        }

        if self.disable_telemetry {
            flags.insert(
                "FFlagDebugDisableTelemetryEphemeralCounter".into(),
                serde_json::json!(true),
            );
            flags.insert(
                "FFlagDebugDisableTelemetryEphemeralStat".into(),
                serde_json::json!(true),
            );
            flags.insert(
                "FFlagDebugDisableTelemetryV2Event".into(),
                serde_json::json!(true),
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Place metadata
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RobloxPlaceInfo {
    pub place_id: u64,
    pub universe_id: Option<u64>,
    pub name: String,
    pub builder: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RobloxPlace {
    pub place_id: u64,
    pub name: String,
    pub last_played_timestamp: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub universe_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub launch_url: Option<String>,
}

// ---------------------------------------------------------------------------
// Windows Multi-Instance Mutex guard
// ---------------------------------------------------------------------------

struct MutexGuardHandle {
    #[cfg(windows)]
    raw_handle: isize,
}

#[cfg(windows)]
impl Drop for MutexGuardHandle {
    fn drop(&mut self) {
        if self.raw_handle != 0 {
            unsafe {
                windows_sys::Win32::System::Threading::ReleaseMutex(self.raw_handle as _);
                windows_sys::Win32::Foundation::CloseHandle(self.raw_handle as _);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Connector
// ---------------------------------------------------------------------------

pub struct RobloxPlatform {
    http: reqwest::Client,
    state_dir: PathBuf,
    multi_instance: StdMutex<Option<MutexGuardHandle>>,
}

impl RobloxPlatform {
    pub fn new(http: reqwest::Client, state_dir: &Path) -> Self {
        Self {
            http,
            state_dir: state_dir.to_path_buf(),
            multi_instance: StdMutex::new(None),
        }
    }

    /// Locate the active Roblox version folder containing `RobloxPlayerBeta.exe`.
    pub fn locate_version_dir() -> Result<PathBuf> {
        #[cfg(windows)]
        {
            use winreg::{enums::*, RegKey};

            let hkcu = RegKey::predef(HKEY_CURRENT_USER);
            if let Ok(key) = hkcu.open_subkey(r"Software\Classes\roblox-player\shell\open\command")
            {
                if let Ok(cmd) = key.get_value::<String, _>("") {
                    if let Some(exe_path) = extract_quoted_path(&cmd) {
                        if let Some(dir) = exe_path.parent() {
                            if dir.join("RobloxPlayerBeta.exe").exists() {
                                return Ok(dir.to_path_buf());
                            }
                        }
                    }
                }
            }
        }

        let local_app_data = dirs::data_local_dir()
            .ok_or(LauncherError::PlatformNotFound("Roblox"))?;
        let versions_root = local_app_data.join("Roblox").join("Versions");
        if !versions_root.exists() {
            return Err(LauncherError::PlatformNotFound("Roblox"));
        }

        let entries = std::fs::read_dir(&versions_root).with_path(&versions_root)?;
        let mut newest: Option<(SystemTime, PathBuf)> = None;

        for entry in entries.flatten() {
            let dir = entry.path();
            if !dir.is_dir() || !dir.join("RobloxPlayerBeta.exe").exists() {
                continue;
            }
            let modified = entry
                .metadata()
                .and_then(|m| m.modified())
                .unwrap_or(SystemTime::UNIX_EPOCH);
            if newest.as_ref().map_or(true, |(best, _)| modified > *best) {
                newest = Some((modified, dir));
            }
        }

        newest
            .map(|(_, dir)| dir)
            .ok_or(LauncherError::PlatformNotFound("Roblox"))
    }

    pub fn client_settings_path() -> Result<PathBuf> {
        Ok(Self::locate_version_dir()?
            .join("ClientSettings")
            .join("ClientAppSettings.json"))
    }

    /// Register `HKCU\Software\Classes\roblox-player\shell\open\command` so
    /// browser `roblox-player:` launches route through OmniLauncher (or restore
    /// to the active `RobloxPlayerBeta.exe`).
    pub fn register_protocol_handler(use_omnilauncher: bool) -> Result<String> {
        #[cfg(windows)]
        {
            use winreg::{enums::*, RegKey};

            let target_exe = if use_omnilauncher {
                std::env::current_exe().map_err(LauncherError::IoRaw)?
            } else {
                Self::locate_version_dir()?.join("RobloxPlayerBeta.exe")
            };

            let cmd_value = format!("\"{}\" \"%1\"", target_exe.display());
            let hkcu = RegKey::predef(HKEY_CURRENT_USER);
            let (proto_key, _) = hkcu
                .create_subkey(r"Software\Classes\roblox-player")
                .map_err(|e| LauncherError::Registry(e.to_string()))?;
            proto_key
                .set_value("", &"URL: Roblox Protocol")
                .map_err(|e| LauncherError::Registry(e.to_string()))?;
            proto_key
                .set_value("URL Protocol", &"")
                .map_err(|e| LauncherError::Registry(e.to_string()))?;

            let (cmd_key, _) = proto_key
                .create_subkey(r"shell\open\command")
                .map_err(|e| LauncherError::Registry(e.to_string()))?;
            cmd_key
                .set_value("", &cmd_value)
                .map_err(|e| LauncherError::Registry(e.to_string()))?;

            Ok(cmd_value)
        }

        #[cfg(not(windows))]
        {
            let _ = use_omnilauncher;
            Err(LauncherError::Unsupported(
                "roblox-player protocol registration is only supported on Windows",
            ))
        }
    }

    // ---- FastFlags -----------------------------------------------------------

    pub async fn read_fast_flags(&self) -> Result<FastFlagMap> {
        let path = match Self::client_settings_path() {
            Ok(p) => p,
            Err(LauncherError::PlatformNotFound(_)) => {
                // Fall back to our persisted copy in `state_dir` so users can
                // pre-configure flags even before installing Roblox.
                self.state_dir.join("ClientAppSettings.json")
            }
            Err(e) => return Err(e),
        };

        match tokio::fs::read_to_string(&path).await {
            Ok(raw) => serde_json::from_str::<FastFlagMap>(&raw).map_err(|e| {
                LauncherError::Manifest {
                    path,
                    message: e.to_string(),
                }
            }),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(FastFlagMap::new()),
            Err(source) => Err(LauncherError::Io { path, source }),
        }
    }

    pub async fn write_fast_flags(&self, flags: &FastFlagMap) -> Result<PathBuf> {
        let target = Self::client_settings_path()
            .unwrap_or_else(|_| self.state_dir.join("ClientAppSettings.json"));

        if let Some(parent) = target.parent() {
            tokio::fs::create_dir_all(parent).await.with_path(parent)?;
        }
        let serialized = serde_json::to_vec_pretty(flags)?;
        let tmp = target.with_extension("json.tmp");
        tokio::fs::write(&tmp, &serialized).await.with_path(&tmp)?;
        tokio::fs::rename(&tmp, &target).await.with_path(&target)?;

        // Also keep a backup in our app data directory so Roblox updates don't wipe user flags.
        if target != self.state_dir.join("ClientAppSettings.json") {
            let _ = tokio::fs::create_dir_all(&self.state_dir).await;
            let _ = tokio::fs::write(self.state_dir.join("ClientAppSettings.json"), &serialized).await;
        }

        Ok(target)
    }

    pub async fn apply_preset(&self, preset: FastFlagPreset) -> Result<FastFlagMap> {
        let mut flags = self.read_fast_flags().await?;
        preset.apply_to(&mut flags);
        self.write_fast_flags(&flags).await?;
        Ok(flags)
    }

    // ---- Multi-Instance Mutex ------------------------------------------------

    /// Enable or disable the `ROBLOX_singletonMutex` holder.
    ///
    /// When enabled, OmniLauncher creates and owns `ROBLOX_singletonMutex`
    /// before any Roblox client starts. Subsequent Roblox processes see the
    /// already-held mutex and skip their single-instance termination check.
    pub fn set_multi_instance(&self, enabled: bool) -> Result<bool> {
        let mut guard = self
            .multi_instance
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        if !enabled {
            *guard = None;
            return Ok(false);
        }

        if guard.is_some() {
            return Ok(true);
        }

        #[cfg(windows)]
        {
            use windows_sys::Win32::Foundation::GetLastError;
            use windows_sys::Win32::System::Threading::CreateMutexW;

            let mut wide: Vec<u16> = "ROBLOX_singletonMutex".encode_utf16().collect();
            wide.push(0);

            let handle = unsafe { CreateMutexW(std::ptr::null(), 1, wide.as_ptr()) };
            if handle.is_null() {
                let code = unsafe { GetLastError() };
                return Err(LauncherError::WinApi(format!(
                    "CreateMutexW(ROBLOX_singletonMutex) failed with Win32 error {code}"
                )));
            }
            *guard = Some(MutexGuardHandle {
                raw_handle: handle as isize,
            });
            tracing::info!("acquired ROBLOX_singletonMutex for multi-instance mode");
            Ok(true)
        }

        #[cfg(not(windows))]
        {
            Err(LauncherError::Unsupported(
                "Roblox multi-instance mutex is only available on Windows",
            ))
        }
    }

    pub fn is_multi_instance_enabled(&self) -> bool {
        self.multi_instance
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .is_some()
    }

    // ---- Place Detection -----------------------------------------------------

    /// Inspect the most recent Roblox log file in `%LOCALAPPDATA%\Roblox\logs`
    /// to identify the last joined `placeId` and resolve its title.
    pub async fn detect_active_place(&self) -> Result<Option<RobloxPlaceInfo>> {
        let place_id = tokio::task::spawn_blocking(Self::scan_latest_log_for_place_id).await??;
        let Some(place_id) = place_id else {
            return Ok(None);
        };

        let url = format!("https://apis.roblox.com/universes/v1/places/{place_id}/universe");
        let universe_id = match self.http.get(&url).send().await {
            Ok(resp) if resp.status().is_success() => resp
                .json::<UniverseLookup>()
                .await
                .ok()
                .and_then(|u| u.universe_id),
            _ => None,
        };

        if let Some(uid) = universe_id {
            let details_url = format!("https://games.roblox.com/v1/games?universeIds={uid}");
            if let Ok(resp) = self.http.get(&details_url).send().await {
                if let Ok(envelope) = resp.json::<GamesDetailEnvelope>().await {
                    if let Some(item) = envelope.data.into_iter().next() {
                        return Ok(Some(RobloxPlaceInfo {
                            place_id,
                            universe_id: Some(uid),
                            name: item.name,
                            builder: item.creator.map(|c| c.name),
                            description: item.description,
                        }));
                    }
                }
            }
        }

        Ok(Some(RobloxPlaceInfo {
            place_id,
            universe_id,
            name: format!("Roblox Place #{place_id}"),
            builder: None,
            description: None,
        }))
    }

    fn scan_latest_log_for_place_id() -> Result<Option<u64>> {
        let Some(local) = dirs::data_local_dir() else {
            return Ok(None);
        };
        let logs_dir = local.join("Roblox").join("logs");
        if !logs_dir.exists() {
            return Ok(None);
        }

        let entries = std::fs::read_dir(&logs_dir).with_path(&logs_dir)?;
        let mut newest_log: Option<(SystemTime, PathBuf)> = None;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("log") {
                continue;
            }
            let modified = entry
                .metadata()
                .and_then(|m| m.modified())
                .unwrap_or(SystemTime::UNIX_EPOCH);
            if newest_log.as_ref().map_or(true, |(best, _)| modified > *best) {
                newest_log = Some((modified, path));
            }
        }

        let Some((_, log_path)) = newest_log else {
            return Ok(None);
        };
        let Ok(raw) = std::fs::read_to_string(&log_path) else {
            return Ok(None);
        };

        let re = Regex::new(r"(?i)placeid[:=\s]+(\d{5,})").expect("valid placeId regex");
        let found = re
            .captures_iter(&raw)
            .last()
            .and_then(|cap| cap.get(1))
            .and_then(|m| m.as_str().parse::<u64>().ok());

        Ok(found)
    }

    /// Multi-log parser that scans `%LOCALAPPDATA%\Roblox\logs` for recently played places,
    /// deduplicates them chronologically, and resolves 512x512 thumbnails.
    pub async fn get_recent_places(&self) -> Result<Vec<RobloxPlace>> {
        let Some(local) = dirs::data_local_dir() else {
            return Ok(Vec::new());
        };
        let logs_dir = local.join("Roblox").join("logs");
        if !logs_dir.exists() {
            return Ok(Vec::new());
        }

        let Ok(entries) = std::fs::read_dir(&logs_dir) else {
            return Ok(Vec::new());
        };

        let mut log_files: Vec<(SystemTime, PathBuf)> = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("log") {
                let modified = entry
                    .metadata()
                    .and_then(|m| m.modified())
                    .unwrap_or(SystemTime::UNIX_EPOCH);
                log_files.push((modified, path));
            }
        }

        // Sort descending by modified time and inspect up to 25 logs
        log_files.sort_by(|a, b| b.0.cmp(&a.0));
        log_files.truncate(25);

        let re_join_full = Regex::new(r"Joining place (\d+) \[(.+)\] with universe (\d+)").expect("regex");
        let re_place_simple = Regex::new(r"(?i)placeid[:=\s]+(\d{5,})").expect("regex");
        let re_network_place = Regex::new(r"(?i)placeId:\s*(\d{5,})").expect("regex");

        let mut places_map: std::collections::HashMap<u64, RobloxPlace> = std::collections::HashMap::new();

        for (mod_time, path) in log_files {
            let Ok(bytes) = std::fs::read(&path) else {
                continue;
            };
            let raw = String::from_utf8_lossy(&bytes);
            let file_timestamp = mod_time
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or_else(|_| Utc::now().timestamp());

            for cap in re_join_full.captures_iter(&raw) {
                if let (Some(pid_match), Some(name_match)) = (cap.get(1), cap.get(2)) {
                    if let Ok(place_id) = pid_match.as_str().parse::<u64>() {
                        let name = name_match.as_str().trim().to_string();
                        let universe_id = cap.get(3).and_then(|m| m.as_str().parse::<u64>().ok());
                        places_map
                            .entry(place_id)
                            .and_modify(|existing| {
                                if file_timestamp > existing.last_played_timestamp {
                                    existing.last_played_timestamp = file_timestamp;
                                    existing.name = name.clone();
                                    existing.universe_id = universe_id;
                                }
                            })
                            .or_insert_with(|| RobloxPlace {
                                place_id,
                                name,
                                last_played_timestamp: file_timestamp,
                                thumbnail_url: Some(format!(
                                    "https://thumbnails.roblox.com/v1/places/gameicons?placeIds={place_id}&size=512x512&format=Png&isCircular=false"
                                )),
                                universe_id,
                                launch_url: Some(format!("roblox://experiences/start?placeId={place_id}")),
                            });
                    }
                }
            }

            for cap in re_network_place.captures_iter(&raw).chain(re_place_simple.captures_iter(&raw)) {
                if let Some(pid_match) = cap.get(1) {
                    if let Ok(place_id) = pid_match.as_str().parse::<u64>() {
                        places_map
                            .entry(place_id)
                            .and_modify(|existing| {
                                if file_timestamp > existing.last_played_timestamp {
                                    existing.last_played_timestamp = file_timestamp;
                                }
                            })
                            .or_insert_with(|| RobloxPlace {
                                place_id,
                                name: format!("Roblox Experience #{place_id}"),
                                last_played_timestamp: file_timestamp,
                                thumbnail_url: Some(format!(
                                    "https://thumbnails.roblox.com/v1/places/gameicons?placeIds={place_id}&size=512x512&format=Png&isCircular=false"
                                )),
                                universe_id: None,
                                launch_url: Some(format!("roblox://experiences/start?placeId={place_id}")),
                            });
                    }
                }
            }
        }

        let mut list: Vec<RobloxPlace> = places_map.into_values().collect();
        list.sort_by(|a, b| b.last_played_timestamp.cmp(&a.last_played_timestamp));
        Ok(list)
    }
}

#[async_trait]
impl PlatformConnector for RobloxPlatform {
    fn kind(&self) -> PlatformKind {
        PlatformKind::Roblox
    }

    async fn is_available(&self) -> bool {
        Self::locate_version_dir().is_ok()
    }

    async fn detect_installed_games(&self) -> Result<Vec<Game>> {
        let version_dir = Self::locate_version_dir()?;
        let exe = version_dir.join("RobloxPlayerBeta.exe");

        let mut player = Game::new(PlatformKind::Roblox, "player", "Roblox");
        player.installed = true;
        player.install_dir = Some(version_dir);
        player.executable = Some(exe);
        player.tags = vec!["Bootstrapper".into(), "FastFlags".into(), "Multi-Instance".into()];
        player.art = Artwork {
            poster: Some(ArtRef::Remote(
                "https://images.rbxcdn.com/c83761712c58384892d63501cad3a1ee".into(),
            )),
            hero: None,
            logo: None,
            icon: None,
        };

        Ok(vec![player])
    }

    async fn launch_game(&self, native_id: &str) -> Result<LaunchHandle> {
        let game_id = GameId::new(PlatformKind::Roblox, native_id);

        // Support `place:<placeId>` deep links as well as raw numeric place IDs.
        if let Some(place_id) = native_id.strip_prefix("place:") {
            let pid: u64 = place_id.trim().parse().map_err(|_| {
                LauncherError::InvalidInput(format!("invalid Roblox PlaceId `{place_id}`"))
            })?;
            let uri = format!("roblox://experiences/start?placeId={pid}");
            return launch_uri(game_id, uri).await;
        } else if let Ok(pid) = native_id.trim().parse::<u64>() {
            let uri = format!("roblox://experiences/start?placeId={pid}");
            return launch_uri(game_id, uri).await;
        }

        if let Ok(dir) = Self::locate_version_dir() {
            let exe = dir.join("RobloxPlayerBeta.exe");
            if exe.exists() {
                return launch_executable(game_id, exe, vec!["--app".into()], Some(dir)).await;
            }
        }

        launch_uri(game_id, "roblox-player:1+launchmode:app".into()).await
    }

    async fn install_game(&self, native_id: &str) -> Result<LaunchHandle> {
        self.launch_game(native_id).await
    }

    async fn sync_profile(&self) -> Result<ProfileSync> {
        let active_place = self.detect_active_place().await.ok().flatten();
        Ok(ProfileSync {
            platform: PlatformKind::Roblox,
            account_id: None,
            display_name: active_place
                .map(|p| format!("Playing {}", p.name))
                .or_else(|| Some("Roblox Client".into())),
            avatar_url: None,
            owned_games: 1,
            synced_at: Utc::now(),
        })
    }
}

fn extract_quoted_path(cmd: &str) -> Option<PathBuf> {
    let trimmed = cmd.trim();
    if let Some(rest) = trimmed.strip_prefix('"') {
        let end = rest.find('"')?;
        Some(PathBuf::from(&rest[..end]))
    } else {
        trimmed.split_whitespace().next().map(PathBuf::from)
    }
}

#[derive(Debug, Deserialize)]
struct UniverseLookup {
    #[serde(rename = "universeId")]
    universe_id: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct GamesDetailEnvelope {
    #[serde(default)]
    data: Vec<GameDetailItem>,
}

#[derive(Debug, Deserialize)]
struct GameDetailItem {
    name: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    creator: Option<GameCreator>,
}

#[derive(Debug, Deserialize)]
struct GameCreator {
    name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fast_flag_preset_sets_fps_and_vulkan() {
        let preset = FastFlagPreset {
            target_fps: Some(240),
            graphics_api: Some(GraphicsApi::Vulkan),
            lighting: Some(LightingTechnology::Future),
            disable_telemetry: true,
        };
        let mut map = FastFlagMap::new();
        preset.apply_to(&mut map);
        assert_eq!(map.get("DFIntTaskSchedulerTargetFps"), Some(&serde_json::json!(240)));
        assert_eq!(map.get("FFlagDebugGraphicsPreferVulkan"), Some(&serde_json::json!(true)));
        assert_eq!(
            map.get("FFlagDebugForceFutureIsBrightPhase3"),
            Some(&serde_json::json!(true))
        );
    }
}
