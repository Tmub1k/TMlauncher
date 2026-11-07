//! Epic Games Store connector (`EpicPlatform`).
//!
//! Scans `C:\ProgramData\Epic\EpicGamesLauncher\Data\Manifests\*.item` (and
//! registry fallback), extracts `AppName`, `DisplayName`, `CatalogNamespace`,
//! `CatalogItemId`, `InstallLocation`, `LaunchExecutable`, `InstallSize`, and
//! launches via the official silent protocol:
//! `com.epicgames.launcher://apps/{CatalogNamespace}%3A{CatalogItemId}%3A{AppName}?action=launch&silent=true`.

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::core::platform::{
    launch_executable, launch_uri, ArtRef, Artwork, Game, GameId, LaunchHandle, PlatformConnector, PlatformKind,
    ProfileSync,
};
use crate::error::{LauncherError, Result};

/// Raw `.item` JSON manifest written by Epic Games Launcher.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct EpicItemManifest {
    pub app_name: String,
    pub display_name: String,
    pub catalog_namespace: String,
    pub catalog_item_id: String,
    #[serde(alias = "InstallationDir", default)]
    pub install_location: PathBuf,
    #[serde(default)]
    pub launch_executable: String,
    #[serde(default)]
    pub launch_command: String,
    #[serde(default)]
    pub install_size: Option<u64>,
    #[serde(rename = "bIsIncompleteInstall", default)]
    pub is_incomplete_install: bool,
    #[serde(default)]
    pub app_categories: Vec<String>,
}

impl EpicItemManifest {
    /// True if this manifest represents a playable game rather than an engine
    /// plugin or launcher component.
    pub fn is_playable_game(&self) -> bool {
        if self.app_name.trim().is_empty() {
            return false;
        }
        // Filter out pure Unreal Engine plugins/vault cache entries if categories are present.
        if !self.app_categories.is_empty()
            && !self
                .app_categories
                .iter()
                .any(|c| c.eq_ignore_ascii_case("games") || c.eq_ignore_ascii_case("applications"))
        {
            return false;
        }
        true
    }

    /// Build the silent Epic Games Launcher launch URI with percent-encoded `:` (`%3A`).
    pub fn launch_uri(&self) -> String {
        format!(
            "com.epicgames.launcher://apps/{}%3A{}%3A{}?action=launch&silent=true",
            url_encode_segment(&self.catalog_namespace),
            url_encode_segment(&self.catalog_item_id),
            url_encode_segment(&self.app_name),
        )
    }

    pub fn executable_path(&self) -> PathBuf {
        self.install_location.join(&self.launch_executable)
    }
}

pub struct EpicPlatform {
    manifests_override: Option<PathBuf>,
    manifest_cache: RwLock<HashMap<String, EpicItemManifest>>,
    http: reqwest::Client,
}

