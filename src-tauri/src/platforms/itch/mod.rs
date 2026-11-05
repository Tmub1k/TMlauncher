//! itch.io connector (`ItchPlatform`): `.itch.toml` manifest parser and local
//! `%APPDATA%\itch\apps` scanner.

use std::path::{Path, PathBuf};

use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::core::platform::{
    launch_executable, launch_uri, Game, GameId, LaunchHandle, PlatformConnector, PlatformKind, ProfileSync,
};
use crate::error::{LauncherError, Result};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ItchManifest {
    #[serde(default)]
    pub actions: Vec<ItchAction>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ItchAction {
    pub name: String,
    pub path: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub platform: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItchStoreGame {
    pub id: String,
    pub title: String,
    pub price: String,
    pub currency: String,
    pub cover_url: Option<String>,
    pub url: String,
    pub description: String,
    pub platforms: Vec<String>,
}

pub struct ItchPlatform;

impl ItchPlatform {
    pub fn new() -> Self {
        Self
    }

    /// Fetch and parse itch.io XML store feeds (top-sellers, on-sale, new-and-popular, free).
    pub async fn fetch_store_feed(http: &reqwest::Client, category: &str) -> Result<Vec<ItchStoreGame>> {
        let feed_name = match category {
            "on-sale" | "sale" => "on-sale",
            "new" | "popular" | "new-and-popular" => "new-and-popular",
            "free" => "free",
            _ => "top-sellers",
        };
        let url = format!("https://itch.io/games/{feed_name}.xml");
        let xml = http
            .get(&url)
            .send()
            .await?
            .text()
            .await?;

        let mut games = Vec::new();
        for item_chunk in xml.split("<item>").skip(1) {
            let Some((content, _)) = item_chunk.split_once("</item>") else {
                continue;
            };

            let extract = |tag: &str| -> Option<String> {
                let start_tag = format!("<{tag}>");
                let end_tag = format!("</{tag}>");
                let start = content.find(&start_tag)? + start_tag.len();
                let end = content[start..].find(&end_tag)?;
                let val = &content[start..start + end];
                Some(val.trim().to_string())
            };

            let plain_title = extract("plainTitle").or_else(|| extract("title")).unwrap_or_default();
            if plain_title.is_empty() {
                continue;
            }

            let link = extract("link").or_else(|| extract("guid")).unwrap_or_default();
            let price = extract("price").unwrap_or_else(|| "$0.00".into());
            let currency = extract("currency").unwrap_or_else(|| "USD".into());
            let image_url = extract("imageurl");

            let mut description = String::new();
            if let Some(cdata_start) = content.find("<![CDATA[") {
                let start = cdata_start + "<![CDATA[".len();
                if let Some(cdata_end) = content[start..].find("]]>") {
                    let desc_raw = &content[start..start + cdata_end];
                    // strip html img tags if present
                    if let Some((clean, _)) = desc_raw.split_once("<img") {
                        description = clean.trim().to_string();
                    } else {
                        description = desc_raw.trim().to_string();
                    }
                }
            }

            let mut platforms = Vec::new();
            if content.contains("<windows>yes</windows>") {
                platforms.push("Windows".into());
            }
            if content.contains("<osx>yes</osx>") {
                platforms.push("macOS".into());
            }
            if content.contains("<linux>yes</linux>") {
                platforms.push("Linux".into());
            }

            let id = link.clone();
            games.push(ItchStoreGame {
                id,
                title: plain_title,
                price,
                currency,
                cover_url: image_url,
                url: link,
                description,
                platforms,
            });
        }

        Ok(games)
    }

    pub fn parse_manifest(path: &Path) -> Result<ItchManifest> {
        let raw = std::fs::read_to_string(path).map_err(|source| LauncherError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        toml::from_str(&raw).map_err(|e| LauncherError::Manifest {
            path: path.to_path_buf(),
            message: e.to_string(),
        })
    }

    pub fn apps_dir() -> Result<PathBuf> {
        let dir = dirs::config_dir()
            .ok_or(LauncherError::PlatformNotFound("itch.io"))?
            .join("itch")
            .join("apps");
        if dir.exists() {
            Ok(dir)
        } else {
            Err(LauncherError::PlatformNotFound("itch.io"))
        }
    }

    pub fn butler_exe_path() -> PathBuf {
        dirs::config_dir()
            .map(|d| d.join("itch").join("broth").join("butler").join("butler.exe"))
            .filter(|p| p.exists())
            .unwrap_or_else(|| PathBuf::from(if cfg!(windows) { "butler.exe" } else { "butler" }))
    }

    /// Run `butler verify <signature> <gamedir> --heal=archive` to verify and
    /// differentially heal an itch.io DRM-free installation.
    pub async fn verify_or_heal(game_dir: &Path, heal: bool) -> Result<String> {
        let butler = Self::butler_exe_path();
        let mut cmd = tokio::process::Command::new(&butler);
        cmd.arg("status").arg(game_dir).arg("--json");
        if heal {
            cmd.arg("--heal");
        }
        let output = cmd
            .output()
            .await
            .map_err(|e| LauncherError::Launch(format!("butler invocation failed: {e}")))?;
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
}

impl Default for ItchPlatform {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PlatformConnector for ItchPlatform {
    fn kind(&self) -> PlatformKind {
        PlatformKind::Itch
    }

    async fn is_available(&self) -> bool {
        Self::apps_dir().is_ok()
    }

    async fn detect_installed_games(&self) -> Result<Vec<Game>> {
        let root = Self::apps_dir()?;
        tokio::task::spawn_blocking(move || {
            let mut games = Vec::new();
            let Ok(entries) = std::fs::read_dir(&root) else {
                return Ok(games);
            };
            for entry in entries.flatten() {
                let dir = entry.path();
                if !dir.is_dir() {
                    continue;
                }
                let manifest_path = dir.join(".itch.toml");
                if !manifest_path.exists() {
                    continue;
                }
                if let Ok(manifest) = Self::parse_manifest(&manifest_path) {
                    if let Some(action) = manifest.actions.first() {
                        let slug = dir
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("itch-game")
                            .to_string();
                        let mut game = Game::new(PlatformKind::Itch, slug.clone(), slug);
                        game.installed = true;
                        game.executable = Some(dir.join(&action.path));
                        game.install_dir = Some(dir);
                        games.push(game);
                    }
                }
            }
            Ok(games)
        })
        .await?
    }

    async fn launch_game(&self, native_id: &str) -> Result<LaunchHandle> {
        let dir = Self::apps_dir()?.join(native_id);
        let manifest = Self::parse_manifest(&dir.join(".itch.toml"))?;
        let action = manifest
            .actions
            .into_iter()
            .next()
            .ok_or_else(|| LauncherError::Launch(format!("no launch action in {native_id}")))?;
        let exe = dir.join(action.path);
        launch_executable(
            GameId::new(PlatformKind::Itch, native_id),
            exe,
            action.args,
            Some(dir),
        )
        .await
    }

    async fn install_game(&self, native_id: &str) -> Result<LaunchHandle> {
        let uri = if native_id.starts_with("http://") || native_id.starts_with("https://") {
            native_id.to_string()
        } else {
            format!("https://itch.io/games/{native_id}")
        };
        let game_id = GameId::new(PlatformKind::Itch, native_id);
        launch_uri(game_id, uri).await
    }

    async fn sync_profile(&self) -> Result<ProfileSync> {
        Ok(ProfileSync {
            platform: PlatformKind::Itch,
            account_id: None,
            display_name: Some("itch.io Local Library".into()),
            avatar_url: None,
            owned_games: 0,
            synced_at: Utc::now(),
        })
    }
}
