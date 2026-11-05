//! Save-State Time Machine (`commands/save_vault.rs`).
//!
//! Provides automatic pre/post-session snapshotting, manual bookmarking,
//! SHA-256 integrity checks, Zip-Slip-resistant extraction, and automatic
//! PreRestore safety backups.

use std::{
    fs::{self, File},
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use zip::{write::SimpleFileOptions, CompressionMethod, ZipArchive, ZipWriter};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveSnapshot {
    pub snapshot_id: String,
    pub game_id: String,
    pub timestamp: i64,
    pub archive_size_bytes: u64,
    pub sha256_checksum: String,
    pub title: String,
    pub label: String,
    pub kind: String,
    pub file_count: usize,
    pub uncompressed_bytes: u64,
    pub source_path: String,
    pub archive_path: String,
    pub pinned: bool,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveLocationCandidate {
    pub path: String,
    pub exists: bool,
    pub file_count: usize,
    pub total_size_bytes: u64,
    pub source: String,
    pub is_primary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreReport {
    pub success: bool,
    pub snapshot_id: String,
    pub target_path: String,
    pub restored_files_count: usize,
    pub safety_snapshot_id: Option<String>,
    pub restored_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultSummary {
    pub total_snapshots: usize,
    pub total_games: usize,
    pub total_vault_bytes: u64,
    pub oldest_snapshot_timestamp: Option<i64>,
    pub newest_snapshot_timestamp: Option<i64>,
}

fn get_vault_root() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("OmniLauncher")
        .join("save_vault")
}

fn get_game_vault_dir(game_id: &str) -> PathBuf {
    get_vault_root().join(game_id)
}

fn get_manifest_path(game_id: &str) -> PathBuf {
    get_game_vault_dir(game_id).join("manifest.json")
}

fn load_snapshots_manifest(game_id: &str) -> Vec<SaveSnapshot> {
    let p = get_manifest_path(game_id);
    if !p.exists() {
        return Vec::new();
    }
    match fs::read_to_string(&p) {
        Ok(json) => serde_json::from_str(&json).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

fn save_snapshots_manifest(game_id: &str, snapshots: &[SaveSnapshot]) -> io::Result<()> {
    let p = get_manifest_path(game_id);
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(snapshots)?;
    fs::write(&p, json)?;
    Ok(())
}

/// Automatically locate or guess save directory for a game.
pub fn resolve_save_directory(game_id: &str, custom_path: Option<&str>) -> Option<PathBuf> {
    if let Some(cp) = custom_path {
        let p = PathBuf::from(cp);
        if p.exists() {
            return Some(p);
        }
    }

    let candidates = detect_save_locations_sync(game_id);
    candidates
        .into_iter()
        .find(|c| c.exists && c.file_count > 0)
        .map(|c| PathBuf::from(c.path))
}

fn detect_save_locations_sync(game_id: &str) -> Vec<SaveLocationCandidate> {
    let mut results = Vec::new();

    // 1. Saved Games (%USERPROFILE%\Saved Games\<title>)
    if let Some(user_profile) = dirs::home_dir() {
        let p = user_profile.join("Saved Games").join(game_id);
        results.push(make_candidate(&p, "saved_games", true));
    }

    // 2. LocalAppData (%LOCALAPPDATA%\<title>\Saved\SaveGames)
    if let Some(local) = dirs::data_local_dir() {
        let p1 = local.join(game_id).join("Saved").join("SaveGames");
        let p2 = local.join(game_id);
        results.push(make_candidate(&p1, "appdata_local_unreal", false));
        results.push(make_candidate(&p2, "appdata_local", false));
    }

    // 3. Roaming AppData (%APPDATA%\<title>)
    if let Some(roaming) = dirs::config_dir() {
        let p = roaming.join(game_id);
        results.push(make_candidate(&p, "appdata_roaming", false));
    }

    // 4. Documents / My Games (%USERPROFILE%\Documents\My Games\<title>)
    if let Some(docs) = dirs::document_dir() {
        let p = docs.join("My Games").join(game_id);
        results.push(make_candidate(&p, "documents_my_games", false));
    }

    // 5. Steam userdata if game_id is numeric app_id
    if let Ok(app_id) = game_id.parse::<u32>() {
        if let Ok(steam_root) = crate::platforms::steam::SteamPlatform::locate_steam_root() {
            let userdata = steam_root.join("userdata");
            if userdata.exists() {
                if let Ok(entries) = fs::read_dir(&userdata) {
                    for entry in entries.flatten() {
                        let remote = entry.path().join(app_id.to_string()).join("remote");
                        if remote.exists() {
                            results.push(make_candidate(&remote, "steam_userdata", false));
                        }
                    }
                }
            }
        }
    }

    results
}

fn make_candidate(p: &Path, source: &str, is_primary: bool) -> SaveLocationCandidate {
    let exists = p.exists();
    let mut file_count = 0;
    let mut total_size = 0;

    if exists {
        if let Ok(entries) = fs::read_dir(p) {
            for entry in entries.flatten() {
                if let Ok(meta) = entry.metadata() {
                    if meta.is_file() {
                        file_count += 1;
                        total_size += meta.len();
                    }
                }
            }
        }
    }

    SaveLocationCandidate {
        path: p.to_string_lossy().to_string(),
        exists,
        file_count,
        total_size_bytes: total_size,
        source: source.to_string(),
        is_primary,
    }
}

/// Compute SHA256 of file.
fn compute_sha256(path: &Path) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }
    Ok(hex::encode(hasher.finalize()))
}

#[tauri::command]
pub async fn create_save_snapshot(
    game_id: String,
    label: Option<String>,
    kind: Option<String>,
    custom_path: Option<String>,
) -> Result<SaveSnapshot, String> {
    tokio::task::spawn_blocking(move || {
        let save_dir = resolve_save_directory(&game_id, custom_path.as_deref())
            .ok_or_else(|| format!("No active save directory found for game: {game_id}"))?;

        let vault_dir = get_game_vault_dir(&game_id);
        fs::create_dir_all(&vault_dir).map_err(|e| e.to_string())?;

        let timestamp = Utc::now().timestamp();
        let iso_time = Utc::now().to_rfc3339().replace([':', '.'], "-");
        let snapshot_id = format!("snap_{iso_time}_{game_id}");
        let filename = format!("save_vault_{game_id}_{iso_time}.zip");
        let archive_path = vault_dir.join(&filename);

        let file = File::create(&archive_path).map_err(|e| e.to_string())?;
        let mut zip = ZipWriter::new(file);
        let options = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .unix_permissions(0o755);

        let mut file_count = 0;
        let mut uncompressed_bytes = 0;

        // Recursive directory walk
        let mut dirs_to_visit = vec![save_dir.clone()];
        while let Some(current_dir) = dirs_to_visit.pop() {
            let entries = fs::read_dir(&current_dir).map_err(|e| e.to_string())?;
            for entry in entries.flatten() {
                let entry_path = entry.path();
                if entry_path.is_dir() {
                    dirs_to_visit.push(entry_path);
                } else if entry_path.is_file() {
                    let relative = entry_path
                        .strip_prefix(&save_dir)
                        .unwrap_or(&entry_path)
                        .to_string_lossy()
                        .replace('\\', "/");

                    zip.start_file(relative, options).map_err(|e| e.to_string())?;
                    let mut src_file = File::open(&entry_path).map_err(|e| e.to_string())?;
                    let mut buf = Vec::new();
                    src_file.read_to_end(&mut buf).map_err(|e| e.to_string())?;
                    zip.write_all(&buf).map_err(|e| e.to_string())?;

                    file_count += 1;
                    uncompressed_bytes += buf.len() as u64;
                }
            }
        }

        zip.finish().map_err(|e| e.to_string())?;

        let archive_size_bytes = fs::metadata(&archive_path)
            .map(|m| m.len())
            .unwrap_or(0);
        let sha256_checksum = compute_sha256(&archive_path).map_err(|e| e.to_string())?;

        let snapshot = SaveSnapshot {
            snapshot_id: snapshot_id.clone(),
            game_id: game_id.clone(),
            timestamp,
            archive_size_bytes,
            sha256_checksum,
            title: game_id.clone(),
            label: label.unwrap_or_else(|| "Snapshot".to_string()),
            kind: kind.unwrap_or_else(|| "manual".to_string()),
            file_count,
            uncompressed_bytes,
            source_path: save_dir.to_string_lossy().to_string(),
            archive_path: archive_path.to_string_lossy().to_string(),
            pinned: false,
            note: None,
        };

        // Append to manifest and enforce rotation limit
        let mut snapshots = load_snapshots_manifest(&game_id);
        snapshots.insert(0, snapshot.clone());

        // Pruning: Keep pinned + up to 10 unpinned snapshots
        const MAX_UNPINNED: usize = 10;
        let mut unpinned_count = 0;
        snapshots.retain(|s| {
            if s.pinned {
                true
            } else {
                unpinned_count += 1;
                if unpinned_count > MAX_UNPINNED {
                    let _ = fs::remove_file(&s.archive_path);
                    false
                } else {
                    true
                }
            }
        });

        let _ = save_snapshots_manifest(&game_id, &snapshots);

        Ok(snapshot)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn list_save_snapshots(game_id: String) -> Result<Vec<SaveSnapshot>, String> {
    tokio::task::spawn_blocking(move || {
        Ok(load_snapshots_manifest(&game_id))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn restore_save_snapshot(
    game_id: String,
    snapshot_id: String,
) -> Result<RestoreReport, String> {
    tokio::task::spawn_blocking(move || {
        let snapshots = load_snapshots_manifest(&game_id);
        let target_snap = snapshots
            .iter()
            .find(|s| s.snapshot_id == snapshot_id)
            .ok_or_else(|| format!("Snapshot {snapshot_id} not found"))?
            .clone();

        let target_dir = PathBuf::from(&target_snap.source_path);

        // 1. Verify archive integrity
        let computed_sha = compute_sha256(Path::new(&target_snap.archive_path))
            .map_err(|e| format!("Failed to read archive: {e}"))?;
        if computed_sha != target_snap.sha256_checksum {
            return Err("Archive integrity check failed: SHA-256 checksum mismatch".to_string());
        }

        // 2. Create emergency safety backup if target directory exists
        let mut safety_snapshot_id = None;
        if target_dir.exists() {
            let iso_time = Utc::now().to_rfc3339().replace([':', '.'], "-");
            let s_id = format!("safety_prerestore_{iso_time}_{game_id}");
            let s_path = get_game_vault_dir(&game_id).join(format!("safety_{s_id}.zip"));

            if let Ok(file) = File::create(&s_path) {
                let mut zip = ZipWriter::new(file);
                let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
                let mut dirs = vec![target_dir.clone()];
                let mut s_file_count = 0usize;
                let mut s_uncompressed = 0u64;
                while let Some(cd) = dirs.pop() {
                    if let Ok(entries) = fs::read_dir(&cd) {
                        for entry in entries.flatten() {
                            let ep = entry.path();
                            if ep.is_dir() {
                                dirs.push(ep);
                            } else if ep.is_file() {
                                if let Ok(rel) = ep.strip_prefix(&target_dir) {
                                    let _ = zip.start_file(rel.to_string_lossy().replace('\\', "/"), options);
                                    if let Ok(bytes) = fs::read(&ep) {
                                        s_uncompressed += bytes.len() as u64;
                                        s_file_count += 1;
                                        let _ = zip.write_all(&bytes);
                                    }
                                }
                            }
                        }
                    }
                }
                let _ = zip.finish();
                let s_archive_size = fs::metadata(&s_path).map(|m| m.len()).unwrap_or(0);
                let s_sha = compute_sha256(&s_path).unwrap_or_default();
                let safety_snap = SaveSnapshot {
                    snapshot_id: s_id.clone(),
                    game_id: game_id.clone(),
                    timestamp: Utc::now().timestamp(),
                    archive_size_bytes: s_archive_size,
                    sha256_checksum: s_sha,
                    title: game_id.clone(),
                    label: "Auto Pre-Restore Safeguard".to_string(),
                    kind: "safeguard".to_string(),
                    file_count: s_file_count,
                    uncompressed_bytes: s_uncompressed,
                    source_path: target_dir.to_string_lossy().to_string(),
                    archive_path: s_path.to_string_lossy().to_string(),
                    pinned: false,
                    note: Some(format!("Captured automatically before restoring {snapshot_id}")),
                };
                let mut updated_snapshots = load_snapshots_manifest(&game_id);
                updated_snapshots.insert(0, safety_snap);
                let _ = save_snapshots_manifest(&game_id, &updated_snapshots);
                safety_snapshot_id = Some(s_id);
            }
        }

        // 3. Extract target archive with Zip-Slip protection
        let zip_file = File::open(&target_snap.archive_path).map_err(|e| e.to_string())?;
        let mut archive = ZipArchive::new(zip_file).map_err(|e| e.to_string())?;
        let mut restored_count = 0;

        fs::create_dir_all(&target_dir).map_err(|e| e.to_string())?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
            let enclosed = file.enclosed_name().ok_or_else(|| {
                format!("Zip-Slip vulnerability detected in entry: {}", file.name())
            })?;

            let out_path = target_dir.join(enclosed);
            if file.is_dir() {
                fs::create_dir_all(&out_path).map_err(|e| e.to_string())?;
            } else {
                if let Some(parent) = out_path.parent() {
                    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                }
                let mut out_file = File::create(&out_path).map_err(|e| e.to_string())?;
                io::copy(&mut file, &mut out_file).map_err(|e| e.to_string())?;
                restored_count += 1;
            }
        }

        Ok(RestoreReport {
            success: true,
            snapshot_id,
            target_path: target_dir.to_string_lossy().to_string(),
            restored_files_count: restored_count,
            safety_snapshot_id,
            restored_at: Utc::now().timestamp(),
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn delete_save_snapshot(
    game_id: String,
    snapshot_id: String,
) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let mut snapshots = load_snapshots_manifest(&game_id);
        if let Some(pos) = snapshots.iter().position(|s| s.snapshot_id == snapshot_id) {
            let snap = snapshots.remove(pos);
            let _ = fs::remove_file(&snap.archive_path);
            let _ = save_snapshots_manifest(&game_id, &snapshots);
        }
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn toggle_snapshot_pin(
    game_id: String,
    snapshot_id: String,
) -> Result<bool, String> {
    tokio::task::spawn_blocking(move || {
        let mut snapshots = load_snapshots_manifest(&game_id);
        let mut new_pinned = false;
        if let Some(snap) = snapshots.iter_mut().find(|s| s.snapshot_id == snapshot_id) {
            snap.pinned = !snap.pinned;
            new_pinned = snap.pinned;
            let _ = save_snapshots_manifest(&game_id, &snapshots);
        }
        Ok(new_pinned)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn open_save_folder(game_id: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        let path = resolve_save_directory(&game_id, None)
            .unwrap_or_else(|| get_game_vault_dir(&game_id));

        if !path.exists() {
            let _ = fs::create_dir_all(&path);
        }

        #[cfg(windows)]
        {
            let _ = std::process::Command::new("explorer.exe")
                .arg(&path)
                .spawn();
        }

        Ok(path.to_string_lossy().to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn detect_game_save_locations(game_id: String) -> Result<Vec<SaveLocationCandidate>, String> {
    tokio::task::spawn_blocking(move || {
        Ok(detect_save_locations_sync(&game_id))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn get_save_vault_summary() -> Result<VaultSummary, String> {
    tokio::task::spawn_blocking(move || {
        let root = get_vault_root();
        let mut total_snapshots = 0;
        let mut total_games = 0;
        let mut total_vault_bytes = 0;
        let mut oldest = None;
        let mut newest = None;

        if let Ok(entries) = fs::read_dir(&root) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    total_games += 1;
                    let manifest = entry.path().join("manifest.json");
                    if let Ok(content) = fs::read_to_string(&manifest) {
                        if let Ok(snaps) = serde_json::from_str::<Vec<SaveSnapshot>>(&content) {
                            for s in snaps {
                                total_snapshots += 1;
                                total_vault_bytes += s.archive_size_bytes;
                                oldest = Some(oldest.map_or(s.timestamp, |o: i64| o.min(s.timestamp)));
                                newest = Some(newest.map_or(s.timestamp, |n: i64| n.max(s.timestamp)));
                            }
                        }
                    }
                }
            }
        }

        Ok(VaultSummary {
            total_snapshots,
            total_games,
            total_vault_bytes,
            oldest_snapshot_timestamp: oldest,
            newest_snapshot_timestamp: newest,
        })
    })
    .await
    .map_err(|e| e.to_string())?
}
