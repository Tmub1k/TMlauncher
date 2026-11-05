//! Valve KeyValues (VDF / ACF) parsers for Steam library discovery.
//!
//! Parses:
//! - `<SteamPath>/steamapps/libraryfolders.vdf` (both modern object format and
//!   legacy string-path format).
//! - `<library>/steamapps/appmanifest_<appid>.acf` (extracting `appid`, `name`,
//!   `installdir`, `StateFlags`, `SizeOnDisk`, `LastPlayed`).
//! - `<SteamPath>/userdata/<AccountID>/<appid>/remotecache.vdf` for cloud save
//!   file tracking.

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use chrono::{DateTime, TimeZone, Utc};
use serde::{de::Deserializer, Deserialize, Serialize};

use crate::error::{LauncherError, Result};

/// Steam `StateFlags == 4` (`k_EAppStateFullyInstalled`) marks a clean, ready-to-launch install.
pub const STATE_FULLY_INSTALLED: u32 = 4;

// ---------------------------------------------------------------------------
// libraryfolders.vdf
// ---------------------------------------------------------------------------

/// An entry inside `libraryfolders.vdf` can be either a nested table with a
/// `"path"` key (modern Steam) or a bare string path (older Steam clients).
#[derive(Debug, Clone)]
pub struct LibraryFolderEntry {
    pub path: PathBuf,
    pub label: Option<String>,
    pub total_size: Option<u64>,
    pub apps: HashMap<String, String>,
}

impl<'de> Deserialize<'de> for LibraryFolderEntry {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum RawEntry {
            Modern {
                path: String,
                #[serde(default)]
                label: Option<String>,
                #[serde(default, rename = "totalsize")]
                total_size: Option<String>,
                #[serde(default)]
                apps: HashMap<String, String>,
            },
            LegacyPath(String),
        }

        match RawEntry::deserialize(deserializer)? {
            RawEntry::Modern {
                path,
                label,
                total_size,
                apps,
            } => Ok(Self {
                path: normalize_vdf_path(&path),
                label: label.filter(|s| !s.is_empty()),
                total_size: total_size.and_then(|s| s.parse().ok()),
                apps,
            }),
            RawEntry::LegacyPath(path) => Ok(Self {
                path: normalize_vdf_path(&path),
                label: None,
                total_size: None,
                apps: HashMap::new(),
            }),
        }
    }
}

