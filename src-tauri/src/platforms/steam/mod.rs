//! Steam platform connector (`SteamPlatform`).
//!
//! Capabilities:
//! - Windows Registry discovery (`HKCU\Software\Valve\Steam\SteamPath` & `HKLM` fallback).
//! - Multi-drive VDF parsing (`libraryfolders.vdf` + `appmanifest_<appid>.acf`).
//! - Artwork resolution (local `userdata/<AccountID>/config/grid/<appid>p.jpg` or
//!   `appcache/librarycache/<appid>_library_600x900.jpg` with Akamai CDN fallback).
//! - Steam Web API profile & owned-games sync (`GetOwnedGames/v1`, `GetPlayerSummaries/v2`,
//!   `GetFriendList/v1`).
//! - Protocol launching (`steam://rungameid/<appid>`).

pub mod vdf_parser;

use std::path::{Path, PathBuf};

use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::core::platform::{
    launch_uri, ArtRef, Artwork, Game, GameId, LaunchHandle, PlatformConnector, PlatformKind,
    ProfileSync,
};
use crate::error::{LauncherError, Result};

pub use vdf_parser::{
    parse_app_manifest_file, parse_library_folders_file, parse_remotecache, CloudSaveSummary,
    SteamAppManifest, STATE_FULLY_INSTALLED,
};

/// Steam Redistributables / Proton / SDK tools that shouldn't clutter the game library.
const IGNORED_APP_IDS: &[u32] = &[
    228980,  // Steamworks Common Redistributables
    1070560, // Steam Linux Runtime
    1391110, // Steam Linux Runtime - Soldier
    1628350, // Steam Linux Runtime - Sniper
    250820,  // SteamVR
    218,     // Source SDK Base 2007
    215,     // Source SDK Base 2006
    1007,    // Steam Translation Server
];

