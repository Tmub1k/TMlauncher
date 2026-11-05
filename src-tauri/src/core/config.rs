//! Persistent user configuration (`<AppConfig>/config.toml`).

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::core::media_capture::CaptureConfig;
use crate::error::{IoResultExt, LauncherError, Result};
use crate::platforms::steam::SteamCredentials;

const CONFIG_FILE: &str = "config.toml";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct NetworkSettings {
    pub dns_provider: String,
    pub doh_enabled: bool,
    pub custom_doh_url: Option<String>,
    pub proxy_enabled: bool,
    pub proxy_url: Option<String>,
    pub cloudflare_media_enabled: bool,
    pub cloudflare_media_endpoint: Option<String>,
    pub download_directory: Option<String>,
}

impl Default for NetworkSettings {
    fn default() -> Self {
        Self {
            dns_provider: "cloudflare".into(),
            doh_enabled: true,
            custom_doh_url: None,
            proxy_enabled: false,
            proxy_url: None,
            cloudflare_media_enabled: true,
            cloudflare_media_endpoint: Some("https://media.omnilauncher.app/cdn-cgi/image".into()),
            download_directory: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AppConfig {
    pub steam: SteamCredentials,
    pub capture: CaptureConfig,
    pub preferred_currency: String,
    pub network: NetworkSettings,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            steam: SteamCredentials::default(),
            capture: CaptureConfig::default(),
            preferred_currency: "USD".into(),
            network: NetworkSettings::default(),
        }
    }
}

impl AppConfig {
    /// Synchronous on purpose: runs once inside Tauri's `setup` hook.
    pub fn load(dir: &Path) -> Result<Self> {
        let path = dir.join(CONFIG_FILE);
        match std::fs::read_to_string(&path) {
            Ok(raw) => toml::from_str(&raw)
                .map_err(|e| LauncherError::Config(format!("{}: {e}", path.display()))),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(source) => Err(LauncherError::Io { path, source }),
        }
    }
}

/// Async, serialized access to the on-disk configuration.
pub struct ConfigStore {
    dir: PathBuf,
    current: Mutex<AppConfig>,
}

impl ConfigStore {
    pub fn new(dir: PathBuf, initial: AppConfig) -> Self {
        Self {
            dir,
            current: Mutex::new(initial),
        }
    }

    pub async fn snapshot(&self) -> AppConfig {
        self.current.lock().await.clone()
    }

    /// Mutate and atomically persist the configuration.
    pub async fn update<F>(&self, mutate: F) -> Result<AppConfig>
    where
        F: FnOnce(&mut AppConfig) + Send,
    {
        let mut guard = self.current.lock().await;
        let mut next = guard.clone();
        mutate(&mut next);

        let serialized = toml::to_string_pretty(&next).map_err(|e| LauncherError::Config(e.to_string()))?;
        tokio::fs::create_dir_all(&self.dir).await.with_path(&self.dir)?;
        let path = self.dir.join(CONFIG_FILE);
        let tmp = self.dir.join(format!("{CONFIG_FILE}.tmp"));
        tokio::fs::write(&tmp, serialized).await.with_path(&tmp)?;
        tokio::fs::rename(&tmp, &path).await.with_path(&path)?;

        *guard = next.clone();
        Ok(next)
    }
}
