//! The `PlatformConnector` abstraction and the shared game data model.
//!
//! Every store/launcher integration (Steam, Epic, Roblox, itch.io, emulators)
//! implements [`PlatformConnector`]. The [`PlatformRegistry`] fans out scans
//! concurrently and keeps an in-memory cache of the unified catalog.

use std::{
    collections::HashMap,
    fmt,
    path::{Path, PathBuf},
    process::Stdio,
    str::FromStr,
    sync::Arc,
};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use tokio::sync::RwLock;

use crate::error::{LauncherError, Result};

// ---------------------------------------------------------------------------
// Identity
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PlatformKind {
    Steam,
    Epic,
    Roblox,
    Itch,
    Emulator,
    Canonical,
}

impl PlatformKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Steam => "steam",
            Self::Epic => "epic",
            Self::Roblox => "roblox",
            Self::Itch => "itch",
            Self::Emulator => "emulator",
            Self::Canonical => "canonical",
        }
    }
}

impl fmt::Display for PlatformKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for PlatformKind {
    type Err = LauncherError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_ascii_lowercase().as_str() {
            "steam" => Ok(Self::Steam),
            "epic" => Ok(Self::Epic),
            "roblox" => Ok(Self::Roblox),
            "itch" => Ok(Self::Itch),
            "emulator" | "emulators" | "retro" => Ok(Self::Emulator),
            "canonical" | "unified" => Ok(Self::Canonical),
            _ => Err(LauncherError::InvalidInput(format!("unknown platform `{s}`"))),
        }
    }
}

/// Globally unique game identifier, serialized as `"<platform>:<native_id>"`
/// (e.g. `steam:570`, `epic:Fortnite`, `roblox:place:920587237`) or `"unified_<key>"`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GameId {
    pub platform: PlatformKind,
    pub native_id: String,
}

impl GameId {
    pub fn new(platform: PlatformKind, native_id: impl Into<String>) -> Self {
        Self {
            platform,
            native_id: native_id.into(),
        }
    }
}

impl fmt::Display for GameId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.platform == PlatformKind::Canonical {
            write!(f, "unified_{}", self.native_id)
        } else {
            write!(f, "{}:{}", self.platform, self.native_id)
        }
    }
}

impl FromStr for GameId {
    type Err = LauncherError;

    fn from_str(s: &str) -> Result<Self> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Err(LauncherError::InvalidInput("empty game id".into()));
        }

        // Support unified / canonical game IDs (e.g. "unified_cyberpunk2077", "canonical_cyberpunk2077")
        if let Some(key) = trimmed.strip_prefix("unified_") {
            if key.is_empty() {
                return Err(LauncherError::InvalidInput(format!("malformed game id `{s}`")));
            }
            return Ok(Self::new(PlatformKind::Canonical, key));
        }
        if let Some(key) = trimmed.strip_prefix("canonical_") {
            if key.is_empty() {
                return Err(LauncherError::InvalidInput(format!("malformed game id `{s}`")));
            }
            return Ok(Self::new(PlatformKind::Canonical, key));
        }

        if let Some(pos) = trimmed.find(':') {
            let platform_str = &trimmed[..pos];
            let native = &trimmed[pos + 1..];
            if native.is_empty() {
                return Err(LauncherError::InvalidInput(format!("malformed game id `{s}`")));
            }
            let kind = platform_str.parse::<PlatformKind>()?;
            Ok(Self::new(kind, native))
        } else {
            Err(LauncherError::InvalidInput(format!("malformed game id `{s}`")))
        }
    }
}

impl Serialize for GameId {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for GameId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        raw.parse().map_err(serde::de::Error::custom)
    }
}

// ---------------------------------------------------------------------------
// Game model
// ---------------------------------------------------------------------------

