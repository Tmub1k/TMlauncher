//! Shader Cache & Disk Reclaimer (`commands/shader_reclaimer.rs`).
//!
//! Discovers orphaned Steam shader caches, DirectX/GPU driver caches,
//! safely calculates reclaimable disk space, and cleans caches with
//! locked-file resilience.

use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheItem {
    pub id: String,
    pub app_id: Option<u32>,
    pub game_title: String,
    pub path: String,
    pub size_bytes: u64,
    pub is_installed: bool,
    pub category: String,
    pub is_safe_to_delete: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShaderCacheSummary {
    pub total_bytes: u64,
    pub item_count: usize,
    pub caches: Vec<CacheItem>,
    pub scanned_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanRequest {
    pub item_ids: Vec<String>,
    pub only_orphans: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanReport {
    pub total_freed_bytes: u64,
    pub cleaned_items_count: usize,
    pub failed_items_count: usize,
    pub skipped_locked_files: usize,
    pub cleaned_at: i64,
    pub message: String,
}

fn compute_dir_size(path: &Path) -> u64 {
    let mut total = 0;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                total += compute_dir_size(&p);
            } else if let Ok(meta) = entry.metadata() {
                total += meta.len();
            }
        }
    }
    total
}

/// Discovers all Steam libraries and installed app IDs.
fn scan_steam_shader_caches() -> Vec<CacheItem> {
    let mut results = Vec::new();

    let steam_root = match crate::platforms::steam::SteamPlatform::locate_steam_root() {
        Ok(r) => r,
        Err(_) => return results,
    };

    let libraryfolders_vdf = steam_root.join("steamapps").join("libraryfolders.vdf");
    let mut library_roots = vec![steam_root.clone()];

    if libraryfolders_vdf.exists() {
        if let Ok(folders) = crate::platforms::steam::vdf_parser::parse_library_folders_file(&libraryfolders_vdf, &steam_root) {
            for folder in folders {
                if !library_roots.contains(&folder) {
                    library_roots.push(folder);
                }
            }
        }
    }

    for lib_root in library_roots {
        let steamapps = lib_root.join("steamapps");
        let shadercache_dir = steamapps.join("shadercache");

        if !shadercache_dir.exists() {
            continue;
        }

        // Collect installed app IDs from appmanifest_*.acf
        let mut installed_app_ids = HashSet::new();
        if let Ok(entries) = fs::read_dir(&steamapps) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with("appmanifest_") && name.ends_with(".acf") {
                    let num_str = name.trim_start_matches("appmanifest_").trim_end_matches(".acf");
                    if let Ok(id) = num_str.parse::<u32>() {
                        installed_app_ids.insert(id);
                    }
                }
            }
        }

        // Check shadercache folders
        if let Ok(entries) = fs::read_dir(&shadercache_dir) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                if entry_path.is_dir() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if let Ok(app_id) = name.parse::<u32>() {
                        let size = compute_dir_size(&entry_path);
                        if size > 0 {
                            let is_installed = installed_app_ids.contains(&app_id);
                            let title = if is_installed {
                                format!("Steam App {app_id}")
                            } else {
                                format!("Orphaned Steam Game ({app_id})")
                            };

                            results.push(CacheItem {
                                id: format!("steam_{app_id}"),
                                app_id: Some(app_id),
                                game_title: title,
                                path: entry_path.to_string_lossy().to_string(),
                                size_bytes: size,
                                is_installed,
                                category: "steam_shader".to_string(),
                                is_safe_to_delete: !is_installed,
                            });
                        }
                    }
                }
            }
        }
    }

    results
}