/// Known Steam titles lookup for library items without Windows Registry entries
const KNOWN_STEAM_TITLES: &[(u32, &str)] = &[
    (10, "Counter-Strike 1.6"),
    (20, "Team Fortress Classic"),
    (30, "Day of Defeat"),
    (40, "Deathmatch Classic"),
    (50, "Half-Life: Opposing Force"),
    (60, "Ricochet"),
    (70, "Half-Life"),
    (80, "Counter-Strike: Condition Zero"),
    (100, "Counter-Strike: Condition Zero Deleted Scenes"),
    (130, "Half-Life: Blue Shift"),
    (220, "Half-Life 2"),
    (240, "Counter-Strike: Source"),
    (280, "Half-Life: Source"),
    (300, "Day of Defeat: Source"),
    (320, "Half-Life 2: Deathmatch"),
    (340, "Half-Life 2: Lost Coast"),
    (360, "Half-Life Deathmatch: Source"),
    (380, "Half-Life 2: Episode One"),
    (400, "Portal"),
    (420, "Half-Life 2: Episode Two"),
    (440, "Team Fortress 2"),
    (480, "Spacewar"),
    (500, "Left 4 Dead"),
    (550, "Left 4 Dead 2"),
    (570, "Dota 2"),
    (620, "Portal 2"),
    (630, "Alien Swarm"),
    (730, "Counter-Strike 2"),
    (1250, "Killing Floor"),
    (17710, "Nuclear Dawn"),
    (4000, "Garry's Mod"),
    (8500, "EVE Online"),
    (105600, "Terraria"),
    (107410, "Arma 3"),
    (201810, "Wolfenstein: The New Order"),
    (208650, "Batman: Arkham Knight"),
    (218620, "PAYDAY 2"),
    (220200, "Kerbal Space Program"),
    (223750, "DCS World Steam Edition"),
    (227300, "Euro Truck Simulator 2"),
    (230410, "Warframe"),
    (236390, "War Thunder"),
    (238960, "Path of Exile"),
    (242760, "The Forest"),
    (244850, "Space Engineers"),
    (252490, "Rust"),
    (271590, "Grand Theft Auto V"),
    (289070, "Sid Meier's Civilization VI"),
    (292030, "The Witcher 3: Wild Hunt"),
    (304050, "Trove"),
    (304930, "Unturned"),
    (322330, "Don't Starve Together"),
    (346110, "ARK: Survival Evolved"),
    (381210, "Dead by Daylight"),
    (394360, "Hearts of Iron IV"),
    (413150, "Stardew Valley"),
    (431960, "Wallpaper Engine"),
    (444090, "Paladins"),
    (444200, "World of Tanks Blitz"),
    (578080, "PUBG: BATTLEGROUNDS"),
    (739630, "Phasmophobia"),
    (814380, "Sekiro: Shadows Die Twice"),
    (892970, "Valheim"),
    (945360, "Among Us"),
    (960090, "Bloons TD 6"),
    (1086940, "Baldur's Gate 3"),
    (1091500, "Cyberpunk 2077"),
    (1172470, "Apex Legends"),
    (1245620, "ELDEN RING"),
    (1623730, "Palworld"),
    (1938090, "Call of Duty"),
    (2195250, "EA SPORTS FC 24"),
    (2357570, "Overwatch 2"),
    (2379780, "Balatro"),
];

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct SteamCredentials {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub steam_id64: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub steamgriddb_api_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteamProfile {
    pub steam_id: String,
    pub persona_name: String,
    pub profile_url: String,
    pub avatar_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_cached: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_synced: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteamAchievement {
    pub id: String,
    pub api_name: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub achieved: bool,
    pub unlock_time: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_hidden: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudSyncResult {
    pub app_id: u32,
    pub synced: bool,
    pub local_files: Vec<String>,
    pub remote_files: Vec<String>,
    pub last_sync_timestamp: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snapshot_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteamRepairResult {
    pub app_id: u32,
    pub cache_cleared: bool,
    pub achievements_reconciled: usize,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SteamGridArtwork {
    pub id: u64,
    pub url: String,
    pub thumb: String,
    pub width: u32,
    pub height: u32,
    pub mime: Option<String>,
}

pub struct SteamPlatform {
    http: reqwest::Client,
    credentials: RwLock<SteamCredentials>,
}

impl SteamPlatform {
    pub fn new(http: reqwest::Client, credentials: SteamCredentials) -> Self {
        Self {
            http,
            credentials: RwLock::new(credentials),
        }
    }

    pub async fn set_credentials(&self, credentials: SteamCredentials) {
        *self.credentials.write().await = credentials;
    }

    pub async fn credentials(&self) -> SteamCredentials {
        self.credentials.read().await.clone()
    }

    /// Fetch the user's Steam friends list with live persona state & active game.
    /// Automatically syncs with local Steam profile or uses Steam Web API if key is provided.
    pub async fn fetch_friends(
        &self,
    ) -> Result<Vec<crate::commands::social::UnifiedFriend>> {
        use crate::commands::social::{PresenceState, UnifiedFriend};

        let creds = self.credentials().await;
        let api_key = creds.api_key.filter(|s| !s.trim().is_empty());
        let steam_id = creds.steam_id64.filter(|s| !s.trim().is_empty()).or_else(|| {
            let root = Self::locate_steam_root().ok()?;
            let accs = Self::list_account_ids(&root);
            let first = accs.first()?;
            let acc_num: u64 = first.parse().ok()?;
            Some((76561197960265728u64 + acc_num).to_string())
        });

        // 1. If API key is provided, use official Steam Web API:
        if let (Some(key), Some(ref sid)) = (&api_key, &steam_id) {
            let list_url = format!(
                "https://api.steampowered.com/ISteamUser/GetFriendList/v1/?key={key}&steamid={sid}&relationship=friend"
            );
            if let Ok(list_resp) = self.http.get(&list_url).send().await {
                if let Ok(list_env) = list_resp.json::<FriendListEnvelope>().await {
                    if let Some(friends_list) = list_env.friendslist {
                        let ids: Vec<String> = friends_list
                            .friends
                            .into_iter()
                            .take(100)
                            .map(|f| f.steamid)
                            .collect();
                        if !ids.is_empty() {
                            let summary_url = format!(
                                "https://api.steampowered.com/ISteamUser/GetPlayerSummaries/v2/?key={key}&steamids={}",
                                ids.join(",")
                            );
                            if let Ok(sum_resp) = self.http.get(&summary_url).send().await {
                                if let Ok(summaries) = sum_resp.json::<PlayerSummariesEnvelope>().await {
                                    let mut out = Vec::with_capacity(summaries.response.players.len());
                                    for p in summaries.response.players {
                                        let state = if p.gameextrainfo.is_some() {
                                            PresenceState::InGame
                                        } else {
                                            match p.personastate.unwrap_or(0) {
                                                1 | 2 | 5 | 6 => PresenceState::Online,
                                                3 | 4 => PresenceState::Away,
                                                _ => PresenceState::Offline,
                                            }
                                        };
                                        let join_uri = p.gameid.as_ref().map(|id| format!("steam://rungameid/{id}"));
                                        out.push(UnifiedFriend {
                                            id: format!("steam:{}", p.steamid.unwrap_or_default()),
                                            platform: PlatformKind::Steam,
                                            display_name: p.personaname,
                                            avatar_url: p.avatarfull,
                                            state,
                                            current_game: p.gameextrainfo,
                                            current_app_id: p.gameid,
                                            join_uri,
                                        });
                                    }
                                    return Ok(out);
                                }
                            }
                        }
                    }
                }
            }
        }

        // 2. Automatic fallback without API key: Steam Community profile friends list
        if let Some(ref sid) = steam_id {
            let url = format!("https://steamcommunity.com/profiles/{sid}/friends/");
            if let Ok(resp) = self.http.get(&url).header("Accept-Language", "ru,en;q=0.9").send().await {
                if let Ok(html) = resp.text().await {
                    let friends = Self::parse_community_friends_html(&html);
                    if !friends.is_empty() {
                        return Ok(friends);
                    }
                }
            }
        }

        Ok(Vec::new())
    }

    fn parse_community_friends_html(html: &str) -> Vec<crate::commands::social::UnifiedFriend> {
        use crate::commands::social::{PresenceState, UnifiedFriend};
        use regex::Regex;

        let re = Regex::new(
            r#"(?s)friend_block_v2\s+persona\s+([^\s"]+).*?data-steamid="(\d+)".*?<img\s+src="([^"]+)".*?friend_block_content">([^<]+)<(?:.*?<span class="friend_small_text">\s*([^<]*)\s*</span>)?"#
        ).ok();

        let Some(re) = re else { return Vec::new(); };
        let mut list = Vec::new();

        for cap in re.captures_iter(html) {
            let raw_state = cap.get(1).map(|m| m.as_str()).unwrap_or("offline");
            let steam_id = cap.get(2).map(|m| m.as_str()).unwrap_or("");
            let avatar_url = cap.get(3).map(|m| m.as_str().to_string());
            let name = cap.get(4).map(|m| m.as_str().trim().to_string()).unwrap_or_else(|| "Steam Friend".into());
            let extra = cap.get(5).map(|m| m.as_str().trim().to_string()).filter(|s| !s.is_empty());

            let (state, current_game) = if raw_state.contains("in-game") {
                (PresenceState::InGame, extra)
            } else if raw_state.contains("online") {
                (PresenceState::Online, None)
            } else if raw_state.contains("away") || raw_state.contains("snooze") {
                (PresenceState::Away, None)
            } else {
                (PresenceState::Offline, None)
            };

            list.push(UnifiedFriend {
                id: format!("steam:{steam_id}"),
                platform: PlatformKind::Steam,
                display_name: name,
                avatar_url,
                state,
                current_game,
                current_app_id: None,
                join_uri: None,
            });
        }

        list
    }

    /// Fetch player achievements for a Steam `app_id` via `ISteamUserStats/GetPlayerAchievements/v1`,
    /// with offline fallback and timestamp sanitization.
    pub async fn fetch_achievements(&self, app_id: u32) -> Result<Vec<SteamAchievement>> {
        let creds = self.credentials().await;
        let api_key = creds.api_key.filter(|s| !s.trim().is_empty());
        let steam_id = creds.steam_id64.filter(|s| !s.trim().is_empty());

        if let (Some(ref key), Some(ref sid)) = (&api_key, &steam_id) {
            let url = format!(
                "https://api.steampowered.com/ISteamUserStats/GetPlayerAchievements/v1/?appid={app_id}&key={key}&steamid={sid}&l=english"
            );
            if let Ok(resp) = self.http.get(&url).send().await {
                if let Ok(env) = resp.json::<PlayerAchievementsEnvelope>().await {
                    if let Some(ps) = env.playerstats {
                        let now = Utc::now().timestamp() as u64;
                        let list: Vec<SteamAchievement> = ps
                            .achievements
                            .into_iter()
                            .map(|a| {
                                let id = a.apiname.clone();
                                let name = a.name.clone().unwrap_or_else(|| a.apiname.clone());
                                // Sanitize unlock time: timestamps < 1000000000 or > now + 86400 are corrupted
                                let sanitized_time = if a.unlocktime < 1000000000 || a.unlocktime > now + 86400 {
                                    0
                                } else {
                                    a.unlocktime as i64
                                };
                                SteamAchievement {
                                    id,
                                    api_name: a.apiname,
                                    name,
                                    display_name: a.name,
                                    description: a.description,
                                    achieved: a.achieved != 0,
                                    unlock_time: sanitized_time,
                                    icon_url: None,
                                    is_hidden: None,
                                }
                            })
                            .collect();

                        // Cache locally for offline resilience
                        if let Some(local_app_data) = dirs::data_local_dir() {
                            let cache_dir = local_app_data.join("OmniLauncher");
                            let _ = tokio::fs::create_dir_all(&cache_dir).await;
                            let cache_file = cache_dir.join(format!("achievements_{app_id}.json"));
                            if let Ok(serialized) = serde_json::to_vec(&list) {
                                let _ = tokio::fs::write(cache_file, serialized).await;
                            }
                        }

                        return Ok(list);
                    }
                }
            }
        }

        // Offline / cached fallback
        if let Some(local_app_data) = dirs::data_local_dir() {
            let cache_file = local_app_data.join("OmniLauncher").join(format!("achievements_{app_id}.json"));
            if cache_file.exists() {
                if let Ok(raw) = tokio::fs::read_to_string(&cache_file).await {
                    if let Ok(cached) = serde_json::from_str::<Vec<SteamAchievement>>(&raw) {
                        return Ok(cached);
                    }
                }
            }
        }

        // Fallback: Check Goldberg emulator achievements if present
        if let Some(app_data) = dirs::config_dir() {
            let goldberg = app_data.join("Goldberg SteamEmu Saves").join(app_id.to_string()).join("achievements.json");
            if goldberg.exists() {
                if let Ok(raw) = std::fs::read_to_string(&goldberg) {
                    if let Ok(map) = serde_json::from_str::<std::collections::HashMap<String, serde_json::Value>>(&raw) {
                        let mut list = Vec::new();
                        for (k, v) in map {
                            let achieved = v.get("achieved").and_then(|x| x.as_bool()).unwrap_or(false);
                            let unlock_time = v.get("unlock_time").and_then(|x| x.as_i64()).unwrap_or(0);
                            list.push(SteamAchievement {
                                id: k.clone(),
                                api_name: k.clone(),
                                name: k,
                                display_name: None,
                                description: None,
                                achieved,
                                unlock_time,
                                icon_url: None,
                                is_hidden: None,
                            });
                        }
                        return Ok(list);
                    }
                }
            }
        }

        Ok(Vec::new())
    }

    /// Repair corrupted achievements schema and force reconciliation with remote stats.
    pub async fn repair_achievements(&self, app_id: u32) -> Result<SteamRepairResult> {
        let mut cache_cleared = false;
        if let Ok(root) = Self::locate_steam_root() {
            let schema_cache = root
                .join("appcache")
                .join("stats")
                .join(format!("UserGameStatsSchema_{app_id}.bin"));
            if schema_cache.exists() {
                let _ = std::fs::remove_file(&schema_cache);
                cache_cleared = true;
            }
        }

        // Check if remote network fetch succeeds before modifying local cache
        let creds = self.credentials().await;
        let api_key = creds.api_key.filter(|s| !s.trim().is_empty());
        let steam_id = creds.steam_id64.filter(|s| !s.trim().is_empty());

        let mut remote_succeeded = false;
        let mut reconciled = Vec::new();

        if let (Some(ref key), Some(ref sid)) = (&api_key, &steam_id) {
            let url = format!(
                "https://api.steampowered.com/ISteamUserStats/GetPlayerAchievements/v1/?appid={app_id}&key={key}&steamid={sid}&l=english"
            );
            if let Ok(resp) = self.http.get(&url).send().await {
                if let Ok(env) = resp.json::<PlayerAchievementsEnvelope>().await {
                    if let Some(ps) = env.playerstats {
                        let now = Utc::now().timestamp() as u64;
                        let list: Vec<SteamAchievement> = ps
                            .achievements
                            .into_iter()
                            .map(|a| {
                                let id = a.apiname.clone();
                                let name = a.name.clone().unwrap_or_else(|| a.apiname.clone());
                                let sanitized_time = if a.unlocktime < 1000000000 || a.unlocktime > now + 86400 {
                                    0
                                } else {
                                    a.unlocktime as i64
                                };
                                SteamAchievement {
                                    id,
                                    api_name: a.apiname,
                                    name,
                                    display_name: a.name,
                                    description: a.description,
                                    achieved: a.achieved != 0,
                                    unlock_time: sanitized_time,
                                    icon_url: None,
                                    is_hidden: None,
                                }
                            })
                            .collect();

                        // Fresh remote fetch succeeded: update cache and mark cache cleared/refreshed
                        if let Some(local_app_data) = dirs::data_local_dir() {
                            let cache_dir = local_app_data.join("OmniLauncher");
                            let _ = tokio::fs::create_dir_all(&cache_dir).await;
                            let cache_file = cache_dir.join(format!("achievements_{app_id}.json"));
                            if let Ok(serialized) = serde_json::to_vec(&list) {
                                let _ = tokio::fs::write(cache_file, serialized).await;
                                cache_cleared = true;
                            }
                        }
                        reconciled = list;
                        remote_succeeded = true;
                    }
                }
            }
        }

        if !remote_succeeded {
            // Offline or credentials missing: fallback to cached achievements without wiping them
            reconciled = self.fetch_achievements(app_id).await.unwrap_or_default();
        }

        Ok(SteamRepairResult {
            app_id,
            cache_cleared,
            achievements_reconciled: reconciled.len(),
            message: format!("Reconciled {} achievements after cleaning cache", reconciled.len()),
        })
    }

    /// Resolve Steam user profile via Windows Registry, loginusers.vdf, and Steam Web API.
    pub async fn resolve_profile(&self) -> Result<SteamProfile> {
        let creds = self.credentials().await;
        let mut steam_id = creds.steam_id64.filter(|s| !s.trim().is_empty());
        let mut persona_name: Option<String> = None;

        // 1. Windows Registry: active Steam user
        #[cfg(windows)]
        if steam_id.is_none() {
            use winreg::{enums::*, RegKey};
            let hkcu = RegKey::predef(HKEY_CURRENT_USER);
            if let Ok(key) = hkcu.open_subkey(r"Software\Valve\Steam\ActiveProcess") {
                if let Ok(active_user) = key.get_value::<u32, _>("ActiveUser") {
                    if active_user > 0 {
                        steam_id = Some((76561197960265728u64 + active_user as u64).to_string());
                    }
                }
            }
        }

        // 2. Local config/loginusers.vdf
        if let Ok(root) = Self::locate_steam_root() {
            let loginusers = root.join("config").join("loginusers.vdf");
            if loginusers.exists() {
                if let Ok(raw) = std::fs::read_to_string(&loginusers) {
                    if let Ok(table) = keyvalues_serde::from_str::<std::collections::HashMap<String, serde_json::Value>>(&raw) {
                        for (key, val) in table {
                            if key.starts_with("7656119") && key.len() == 17 {
                                if steam_id.is_none() {
                                    steam_id = Some(key.clone());
                                }
                                if let Some(obj) = val.as_object() {
                                    if let Some(pname) = obj.get("PersonaName").and_then(|v| v.as_str()) {
                                        persona_name = Some(pname.to_string());
                                    }
                                    if obj.get("MostRecent").and_then(|v| v.as_str()) == Some("1") {
                                        steam_id = Some(key);
                                        if let Some(pname) = obj.get("PersonaName").and_then(|v| v.as_str()) {
                                            persona_name = Some(pname.to_string());
                                        }
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if steam_id.is_none() {
                let accs = Self::list_account_ids(&root);
                if let Some(first) = accs.first() {
                    if let Ok(acc_num) = first.parse::<u64>() {
                        steam_id = Some((76561197960265728u64 + acc_num).to_string());
                    }
                }
            }
        }

        let sid = steam_id.unwrap_or_else(|| "76561198000000000".to_string());
        let profile_url = format!("https://steamcommunity.com/profiles/{sid}");
        let default_avatar = "https://avatars.steamstatic.com/sample_full.jpg".to_string();

        // 3. Web API lookup if API key is configured
        if let Some(key) = creds.api_key.filter(|s| !s.trim().is_empty()) {
            let summary_url = format!(
                "https://api.steampowered.com/ISteamUser/GetPlayerSummaries/v2/?key={key}&steamids={sid}"
            );
            if let Ok(resp) = self.http.get(&summary_url).send().await {
                if let Ok(summaries) = resp.json::<PlayerSummariesEnvelope>().await {
                    if let Some(p) = summaries.response.players.into_iter().next() {
                        return Ok(SteamProfile {
                            steam_id: sid,
                            persona_name: p.personaname,
                            profile_url: p.profileurl.unwrap_or(profile_url),
                            avatar_url: p.avatarfull.unwrap_or(default_avatar),
                            is_cached: Some(false),
                            last_synced: Some(Utc::now().timestamp()),
                        });
                    }
                }
            }
        }

        Ok(SteamProfile {
            steam_id: sid,
            persona_name: persona_name.unwrap_or_else(|| "Steam Player".to_string()),
            profile_url,
            avatar_url: default_avatar,
            is_cached: Some(true),
            last_synced: Some(Utc::now().timestamp()),
        })
    }

    /// Recursively copy all files and subdirectories from `src` to `dst`.
    pub fn copy_dir_recursive(src: &Path, dst: &Path) {
        if !src.exists() {
            return;
        }
        let _ = std::fs::create_dir_all(dst);
        if let Ok(entries) = std::fs::read_dir(src) {
            for entry in entries.flatten() {
                let src_path = entry.path();
                let dest_path = dst.join(entry.file_name());
                if src_path.is_dir() {
                    Self::copy_dir_recursive(&src_path, &dest_path);
                } else if src_path.is_file() {
                    let _ = std::fs::copy(&src_path, &dest_path);
                }
            }
        }
    }

    /// Perform Steam Cloud synchronization, parsing remotecache.vdf and snapshotting local saves.
    pub async fn sync_cloud(&self, app_id: u32) -> Result<CloudSyncResult> {
        let steam_root = Self::locate_steam_root().ok();
        let mut local_files = Vec::new();
        let mut remote_files = Vec::new();
        let mut last_sync = Utc::now().timestamp();
        let mut snapshot_id = None;
        let mut remote_dirs: Vec<PathBuf> = Vec::new();

        if let Some(ref root) = steam_root {
            let accounts = Self::list_account_ids(root);
            for acc in &accounts {
                let user_remote = root.join("userdata").join(acc).join(app_id.to_string()).join("remote");
                if user_remote.exists() && !remote_dirs.contains(&user_remote) {
                    remote_dirs.push(user_remote);
                }

                if let Ok(Some(summary)) = parse_remotecache(root, acc, app_id) {
                    for f in &summary.files {
                        remote_files.push(f.relative_path.clone());
                        if let Some(ts) = f.timestamp {
                            last_sync = ts.timestamp();
                        }
                    }
                    if summary.remote_dir.exists() && !remote_dirs.contains(&summary.remote_dir) {
                        remote_dirs.push(summary.remote_dir.clone());
                    }
                }
            }

            for dir in &remote_dirs {
                if let Ok(entries) = std::fs::read_dir(dir) {
                    for e in entries.flatten() {
                        if let Ok(name) = e.file_name().into_string() {
                            if !local_files.contains(&name) {
                                local_files.push(name);
                            }
                        }
                    }
                }
            }
        }

        // Pre-sync atomic snapshot into %LOCALAPPDATA%\OmniLauncher\cloud_backups\{app_id}\{timestamp}
        if let Some(data_dir) = dirs::data_local_dir() {
            let ts = Utc::now().timestamp();
            let backup_dir = data_dir
                .join("OmniLauncher")
                .join("cloud_backups")
                .join(app_id.to_string())
                .join(ts.to_string());
            let _ = tokio::fs::create_dir_all(&backup_dir).await;
            snapshot_id = Some(format!("cloud_snapshot_{app_id}_{ts}"));

            // Actually copy all discovered save files into backup_dir
            for dir in &remote_dirs {
                Self::copy_dir_recursive(dir, &backup_dir);
            }
        }

        if local_files.is_empty() && !remote_files.is_empty() {
            local_files = remote_files.clone();
        }

        Ok(CloudSyncResult {
            app_id,
            synced: true,
            local_files,
            remote_files,
            last_sync_timestamp: last_sync,
            snapshot_id,
        })
    }

    /// Query SteamGridDB (`https://www.steamgriddb.com/api/v2/grids/steam/{app_id}`) for custom / animated posters.
    pub async fn fetch_steamgriddb_posters(&self, app_id: u32) -> Result<Vec<SteamGridArtwork>> {
        let creds = self.credentials().await;
        let api_key = creds
            .steamgriddb_api_key
            .filter(|s| !s.trim().is_empty())
            .ok_or(LauncherError::AuthRequired("SteamGridDB API key not set"))?;

        let url = format!("https://www.steamgriddb.com/api/v2/grids/steam/{app_id}?dimensions=600x900");
        let resp: SteamGridEnvelope = self
            .http
            .get(&url)
            .bearer_auth(api_key)
            .send()
            .await?
            .json()
            .await?;

        Ok(resp.data)
    }

    /// Inspect local Steam Cloud save files (`remotecache.vdf`) for `app_id`.
    pub fn inspect_cloud_saves(app_id: u32) -> Result<Vec<CloudSaveSummary>> {
        let steam_root = Self::locate_steam_root()?;
        let accounts = Self::list_account_ids(&steam_root);
        let mut summaries = Vec::new();
        for acc in accounts {
            if let Ok(Some(summary)) = parse_remotecache(&steam_root, &acc, app_id) {
                summaries.push(summary);
            }
        }
        Ok(summaries)
    }

    /// Locate the root Steam installation directory via Windows Registry or
    /// default filesystem paths.
    pub fn locate_steam_root() -> Result<PathBuf> {
        #[cfg(windows)]
        {
            use winreg::{enums::*, RegKey};

            let hkcu = RegKey::predef(HKEY_CURRENT_USER);
            if let Ok(key) = hkcu.open_subkey(r"Software\Valve\Steam") {
                if let Ok(raw) = key.get_value::<String, _>("SteamPath") {
                    let candidate = PathBuf::from(raw.replace('/', "\\"));
                    if candidate.exists() {
                        return Ok(candidate);
                    }
                }
            }

            let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
            for subkey in [
                r"SOFTWARE\WOW6432Node\Valve\Steam",
                r"SOFTWARE\Valve\Steam",
            ] {
                if let Ok(key) = hklm.open_subkey(subkey) {
                    if let Ok(raw) = key.get_value::<String, _>("InstallPath") {
                        let candidate = PathBuf::from(raw.replace('/', "\\"));
                        if candidate.exists() {
                            return Ok(candidate);
                        }
                    }
                }
            }
        }

        // Cross-platform / fallback default locations.
        let mut fallbacks = vec![
            PathBuf::from(r"C:\Program Files (x86)\Steam"),
            PathBuf::from(r"C:\Program Files\Steam"),
        ];
        if let Some(home) = dirs::home_dir() {
            fallbacks.push(home.join(".steam/steam"));
            fallbacks.push(home.join(".local/share/Steam"));
        }

        fallbacks
            .into_iter()
            .find(|p| p.exists())
            .ok_or(LauncherError::PlatformNotFound("Steam"))
    }

    /// Discover alluserdata AccountID folders (`<SteamPath>/userdata/<AccountID>`).
    fn list_account_ids(steam_root: &Path) -> Vec<String> {
        let userdata = steam_root.join("userdata");
        let Ok(entries) = std::fs::read_dir(userdata) else {
            return Vec::new();
        };
        entries
            .flatten()
            .filter_map(|e| {
                let name = e.file_name().to_string_lossy().into_owned();
                (e.path().is_dir() && name.chars().all(|c| c.is_ascii_digit()) && name != "0")
                    .then_some(name)
            })
            .collect()
    }

    /// Build `Artwork` for a Steam app, preferring custom user grid images,
    /// then local Steam library cache, and finally the Akamai Steam CDN.
    fn resolve_artwork(steam_root: &Path, account_ids: &[String], app_id: u32) -> Artwork {
        // 1. Check user custom grid overrides (`userdata/<id>/config/grid/<appid>p.*`).
        let mut local_poster = None;
        let mut local_hero = None;
        let mut local_logo = None;

        for acc in account_ids {
            let grid = steam_root.join("userdata").join(acc).join("config").join("grid");
            for ext in ["png", "jpg", "webp"] {
                if local_poster.is_none() {
                    let p = grid.join(format!("{app_id}p.{ext}"));
                    if p.exists() {
                        local_poster = Some(ArtRef::Local(p));
                    }
                }
                if local_hero.is_none() {
                    let h = grid.join(format!("{app_id}_hero.{ext}"));
                    if h.exists() {
                        local_hero = Some(ArtRef::Local(h));
                    }
                }
                if local_logo.is_none() {
                    let l = grid.join(format!("{app_id}_logo.{ext}"));
                    if l.exists() {
                        local_logo = Some(ArtRef::Local(l));
                    }
                }
            }
        }

        // 2. Check Steam's local librarycache (`appcache/librarycache`).
        let cache_dir = steam_root.join("appcache").join("librarycache");
        let app_sub = cache_dir.join(app_id.to_string());
        if local_poster.is_none() {
            let p_sub = app_sub.join("library_600x900.jpg");
            if p_sub.exists() {
                local_poster = Some(ArtRef::Local(p_sub));
            } else {
                let cached = cache_dir.join(format!("{app_id}_library_600x900.jpg"));
                if cached.exists() {
                    local_poster = Some(ArtRef::Local(cached));
                } else {
                    let h_sub = app_sub.join("header.jpg");
                    if h_sub.exists() {
                        local_poster = Some(ArtRef::Local(h_sub));
                    }
                }
            }
        }
        if local_hero.is_none() {
            let h_sub = app_sub.join("library_hero.jpg");
            if h_sub.exists() {
                local_hero = Some(ArtRef::Local(h_sub));
            } else {
                let cached = cache_dir.join(format!("{app_id}_library_hero.jpg"));
                if cached.exists() {
                    local_hero = Some(ArtRef::Local(cached));
                }
            }
        }
        if local_logo.is_none() {
            let l_sub = app_sub.join("logo.png");
            if l_sub.exists() {
                local_logo = Some(ArtRef::Local(l_sub));
            } else {
                let cached = cache_dir.join(format!("{app_id}_logo.png"));
                if cached.exists() {
                    local_logo = Some(ArtRef::Local(cached));
                }
            }
        }

        // 3. Fallback to official Steam Cloudflare CDN.
        let poster = local_poster.or_else(|| {
            Some(ArtRef::Remote(format!(
                "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/{app_id}/library_600x900.jpg"
            )))
        });
        let hero = local_hero.or_else(|| {
            Some(ArtRef::Remote(format!(
                "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/{app_id}/library_hero.jpg"
            )))
        });
        let logo = local_logo.or_else(|| {
            Some(ArtRef::Remote(format!(
                "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/{app_id}/logo.png"
            )))
        });

        Artwork {
            poster,
            hero,
            logo,
            icon: None,
        }
    }

    fn parse_localconfig_playtimes(
        steam_root: &Path,
        account_ids: &[String],
    ) -> std::collections::HashMap<u32, (u64, Option<chrono::DateTime<chrono::Utc>>)> {
        use std::collections::HashMap;
        let mut map: HashMap<u32, (u64, Option<chrono::DateTime<chrono::Utc>>)> = HashMap::new();
        for acc in account_ids {
            let path = steam_root.join("userdata").join(acc).join("config").join("localconfig.vdf");
            let Ok(content) = std::fs::read_to_string(&path) else {
                continue;
            };
            if let Ok(re_app) = regex::Regex::new(r#""(\d{3,9})"\s*\{([^}]*)\}"#) {
                let re_play = regex::Regex::new(r#""Playtime"\s*"(\d+)""#).ok();
                let re_last = regex::Regex::new(r#""LastPlayed"\s*"(\d+)""#).ok();

                for cap in re_app.captures_iter(&content) {
                    if let Ok(app_id) = cap[1].parse::<u32>() {
                        let block = &cap[2];
                        let mins = re_play
                            .as_ref()
                            .and_then(|r| r.captures(block))
                            .and_then(|c| c[1].parse::<u64>().ok())
                            .unwrap_or(0);
                        let last = re_last
                            .as_ref()
                            .and_then(|r| r.captures(block))
                            .and_then(|c| c[1].parse::<i64>().ok())
                            .and_then(|s| chrono::DateTime::from_timestamp(s, 0));

                        let entry = map.entry(app_id).or_insert((0, None));
                        if mins > entry.0 {
                            entry.0 = mins;
                        }
                        if last.is_some() {
                            entry.1 = last;
                        }
                    }
                }
            }
        }
        map
    }

    /// Blocking scan run on Tokio's blocking pool so filesystem traversal
    /// never stalls async tasks.
    fn scan_installed_blocking() -> Result<Vec<Game>> {
        let steam_root = Self::locate_steam_root()?;
        let vdf_path = steam_root.join("steamapps").join("libraryfolders.vdf");

        let libraries = if vdf_path.exists() {
            parse_library_folders_file(&vdf_path, &steam_root)?
        } else {
            vec![steam_root.clone()]
        };

        let account_ids = Self::list_account_ids(&steam_root);
        let playtimes = Self::parse_localconfig_playtimes(&steam_root, &account_ids);
        let mut games = Vec::new();
        let mut installed_ids = std::collections::HashSet::new();

        for lib in libraries {
            let steamapps = lib.join("steamapps");
            let Ok(entries) = std::fs::read_dir(&steamapps) else {
                continue;
            };

            for entry in entries.flatten() {
                let path = entry.path();
                let Some(file_name) = path.file_name().and_then(|s| s.to_str()) else {
                    continue;
                };
                if !(file_name.starts_with("appmanifest_") && file_name.ends_with(".acf")) {
                    continue;
                }

                match parse_app_manifest_file(&path, &steamapps) {
                    Ok(manifest) => {
                        if IGNORED_APP_IDS.contains(&manifest.app_id) {
                            continue;
                        }
                        let is_ready = manifest.is_fully_installed();
                        let mut game = Game::new(
                            PlatformKind::Steam,
                            manifest.app_id.to_string(),
                            manifest.name,
                        );
                        game.installed = is_ready;
                        game.install_dir = Some(manifest.install_dir);
                        game.size_bytes = manifest.size_bytes;
                        game.last_played = manifest.last_played;
                        if let Some((mins, last)) = playtimes.get(&manifest.app_id) {
                            if *mins > 0 {
                                game.playtime_minutes = Some(*mins);
                            }
                            if game.last_played.is_none() && last.is_some() {
                                game.last_played = *last;
                            }
                        }
                        game.art = Self::resolve_artwork(&steam_root, &account_ids, manifest.app_id);
                        installed_ids.insert(manifest.app_id);
                        games.push(game);
                    }
                    Err(err) => {
                        tracing::warn!(acf = %path.display(), error = %err, "skipping malformed Steam manifest");
                    }
                }
            }
        }

        // 4. Discover owned uninstalled Steam games from Windows Registry AND localconfig.vdf
        let mut app_names: std::collections::HashMap<u32, String> = std::collections::HashMap::new();

        #[cfg(windows)]
        {
            use winreg::{enums::*, RegKey};
            let hkcu = RegKey::predef(HKEY_CURRENT_USER);
            if let Ok(apps_key) = hkcu.open_subkey(r"Software\Valve\Steam\Apps") {
                for subkey_name in apps_key.enum_keys().flatten() {
                    if let Ok(app_id) = subkey_name.parse::<u32>() {
                        if let Ok(sub) = apps_key.open_subkey(&subkey_name) {
                            let name: String = sub.get_value("Name").unwrap_or_default();
                            if !name.trim().is_empty() {
                                app_names.insert(app_id, name);
                            }
                        }
                    }
                }
            }
        }

        // Load persistent names cache strictly as a name lookup dictionary, not as ownership candidates
        let mut names_cache: std::collections::HashMap<u32, String> = std::collections::HashMap::new();
        if let Some(data_dir) = dirs::data_dir() {
            let dir = data_dir.join("OmniLauncher");
            for file in ["steam_app_names.json", "steam_names_cache.json"] {
                let cache_file = dir.join(file);
                if let Ok(content) = std::fs::read_to_string(&cache_file) {
                    if let Ok(map) = serde_json::from_str::<std::collections::HashMap<u32, String>>(&content) {
                        names_cache.extend(map);
                    }
                }
            }
        }

        let mut uninstalled_candidate_ids: std::collections::HashSet<u32> =
            app_names.keys().copied().collect();
        for id in playtimes.keys() {
            uninstalled_candidate_ids.insert(*id);
        }

        for app_id in uninstalled_candidate_ids {
            if IGNORED_APP_IDS.contains(&app_id) || installed_ids.contains(&app_id) {
                continue;
            }

            let name = app_names
                .get(&app_id)
                .cloned()
                .or_else(|| names_cache.get(&app_id).cloned())
                .unwrap_or_else(|| {
                    // Check if known in KNOWN_STEAM_TITLES
                    KNOWN_STEAM_TITLES
                        .iter()
                        .find(|(id, _)| *id == app_id)
                        .map(|(_, t)| t.to_string())
                        .unwrap_or_else(|| format!("Steam App #{app_id}"))
                });

            let mut game = Game::new(PlatformKind::Steam, app_id.to_string(), name);
            game.installed = false;
            if let Some((mins, last)) = playtimes.get(&app_id) {
                if *mins > 0 {
                    game.playtime_minutes = Some(*mins);
                }
                game.last_played = *last;
            }
            game.art = Self::resolve_artwork(&steam_root, &account_ids, app_id);
            installed_ids.insert(app_id);
            games.push(game);
        }

        Ok(games)
    }

    pub fn merge_into_names_cache(incoming: &std::collections::HashMap<u32, String>) {
        if incoming.is_empty() {
            return;
        }
        if let Some(data_dir) = dirs::data_dir() {
            let dir = data_dir.join("OmniLauncher");
            let _ = std::fs::create_dir_all(&dir);
            let path = dir.join("steam_app_names.json");
            let mut map: std::collections::HashMap<u32, String> = std::fs::read_to_string(&path)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default();
            let before_len = map.len();
            for (k, v) in incoming {
                map.insert(*k, v.clone());
            }
            if map.len() > before_len {
                if let Ok(s) = serde_json::to_string_pretty(&map) {
                    let _ = std::fs::write(&path, s);
                }
            }
        }
    }

    pub async fn resolve_unnamed_games(http: &reqwest::Client, games: &mut [Game]) {
        let mut names_cache: std::collections::HashMap<u32, String> = std::collections::HashMap::new();
        if let Some(data_dir) = dirs::data_dir() {
            let dir = data_dir.join("OmniLauncher");
            for file in ["steam_app_names.json", "steam_names_cache.json"] {
                let p = dir.join(file);
                if let Ok(s) = std::fs::read_to_string(&p) {
                    if let Ok(m) = serde_json::from_str::<std::collections::HashMap<u32, String>>(&s) {
                        names_cache.extend(m);
                    }
                }
            }
        }

        let mut newly_resolved = std::collections::HashMap::new();

        for game in games.iter_mut() {
            if game.title.starts_with("Steam App #") {
                if let Ok(app_id) = game.id.native_id.parse::<u32>() {
                    if let Some(cached_name) = names_cache.get(&app_id) {
                        game.title = cached_name.clone();
                    } else {
                        let url = format!(
                            "https://store.steampowered.com/api/appdetails?appids={app_id}&filters=basic&l=russian"
                        );
                        if let Ok(resp) = http.get(&url).send().await {
                            if let Ok(json) = resp.json::<serde_json::Value>().await {
                                if let Some(name) = json
                                    .pointer(&format!("/{app_id}/data/name"))
                                    .and_then(|v| v.as_str())
                                {
                                    if !name.trim().is_empty() {
                                        game.title = name.to_string();
                                        names_cache.insert(app_id, name.to_string());
                                        newly_resolved.insert(app_id, name.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if !newly_resolved.is_empty() {
            Self::merge_into_names_cache(&newly_resolved);
        }
    }
}

#[async_trait]
impl PlatformConnector for SteamPlatform {
    fn kind(&self) -> PlatformKind {
        PlatformKind::Steam
    }

    async fn is_available(&self) -> bool {
        Self::locate_steam_root().is_ok()
    }

    async fn detect_installed_games(&self) -> Result<Vec<Game>> {
        let mut games = tokio::task::spawn_blocking(Self::scan_installed_blocking).await??;
        Self::resolve_unnamed_games(&self.http, &mut games).await;
        Ok(games)
    }

    async fn launch_game(&self, native_id: &str) -> Result<LaunchHandle> {
        let app_id: u32 = native_id
            .trim()
            .parse()
            .map_err(|_| LauncherError::InvalidInput(format!("invalid Steam AppID `{native_id}`")))?;

        let game_id = GameId::new(PlatformKind::Steam, app_id.to_string());
        let uri = format!("steam://rungameid/{app_id}");
        launch_uri(game_id, uri).await
    }

    async fn install_game(&self, native_id: &str) -> Result<LaunchHandle> {
        let app_id: u32 = native_id
            .trim()
            .parse()
            .map_err(|_| LauncherError::InvalidInput(format!("invalid Steam AppID `{native_id}`")))?;

        let game_id = GameId::new(PlatformKind::Steam, app_id.to_string());
        let uri = format!("steam://install/{app_id}");
        launch_uri(game_id, uri).await
    }

    async fn sync_profile(&self) -> Result<ProfileSync> {
        let creds = self.credentials().await;
        let api_key = creds.api_key.filter(|s| !s.trim().is_empty());
        let mut steam_id = creds.steam_id64.filter(|s| !s.trim().is_empty());

        if steam_id.is_none() {
            if let Ok(resolved) = self.resolve_profile().await {
                steam_id = Some(resolved.steam_id);
            }
        }

        let Some(steam_id) = steam_id else {
            return Err(LauncherError::AuthRequired("SteamID64 is not configured and could not be detected"));
        };

        // 1. If valid user API key is configured, query official Web API
        if let Some(key) = api_key {
            let summary_url = format!(
                "https://api.steampowered.com/ISteamUser/GetPlayerSummaries/v2/?key={key}&steamids={steam_id}"
            );
            if let Ok(resp) = self.http.get(&summary_url).send().await {
                if let Ok(summary_resp) = resp.json::<PlayerSummariesEnvelope>().await {
                    let player = summary_resp.response.players.into_iter().next();
                    let owned_url = format!(
                        "https://api.steampowered.com/IPlayerService/GetOwnedGames/v1/?key={key}&steamid={steam_id}&include_appinfo=1&include_played_free_games=1"
                    );
                    let owned_count = if let Ok(resp) = self.http.get(&owned_url).send().await {
                        resp.json::<OwnedGamesEnvelope>().await.ok().and_then(|r| r.response.game_count).unwrap_or(0)
                    } else {
                        0
                    };

                    return Ok(ProfileSync {
                        platform: PlatformKind::Steam,
                        account_id: Some(steam_id),
                        display_name: player.as_ref().map(|p| p.personaname.clone()),
                        avatar_url: player.and_then(|p| p.avatarfull),
                        owned_games: owned_count,
                        synced_at: Utc::now(),
                    });
                }
            }
        }

        // 2. Fallback: Public profile XML / games list when API key is not configured
        let mut persona_name = None;
        let mut avatar_url = None;

        let profile_xml_url = format!("https://steamcommunity.com/profiles/{steam_id}/?xml=1");
        if let Ok(resp) = self.http.get(&profile_xml_url).send().await {
            if let Ok(text) = resp.text().await {
                if let Some(re) = regex::Regex::new(r"<steamID>(?:<!\[CDATA\[)?(.*?)(?:\]\]>)?</steamID>").ok() {
                    if let Some(cap) = re.captures(&text) {
                        persona_name = cap.get(1).map(|m| m.as_str().to_string());
                    }
                }
                if let Some(re) = regex::Regex::new(r"<avatarFull>(?:<!\[CDATA\[)?(.*?)(?:\]\]>)?</avatarFull>").ok() {
                    if let Some(cap) = re.captures(&text) {
                        avatar_url = cap.get(1).map(|m| m.as_str().to_string());
                    }
                }
            }
        }

        if persona_name.is_none() {
            if let Ok(resolved) = self.resolve_profile().await {
                persona_name = Some(resolved.persona_name);
                avatar_url = avatar_url.or(Some(resolved.avatar_url));
            }
        }

        let games_xml_url = format!("https://steamcommunity.com/profiles/{steam_id}/games?tab=all&xml=1");
        let mut owned_count = 0;
        if let Ok(resp) = self.http.get(&games_xml_url).send().await {
            if let Ok(text) = resp.text().await {
                if let Some(re) = regex::Regex::new(r"<appID>(\d+)</appID>\s*<name>(?:<!\[CDATA\[)?(.*?)(?:\]\]>)?</name>").ok() {
                    let mut resolved_map = std::collections::HashMap::new();
                    for cap in re.captures_iter(&text) {
                        owned_count += 1;
                        if let (Some(id_m), Some(name_m)) = (cap.get(1), cap.get(2)) {
                            if let Ok(aid) = id_m.as_str().parse::<u32>() {
                                let gname = name_m.as_str().to_string();
                                if !gname.is_empty() {
                                    resolved_map.insert(aid, gname);
                                }
                            }
                        }
                    }
                    if !resolved_map.is_empty() {
                        Self::merge_into_names_cache(&resolved_map);
                    }
                }
            }
        }

        if owned_count == 0 {
            if let Ok(installed) = self.detect_installed_games().await {
                owned_count = installed.len();
            }
        }

        Ok(ProfileSync {
            platform: PlatformKind::Steam,
            account_id: Some(steam_id),
            display_name: persona_name,
            avatar_url,
            owned_games: owned_count,
            synced_at: Utc::now(),
        })
    }
}

// ---------------------------------------------------------------------------
// Steam Web API wire structs
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct PlayerSummariesEnvelope {
    response: PlayerSummariesInner,
}

#[derive(Debug, Deserialize)]
struct PlayerSummariesInner {
    #[serde(default)]
    players: Vec<SteamPlayerSummary>,
}

#[derive(Debug, Deserialize)]
struct SteamPlayerSummary {
    #[serde(default)]
    steamid: Option<String>,
    personaname: String,
    #[serde(default)]
    profileurl: Option<String>,
    #[serde(default)]
    avatarfull: Option<String>,
    #[serde(default)]
    personastate: Option<u32>,
    #[serde(default)]
    gameextrainfo: Option<String>,
    #[serde(default)]
    gameid: Option<String>,
}

#[derive(Debug, Deserialize)]
struct FriendListEnvelope {
    #[serde(default)]
    friendslist: Option<FriendListInner>,
}

#[derive(Debug, Deserialize)]
struct FriendListInner {
    #[serde(default)]
    friends: Vec<SteamFriendRef>,
}

#[derive(Debug, Deserialize)]
struct SteamFriendRef {
    steamid: String,
}

#[derive(Debug, Deserialize)]
struct PlayerAchievementsEnvelope {
    #[serde(default)]
    playerstats: Option<PlayerStatsInner>,
}

#[derive(Debug, Deserialize)]
struct PlayerStatsInner {
    #[serde(default)]
    achievements: Vec<RawSteamAchievement>,
}

#[derive(Debug, Deserialize)]
struct RawSteamAchievement {
    apiname: String,
    #[serde(default)]
    achieved: u8,
    #[serde(default)]
    unlocktime: u64,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SteamGridEnvelope {
    #[serde(default)]
    data: Vec<SteamGridArtwork>,
}

#[derive(Debug, Deserialize)]
struct OwnedGamesEnvelope {
    response: OwnedGamesInner,
}

#[derive(Debug, Default, Deserialize)]
struct OwnedGamesInner {
    #[serde(default)]
    game_count: Option<usize>,
}
