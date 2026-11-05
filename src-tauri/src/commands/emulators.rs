//! Curated GitHub Open-Source Emulators Catalog & 1-Click Downloader/Extractor.

use std::fs::File;
use std::path::{Path, PathBuf};
use std::time::Instant;

use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};
use tokio::io::AsyncWriteExt;
use zip::ZipArchive;

use crate::EVT_LIBRARY_UPDATED;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmulatorCatalogEntry {
    pub id: String,
    pub name: String,
    pub repo: String,
    pub version: String,
    pub installed: bool,
    pub executable_name: String,
    #[serde(rename = "executableName")]
    pub executable_name_camel: String,
    pub arch: String,
    pub system: String,
    pub short_system: String,
    pub author: String,
    pub description: String,
    pub supported_extensions: Vec<String>,
    pub install_path: Option<String>,
    pub executable_path: Option<String>,
    pub is_rate_limited: bool,
    pub website: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmulatorInstallResult {
    pub success: bool,
    pub installed: bool,
    pub emulator_id: String,
    pub installed_location: String,
    #[serde(rename = "installedLocation")]
    pub installed_location_camel: String,
    pub used_builtin_picker: bool,
    #[serde(rename = "usedBuiltinPicker")]
    pub used_builtin_picker_camel: bool,
    pub dialog_type: String,
    #[serde(rename = "dialogType")]
    pub dialog_type_camel: String,
    pub registered_game_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmulatorProgressEvent {
    pub emulator_id: String,
    pub phase: String, // "downloading" | "extracting" | "registering" | "completed" | "error"
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub percentage: f64,
    pub speed_mb_s: f64,
    pub error: Option<String>,
}

#[allow(dead_code)]
struct StaticEmuPreset {
    id: &'static str,
    name: &'static str,
    repo: &'static str,
    version: &'static str,
    executable_name: &'static str,
    system: &'static str,
    short_system: &'static str,
    author: &'static str,
    description: &'static str,
    supported_extensions: &'static [&'static str],
    website: &'static str,
    fallback_download_url: &'static str,
}

const PRESETS: &[StaticEmuPreset] = &[
    StaticEmuPreset {
        id: "retroarch",
        name: "RetroArch",
        repo: "libretro/RetroArch",
        version: "v1.19.1",
        executable_name: "retroarch.exe",
        system: "Multi-System Retro (8-bit / 16-bit / 32-bit)",
        short_system: "MULTI",
        author: "Libretro",
        description: "Modular multi-system retro game emulator and game engine frontend.",
        supported_extensions: &[".zip", ".7z", ".nes", ".snes", ".bin", ".cue", ".iso", ".gba", ".md", ".n64"],
        website: "https://www.retroarch.com",
        fallback_download_url: "https://buildbot.libretro.com/stable/1.19.1/windows/x86_64/RetroArch.7z",
    },
    StaticEmuPreset {
        id: "dolphin",
        name: "Dolphin",
        repo: "dolphin-emu/dolphin",
        version: "2407",
        executable_name: "Dolphin.exe",
        system: "Nintendo GameCube / Wii",
        short_system: "GAMECUBE/WII",
        author: "Dolphin Team",
        description: "Open-source Nintendo GameCube and Wii emulator with 4K enhancement support.",
        supported_extensions: &[".iso", ".gcm", ".rvz", ".wbfs", ".wad", ".elf", ".dol"],
        website: "https://dolphin-emu.org",
        fallback_download_url: "https://dl.dolphin-emu.org/builds/dolphin-master-2407-x64.7z",
    },
    StaticEmuPreset {
        id: "pcsx2",
        name: "PCSX2",
        repo: "PCSX2/pcsx2",
        version: "v2.0.2",
        executable_name: "pcsx2-qt.exe",
        system: "Sony PlayStation 2",
        short_system: "PS2",
        author: "PCSX2 Team",
        description: "High-performance open-source Sony PlayStation 2 emulator.",
        supported_extensions: &[".iso", ".bin", ".chd", ".cso", ".mdf", ".img", ".gz"],
        website: "https://pcsx2.net",
        fallback_download_url: "https://github.com/PCSX2/pcsx2/releases/download/v2.0.2/pcsx2-v2.0.2-windows-x64-Qt.7z",
    },
    StaticEmuPreset {
        id: "rpcs3",
        name: "RPCS3",
        repo: "RPCS3/rpcs3-binaries-win",
        version: "v0.0.32",
        executable_name: "rpcs3.exe",
        system: "Sony PlayStation 3",
        short_system: "PS3",
        author: "RPCS3 Team",
        description: "The world's first open-source Sony PlayStation 3 emulator and debugger.",
        supported_extensions: &[".pkg", ".iso", ".elf", ".bin", ".self"],
        website: "https://rpcs3.net",
        fallback_download_url: "https://github.com/RPCS3/rpcs3-binaries-win/releases/download/build-2024-11-01/rpcs3-v0.0.32.7z",
    },
    StaticEmuPreset {
        id: "ppsspp",
        name: "PPSSPP",
        repo: "hrydgard/ppsspp",
        version: "v1.17.1",
        executable_name: "PPSSPPWindows64.exe",
        system: "Sony PlayStation Portable (PSP)",
        short_system: "PSP",
        author: "Henrik Rydgård",
        description: "Fast, cross-platform Sony PSP emulator running in full HD resolution.",
        supported_extensions: &[".iso", ".cso", ".chd", ".pbp", ".elf"],
        website: "https://www.ppsspp.org",
        fallback_download_url: "https://www.ppsspp.org/files/1_17_1/ppsspp_win.zip",
    },
    StaticEmuPreset {
        id: "duckstation",
        name: "DuckStation",
        repo: "stenzek/duckstation",
        version: "latest",
        executable_name: "duckstation-qt-x64-Release-LTCG.exe",
        system: "Sony PlayStation 1",
        short_system: "PS1",
        author: "Stenzek",
        description: "Sony PlayStation 1 (PSX/PS1) emulator focusing on playability and speed.",
        supported_extensions: &[".cue", ".bin", ".iso", ".chd", ".pbp", ".m3u"],
        website: "https://github.com/stenzek/duckstation",
        fallback_download_url: "https://github.com/stenzek/duckstation/releases/download/latest/duckstation-windows-x64-release.zip",
    },
    StaticEmuPreset {
        id: "ryujinx",
        name: "Ryujinx",
        repo: "Ryujinx/release-channel-master",
        version: "1.1.1300",
        executable_name: "Ryujinx.exe",
        system: "Nintendo Switch",
        short_system: "SWITCH",
        author: "Ryubing / Community",
        description: "Experimental Nintendo Switch emulator written in C#.",
        supported_extensions: &[".nsp", ".xci", ".nca", ".nso"],
        website: "https://github.com/Ryubing/Ryujinx",
        fallback_download_url: "https://github.com/Ryubing/Ryujinx/releases/download/1.1.1300/ryujinx-1.1.1300-win_x64.zip",
    },
    StaticEmuPreset {
        id: "lime3ds",
        name: "Lime3DS",
        repo: "Lime3DS/Lime3DS",
        version: "2115",
        executable_name: "lime3ds-gui.exe",
        system: "Nintendo 3DS",
        short_system: "3DS",
        author: "Lime3DS Project",
        description: "Open-source Nintendo 3DS emulator fork continuing Citra legacy.",
        supported_extensions: &[".3ds", ".cia", ".cxi", ".app", ".cci"],
        website: "https://lime3ds.github.io",
        fallback_download_url: "https://github.com/Lime3DS/Lime3DS/releases/download/2115/lime3ds-2115-windows-msvc.zip",
    },
];

fn get_default_install_dir(app: &AppHandle, emu_id: &str) -> PathBuf {
    app.path()
        .app_data_dir()
        .unwrap_or_else(|_| dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("C:\\OmniLauncher")).join("OmniLauncher"))
        .join("emulators")
        .join(emu_id)
}