fn scan_gpu_driver_caches() -> Vec<CacheItem> {
    let mut results = Vec::new();

    if let Some(local_app_data) = dirs::data_local_dir() {
        let targets = [
            ("directx_d3d", "DirectX D3D Shader Cache", local_app_data.join("D3DSCache"), "directx"),
            ("nvidia_dxcache", "NVIDIA DXCache", local_app_data.join("NVIDIA").join("DXCache"), "nvidia"),
            ("nvidia_glcache", "NVIDIA GLCache", local_app_data.join("NVIDIA").join("GLCache"), "nvidia"),
            ("amd_dxcache", "AMD Driver DxCache", local_app_data.join("AMD").join("DxCache"), "amd"),
            ("launcher_temp", "Launcher Image & HTTP Cache", local_app_data.join("OmniLauncher").join("cache"), "launcher_temp"),
        ];

        for (id, label, path, category) in targets {
            if path.exists() {
                let size = compute_dir_size(&path);
                if size > 0 {
                    results.push(CacheItem {
                        id: id.to_string(),
                        app_id: None,
                        game_title: label.to_string(),
                        path: path.to_string_lossy().to_string(),
                        size_bytes: size,
                        is_installed: false, // driver caches are shared/global
                        category: category.to_string(),
                        is_safe_to_delete: true,
                    });
                }
            }
        }
    }

    results
}

#[tauri::command]
pub async fn scan_shader_caches() -> Result<ShaderCacheSummary, String> {
    tokio::task::spawn_blocking(move || {
        let mut all_caches = scan_steam_shader_caches();
        all_caches.extend(scan_gpu_driver_caches());

        let total_bytes = all_caches.iter().map(|c| c.size_bytes).sum();
        let item_count = all_caches.len();

        Ok(ShaderCacheSummary {
            total_bytes,
            item_count,
            caches: all_caches,
            scanned_at: Utc::now().timestamp(),
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

fn delete_directory_contents_safe(path: &Path, freed_bytes: &mut u64, skipped_locked: &mut usize, failed_count: &mut usize) {
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let ep = entry.path();
            if ep.is_dir() {
                delete_directory_contents_safe(&ep, freed_bytes, skipped_locked, failed_count);
                let _ = fs::remove_dir(&ep);
            } else if let Ok(meta) = entry.metadata() {
                let len = meta.len();
                match fs::remove_file(&ep) {
                    Ok(()) => {
                        *freed_bytes += len;
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied || e.raw_os_error() == Some(32) => {
                        *skipped_locked += 1;
                    }
                    Err(_) => {
                        *failed_count += 1;
                    }
                }
            }
        }
    }
}

#[tauri::command]
pub async fn clean_shader_caches(request: CleanRequest) -> Result<CleanReport, String> {
    tokio::task::spawn_blocking(move || {
        let summary = scan_shader_caches_internal();
        let mut total_freed_bytes = 0u64;
        let mut cleaned_items_count = 0;
        let mut failed_items_count = 0;
        let mut skipped_locked_files = 0;

        let target_ids: HashSet<String> = request.item_ids.into_iter().collect();

        for item in summary.caches {
            if !target_ids.is_empty() && !target_ids.contains(&item.id) {
                continue;
            }

            if request.only_orphans && item.is_installed {
                continue;
            }

            let path = PathBuf::from(&item.path);
            if !path.exists() {
                continue;
            }

            let mut freed_for_item = 0;
            delete_directory_contents_safe(&path, &mut freed_for_item, &mut skipped_locked_files, &mut failed_items_count);

            if !item.is_installed && item.category == "steam_shader" {
                let _ = fs::remove_dir_all(&path);
            }

            total_freed_bytes += freed_for_item;
            cleaned_items_count += 1;
        }

        let message = if cleaned_items_count == 0 {
            "All shader caches are active. No orphaned files found.".to_string()
        } else {
            format!("Cleaned {cleaned_items_count} cache targets, freed {:.2} MB", total_freed_bytes as f64 / (1024.0 * 1024.0))
        };

        Ok(CleanReport {
            total_freed_bytes,
            cleaned_items_count,
            failed_items_count,
            skipped_locked_files,
            cleaned_at: Utc::now().timestamp(),
            message,
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

fn scan_shader_caches_internal() -> ShaderCacheSummary {
    let mut all_caches = scan_steam_shader_caches();
    all_caches.extend(scan_gpu_driver_caches());
    let total_bytes = all_caches.iter().map(|c| c.size_bytes).sum();
    let item_count = all_caches.len();
    ShaderCacheSummary {
        total_bytes,
        item_count,
        caches: all_caches,
        scanned_at: Utc::now().timestamp(),
    }
}
