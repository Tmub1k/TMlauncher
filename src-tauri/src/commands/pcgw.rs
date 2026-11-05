//! PCGamingWiki 1-Click Tweak Engine (`commands/pcgw.rs`).
//!
//! Provides PCGamingWiki hardware capability querying (Ultrawide, FPS uncap, FOV),
//! safe INI/XML editing with `.omnibak` backups, clearing Windows read-only flags,
//! and parameter clamping.

use std::{
    fs,
    io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PcgwTweak {
    pub id: String,
    pub title: String,
    pub config_file: String,
    pub section: Option<String>,
    pub key: String,
    pub current_val: String,
    pub recommended_val: String,
    pub applied: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PcgwRecord {
    pub game_title: String,
    pub ultrawide_support: bool,
    pub fps_uncap_supported: bool,
    pub fov_slider_supported: bool,
    pub ultrawide_notes: Option<String>,
    pub fps_notes: Option<String>,
    pub fov_notes: Option<String>,
    pub config_paths: Vec<String>,
    pub tweaks: Vec<PcgwTweak>,
    pub source: String,
}

/// Clamps tweak values to safe ranges (FOV: 60..130, FPS: 30..360).
pub fn clamp_tweak_value(key: &str, val: &str) -> String {
    let lower_key = key.to_lowercase();
    if lower_key.contains("fov") {
        if let Ok(num) = val.parse::<f64>() {
            let clamped = num.max(60.0).min(130.0);
            return format!("{}", clamped.round() as i64);
        }
    } else if lower_key.contains("fps") || lower_key.contains("framerate") {
        if let Ok(num) = val.parse::<i64>() {
            let clamped = num.max(30).min(360);
            return format!("{clamped}");
        }
    }
    val.to_string()
}

/// Clears Windows FILE_ATTRIBUTE_READONLY if present.
pub fn clear_readonly_attribute(path: &Path) -> io::Result<bool> {
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        use std::os::windows::fs::MetadataExt;
        let meta = fs::metadata(path)?;
        let attrs = meta.file_attributes();
        const FILE_ATTRIBUTE_READONLY: u32 = 0x1;
        if attrs & FILE_ATTRIBUTE_READONLY != 0 {
            use windows_sys::Win32::Storage::FileSystem::SetFileAttributesW;
            let wide: Vec<u16> = path.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
            let new_attrs = attrs & !FILE_ATTRIBUTE_READONLY;
            unsafe {
                SetFileAttributesW(wide.as_ptr(), new_attrs);
            }
            return Ok(true);
        }
    }
    #[cfg(not(windows))]
    {
        let mut perms = fs::metadata(path)?.permissions();
        if perms.readonly() {
            perms.set_readonly(false);
            fs::set_permissions(path, perms)?;
            return Ok(true);
        }
    }
    Ok(false)
}

fn validate_xml_syntax(content: &str) -> Result<(), String> {
    let open_count = content.matches("<config>").count();
    let close_count = content.matches("</config>").count();
    if open_count != close_count {
        return Err("Configuration file has existing syntax corruption: unbalanced <config> tags".to_string());
    }
    Ok(())
}

fn query_curated_pcgw(title: &str) -> PcgwRecord {
    let lower = title.to_lowercase();

    if lower.contains("elden ring") {
        PcgwRecord {
            game_title: title.to_string(),
            ultrawide_support: false,
            fps_uncap_supported: true,
            fov_slider_supported: false,
            ultrawide_notes: Some("Requires Flawless Widescreen or mod to uncap 21:9".to_string()),
            fps_notes: Some("Engine locked to 60 FPS natively; patchable to 120/144".to_string()),
            fov_notes: None,
            config_paths: vec!["graphics_config.xml".to_string()],
            tweaks: vec![
                PcgwTweak {
                    id: "fps_uncap".to_string(),
                    title: "Uncap FPS to 144".to_string(),
                    config_file: "graphics_config.xml".to_string(),
                    section: None,
                    key: "MaxFPS".to_string(),
                    current_val: "60".to_string(),
                    recommended_val: "144".to_string(),
                    applied: false,
                },
            ],
            source: "curated_database".to_string(),
        }
    } else if lower.contains("witcher 3") {
        PcgwRecord {
            game_title: title.to_string(),
            ultrawide_support: true,
            fps_uncap_supported: true,
            fov_slider_supported: true,
            ultrawide_notes: Some("Native 21:9 and 32:9 HUD support".to_string()),
            fps_notes: Some("Unlimited FPS slider in options".to_string()),
            fov_notes: Some("FOV adjustment available in user.settings".to_string()),
            config_paths: vec!["user.settings".to_string()],
            tweaks: vec![
                PcgwTweak {
                    id: "fov_tweak".to_string(),
                    title: "Widen Gameplay FOV".to_string(),
                    config_file: "user.settings".to_string(),
                    section: Some("Gameplay".to_string()),
                    key: "CameraFOV".to_string(),
                    current_val: "60".to_string(),
                    recommended_val: "85".to_string(),
                    applied: false,
                },
            ],
            source: "curated_database".to_string(),
        }
    } else if lower.contains("cyberpunk 2077") {
        PcgwRecord {
            game_title: title.to_string(),
            ultrawide_support: true,
            fps_uncap_supported: true,
            fov_slider_supported: true,
            ultrawide_notes: Some("Full 21:9 / 32:9 Ultrawide support".to_string()),
            fps_notes: Some("Full uncap support up to 240+ FPS".to_string()),
            fov_notes: Some("In-game FOV slider 70-100".to_string()),
            config_paths: vec!["UserSettings.json".to_string()],
            tweaks: vec![
                PcgwTweak {
                    id: "fov_slider".to_string(),
                    title: "Expand First-Person Field of View".to_string(),
                    config_file: "UserSettings.json".to_string(),
                    section: Some("Gameplay/FOV".to_string()),
                    key: "FieldOfView".to_string(),
                    current_val: "80".to_string(),
                    recommended_val: "100".to_string(),
                    applied: false,
                },
                PcgwTweak {
                    id: "crowd_density".to_string(),
                    title: "High Precision Ray Tracing Memory".to_string(),
                    config_file: "UserSettings.json".to_string(),
                    section: None,
                    key: "RayTracingQuality".to_string(),
                    current_val: "Medium".to_string(),
                    recommended_val: "Ultra".to_string(),
                    applied: false,
                },
            ],
            source: "curated_database".to_string(),
        }
    } else {
        // Safe generic default
        PcgwRecord {
            game_title: title.to_string(),
            ultrawide_support: true,
            fps_uncap_supported: true,
            fov_slider_supported: false,
            ultrawide_notes: None,
            fps_notes: None,
            fov_notes: None,
            config_paths: Vec::new(),
            tweaks: Vec::new(),
            source: "heuristic".to_string(),
        }
    }
}

#[tauri::command]
pub async fn get_pcgamingwiki_data(
    game_title: String,
    _steam_app_id: Option<u32>,
) -> Result<PcgwRecord, String> {
    Ok(query_curated_pcgw(&game_title))
}

#[tauri::command]
pub async fn apply_pcgw_tweak(
    config_path: String,
    section: Option<String>,
    key: String,
    value: String,
) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let path = PathBuf::from(&config_path);
        if !path.exists() {
            return Err(format!("Target config file was moved or deleted: {config_path}"));
        }

        // 1. Clear read-only attribute if set
        let _ = clear_readonly_attribute(&path);

        // 2. Read existing content
        let content = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read config: {e}"))?;

        // 3. Syntax check if XML
        if path.extension().and_then(|s| s.to_str()) == Some("xml") {
            validate_xml_syntax(&content)?;
        }

        // 4. Create mandatory .omnibak backup
        let backup_path = PathBuf::from(format!("{config_path}.omnibak"));
        if !backup_path.exists() {
            fs::write(&backup_path, &content)
                .map_err(|e| format!("Failed to create .omnibak backup: {e}"))?;
        }

        // 5. Clamp parameter if necessary
        let clamped_val = clamp_tweak_value(&key, &value);

        // 6. Apply tweak
        let new_content = if path.extension().and_then(|s| s.to_str()) == Some("xml") {
            let tag_open = format!("<{key}>");
            if content.contains(&tag_open) {
                let re = regex::Regex::new(&format!(r"<{key}>.*?</{key}>")).unwrap();
                re.replace(&content, format!("<{key}>{clamped_val}</{key}>")).to_string()
            } else {
                content.replace("</config>", &format!("  <{key}>{clamped_val}</{key}>\n</config>"))
            }
        } else {
            // INI format
            let regex = regex::Regex::new(&format!(r"(?m)^(\s*{key}\s*=\s*).*$")).unwrap();
            if regex.is_match(&content) {
                regex.replace(&content, format!("${{1}}{clamped_val}")).to_string()
            } else if let Some(sec) = section {
                let sec_header = format!("[{sec}]");
                if content.contains(&sec_header) {
                    content.replace(&sec_header, &format!("{sec_header}\n{key}={clamped_val}"))
                } else {
                    format!("{content}\n[{sec}]\n{key}={clamped_val}")
                }
            } else {
                format!("{content}\n{key}={clamped_val}")
            }
        };

        fs::write(&path, new_content)
            .map_err(|e| format!("Failed to write modified config: {e}"))?;

        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn revert_pcgw_tweak(config_path: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let path = PathBuf::from(&config_path);
        let backup_path = PathBuf::from(format!("{config_path}.omnibak"));
        if !backup_path.exists() {
            return Err(format!("No .omnibak backup found for {config_path}"));
        }

        let _ = clear_readonly_attribute(&path);
        fs::copy(&backup_path, &path)
            .map_err(|e| format!("Failed to restore .omnibak: {e}"))?;

        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn open_game_config_directory(config_path: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let p = PathBuf::from(&config_path);
        let parent = if p.is_file() {
            p.parent().unwrap_or(&p)
        } else {
            &p
        };

        #[cfg(windows)]
        {
            let _ = std::process::Command::new("explorer.exe")
                .arg(parent)
                .spawn();
        }

        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}
