//! In-Launcher Drive/Directory Navigation & Network/Cloudflare Settings.

use std::path::{Path, PathBuf};
use std::time::Instant;

use regex::Regex;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::core::config::{ConfigStore, NetworkSettings};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriveInfo {
    pub drive: String,
    pub mount_point: String,
    pub label: String,
    pub drive_type: String,
    pub total_bytes: u64,
    pub free_bytes: u64,
    pub used_bytes: u64,
    pub total_gb: f64,
    pub free_gb: f64,
    pub used_percentage: f64,
    pub is_system: bool,
    pub is_read_only: bool,
    pub is_low_space: bool,
    pub file_system: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectoryListing {
    pub current_path: String,
    pub parent_path: Option<String>,
    pub can_write: bool,
    pub entries: Vec<DirectoryEntryItem>,
    pub total_folders: usize,
    pub free_space_bytes: u64,
    pub free_space_gb: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectoryEntryItem {
    pub name: String,
    pub path: String,
    pub is_directory: bool,
    pub is_hidden: bool,
    pub is_system: bool,
    pub is_accessible: bool,
    pub modified_time: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionTestResult {
    pub success: bool,
    pub latency_ms: u64,
    pub resolved_ip: Option<String>,
    pub doh_active: bool,
    pub proxy_active: bool,
    pub error: Option<String>,
}

#[tauri::command]
pub async fn get_system_drives() -> Result<Vec<DriveInfo>, String> {
    #[cfg(windows)]
    {
        use std::ffi::OsString;
        use std::os::windows::ffi::OsStringExt;
        use windows_sys::Win32::Storage::FileSystem::{
            GetDiskFreeSpaceExW, GetDriveTypeW, GetLogicalDrives, GetVolumeInformationW,
        };

        const DRIVE_REMOVABLE: u32 = 2;
        const DRIVE_FIXED: u32 = 3;
        const DRIVE_REMOTE: u32 = 4;
        const DRIVE_CDROM: u32 = 5;
        const DRIVE_RAMDISK: u32 = 6;

        let mask = unsafe { GetLogicalDrives() };
        let mut drives = Vec::new();
        let sys_drive = std::env::var("SystemDrive").unwrap_or_else(|_| "C:".into());

        for i in 0..26 {
            if (mask & (1 << i)) != 0 {
                let letter = (b'A' + i) as char;
                let root_str = format!("{}:\\", letter);
                let wide_root: Vec<u16> = root_str.encode_utf16().chain(std::iter::once(0)).collect();

                let drive_type_raw = unsafe { GetDriveTypeW(wide_root.as_ptr()) };
                let (drive_type, is_read_only) = match drive_type_raw {
                    DRIVE_FIXED => ("Fixed", false),
                    DRIVE_REMOVABLE => ("Removable", false),
                    DRIVE_REMOTE => ("Network", false),
                    DRIVE_CDROM => ("CDROM", true),
                    DRIVE_RAMDISK => ("RamDisk", false),
                    _ => ("Unknown", false),
                };

                let mut free_avail: u64 = 0;
                let mut total: u64 = 0;
                let mut total_free: u64 = 0;
                let space_ok = unsafe {
                    GetDiskFreeSpaceExW(
                        wide_root.as_ptr(),
                        &mut free_avail,
                        &mut total,
                        &mut total_free,
                    )
                };

                let (free_bytes, total_bytes) = if space_ok != 0 {
                    (free_avail, total)
                } else {
                    (0, 0)
                };

                // Volume label
                let mut vol_name = [0u16; 261];
                let mut fs_name = [0u16; 261];
                let vol_ok = unsafe {
                    GetVolumeInformationW(
                        wide_root.as_ptr(),
                        vol_name.as_mut_ptr(),
                        vol_name.len() as u32,
                        std::ptr::null_mut(),
                        std::ptr::null_mut(),
                        std::ptr::null_mut(),
                        fs_name.as_mut_ptr(),
                        fs_name.len() as u32,
                    )
                };

                let label = if vol_ok != 0 {
                    let len = vol_name.iter().position(|&c| c == 0).unwrap_or(0);
                    let s = OsString::from_wide(&vol_name[..len]).to_string_lossy().to_string();
                    if s.trim().is_empty() {
                        format!("Local Disk ({}:)", letter)
                    } else {
                        format!("{} ({}:)", s, letter)
                    }
                } else {
                    format!("Local Disk ({}:)", letter)
                };

                let fs = if vol_ok != 0 {
                    let len = fs_name.iter().position(|&c| c == 0).unwrap_or(0);
                    Some(OsString::from_wide(&fs_name[..len]).to_string_lossy().to_string())
                } else {
                    None
                };

                let used_bytes = total_bytes.saturating_sub(free_bytes);
                let total_gb = (total_bytes as f64) / (1024.0 * 1024.0 * 1024.0);
                let free_gb = (free_bytes as f64) / (1024.0 * 1024.0 * 1024.0);
                let used_percentage = if total_bytes > 0 {
                    ((used_bytes as f64) / (total_bytes as f64)) * 100.0
                } else {
                    0.0
                };

                drives.push(DriveInfo {
                    drive: format!("{}:", letter),
                    mount_point: root_str,
                    label,
                    drive_type: drive_type.into(),
                    total_bytes,
                    free_bytes,
                    used_bytes,
                    total_gb,
                    free_gb,
                    used_percentage,
                    is_system: format!("{}:", letter).eq_ignore_ascii_case(&sys_drive),
                    is_read_only,
                    is_low_space: free_bytes < (10 * 1024 * 1024 * 1024),
                    file_system: fs,
                });
            }
        }
        Ok(drives)
    }

    #[cfg(not(windows))]
    {
        Ok(vec![DriveInfo {
            drive: "C:".into(),
            mount_point: "C:\\".into(),
            label: "Local Disk (C:)".into(),
            drive_type: "Fixed".into(),
            total_bytes: 500_000_000_000,
            free_bytes: 150_000_000_000,
            used_bytes: 350_000_000_000,
            total_gb: 465.6,
            free_gb: 139.7,
            used_percentage: 70.0,
            is_system: true,
            is_read_only: false,
            is_low_space: false,
            file_system: Some("NTFS".into()),
        }])
    }
}

#[tauri::command]
pub async fn list_directory_contents(path: String) -> Result<DirectoryListing, String> {
    let p = if path.trim().is_empty() {
        dirs::home_dir().unwrap_or_else(|| PathBuf::from("C:\\"))
    } else {
        PathBuf::from(path.trim())
    };

    if !p.exists() {
        return Err(format!("Directory does not exist: {}", p.display()));
    }

    let parent_path = p.parent().map(|par| par.to_string_lossy().to_string());
    let mut entries = Vec::new();
    let mut total_folders = 0;

    let read_res = std::fs::read_dir(&p).map_err(|e| format!("Failed to read directory: {e}"))?;

    for entry in read_res.flatten() {
        let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
        if !is_dir {
            continue; // Directory picker only displays directories
        }

        let name = entry.file_name().to_string_lossy().to_string();
        let item_path = entry.path().to_string_lossy().to_string();
        let metadata = entry.metadata().ok();
        let modified_time = metadata
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs());

        let is_hidden = name.starts_with('$') || name.starts_with('.');
        let is_system = name.eq_ignore_ascii_case("System Volume Information")
            || name.eq_ignore_ascii_case("Recovery")
            || name.eq_ignore_ascii_case("$Recycle.Bin");

        total_folders += 1;
        entries.push(DirectoryEntryItem {
            name,
            path: item_path,
            is_directory: true,
            is_hidden,
            is_system,
            is_accessible: true,
            modified_time,
        });
    }

    entries.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    // Test write permissions
    let test_file = p.join(".omni_write_test");
    let can_write = match std::fs::write(&test_file, b"test") {
        Ok(_) => {
            let _ = std::fs::remove_file(test_file);
            true
        }
        Err(_) => false,
    };

    let (free_space_bytes, free_space_gb) = {
        #[cfg(windows)]
        {
            use windows_sys::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;
            let root = p.ancestors().find(|a| a.is_dir()).unwrap_or(Path::new("C:\\"));
            let mut root_str = root.to_string_lossy().to_string();
            if root_str.ends_with(':') {
                root_str.push('\\');
            }
            let wide: Vec<u16> = root_str.encode_utf16().chain(std::iter::once(0)).collect();
            let mut free_avail = 0u64;
            let mut total = 0u64;
            let mut total_free = 0u64;
            let ok = unsafe { GetDiskFreeSpaceExW(wide.as_ptr(), &mut free_avail, &mut total, &mut total_free) };
            if ok != 0 {
                (free_avail, (free_avail as f64) / (1024.0 * 1024.0 * 1024.0))
            } else {
                (0, 0.0)
            }
        }
        #[cfg(not(windows))]
        {
            (150_000_000_000, 139.7)
        }
    };

    Ok(DirectoryListing {
        current_path: p.to_string_lossy().to_string(),
        parent_path,
        can_write,
        entries,
        total_folders,
        free_space_bytes,
        free_space_gb,
    })
}

#[tauri::command]
pub async fn create_directory(path: String) -> Result<(), String> {
    let p = Path::new(path.trim());
    if let Some(file_name) = p.file_name().and_then(|f| f.to_str()) {
        let name_trimmed = file_name.trim().trim_end_matches('.');
        let stem = name_trimmed.split('.').next().unwrap_or(name_trimmed).trim();
        let stem_upper = stem.to_uppercase();

        let reserved = [
            "CON", "PRN", "AUX", "NUL", "COM0", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6",
            "COM7", "COM8", "COM9", "LPT0", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6",
            "LPT7", "LPT8", "LPT9",
        ];
        if reserved.contains(&stem_upper.as_str()) {
            return Err(format!("'{}' is a reserved Windows device name", file_name));
        }

        if file_name.contains(&['<', '>', ':', '"', '/', '\\', '|', '?', '*'][..]) {
            return Err("Folder name contains invalid characters (< > : \" / \\ | ? *)".into());
        }
    }

    tokio::fs::create_dir_all(p)
        .await
        .map_err(|e| format!("Failed to create folder: {e}"))?;

    Ok(())
}

#[tauri::command]
pub async fn get_network_settings(store: State<'_, ConfigStore>) -> Result<NetworkSettings, String> {
    let cfg = store.snapshot().await;
    Ok(cfg.network)
}

#[tauri::command]
pub async fn update_network_settings(
    settings: NetworkSettings,
    store: State<'_, ConfigStore>,
) -> Result<(), String> {
    // Validate proxy URI if enabled
    if settings.proxy_enabled {
        if let Some(ref uri) = settings.proxy_url {
            let re = Regex::new(r"^(socks5|http|https)://(?:([^:]+):([^@]+)@)?(\[[0-9a-fA-F:]+\]|[^:]+):(\d+)$")
                .map_err(|e| e.to_string())?;
            let caps = re
                .captures(uri)
                .ok_or_else(|| "Invalid proxy URI format. Example: socks5://127.0.0.1:1080 or socks5://[::1]:1080".to_string())?;

            let host = &caps[4];
            if host.starts_with('[') && host.ends_with(']') {
                let inner = &host[1..host.len() - 1];
                if inner.parse::<std::net::Ipv6Addr>().is_err() {
                    return Err(format!("Invalid IPv6 proxy address: '{inner}'"));
                }
            } else if host.trim().is_empty() {
                return Err("Proxy hostname cannot be empty".to_string());
            }

            let port: u32 = caps[5]
                .parse()
                .map_err(|_| "Invalid proxy port number".to_string())?;

            if !(1..=65535).contains(&port) {
                return Err("Proxy port must be between 1 and 65535".to_string());
            }
        }
    }

    // Validate DoH endpoint
    if settings.dns_provider == "custom" {
        if let Some(ref doh) = settings.custom_doh_url {
            if !doh.starts_with("https://") || doh.len() <= 8 {
                return Err("Custom DoH endpoint must be a valid https:// URL".to_string());
            }
        }
    }

    store
        .update(|cfg| {
            cfg.network = settings;
        })
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[derive(Debug, Deserialize)]
struct DohRecord {
    #[allow(dead_code)]
    name: Option<String>,
    #[serde(rename = "type")]
    record_type: Option<u32>,
    #[serde(rename = "TTL")]
    #[allow(dead_code)]
    ttl: Option<u32>,
    data: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DohResponse {
    #[serde(rename = "Status")]
    status: Option<i32>,
    #[serde(rename = "Answer")]
    answer: Option<Vec<DohRecord>>,
}

#[tauri::command]
pub async fn test_network_connection(
    provider: Option<String>,
    proxy_url: Option<String>,
) -> Result<ConnectionTestResult, String> {
    let start = Instant::now();
    let dns = provider.as_deref().unwrap_or("cloudflare");

    // System DNS branch
    if dns == "system" {
        return match tokio::net::lookup_host("cloudflare.com:443").await {
            Ok(mut addrs) => {
                let latency_ms = start.elapsed().as_millis() as u64;
                let resolved = addrs.next().map(|a| a.ip().to_string());
                Ok(ConnectionTestResult {
                    success: resolved.is_some(),
                    latency_ms,
                    resolved_ip: resolved,
                    doh_active: false,
                    proxy_active: proxy_url.is_some(),
                    error: None,
                })
            }
            Err(err) => {
                let latency_ms = start.elapsed().as_millis() as u64;
                Ok(ConnectionTestResult {
                    success: false,
                    latency_ms,
                    resolved_ip: None,
                    doh_active: false,
                    proxy_active: proxy_url.is_some(),
                    error: Some(format!("System DNS resolution failed: {err}")),
                })
            }
        };
    }

    let test_endpoint = match dns {
        "google" => "https://dns.google/resolve?name=cloudflare.com&type=A".to_string(),
        "quad9" => "https://dns.quad9.net/dns-query?name=cloudflare.com".to_string(),
        custom if custom.starts_with("https://") => {
            if custom.contains('?') {
                format!("{custom}&name=cloudflare.com&type=A")
            } else {
                format!("{custom}?name=cloudflare.com&type=A")
            }
        }
        _ => "https://cloudflare-dns.com/dns-query?name=cloudflare.com&type=A".to_string(),
    };

    let mut builder = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .user_agent("OmniLauncher/0.1.0");

    if let Some(ref proxy) = proxy_url {
        if !proxy.trim().is_empty() {
            if let Ok(p) = reqwest::Proxy::all(proxy) {
                builder = builder.proxy(p);
            }
        }
    }

    let client = builder.build().map_err(|e| e.to_string())?;

    match client.get(&test_endpoint).header("accept", "application/dns-json").send().await {
        Ok(resp) => {
            let latency_ms = start.elapsed().as_millis() as u64;
            if !resp.status().is_success() {
                return Ok(ConnectionTestResult {
                    success: false,
                    latency_ms,
                    resolved_ip: None,
                    doh_active: false,
                    proxy_active: proxy_url.is_some(),
                    error: Some(format!("HTTP {}", resp.status())),
                });
            }

            match resp.json::<DohResponse>().await {
                Ok(body) => {
                    let dns_success = body.status.unwrap_or(-1) == 0;
                    let resolved_ip = body.answer.as_ref().and_then(|ans| {
                        ans.iter()
                            .find(|r| r.record_type == Some(1)) // Type 1: A record
                            .and_then(|r| r.data.clone())
                            .or_else(|| {
                                ans.iter()
                                    .find(|r| r.record_type == Some(28)) // Type 28: AAAA record
                                    .and_then(|r| r.data.clone())
                            })
                            .or_else(|| ans.first().and_then(|r| r.data.clone()))
                    });

                    let success = dns_success && resolved_ip.is_some();
                    let error = if !dns_success {
                        Some(format!("DNS query returned RCODE {}", body.status.unwrap_or(-1)))
                    } else if resolved_ip.is_none() {
                        Some("No Answer records returned by DoH resolver".to_string())
                    } else {
                        None
                    };

                    Ok(ConnectionTestResult {
                        success,
                        latency_ms,
                        resolved_ip,
                        doh_active: success,
                        proxy_active: proxy_url.is_some(),
                        error,
                    })
                }
                Err(err) => Ok(ConnectionTestResult {
                    success: false,
                    latency_ms,
                    resolved_ip: None,
                    doh_active: false,
                    proxy_active: proxy_url.is_some(),
                    error: Some(format!("Failed to parse DoH JSON: {err}")),
                }),
            }
        }
        Err(err) => {
            let latency_ms = start.elapsed().as_millis() as u64;
            Ok(ConnectionTestResult {
                success: false,
                latency_ms,
                resolved_ip: None,
                doh_active: false,
                proxy_active: proxy_url.is_some(),
                error: Some(err.to_string()),
            })
        }
    }
}