fn find_executable_recursive(dir: &Path, target_exe: &str) -> Option<PathBuf> {
    if !dir.exists() {
        return None;
    }
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.eq_ignore_ascii_case(target_exe) {
                        return Some(path);
                    }
                }
            } else if path.is_dir() {
                if let Some(found) = find_executable_recursive(&path, target_exe) {
                    return Some(found);
                }
            }
        }
    }
    None
}

#[tauri::command]
pub async fn get_emulator_catalog(app: AppHandle) -> Result<Vec<EmulatorCatalogEntry>, String> {
    let mut catalog = Vec::new();

    for preset in PRESETS {
        let install_dir = get_default_install_dir(&app, preset.id);
        let found_exe = find_executable_recursive(&install_dir, preset.executable_name);
        let installed = found_exe.is_some();

        catalog.push(EmulatorCatalogEntry {
            id: preset.id.to_string(),
            name: preset.name.to_string(),
            repo: preset.repo.to_string(),
            version: preset.version.to_string(),
            installed,
            executable_name: preset.executable_name.to_string(),
            executable_name_camel: preset.executable_name.to_string(),
            arch: "x86_64".to_string(),
            system: preset.system.to_string(),
            short_system: preset.short_system.to_string(),
            author: preset.author.to_string(),
            description: preset.description.to_string(),
            supported_extensions: preset.supported_extensions.iter().map(|&s| s.to_string()).collect(),
            install_path: if installed { Some(install_dir.to_string_lossy().to_string()) } else { None },
            executable_path: found_exe.map(|p| p.to_string_lossy().to_string()),
            is_rate_limited: false,
            website: Some(preset.website.to_string()),
        });
    }

    Ok(catalog)
}

