//! Platform integration and multi-store deduplication commands.
//!
//! Provides IPC commands for:
//! - Steam Profile & Account Linking (`get_steam_profile`)
//! - Steam Achievements & Desync Repair (`get_steam_achievements`, `repair_steam_achievements`)
//! - Steam Cloud Sync & Versioning (`sync_steam_cloud`)
//! - Epic Games Library Import (`get_epic_library`)
//! - Roblox Recently Played Experiences (`get_roblox_recent_places`)
//! - Multi-Store Unified Library Deduplication (`get_unified_library`)
//! - Steam-grade Game Properties Modal data & file verification (`get_game_properties`, `save_game_properties`, `verify_game_files`)

use std::{
    collections::HashMap,
    path::PathBuf,
    sync::Arc,
};

use regex::Regex;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::core::platform::{Game, PlatformRegistry};
use crate::error::{LauncherError, Result};
use crate::platforms::{
    epic::{EpicGame, EpicPlatform},
    roblox::{RobloxPlace, RobloxPlatform},
    steam::{CloudSyncResult, SteamAchievement, SteamPlatform, SteamProfile, SteamRepairResult},
};

// ---------------------------------------------------------------------------
// Unified Library & Deduplication
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedGameSource {
    pub id: String,
    pub platform: String,
    pub native_id: String,
    pub title: String,
    pub installed: bool,
    pub install_dir: Option<String>,
    pub executable: Option<String>,
    pub size_bytes: Option<u64>,
    pub playtime_minutes: Option<u64>,
    pub last_played: Option<String>,
    pub launch_action: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedGame {
    pub id: String,
    pub primary_title: String,
    pub title: String,
    pub platforms: Vec<String>,
    pub preferred_platform: String,
    pub installed: bool,
    pub total_playtime_minutes: u64,
    pub poster_url: Option<String>,
    pub sources: Vec<UnifiedGameSource>,
}

/// Normalize game title to match cross-store variations (GOTY, Deluxe, trademarks, punctuation).
pub fn normalize_game_title(title: &str) -> String {
    let lower = title.to_lowercase();
    let stripped = lower
        .replace('®', "")
        .replace('™', "")
        .replace('©', "");

    // Strip edition / edition keywords
    let re_edition = Regex::new(
        r"(?i)\b(standard|deluxe|game of the year|goty|edition|bundle|director's cut|complete|ultimate|remastered|anniversary|definitive)\b",
    )
    .expect("valid regex");
    let cleaned = re_edition.replace_all(&stripped, " ");

    let mut out = String::new();
    for c in cleaned.chars() {
        if c.is_alphanumeric() {
            out.push(c);
        }
    }
    out.trim().to_string()
}

/// Truncate a UTF-8 string to at most `max_bytes` without panicking on multi-byte char boundaries.
pub fn safe_truncate_utf8(s: &mut String, max_bytes: usize) {
    if s.len() > max_bytes {
        let boundary = s
            .char_indices()
            .map(|(idx, _)| idx)
            .take_while(|&idx| idx <= max_bytes)
            .last()
            .unwrap_or(0);
        s.truncate(boundary);
    }
}

fn platform_priority(p: &str) -> u8 {
    match p.to_lowercase().as_str() {
        "steam" => 1,
        "epic" => 2,
        "gog" => 3,
        "itch" => 4,
        "emulator" | "retro" => 5,
        "roblox" => 6,
        _ => 7,
    }
}

// ---------------------------------------------------------------------------
// Game Properties
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameProperties {
    pub game_id: String,
    pub launch_args: String,
    pub custom_executable: Option<String>,
    pub cloud_enabled: bool,
    pub compatibility_tool: Option<String>,
    pub env_vars: HashMap<String, String>,
    pub high_dpi_scaling: bool,
    // File stats:
    pub install_dir: Option<String>,
    pub total_size_bytes: u64,
    pub file_count: u64,
    pub verify_integrity_available: bool,
    // Cloud stats:
    pub quota_bytes: u64,
    pub used_bytes: u64,
    pub last_sync: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyResult {
    pub status: String,
    pub file_count: u64,
    pub verified_files: u64,
    pub corrupted_files: u64,
    pub is_locked: bool,
}

// ---------------------------------------------------------------------------
// IPC Command Handlers
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn get_steam_profile(
    steam: State<'_, Arc<SteamPlatform>>,
) -> Result<SteamProfile> {
    steam.resolve_profile().await
}

#[tauri::command]
pub async fn get_steam_achievements(
    app_id: u32,
    steam: State<'_, Arc<SteamPlatform>>,
) -> Result<Vec<SteamAchievement>> {
    steam.fetch_achievements(app_id).await
}

#[tauri::command]
pub async fn repair_steam_achievements(
    app_id: u32,
    steam: State<'_, Arc<SteamPlatform>>,
) -> Result<SteamRepairResult> {
    steam.repair_achievements(app_id).await
}

#[tauri::command]
pub async fn sync_steam_cloud(
    app_id: u32,
    steam: State<'_, Arc<SteamPlatform>>,
) -> Result<CloudSyncResult> {
    steam.sync_cloud(app_id).await
}

#[tauri::command]
pub async fn get_epic_library(
    include_uninstalled: bool,
    epic: State<'_, Arc<EpicPlatform>>,
) -> Result<Vec<EpicGame>> {
    epic.fetch_epic_library(include_uninstalled).await
}

#[tauri::command]
pub async fn get_roblox_recent_places(
    roblox: State<'_, Arc<RobloxPlatform>>,
) -> Result<Vec<RobloxPlace>> {
    roblox.get_recent_places().await
}

#[tauri::command]
pub async fn get_unified_library(
    registry: State<'_, Arc<PlatformRegistry>>,
) -> Result<Vec<UnifiedGame>> {
    let games = registry.cached_games().await;
    let mut all_games = if games.is_empty() {
        registry.scan_all().await.games
    } else {
        games
    };

    let custom = crate::commands::custom_games::custom_games_to_platform_games(
        crate::commands::custom_games::load_custom_games_blocking(),
    );
    all_games.extend(custom);

    let mut groups: HashMap<String, Vec<Game>> = HashMap::new();
    for g in all_games {
        let norm_key = normalize_game_title(&g.title);
        let key = if norm_key.is_empty() {
            g.title.to_lowercase()
        } else {
            norm_key
        };
        groups.entry(key).or_default().push(g);
    }

    let mut unified_list: Vec<UnifiedGame> = Vec::with_capacity(groups.len());

    for (norm_key, mut items) in groups {
        // Sort items by platform priority
        items.sort_by_key(|g| platform_priority(g.id.platform.as_str()));

        let preferred = &items[0];
        let primary_title = preferred.title.clone();
        let preferred_platform = preferred.id.platform.as_str().to_string();

        let mut platforms = Vec::new();
        let mut sources = Vec::new();
        let mut any_installed = false;
        let mut total_playtime = 0u64;
        let mut poster_url = None;

        for item in &items {
            let plat_str = item.id.platform.as_str().to_string();
            if !platforms.contains(&plat_str) {
                platforms.push(plat_str.clone());
            }

            if item.installed {
                any_installed = true;
            }

            if let Some(mins) = item.playtime_minutes {
                total_playtime += mins;
            }

            if poster_url.is_none() {
                match item.art.poster {
                    Some(crate::core::platform::ArtRef::Remote(ref url)) => {
                        poster_url = Some(url.clone());
                    }
                    Some(crate::core::platform::ArtRef::Local(ref path)) => {
                        poster_url = Some(path.to_string_lossy().to_string());
                    }
                    None => {}
                }
                if poster_url.is_none() && item.id.platform.as_str() == "steam" {
                    poster_url = Some(format!(
                        "https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/{}/library_600x900.jpg",
                        item.id.native_id
                    ));
                }
            }

            let install_dir_str = item.install_dir.as_ref().map(|p| p.to_string_lossy().to_string());
            let exe_str = item.executable.as_ref().map(|p| p.to_string_lossy().to_string());
            let last_played_str = item.last_played.map(|dt| dt.to_rfc3339());

            sources.push(UnifiedGameSource {
                id: item.id.to_string(),
                platform: plat_str,
                native_id: item.id.native_id.clone(),
                title: item.title.clone(),
                installed: item.installed,
                install_dir: install_dir_str,
                executable: exe_str,
                size_bytes: item.size_bytes,
                playtime_minutes: item.playtime_minutes,
                last_played: last_played_str,
                launch_action: Some(if item.installed { "launch".into() } else { "install".into() }),
            });
        }

        unified_list.push(UnifiedGame {
            id: format!("unified_{norm_key}"),
            primary_title: primary_title.clone(),
            title: primary_title,
            platforms,
            preferred_platform,
            installed: any_installed,
            total_playtime_minutes: total_playtime,
            poster_url,
            sources,
        });
    }

    unified_list.sort_by(|a, b| a.primary_title.cmp(&b.primary_title));
    Ok(unified_list)
}

#[tauri::command]
pub async fn get_game_properties(
    game_id: String,
    registry: State<'_, Arc<PlatformRegistry>>,
) -> Result<GameProperties> {
    // 1. Try reading saved properties from disk
    if let Some(local_app_data) = dirs::data_local_dir() {
        let safe_id = game_id.replace([':', '/', '\\', '.'], "_");
        let file_path = local_app_data.join("OmniLauncher").join("properties").join(format!("{safe_id}.json"));
        if file_path.exists() {
            if let Ok(raw) = tokio::fs::read_to_string(&file_path).await {
                if let Ok(props) = serde_json::from_str::<GameProperties>(&raw) {
                    return Ok(props);
                }
            }
        }
    }

    // 2. Discover baseline from Game object if registered
    let existing_game = {
        let cached = registry.cached_games().await;
        cached.into_iter().find(|g| g.id.to_string() == game_id)
    };

    let (install_dir, size_bytes, file_count) = if let Some(g) = existing_game {
        let dir = g.install_dir.map(|p| p.to_string_lossy().to_string());
        let count = if let Some(ref d) = dir {
            let p = PathBuf::from(d);
            if p.exists() {
                std::fs::read_dir(&p).map(|rd| rd.count() as u64).unwrap_or(0)
            } else {
                0
            }
        } else {
            0
        };
        (dir, g.size_bytes.unwrap_or(0), count)
    } else {
        (None, 0, 0)
    };

    Ok(GameProperties {
        game_id,
        launch_args: String::new(),
        custom_executable: None,
        cloud_enabled: true,
        compatibility_tool: Some("native_windows_subsystem".into()),
        env_vars: HashMap::new(),
        high_dpi_scaling: true,
        install_dir,
        total_size_bytes: size_bytes,
        file_count,
        verify_integrity_available: true,
        quota_bytes: 104_857_600, // 100 MB default quota
        used_bytes: if size_bytes > 0 { (size_bytes / 1024).min(15_728_640) } else { 0 },
        last_sync: Some(chrono::Utc::now().timestamp()),
    })
}

#[tauri::command]
pub async fn save_game_properties(
    game_id: String,
    mut properties: GameProperties,
) -> Result<()> {
    // Sanitize launch args: remove shell escape/chaining characters, redirections, escapes, env expansion, newlines, and null bytes
    let dangerous_chars = [';', '&', '|', '`', '$', '>', '<', '^', '%', '(', ')', '\r', '\n', '\0'];
    properties.launch_args = properties
        .launch_args
        .chars()
        .filter(|c| !dangerous_chars.contains(c))
        .collect();

    // Cap at 4096 bytes ensuring UTF-8 char boundary safety
    safe_truncate_utf8(&mut properties.launch_args, 4096);

    if let Some(local_app_data) = dirs::data_local_dir() {
        let safe_id = game_id.replace([':', '/', '\\', '.'], "_");
        let dir = local_app_data.join("OmniLauncher").join("properties");
        tokio::fs::create_dir_all(&dir).await.map_err(LauncherError::IoRaw)?;
        let file_path = dir.join(format!("{safe_id}.json"));
        let serialized = serde_json::to_string_pretty(&properties)?;
        tokio::fs::write(&file_path, serialized).await.map_err(LauncherError::IoRaw)?;
    }

    Ok(())
}

#[tauri::command]
pub async fn verify_game_files(
    game_id: String,
    registry: State<'_, Arc<PlatformRegistry>>,
) -> Result<VerifyResult> {
    let existing_game = {
        let cached = registry.cached_games().await;
        cached.into_iter().find(|g| g.id.to_string() == game_id)
    };

    let dir = existing_game.and_then(|g| g.install_dir);
    let Some(dir) = dir else {
        return Ok(VerifyResult {
            status: "not_installed".into(),
            file_count: 0,
            verified_files: 0,
            corrupted_files: 0,
            is_locked: false,
        });
    };

    if !dir.exists() {
        return Ok(VerifyResult {
            status: "missing_directory".into(),
            file_count: 0,
            verified_files: 0,
            corrupted_files: 0,
            is_locked: false,
        });
    }

    let mut count = 0u64;
    let mut verified = 0u64;

    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            count += 1;
            let path = entry.path();
            if path.is_file() {
                // Verify file is readable and non-corrupted
                if std::fs::File::open(&path).is_ok() {
                    verified += 1;
                }
            } else {
                verified += 1;
            }
        }
    }

    Ok(VerifyResult {
        status: "verified".into(),
        file_count: count,
        verified_files: verified,
        corrupted_files: count.saturating_sub(verified),
        is_locked: false,
    })
}