impl EpicPlatform {
    pub fn new() -> Self {
        Self {
            manifests_override: None,
            manifest_cache: RwLock::new(HashMap::new()),
            http: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(5))
                .build()
                .unwrap_or_default(),
        }
    }

    pub fn with_manifests_dir(dir: PathBuf) -> Self {
        Self {
            manifests_override: Some(dir),
            manifest_cache: RwLock::new(HashMap::new()),
            http: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(5))
                .build()
                .unwrap_or_default(),
        }
    }

    /// Resolve `C:\ProgramData\Epic\EpicGamesLauncher\Data\Manifests` via
    /// Windows Registry (`AppDataPath`) or the standard `%ProgramData%` path.
    pub fn locate_manifests_dir(&self) -> Result<PathBuf> {
        if let Some(custom) = &self.manifests_override {
            if custom.exists() {
                return Ok(custom.clone());
            }
        }

        #[cfg(windows)]
        {
            use winreg::{enums::*, RegKey};

            let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
            for subkey in [
                r"SOFTWARE\WOW6432Node\Epic Games\EpicGamesLauncher",
                r"SOFTWARE\Epic Games\EpicGamesLauncher",
            ] {
                if let Ok(key) = hklm.open_subkey(subkey) {
                    if let Ok(app_data) = key.get_value::<String, _>("AppDataPath") {
                        let candidate = PathBuf::from(app_data).join("Manifests");
                        if candidate.exists() {
                            return Ok(candidate);
                        }
                    }
                }
            }
        }

        let program_data = std::env::var_os("ProgramData")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(r"C:\ProgramData"));
        let default_dir = program_data
            .join("Epic")
            .join("EpicGamesLauncher")
            .join("Data")
            .join("Manifests");

        if default_dir.exists() {
            Ok(default_dir)
        } else {
            Err(LauncherError::PlatformNotFound("Epic Games Launcher"))
        }
    }

    pub fn parse_item_file(path: &Path) -> Result<EpicItemManifest> {
        let raw = std::fs::read_to_string(path).map_err(|source| LauncherError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        Self::parse_item_str(&raw, path)
    }

    pub fn parse_item_str(raw: &str, source_path: &Path) -> Result<EpicItemManifest> {
        serde_json::from_str::<EpicItemManifest>(raw).map_err(|e| LauncherError::Manifest {
            path: source_path.to_path_buf(),
            message: e.to_string(),
        })
    }

    fn scan_manifests_blocking(dir: PathBuf) -> Result<Vec<EpicItemManifest>> {
        let entries = std::fs::read_dir(&dir).map_err(|source| LauncherError::Io {
            path: dir.clone(),
            source,
        })?;

        let mut manifests = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("item") {
                continue;
            }
            match Self::parse_item_file(&path) {
                Ok(manifest) if manifest.is_playable_game() => manifests.push(manifest),
                Ok(_) => {}
                Err(err) => {
                    tracing::warn!(item = %path.display(), error = %err, "skipping malformed Epic .item manifest");
                }
            }
        }
        Ok(manifests)
    }

    /// Retrieve full Epic Games library, optionally including uninstalled catalog games.
    pub async fn fetch_epic_library(&self, include_uninstalled: bool) -> Result<Vec<EpicGame>> {
        let mut map: HashMap<String, EpicGame> = HashMap::new();

        // 1. Scan installed manifests
        if let Ok(dir) = self.locate_manifests_dir() {
            if let Ok(manifests) = tokio::task::spawn_blocking(move || Self::scan_manifests_blocking(dir)).await? {
                for m in manifests {
                    let is_installed = !m.is_incomplete_install && m.executable_path().exists();
                    let launch_uri = Some(m.launch_uri());
                    let install_uri = Some(format!(
                        "com.epicgames.launcher://apps/{}?action=install",
                        url_encode_segment(&m.app_name)
                    ));
                    map.insert(
                        m.app_name.clone(),
                        EpicGame {
                            app_name: m.app_name,
                            display_name: m.display_name,
                            catalog_item_id: m.catalog_item_id,
                            installed: is_installed,
                            install_location: if is_installed {
                                Some(m.install_location.to_string_lossy().to_string())
                            } else {
                                None
                            },
                            launch_uri,
                            install_uri,
                        },
                    );
                }
            }
        }

        // 2. If uninstalled games requested, check launcher logs, legendary, heroic, and local cache
        if include_uninstalled {
            if let Some(config_dir) = dirs::config_dir() {
                for base in [config_dir.join("legendary"), config_dir.join("heroic").join("store_cache")] {
                    for filename in ["installed.json", "epic_games.json"] {
                        let path = base.join(filename);
                        if path.exists() {
                            if let Ok(raw) = tokio::fs::read_to_string(&path).await {
                                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&raw) {
                                    if let Some(obj) = val.as_object() {
                                        for (k, item) in obj {
                                            let app_name = item.get("app_name")
                                                .and_then(|v| v.as_str())
                                                .unwrap_or(k)
                                                .to_string();
                                            let display_name = item.get("title")
                                                .or_else(|| item.get("app_title"))
                                                .and_then(|v| v.as_str())
                                                .unwrap_or(&app_name)
                                                .to_string();
                                            let catalog_id = item.get("catalog_item_id")
                                                .and_then(|v| v.as_str())
                                                .unwrap_or(&app_name)
                                                .to_string();
                                            map.entry(app_name.clone()).or_insert_with(|| EpicGame {
                                                app_name: app_name.clone(),
                                                display_name,
                                                catalog_item_id: catalog_id,
                                                installed: false,
                                                install_location: None,
                                                launch_uri: None,
                                                install_uri: Some(format!(
                                                    "com.epicgames.launcher://apps/{}?action=install",
                                                    url_encode_segment(&app_name)
                                                )),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if let Some(local_app_data) = dirs::data_local_dir() {
                let log_path = local_app_data
                    .join("EpicGamesLauncher")
                    .join("Saved")
                    .join("Logs")
                    .join("EpicGamesLauncher.log");
                if log_path.exists() {
                    if let Ok(raw) = tokio::fs::read_to_string(&log_path).await {
                        let re = regex::Regex::new(r#"(?i)CatalogItemId[:=]\s*["']?([a-zA-Z0-9]{10,})["']?.*?AppName[:=]\s*["']?([a-zA-Z0-9_\-]+)["']?"#).ok();
                        if let Some(re) = re {
                            for cap in re.captures_iter(&raw) {
                                let cat_id = cap.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
                                let app = cap.get(2).map(|m| m.as_str().to_string()).unwrap_or_default();
                                if !app.is_empty() && !map.contains_key(&app) {
                                    map.insert(
                                        app.clone(),
                                        EpicGame {
                                            app_name: app.clone(),
                                            display_name: app.clone(),
                                            catalog_item_id: cat_id,
                                            installed: false,
                                            install_location: None,
                                            launch_uri: None,
                                            install_uri: Some(format!(
                                                "com.epicgames.launcher://apps/{}?action=install",
                                                url_encode_segment(&app)
                                            )),
                                        },
                                    );
                                }
                            }
                        }
                    }
                }

                let local_cache = local_app_data.join("OmniLauncher").join("epic_catalog_cache.json");
                if local_cache.exists() {
                    if let Ok(raw) = tokio::fs::read_to_string(&local_cache).await {
                        if let Ok(list) = serde_json::from_str::<Vec<EpicGame>>(&raw) {
                            for g in list {
                                map.entry(g.app_name.clone()).or_insert(g);
                            }
                        }
                    }
                }
            }
        }

        let mut result: Vec<EpicGame> = map.into_values().collect();
        result.sort_by(|a, b| a.display_name.cmp(&b.display_name));
        Ok(result)
    }

    fn preset_art(app_name: &str, title: &str) -> Option<(String, String)> {
        let lower_app = app_name.to_lowercase();
        let lower_title = title.to_lowercase();
        if lower_app.contains("fortnite") || lower_title.contains("fortnite") {
            return Some((
                "https://cdn1.epicgames.com/offer/fn/FNBR_29-00_C5S2_EGS_Launcher_Blade_2560x1440_2560x1440-66e85579fa9a117565ba18451f28b7e7".into(),
                "https://cdn1.epicgames.com/offer/fn/FNBR_29-00_C5S2_EGS_Launcher_Blade_2560x1440_2560x1440-66e85579fa9a117565ba18451f28b7e7".into(),
            ));
        }
        if lower_app.contains("alanwake2") || lower_title.contains("alan wake 2") {
            return Some((
                "https://cdn1.epicgames.com/offer/c4763f236d08423eb47b4c3008779c84/EGS_AlanWake2_RemedyEntertainment_S2_1200x1600-c7c527011d61eb2e4719286073dc0814".into(),
                "https://cdn1.epicgames.com/offer/c4763f236d08423eb47b4c3008779c84/EGS_AlanWake2_RemedyEntertainment_S1_2560x1440-6219ffbb133d17961937ff7fa762391b".into(),
            ));
        }
        None
    }

    async fn harvest_legendary_heroic_art() -> HashMap<String, (Option<String>, Option<String>)> {
        let mut map = HashMap::new();
        if let Some(config_dir) = dirs::config_dir() {
            for base in [config_dir.join("legendary"), config_dir.join("heroic").join("store_cache")] {
                for filename in ["installed.json", "epic_games.json"] {
                    let path = base.join(filename);
                    if path.exists() {
                        if let Ok(raw) = tokio::fs::read_to_string(&path).await {
                            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&raw) {
                                if let Some(obj) = val.as_object() {
                                    for (k, item) in obj {
                                        let app_name = item.get("app_name")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or(k);
                                        let mut poster = None;
                                        let mut hero = None;
                                        if let Some(imgs) = item.get("keyImages").and_then(|v| v.as_array()) {
                                            for img in imgs {
                                                let t = img.get("type").and_then(|v| v.as_str()).unwrap_or("");
                                                let url = img.get("url").and_then(|v| v.as_str()).map(String::from);
                                                if t == "DieselGameBox" || t == "OfferImageTall" || t == "Thumbnail" {
                                                    if poster.is_none() { poster = url.clone(); }
                                                }
                                                if t == "DieselStoreFrontWide" || t == "OfferImageWide" {
                                                    if hero.is_none() { hero = url.clone(); }
                                                }
                                            }
                                        }
                                        if poster.is_some() || hero.is_some() {
                                            map.insert(app_name.to_string(), (poster, hero));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        map
    }

    pub async fn resolve_games_artwork(&self, games: &mut [Game]) {
        let mut cache: HashMap<String, (Option<String>, Option<String>)> = HashMap::new();
        let cache_path = dirs::data_dir().map(|d| d.join("OmniLauncher").join("epic_artwork_cache.json"));
        if let Some(ref p) = cache_path {
            if let Ok(raw) = std::fs::read_to_string(p) {
                if let Ok(map) = serde_json::from_str::<HashMap<String, (Option<String>, Option<String>)>>(&raw) {
                    cache = map;
                }
            }
        }

        let local_harvest = Self::harvest_legendary_heroic_art().await;
        for (k, v) in local_harvest {
            cache.entry(k).or_insert(v);
        }

        let mut changed = false;

        for game in games.iter_mut() {
            let app_name = &game.id.native_id;
            let display_name = &game.title;

            let preset = Self::preset_art(app_name, display_name);

            let (poster_url, hero_url) = if let Some(p) = preset {
                (Some(p.0), Some(p.1))
            } else if let Some(cached) = cache.get(app_name).or_else(|| cache.get(display_name)) {
                cached.clone()
            } else {
                let search_title = crate::commands::store::sanitize_search_query(display_name);
                let mut found_poster = None;
                let mut found_hero = None;

                if !search_title.is_empty() {
                    let search_url = "https://store.steampowered.com/api/storesearch/";
                    if let Ok(resp) = self.http.get(search_url)
                        .query(&[("term", search_title.as_str()), ("l", "russian"), ("cc", "us")])
                        .send()
                        .await
                    {
                        if let Ok(json) = resp.json::<serde_json::Value>().await {
                            if let Some(first_id) = json.pointer("/items/0/id").and_then(|v| v.as_u64()) {
                                let cdn = "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps";
                                found_poster = Some(format!("{cdn}/{first_id}/library_600x900.jpg"));
                                found_hero = Some(format!("{cdn}/{first_id}/library_hero.jpg"));
                            }
                        }
                    }
                }

                cache.insert(app_name.clone(), (found_poster.clone(), found_hero.clone()));
                changed = true;
                (found_poster, found_hero)
            };

            game.art = Artwork {
                poster: poster_url.map(ArtRef::Remote),
                hero: hero_url.map(ArtRef::Remote),
                logo: None,
                icon: None,
            };
        }

        if changed {
            if let Some(ref p) = cache_path {
                if let Some(parent) = p.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                if let Ok(s) = serde_json::to_string_pretty(&cache) {
                    let _ = std::fs::write(p, s);
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpicGame {
    pub app_name: String,
    pub display_name: String,
    pub catalog_item_id: String,
    pub installed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub install_location: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub launch_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub install_uri: Option<String>,
}

impl Default for EpicPlatform {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PlatformConnector for EpicPlatform {
    fn kind(&self) -> PlatformKind {
        PlatformKind::Epic
    }

    async fn is_available(&self) -> bool {
        self.locate_manifests_dir().is_ok()
    }

    async fn detect_installed_games(&self) -> Result<Vec<Game>> {
        let dir = self.locate_manifests_dir().ok();
        let manifests = if let Some(d) = dir {
            tokio::task::spawn_blocking(move || Self::scan_manifests_blocking(d))
                .await
                .unwrap_or(Ok(Vec::new()))
                .unwrap_or_default()
        } else {
            Vec::new()
        };

        let mut cache = self.manifest_cache.write().await;
        cache.clear();

        let mut games = Vec::with_capacity(manifests.len());
        let mut seen = std::collections::HashSet::new();

        for m in manifests {
            let exe = m.executable_path();
            let is_ready = !m.is_incomplete_install && exe.exists();
            let mut game = Game::new(PlatformKind::Epic, m.app_name.clone(), m.display_name.clone());
            game.installed = is_ready;
            game.install_dir = Some(m.install_location.clone());
            game.executable = Some(exe);
            game.size_bytes = m.install_size;
            seen.insert(m.app_name.clone());
            cache.insert(m.app_name.clone(), m);
            games.push(game);
        }

        // Include user's uninstalled Epic library games discovered from manifests/logs/cache
        if let Ok(uninstalled) = self.fetch_epic_library(true).await {
            for eg in uninstalled {
                if !seen.contains(&eg.app_name) {
                    seen.insert(eg.app_name.clone());
                    let mut game = Game::new(PlatformKind::Epic, eg.app_name.clone(), eg.display_name.clone());
                    game.installed = eg.installed;
                    games.push(game);
                }
            }
        }

        self.resolve_games_artwork(&mut games).await;

        Ok(games)
    }

    async fn launch_game(&self, native_id: &str) -> Result<LaunchHandle> {
        let manifest = {
            let cache = self.manifest_cache.read().await;
            cache.get(native_id).cloned()
        };

        // Re-scan on cache miss in case the user asks to launch before the initial scan finishes.
        let manifest = match manifest {
            Some(m) => m,
            None => {
                self.detect_installed_games().await?;
                self.manifest_cache
                    .read()
                    .await
                    .get(native_id)
                    .cloned()
                    .ok_or_else(|| LauncherError::GameNotFound(format!("epic:{native_id}")))?
            }
        };

        let game_id = GameId::new(PlatformKind::Epic, manifest.app_name.clone());
        let uri = manifest.launch_uri();

        match launch_uri(game_id.clone(), uri).await {
            Ok(handle) => Ok(handle),
            Err(uri_err) => {
                tracing::warn!(error = %uri_err, "Epic URI launch failed; attempting direct executable fallback");
                let exe = manifest.executable_path();
                let args: Vec<String> = manifest
                    .launch_command
                    .split_whitespace()
                    .map(String::from)
                    .collect();
                launch_executable(game_id, exe, args, Some(manifest.install_location)).await
            }
        }
    }

    async fn install_game(&self, native_id: &str) -> Result<LaunchHandle> {
        let manifest = {
            let cache = self.manifest_cache.read().await;
            cache.get(native_id).cloned()
        };
        let uri = if let Some(m) = manifest {
            format!(
                "com.epicgames.launcher://apps/{}%3A{}%3A{}?action=install",
                url_encode_segment(&m.catalog_namespace),
                url_encode_segment(&m.catalog_item_id),
                url_encode_segment(&m.app_name),
            )
        } else {
            format!("com.epicgames.launcher://apps/{native_id}?action=install")
        };
        let game_id = GameId::new(PlatformKind::Epic, native_id);
        launch_uri(game_id, uri).await
    }

    async fn sync_profile(&self) -> Result<ProfileSync> {
        let owned_games = self.manifest_cache.read().await.len();
        Ok(ProfileSync {
            platform: PlatformKind::Epic,
            account_id: None,
            display_name: Some("Epic Local Library".into()),
            avatar_url: None,
            owned_games,
            synced_at: Utc::now(),
        })
    }
}

fn url_encode_segment(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for b in input.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_epic_item_and_formats_protocol_uri() {
        let raw = r#"{
            "FormatVersion": 0,
            "bIsIncompleteInstall": false,
            "AppName": "Fortnite",
            "DisplayName": "Fortnite",
            "CatalogNamespace": "fn",
            "CatalogItemId": "4fe75bbc5a674f4f9b356b5c90567da5",
            "InstallLocation": "C:\\Program Files\\Epic Games\\Fortnite",
            "LaunchExecutable": "FortniteGame/Binaries/Win64/FortniteClient-Win64-Shipping.exe",
            "InstallSize": 61234567890,
            "AppCategories": ["games"]
        }"#;

        let item = EpicPlatform::parse_item_str(raw, Path::new("Fortnite.item")).unwrap();
        assert!(item.is_playable_game());
        assert_eq!(
            item.launch_uri(),
            "com.epicgames.launcher://apps/fn%3A4fe75bbc5a674f4f9b356b5c90567da5%3AFortnite?action=launch&silent=true"
        );
    }
}