#[derive(Debug, Deserialize)]
struct GithubRelease {
    assets: Vec<GithubAsset>,
}

#[derive(Debug, Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
}

async fn resolve_github_asset_url(client: &reqwest::Client, repo: &str) -> Result<String, String> {
    let api_url = format!("https://api.github.com/repos/{}/releases/latest", repo);
    let resp = client
        .get(&api_url)
        .header("Accept", "application/vnd.github.v3+json")
        .header("User-Agent", "OmniLauncher/0.1.0")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        return Err(format!("GitHub API status: {}", resp.status()));
    }

    let release = resp.json::<GithubRelease>().await.map_err(|e| e.to_string())?;
    let matched = release
        .assets
        .iter()
        .find(|a| {
            let lower = a.name.to_lowercase();
            lower.ends_with(".zip") && (lower.contains("win") || lower.contains("x64"))
        })
        .or_else(|| {
            release.assets.iter().find(|a| a.name.to_lowercase().ends_with(".zip"))
        });

    matched
        .map(|a| a.browser_download_url.clone())
        .ok_or_else(|| "No suitable release zip asset found".to_string())
}

pub fn safe_extract_archive(archive_path: &Path, dest_dir: &Path) -> Result<(), String> {
    let dest_existed_before = dest_dir.exists();
    if !dest_existed_before {
        std::fs::create_dir_all(dest_dir).map_err(|e| format!("Failed to create destination folder: {e}"))?;
    }

    let mut created_dirs: Vec<PathBuf> = Vec::new();
    let mut created_files: Vec<PathBuf> = Vec::new();

    let extract_result = (|| -> Result<(), String> {
        let file = File::open(archive_path).map_err(|e| format!("Cannot open zip: {e}"))?;
        let mut archive = ZipArchive::new(file).map_err(|e| format!("Invalid zip archive: {e}"))?;

        for i in 0..archive.len() {
            let mut entry = archive.by_index(i).map_err(|e| format!("Zip entry error: {e}"))?;
            let enclosed = entry.enclosed_name().ok_or_else(|| {
                "Zip-Slip vulnerability detected in archive path".to_string()
            })?;

            let outpath = dest_dir.join(enclosed);
            if entry.is_dir() {
                if !outpath.exists() {
                    std::fs::create_dir_all(&outpath).map_err(|e| format!("Dir creation error: {e}"))?;
                    created_dirs.push(outpath);
                }
            } else {
                if let Some(parent) = outpath.parent() {
                    if !parent.exists() {
                        std::fs::create_dir_all(parent).map_err(|e| format!("Parent dir creation error: {e}"))?;
                        created_dirs.push(parent.to_path_buf());
                    }
                }
                let file_existed = outpath.exists();
                let mut outfile = File::create(&outpath).map_err(|e| format!("File creation error: {e}"))?;
                std::io::copy(&mut entry, &mut outfile).map_err(|e| format!("Copy error: {e}"))?;
                if !file_existed {
                    created_files.push(outpath);
                }
            }
        }
        Ok(())
    })();

    if let Err(err) = extract_result {
        // Safe rollback: delete created files first
        for f in created_files.iter().rev() {
            let _ = std::fs::remove_file(f);
        }
        // Delete created dirs in reverse order (deepest first, fails safely if non-empty)
        for d in created_dirs.iter().rev() {
            let _ = std::fs::remove_dir(d);
        }
        // Only delete dest_dir if it was newly created and is empty
        if !dest_existed_before {
            let _ = std::fs::remove_dir(dest_dir);
        }
        return Err(err);
    }

    Ok(())
}