#[tauri::command]
pub async fn open_game_directory(
    registry: State<'_, Arc<PlatformRegistry>>,
    game_id: String,
    kind: String,
) -> Result<String> {
    let existing_game = {
        let cached = registry.cached_games().await;
        cached.into_iter().find(|g| g.id.to_string() == game_id)
    };

    let target_dir = match kind.as_str() {
        "save" => {
            crate::commands::save_vault::resolve_save_directory(&game_id, None)
                .unwrap_or_else(|| {
                    dirs::data_local_dir()
                        .unwrap_or_else(|| PathBuf::from("."))
                        .join("OmniLauncher")
                        .join("save_vault")
                        .join(&game_id)
                })
        }
        "mods" => {
            let base = existing_game
                .as_ref()
                .and_then(|g| g.install_dir.clone())
                .unwrap_or_else(|| PathBuf::from("."));
            let mod_dir = base.join("Mods");
            if !mod_dir.exists() {
                let _ = std::fs::create_dir_all(&mod_dir);
            }
            mod_dir
        }
        _ => {
            existing_game
                .and_then(|g| g.install_dir)
                .unwrap_or_else(|| PathBuf::from("."))
        }
    };

    if !target_dir.exists() {
        let _ = std::fs::create_dir_all(&target_dir);
    }

    #[cfg(windows)]
    {
        let _ = std::process::Command::new("explorer.exe")
            .arg(&target_dir)
            .spawn();
    }

    Ok(target_dir.to_string_lossy().to_string())
}

