//! Emulator & ROM library connector (`EmulatorPlatform`).
//!
//! Scans configured ROM directories for standard disc/cartridge formats
//! (`.iso`, `.chd`, `.nsp`, `.rvz`, `.gba`, `.nds`, `.sfc`) and builds CLI
//! invocations for RetroArch, Dolphin, PCSX2, RPCS3, and Ryujinx.

use std::path::PathBuf;

use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::core::platform::{
    launch_executable, Game, GameId, LaunchHandle, PlatformConnector, PlatformKind, ProfileSync,
};
use crate::error::{LauncherError, Result};

pub const ROM_EXTENSIONS: &[&str] = &["iso", "chd", "nsp", "xci", "rvz", "gba", "nds", "sfc", "z64"];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmulatorProfile {
    pub id: String,
    pub name: String,
    pub executable: PathBuf,
    pub args_template: Vec<String>,
    pub rom_directories: Vec<PathBuf>,
}

pub struct EmulatorPlatform {
    profiles: Vec<EmulatorProfile>,
}

impl EmulatorPlatform {
    pub fn new(profiles: Vec<EmulatorProfile>) -> Self {
        Self { profiles }
    }

    /// Pre-configured CLI launch templates for RetroArch, RPCS3, PCSX2, Dolphin,
    /// Ryujinx, and PPSSPP.
    pub fn default_presets() -> Vec<EmulatorProfile> {
        vec![
            EmulatorProfile {
                id: "retroarch".into(),
                name: "RetroArch".into(),
                executable: PathBuf::from("retroarch.exe"),
                args_template: vec!["-f".into(), "{rom}".into()],
                rom_directories: Vec::new(),
            },
            EmulatorProfile {
                id: "dolphin".into(),
                name: "Dolphin (GC/Wii)".into(),
                executable: PathBuf::from("Dolphin.exe"),
                args_template: vec!["-b".into(), "-e".into(), "{rom}".into()],
                rom_directories: Vec::new(),
            },
            EmulatorProfile {
                id: "pcsx2".into(),
                name: "PCSX2 (PS2)".into(),
                executable: PathBuf::from("pcsx2-qt.exe"),
                args_template: vec!["-fullscreen".into(), "-batch".into(), "--".into(), "{rom}".into()],
                rom_directories: Vec::new(),
            },
            EmulatorProfile {
                id: "rpcs3".into(),
                name: "RPCS3 (PS3)".into(),
                executable: PathBuf::from("rpcs3.exe"),
                args_template: vec!["--no-gui".into(), "{rom}".into()],
                rom_directories: Vec::new(),
            },
            EmulatorProfile {
                id: "ryujinx".into(),
                name: "Ryujinx (Switch)".into(),
                executable: PathBuf::from("Ryujinx.exe"),
                args_template: vec!["--fullscreen".into(), "{rom}".into()],
                rom_directories: Vec::new(),
            },
            EmulatorProfile {
                id: "ppsspp".into(),
                name: "PPSSPP (PSP)".into(),
                executable: PathBuf::from("PPSSPPWindows64.exe"),
                args_template: vec!["--fullscreen".into(), "{rom}".into()],
                rom_directories: Vec::new(),
            },
        ]
    }

    /// Compute MD5 + SHA-256 hashes of a ROM file for RetroAchievements lookup.
    pub async fn hash_rom_file(path: PathBuf) -> Result<(String, String)> {
        use md5::Md5;
        use sha2::{Digest, Sha256};
        use std::io::Read;

        tokio::task::spawn_blocking(move || -> Result<(String, String)> {
            let mut file = std::fs::File::open(&path).map_err(|source| LauncherError::Io {
                path: path.clone(),
                source,
            })?;
            let mut md5_hasher = Md5::new();
            let mut sha_hasher = Sha256::new();
            let mut buf = vec![0u8; 64 * 1024];
            // Hash up to first 64 MiB for instant response on multi-GB ISOs.
            let mut remaining = 64 * 1024 * 1024usize;
            while remaining > 0 {
                let n = file.read(&mut buf).map_err(|source| LauncherError::Io {
                    path: path.clone(),
                    source,
                })?;
                if n == 0 {
                    break;
                }
                md5_hasher.update(&buf[..n]);
                sha_hasher.update(&buf[..n]);
                remaining = remaining.saturating_sub(n);
            }
            Ok((
                hex::encode(md5_hasher.finalize()),
                hex::encode(sha_hasher.finalize()),
            ))
        })
        .await?
    }
}

#[async_trait]
impl PlatformConnector for EmulatorPlatform {
    fn kind(&self) -> PlatformKind {
        PlatformKind::Emulator
    }

    async fn is_available(&self) -> bool {
        !self.profiles.is_empty()
    }

    async fn detect_installed_games(&self) -> Result<Vec<Game>> {
        if self.profiles.is_empty() {
            return Err(LauncherError::PlatformNotFound("Emulators"));
        }
        let mut games = Vec::new();
        for profile in &self.profiles {
            for dir in &profile.rom_directories {
                let Ok(entries) = std::fs::read_dir(dir) else {
                    continue;
                };
                for entry in entries.flatten() {
                    let path = entry.path();
                    let Some(ext) = path
                        .extension()
                        .and_then(|e| e.to_str())
                        .map(|s| s.to_ascii_uppercase())
                    else {
                        continue;
                    };
                    if !ROM_EXTENSIONS.iter().any(|k| k.eq_ignore_ascii_case(&ext)) {
                        continue;
                    }
                    let title = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("ROM")
                        .to_string();
                    let native_id = format!("{}::{}", profile.id, path.display());
                    let mut game = Game::new(PlatformKind::Emulator, native_id, title);
                    game.installed = true;
                    game.executable = Some(profile.executable.clone());
                    game.install_dir = Some(path);
                    game.tags = vec![profile.name.clone(), ext];
                    games.push(game);
                }
            }
        }
        Ok(games)
    }

    async fn launch_game(&self, native_id: &str) -> Result<LaunchHandle> {
        let (profile_id, rom_path) = native_id
            .split_once("::")
            .ok_or_else(|| LauncherError::InvalidInput(format!("invalid ROM id `{native_id}`")))?;
        let profile = self
            .profiles
            .iter()
            .find(|p| p.id == profile_id)
            .ok_or_else(|| LauncherError::GameNotFound(native_id.into()))?;

        let args: Vec<String> = if profile.args_template.is_empty() {
            vec![rom_path.to_string()]
        } else {
            profile
                .args_template
                .iter()
                .map(|a| a.replace("{rom}", rom_path))
                .collect()
        };

        launch_executable(
            GameId::new(PlatformKind::Emulator, native_id),
            profile.executable.clone(),
            args,
            None,
        )
        .await
    }

    async fn sync_profile(&self) -> Result<ProfileSync> {
        Ok(ProfileSync {
            platform: PlatformKind::Emulator,
            account_id: None,
            display_name: Some("RetroAchievements / Local ROMs".into()),
            avatar_url: None,
            owned_games: 0,
            synced_at: Utc::now(),
        })
    }
}