/// Parse `<SteamPath>/steamapps/libraryfolders.vdf` and return every unique
/// Steam library root directory (including `steam_root` itself).
pub fn parse_library_folders_file(path: &Path, steam_root: &Path) -> Result<Vec<PathBuf>> {
    let raw = std::fs::read_to_string(path).map_err(|source| LauncherError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    parse_library_folders_str(&raw, path, steam_root)
}

pub fn parse_library_folders_str(
    raw: &str,
    source_path: &Path,
    steam_root: &Path,
) -> Result<Vec<PathBuf>> {
    let parsed: HashMap<String, serde_json::Value> =
        keyvalues_serde::from_str(raw).map_err(|e| LauncherError::VdfParse {
            path: source_path.to_path_buf(),
            message: e.to_string(),
        })?;

    let mut folders = vec![steam_root.to_path_buf()];

    for (key, value) in parsed {
        // Numeric keys ("0", "1", "2", ...) denote library folder indices.
        if key.chars().all(|c| c.is_ascii_digit()) {
            if let Ok(entry) = serde_json::from_value::<LibraryFolderEntry>(value) {
                if !folders.iter().any(|existing| same_path(existing, &entry.path)) {
                    folders.push(entry.path);
                }
            }
        }
    }

    Ok(folders)
}

// ---------------------------------------------------------------------------
// appmanifest_<appid>.acf
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct RawAppState {
    #[serde(alias = "appid", alias = "AppID")]
    pub appid: String,
    #[serde(alias = "name", alias = "Name", default)]
    pub name: String,
    #[serde(alias = "installdir", alias = "InstallDir", default)]
    pub installdir: String,
    #[serde(alias = "StateFlags", default)]
    pub state_flags: String,
    #[serde(alias = "SizeOnDisk", default)]
    pub size_on_disk: Option<String>,
    #[serde(alias = "LastPlayed", default)]
    pub last_played: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SteamAppManifest {
    pub app_id: u32,
    pub name: String,
    pub install_dir: PathBuf,
    pub state_flags: u32,
    pub size_bytes: Option<u64>,
    pub last_played: Option<DateTime<Utc>>,
}

impl SteamAppManifest {
    /// Returns `true` when `StateFlags` indicates a complete installation
    /// (`StateFlags == 4` or has bit 4 set with no active update errors).
    pub fn is_fully_installed(&self) -> bool {
        self.state_flags == STATE_FULLY_INSTALLED || (self.state_flags & STATE_FULLY_INSTALLED) != 0
    }
}

/// Parse a single `appmanifest_<appid>.acf` file and resolve its `common/<installdir>` path.
pub fn parse_app_manifest_file(acf_path: &Path, steamapps_dir: &Path) -> Result<SteamAppManifest> {
    let raw = std::fs::read_to_string(acf_path).map_err(|source| LauncherError::Io {
        path: acf_path.to_path_buf(),
        source,
    })?;
    parse_app_manifest_str(&raw, acf_path, steamapps_dir)
}

pub fn parse_app_manifest_str(
    raw: &str,
    acf_path: &Path,
    steamapps_dir: &Path,
) -> Result<SteamAppManifest> {
    let state: RawAppState =
        keyvalues_serde::from_str(raw).map_err(|e| LauncherError::VdfParse {
            path: acf_path.to_path_buf(),
            message: e.to_string(),
        })?;

    let app_id: u32 = state.appid.trim().parse().map_err(|_| LauncherError::VdfParse {
        path: acf_path.to_path_buf(),
        message: format!("invalid appid `{}`", state.appid),
    })?;

    let state_flags: u32 = state.state_flags.trim().parse().unwrap_or(0);
    let size_bytes = state
        .size_on_disk
        .as_deref()
        .and_then(|s| s.trim().parse::<u64>().ok());

    let last_played = state
        .last_played
        .as_deref()
        .and_then(|s| s.trim().parse::<i64>().ok())
        .filter(|&ts| ts > 0)
        .and_then(|ts| Utc.timestamp_opt(ts, 0).single());

    let title = if state.name.trim().is_empty() {
        format!("Steam App {app_id}")
    } else {
        state.name.trim().to_string()
    };

    let install_dir = steamapps_dir.join("common").join(state.installdir.trim());

    Ok(SteamAppManifest {
        app_id,
        name: title,
        install_dir,
        state_flags,
        size_bytes,
        last_played,
    })
}

// ---------------------------------------------------------------------------
// remotecache.vdf (Steam Cloud save status)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CloudSaveFile {
    pub relative_path: String,
    pub size_bytes: u64,
    pub timestamp: Option<DateTime<Utc>>,
    pub synced: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CloudSaveSummary {
    pub app_id: u32,
    pub account_id: String,
    pub remote_dir: PathBuf,
    pub files: Vec<CloudSaveFile>,
    pub total_bytes: u64,
}

/// Inspect `<SteamPath>/userdata/<AccountID>/<appid>/remotecache.vdf` if present.
pub fn parse_remotecache(
    steam_root: &Path,
    account_id: &str,
    app_id: u32,
) -> Result<Option<CloudSaveSummary>> {
    let app_user_dir = steam_root
        .join("userdata")
        .join(account_id)
        .join(app_id.to_string());
    let vdf_path = app_user_dir.join("remotecache.vdf");
    if !vdf_path.exists() {
        return Ok(None);
    }

    let raw = std::fs::read_to_string(&vdf_path).map_err(|source| LauncherError::Io {
        path: vdf_path.clone(),
        source,
    })?;

    let table: HashMap<String, serde_json::Value> =
        keyvalues_serde::from_str(&raw).map_err(|e| LauncherError::VdfParse {
            path: vdf_path,
            message: e.to_string(),
        })?;

    let mut files = Vec::new();
    let mut total_bytes = 0u64;

    for (rel_path, entry) in table {
        let Some(obj) = entry.as_object() else {
            continue;
        };
        let size_bytes = obj
            .get("size")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);
        let timestamp = obj
            .get("localtime")
            .or_else(|| obj.get("time"))
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<i64>().ok())
            .filter(|&ts| ts > 0)
            .and_then(|ts| Utc.timestamp_opt(ts, 0).single());
        let sync_state = obj
            .get("syncstate")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0);

        total_bytes += size_bytes;
        files.push(CloudSaveFile {
            relative_path: rel_path,
            size_bytes,
            timestamp,
            synced: sync_state == 1 || sync_state == 4,
        });
    }

    Ok(Some(CloudSaveSummary {
        app_id,
        account_id: account_id.to_string(),
        remote_dir: app_user_dir.join("remote"),
        files,
        total_bytes,
    }))
}

// ---------------------------------------------------------------------------
// Path normalization helpers
// ---------------------------------------------------------------------------

fn normalize_vdf_path(raw: &str) -> PathBuf {
    let unescaped = raw.replace("\\\\", "\\");
    PathBuf::from(unescaped.trim())
}

fn same_path(a: &Path, b: &Path) -> bool {
    a.to_string_lossy()
        .replace('/', "\\")
        .eq_ignore_ascii_case(&b.to_string_lossy().replace('/', "\\"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_modern_library_folders_vdf() {
        let sample = r#"
"libraryfolders"
{
    "0"
    {
        "path"		"C:\\Program Files (x86)\\Steam"
        "label"		""
        "totalsize"		"0"
        "apps"
        {
            "228980"		"412356789"
            "570"		"32145678900"
        }
    }
    "1"
    {
        "path"		"D:\\SteamLibrary"
        "label"		"NVMe Games"
        "totalsize"		"1000202039296"
        "apps"
        {
            "1091500"		"70123456789"
        }
    }
}
"#;
        let root = Path::new(r"C:\Program Files (x86)\Steam");
        let libs = parse_library_folders_str(sample, Path::new("libraryfolders.vdf"), root).unwrap();
        assert_eq!(libs.len(), 2);
        assert_eq!(libs[1], PathBuf::from(r"D:\SteamLibrary"));
    }

    #[test]
    fn parses_app_manifest_acf() {
        let sample = r#"
"AppState"
{
    "appid"		"1091500"
    "Universe"		"1"
    "name"		"Cyberpunk 2077"
    "StateFlags"		"4"
    "installdir"		"Cyberpunk 2077"
    "LastUpdated"		"1700000000"
    "SizeOnDisk"		"75400123456"
    "LastPlayed"		"1705000000"
}
"#;
        let manifest = parse_app_manifest_str(
            sample,
            Path::new("appmanifest_1091500.acf"),
            Path::new(r"D:\SteamLibrary\steamapps"),
        )
        .unwrap();
        assert_eq!(manifest.app_id, 1091500);
        assert_eq!(manifest.name, "Cyberpunk 2077");
        assert!(manifest.is_fully_installed());
        assert_eq!(manifest.size_bytes, Some(75_400_123_456));
    }
}