/// Reference to an image: a local file (served through Tauri's asset protocol)
/// or a remote URL.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "lowercase")]
pub enum ArtRef {
    Local(PathBuf),
    Remote(String),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Artwork {
    /// Vertical 2:3 cover.
    pub poster: Option<ArtRef>,
    /// Wide hero/banner background.
    pub hero: Option<ArtRef>,
    /// Transparent logo.
    pub logo: Option<ArtRef>,
    pub icon: Option<ArtRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    pub id: GameId,
    pub platform: PlatformKind,
    pub native_id: String,
    pub title: String,
    pub installed: bool,
    pub install_dir: Option<PathBuf>,
    pub executable: Option<PathBuf>,
    pub size_bytes: Option<u64>,
    pub playtime_minutes: Option<u64>,
    pub last_played: Option<DateTime<Utc>>,
    pub art: Artwork,
    pub tags: Vec<String>,
}

impl Game {
    pub fn new(platform: PlatformKind, native_id: impl Into<String>, title: impl Into<String>) -> Self {
        let native_id = native_id.into();
        Self {
            id: GameId::new(platform, native_id.clone()),
            platform,
            native_id,
            title: title.into(),
            installed: false,
            install_dir: None,
            executable: None,
            size_bytes: None,
            playtime_minutes: None,
            last_played: None,
            art: Artwork::default(),
            tags: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum LaunchMethod {
    Uri { uri: String },
    Executable { path: PathBuf, args: Vec<String> },
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchHandle {
    pub game_id: GameId,
    pub method: LaunchMethod,
    /// Only known for direct executable launches (protocol launches are
    /// brokered by the store client).
    pub pid: Option<u32>,
    pub started_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileSync {
    pub platform: PlatformKind,
    pub account_id: Option<String>,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    /// Number of owned titles reported by the online service.
    pub owned_games: usize,
    pub synced_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// The connector trait
// ---------------------------------------------------------------------------

#[async_trait]
pub trait PlatformConnector: Send + Sync {
    fn kind(&self) -> PlatformKind;

    /// Cheap check whether the platform client is present on this machine.
    async fn is_available(&self) -> bool;

    /// Enumerate locally installed games. Must return
    /// [`LauncherError::PlatformNotFound`] when the client is absent so the
    /// registry can treat it as "nothing to show" rather than a failure.
    async fn detect_installed_games(&self) -> Result<Vec<Game>>;

    /// Launch a game by its platform-native id.
    async fn launch_game(&self, native_id: &str) -> Result<LaunchHandle>;

    /// Start downloading or installing a game by its platform-native id.
    async fn install_game(&self, native_id: &str) -> Result<LaunchHandle> {
        self.launch_game(native_id).await
    }

    /// Pull account data (playtime, avatar, ownership) from the online service.
    async fn sync_profile(&self) -> Result<ProfileSync>;
}

// ---------------------------------------------------------------------------
// Process and Launcher Automation Helpers
// ---------------------------------------------------------------------------

/// Check whether a specific process (e.g. "steam.exe") is currently running on the system.
pub async fn is_process_running(process_name: &str) -> bool {
    let name = process_name.to_lowercase();
    tokio::task::spawn_blocking(move || {
        #[cfg(windows)]
        {
            use std::process::Command;
            if let Ok(output) = Command::new("tasklist")
                .args(["/FI", &format!("IMAGENAME eq {name}"), "/NH"])
                .output()
            {
                let text = String::from_utf8_lossy(&output.stdout).to_lowercase();
                return text.contains(&name);
            }
        }
        false
    })
    .await
    .unwrap_or(false)
}

/// Ensure the platform's native client (Steam, Epic Games, itch.io) is running in the background.
/// If not running, launches it in silent/minimized background mode.
pub async fn ensure_launcher_running(platform: PlatformKind) -> Result<()> {
    match platform {
        PlatformKind::Steam => {
            if !is_process_running("steam.exe").await {
                if let Ok(steam_root) = crate::platforms::steam::SteamPlatform::locate_steam_root() {
                    let exe = steam_root.join("steam.exe");
                    if exe.exists() {
                        tracing::info!("Launching Steam in background (-silent)...");
                        let _ = launch_executable(
                            GameId::new(PlatformKind::Steam, "launcher"),
                            exe,
                            vec!["-silent".into()],
                            Some(steam_root),
                        )
                        .await;
                        tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
                    }
                }
            }
        }
        PlatformKind::Epic => {
            if !is_process_running("EpicGamesLauncher.exe").await {
                let candidates = [
                    PathBuf::from(r"C:\Program Files\Epic Games\Launcher\Portal\Binaries\Win64\EpicGamesLauncher.exe"),
                    PathBuf::from(r"C:\Program Files (x86)\Epic Games\Launcher\Portal\Binaries\Win64\EpicGamesLauncher.exe"),
                ];
                if let Some(exe) = candidates.into_iter().find(|p| p.exists()) {
                    tracing::info!("Launching Epic Games Launcher in background (-silent -minimized)...");
                    let dir = exe.parent().map(Path::to_path_buf);
                    let _ = launch_executable(
                        GameId::new(PlatformKind::Epic, "launcher"),
                        exe,
                        vec!["-silent".into(), "-minimized".into()],
                        dir,
                    )
                    .await;
                    tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
                }
            }
        }
        PlatformKind::Itch => {
            if !is_process_running("itch.exe").await {
                if let Some(app_data) = dirs::data_local_dir() {
                    let exe = app_data.join("itch").join("itch.exe");
                    if exe.exists() {
                        tracing::info!("Launching itch.io in background (--silent)...");
                        let _ = launch_executable(
                            GameId::new(PlatformKind::Itch, "launcher"),
                            exe,
                            vec!["--silent".into()],
                            None,
                        )
                        .await;
                        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
                    }
                }
            }
        }
        _ => {}
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Launch helpers shared by connectors
// ---------------------------------------------------------------------------

/// Hand a URI (`steam://`, `com.epicgames.launcher://`, `roblox://`) to the
/// OS protocol handler.
pub async fn launch_uri(game_id: GameId, uri: String) -> Result<LaunchHandle> {
    let target = uri.clone();
    tokio::task::spawn_blocking(move || open::that_detached(&target))
        .await?
        .map_err(|e| LauncherError::Launch(format!("could not open `{uri}`: {e}")))?;

    tracing::info!(%game_id, %uri, "launched via protocol");
    Ok(LaunchHandle {
        game_id,
        method: LaunchMethod::Uri { uri },
        pid: None,
        started_at: Utc::now(),
    })
}

/// Spawn a detached executable that outlives the launcher.
pub async fn launch_executable(
    game_id: GameId,
    path: PathBuf,
    args: Vec<String>,
    cwd: Option<PathBuf>,
) -> Result<LaunchHandle> {
    if !tokio::fs::try_exists(&path).await.unwrap_or(false) {
        return Err(LauncherError::Launch(format!(
            "executable `{}` does not exist",
            path.display()
        )));
    }

    let working_dir = cwd.or_else(|| path.parent().map(Path::to_path_buf));
    let (exe, argv) = (path.clone(), args.clone());

    let pid = tokio::task::spawn_blocking(move || -> Result<u32> {
        let mut cmd = std::process::Command::new(&exe);
        cmd.args(&argv)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        if let Some(dir) = working_dir {
            cmd.current_dir(dir);
        }
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const DETACHED_PROCESS: u32 = 0x0000_0008;
            const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
            cmd.creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP);
        }
        let child = cmd
            .spawn()
            .map_err(|e| LauncherError::Launch(format!("failed to start `{}`: {e}", exe.display())))?;
        // Dropping a std `Child` does not kill the process.
        Ok(child.id())
    })
    .await??;

    tracing::info!(%game_id, pid, exe = %path.display(), "launched executable");
    Ok(LaunchHandle {
        game_id,
        method: LaunchMethod::Executable { path, args },
        pid: Some(pid),
        started_at: Utc::now(),
    })
}

// ---------------------------------------------------------------------------
// Registry
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformFailure {
    pub platform: PlatformKind,
    pub error: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanReport {
    pub games: Vec<Game>,
    pub errors: Vec<PlatformFailure>,
    pub scanned_at: DateTime<Utc>,
}

pub struct PlatformRegistry {
    connectors: HashMap<PlatformKind, Arc<dyn PlatformConnector>>,
    cache: RwLock<Vec<Game>>,
}

impl PlatformRegistry {
    pub fn new(connectors: Vec<Arc<dyn PlatformConnector>>) -> Self {
        Self {
            connectors: connectors.into_iter().map(|c| (c.kind(), c)).collect(),
            cache: RwLock::new(Vec::new()),
        }
    }

    pub fn connector(&self, kind: PlatformKind) -> Result<Arc<dyn PlatformConnector>> {
        self.connectors
            .get(&kind)
            .cloned()
            .ok_or_else(|| LauncherError::InvalidInput(format!("platform `{kind}` is not enabled")))
    }

    /// Scan every platform concurrently. One broken platform never hides the
    /// results of the others.
    pub async fn scan_all(&self) -> ScanReport {
        let scans = self.connectors.values().cloned().map(|c| async move {
            let kind = c.kind();
            (kind, c.detect_installed_games().await)
        });

        let mut games = Vec::new();
        let mut errors = Vec::new();
        for (platform, result) in futures::future::join_all(scans).await {
            match result {
                Ok(found) => games.extend(found),
                Err(LauncherError::PlatformNotFound(_)) => {
                    tracing::debug!(%platform, "platform not installed; skipping");
                }
                Err(err) => {
                    tracing::warn!(%platform, error = %err, "platform scan failed");
                    errors.push(PlatformFailure {
                        platform,
                        error: err.to_string(),
                    });
                }
            }
        }

        games.sort_by_cached_key(|g| g.title.to_lowercase());
        *self.cache.write().await = games.clone();

        ScanReport {
            games,
            errors,
            scanned_at: Utc::now(),
        }
    }

    pub async fn cached_games(&self) -> Vec<Game> {
        self.cache.read().await.clone()
    }

    pub async fn resolve_canonical(&self, canonical_key: &str) -> Option<Game> {
        let cached = self.cache.read().await;
        let mut matches: Vec<Game> = cached
            .iter()
            .filter(|g| {
                crate::commands::platforms::normalize_game_title(&g.title) == canonical_key
                    || g.title.eq_ignore_ascii_case(canonical_key)
            })
            .cloned()
            .collect();

        if matches.is_empty() {
            return None;
        }

        matches.sort_by_key(|g| {
            let installed_rank = if g.installed { 0 } else { 1 };
            let plat_rank = match g.id.platform {
                PlatformKind::Steam => 1,
                PlatformKind::Epic => 2,
                PlatformKind::Itch => 3,
                PlatformKind::Emulator => 4,
                PlatformKind::Roblox => 5,
                PlatformKind::Canonical => 6,
            };
            (installed_rank, plat_rank)
        });

        Some(matches[0].clone())
    }

    pub async fn find(&self, id: &GameId) -> Option<Game> {
        if id.platform == PlatformKind::Canonical {
            self.resolve_canonical(&id.native_id).await
        } else {
            self.cache.read().await.iter().find(|g| &g.id == id).cloned()
        }
    }

    pub async fn launch(&self, id: &GameId) -> Result<LaunchHandle> {
        let resolved = if id.platform == PlatformKind::Canonical {
            let g = self
                .resolve_canonical(&id.native_id)
                .await
                .ok_or_else(|| LauncherError::GameNotFound(format!("canonical id `{}`", id.native_id)))?;
            g.id
        } else {
            id.clone()
        };

        let _ = ensure_launcher_running(resolved.platform).await;
        let game = self.find(&resolved).await;
        let connector = self.connector(resolved.platform)?;
        if let Some(g) = game {
            if !g.installed {
                tracing::info!(%resolved, "game is not installed; delegating to platform installer");
                return connector.install_game(&resolved.native_id).await;
            }
        }
        connector.launch_game(&resolved.native_id).await
    }

    pub async fn install(&self, id: &GameId) -> Result<LaunchHandle> {
        let resolved = if id.platform == PlatformKind::Canonical {
            let g = self
                .resolve_canonical(&id.native_id)
                .await
                .ok_or_else(|| LauncherError::GameNotFound(format!("canonical id `{}`", id.native_id)))?;
            g.id
        } else {
            id.clone()
        };

        let _ = ensure_launcher_running(resolved.platform).await;
        self.connector(resolved.platform)?.install_game(&resolved.native_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_id_roundtrip() {
        let id: GameId = "roblox:place:920587237".parse().unwrap();
        assert_eq!(id.platform, PlatformKind::Roblox);
        assert_eq!(id.native_id, "place:920587237");
        assert_eq!(id.to_string(), "roblox:place:920587237");
        assert!("steam:".parse::<GameId>().is_err());
        assert!("nope".parse::<GameId>().is_err());

        let id_canon: GameId = "unified_cyberpunk2077".parse().unwrap();
        assert_eq!(id_canon.platform, PlatformKind::Canonical);
        assert_eq!(id_canon.native_id, "cyberpunk2077");
        assert_eq!(id_canon.to_string(), "unified_cyberpunk2077");
    }
}