#[tauri::command]
pub async fn install_emulator(
    app: AppHandle,
    emulator_id: String,
    target_dir: Option<String>,
) -> Result<EmulatorInstallResult, String> {
    let preset = PRESETS
        .iter()
        .find(|p| p.id.eq_ignore_ascii_case(&emulator_id))
        .ok_or_else(|| format!("Unknown emulator ID: {}", emulator_id))?;

    let dest_dir = if let Some(ref custom_dir) = target_dir {
        PathBuf::from(custom_dir)
    } else {
        get_default_install_dir(&app, preset.id)
    };

    // Free disk space check: ensure drive has sufficient space (require at least 500MB)
    let estimated_required_bytes = 500 * 1024 * 1024; // 500MB
    #[cfg(windows)]
    {
        use windows_sys::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;
        let root = dest_dir.ancestors().find(|p| p.is_dir()).unwrap_or(Path::new("C:\\"));
        let mut root_str = root.to_string_lossy().to_string();
        if root_str.ends_with(':') {
            root_str.push('\\');
        }
        let wide: Vec<u16> = root_str.encode_utf16().chain(std::iter::once(0)).collect();
        let mut free_avail = 0u64;
        let mut total = 0u64;
        let mut total_free = 0u64;
        let ok = unsafe { GetDiskFreeSpaceExW(wide.as_ptr(), &mut free_avail, &mut total, &mut total_free) };
        if ok != 0 && free_avail < estimated_required_bytes {
            return Err("Insufficient disk space for archive extraction".to_string());
        }
    }

    tokio::fs::create_dir_all(&dest_dir)
        .await
        .map_err(|e| format!("Failed to create destination folder: {e}"))?;

    // Phase: downloading
    let _ = app.emit(
        "emulator://progress",
        EmulatorProgressEvent {
            emulator_id: emulator_id.clone(),
            phase: "downloading".to_string(),
            downloaded_bytes: 0,
            total_bytes: 0,
            percentage: 0.0,
            speed_mb_s: 0.0,
            error: None,
        },
    );

    // Prepare temp zip file in OS temp folder
    let tmp_zip = std::env::temp_dir().join(format!("omni_emu_{}_{}.zip", preset.id, std::process::id()));

    let client = reqwest::Client::builder()
        .user_agent("OmniLauncher/0.1.0")
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {e}"))?;

    // Resolve asset download URL: query GitHub API first, fallback to configured URL
    let download_url = match resolve_github_asset_url(&client, preset.repo).await {
        Ok(url) => url,
        Err(_) => preset.fallback_download_url.to_string(),
    };

    let resp = client
        .get(&download_url)
        .send()
        .await
        .map_err(|e| {
            let err_msg = format!("Download request failed: {e}");
            let _ = app.emit(
                "emulator://progress",
                EmulatorProgressEvent {
                    emulator_id: emulator_id.clone(),
                    phase: "error".to_string(),
                    downloaded_bytes: 0,
                    total_bytes: 0,
                    percentage: 0.0,
                    speed_mb_s: 0.0,
                    error: Some(err_msg.clone()),
                },
            );
            err_msg
        })?;

    if !resp.status().is_success() {
        let err_msg = format!("Download failed with HTTP status {}", resp.status());
        let _ = app.emit(
            "emulator://progress",
            EmulatorProgressEvent {
                emulator_id: emulator_id.clone(),
                phase: "error".to_string(),
                downloaded_bytes: 0,
                total_bytes: 0,
                percentage: 0.0,
                speed_mb_s: 0.0,
                error: Some(err_msg.clone()),
            },
        );
        return Err(err_msg);
    }

    let total_bytes = resp.content_length().unwrap_or(0);
    let mut downloaded_bytes = 0u64;
    let start_time = Instant::now();
    let mut last_emit = Instant::now();

    let mut out_file = tokio::fs::File::create(&tmp_zip)
        .await
        .map_err(|e| format!("Failed to create temporary archive: {e}"))?;

    let mut stream = resp.bytes_stream();
    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.map_err(|e| format!("Network stream error: {e}"))?;
        out_file.write_all(&chunk).await.map_err(|e| format!("Disk write error: {e}"))?;
        downloaded_bytes += chunk.len() as u64;

        if last_emit.elapsed() >= std::time::Duration::from_millis(200)
            || (total_bytes > 0 && downloaded_bytes >= total_bytes)
        {
            let elapsed_s = start_time.elapsed().as_secs_f64().max(0.001);
            let speed_mb_s = (downloaded_bytes as f64) / (1024.0 * 1024.0 * elapsed_s);
            let percentage = if total_bytes > 0 {
                ((downloaded_bytes as f64) / (total_bytes as f64) * 100.0).clamp(0.0, 100.0)
            } else {
                50.0
            };

            let _ = app.emit(
                "emulator://progress",
                EmulatorProgressEvent {
                    emulator_id: emulator_id.clone(),
                    phase: "downloading".to_string(),
                    downloaded_bytes,
                    total_bytes,
                    percentage,
                    speed_mb_s,
                    error: None,
                },
            );
            last_emit = Instant::now();
        }
    }
    out_file.flush().await.map_err(|e| format!("Flush error: {e}"))?;
    drop(out_file);

    // Verify archive size (>0 bytes)
    let meta = std::fs::metadata(&tmp_zip).map_err(|e| format!("Metadata error: {e}"))?;
    if meta.len() == 0 {
        let _ = std::fs::remove_file(&tmp_zip);
        return Err("Archive is empty (0 bytes)".to_string());
    }

    // Phase: extracting
    let _ = app.emit(
        "emulator://progress",
        EmulatorProgressEvent {
            emulator_id: emulator_id.clone(),
            phase: "extracting".to_string(),
            downloaded_bytes: meta.len(),
            total_bytes: meta.len(),
            percentage: 50.0,
            speed_mb_s: 15.0,
            error: None,
        },
    );

    let extract_result = safe_extract_archive(&tmp_zip, &dest_dir);
    let _ = std::fs::remove_file(&tmp_zip);

    if let Err(err) = extract_result {
        let _ = app.emit(
            "emulator://progress",
            EmulatorProgressEvent {
                emulator_id: emulator_id.clone(),
                phase: "error".to_string(),
                downloaded_bytes: 0,
                total_bytes: 0,
                percentage: 0.0,
                speed_mb_s: 0.0,
                error: Some(err.clone()),
            },
        );
        return Err(err);
    }

    // Discover installed executable
    let found_exe = find_executable_recursive(&dest_dir, preset.executable_name)
        .ok_or_else(|| format!("Executable '{}' not found after extraction", preset.executable_name))?;

    let found_exe_str = found_exe.to_string_lossy().to_string();

    // Phase: registering in Library
    let _ = app.emit(
        "emulator://progress",
        EmulatorProgressEvent {
            emulator_id: emulator_id.clone(),
            phase: "registering".to_string(),
            downloaded_bytes: meta.len(),
            total_bytes: meta.len(),
            percentage: 90.0,
            speed_mb_s: 0.0,
            error: None,
        },
    );

    let registered_game_id = format!("emu_{}", preset.id);

    // Emit library update so the UI reacts instantly
    let _ = app.emit(EVT_LIBRARY_UPDATED, ());

    // Phase: completed
    let _ = app.emit(
        "emulator://progress",
        EmulatorProgressEvent {
            emulator_id: emulator_id.clone(),
            phase: "completed".to_string(),
            downloaded_bytes: meta.len(),
            total_bytes: meta.len(),
            percentage: 100.0,
            speed_mb_s: 0.0,
            error: None,
        },
    );

    Ok(EmulatorInstallResult {
        success: true,
        installed: true,
        emulator_id: preset.id.to_string(),
        installed_location: found_exe_str.clone(),
        installed_location_camel: found_exe_str,
        used_builtin_picker: true,
        used_builtin_picker_camel: true,
        dialog_type: "in_launcher_svelte_modal".to_string(),
        dialog_type_camel: "in_launcher_svelte_modal".to_string(),
        registered_game_id,
    })
}

#[tauri::command]
pub async fn launch_emulator(app: AppHandle, emulator_id: String) -> Result<(), String> {
    let preset = PRESETS
        .iter()
        .find(|p| p.id.eq_ignore_ascii_case(&emulator_id))
        .ok_or_else(|| format!("Unknown emulator: {}", emulator_id))?;

    let install_dir = get_default_install_dir(&app, preset.id);
    let exe_path = find_executable_recursive(&install_dir, preset.executable_name)
        .ok_or_else(|| format!("Emulator '{}' is not installed or executable missing", preset.name))?;

    open::that_detached(&exe_path)
        .map_err(|e| format!("Failed to launch emulator process: {e}"))?;

    Ok(())
}
