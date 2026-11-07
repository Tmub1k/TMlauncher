//! Native Windows 11 Mica / Tabbed (Mica Alt) / Acrylic window vibrancy control
//! and in-app / client store window management.

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, WebviewWindow};

use crate::error::{LauncherError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WindowEffectKind {
    Mica,
    Tabbed,
    Acrylic,
    None,
}

pub fn apply_effect(window: &WebviewWindow, effect: WindowEffectKind) -> Result<()> {
    #[cfg(windows)]
    {
        use window_vibrancy::{
            apply_acrylic, apply_mica, apply_tabbed, clear_acrylic, clear_mica, clear_tabbed,
        };

        match effect {
            WindowEffectKind::Mica => {
                apply_mica(window, Some(true))
                    .map_err(|e| LauncherError::WinApi(format!("apply_mica failed: {e}")))?;
            }
            WindowEffectKind::Tabbed => {
                apply_tabbed(window, Some(true))
                    .map_err(|e| LauncherError::WinApi(format!("apply_tabbed failed: {e}")))?;
            }
            WindowEffectKind::Acrylic => {
                apply_acrylic(window, Some((9, 9, 11, 210)))
                    .map_err(|e| LauncherError::WinApi(format!("apply_acrylic failed: {e}")))?;
            }
            WindowEffectKind::None => {
                let _ = clear_mica(window);
                let _ = clear_tabbed(window);
                let _ = clear_acrylic(window);
            }
        }
    }

    #[cfg(not(windows))]
    {
        let _ = (window, effect);
    }

    Ok(())
}

#[tauri::command]
pub async fn set_window_effect(window: WebviewWindow, effect: WindowEffectKind) -> Result<()> {
    apply_effect(&window, effect)
}

#[tauri::command]
pub async fn get_store_page_html(url: String) -> Result<String> {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36")
        .build()?;

    let text = client
        .get(&url)
        .header("Accept-Language", "ru,en;q=0.9")
        .send()
        .await?
        .text()
        .await?;

    let injection = r#"
        <meta name="referrer" content="no-referrer">
        <link rel="stylesheet" href="https://store.akamai.steamstatic.com/public/shared/css/motiva_sans.css">
        <link rel="stylesheet" href="https://store.akamai.steamstatic.com/public/shared/css/shared_global.css">
        <link rel="stylesheet" href="https://store.akamai.steamstatic.com/public/css/v6/game.css">
        <style>
            body { background-color: #171a21 !important; color: #c6d4df !important; font-family: 'Motiva Sans', sans-serif !important; }
            a { color: #66c0f4 !important; }
            .global_header, #global_header, .responsive_header { background: #171a21 !important; }
        </style>
    "#;

    let base_tag = format!("<base href=\"{url}\" target=\"_blank\">\n{injection}");
    let html = if let Some(idx) = text.find("<head>") {
        let (head, rest) = text.split_at(idx + 6);
        format!("{head}\n{base_tag}\n{rest}")
    } else {
        format!("{base_tag}\n{text}")
    };
    Ok(html)
}

#[tauri::command]
pub async fn open_in_store_webview(
    app: AppHandle,
    url: String,
    title: String,
) -> Result<()> {
    let parsed: tauri::Url = url.parse().map_err(|e| LauncherError::InvalidInput(format!("{e}")))?;

    if let Some(w) = app.get_webview_window("store_window") {
        let _ = w.navigate(parsed);
        let _ = w.eval(&format!("window.location.href = {:?};", url));
        let _ = w.set_title(&title);
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
        return Ok(());
    }

    let mut builder = tauri::WebviewWindowBuilder::new(
        &app,
        "store_window",
        tauri::WebviewUrl::External(parsed),
    )
    .title(&title)
    .inner_size(1280.0, 840.0)
    .center()
    .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36");

    if let Ok(data_dir) = app.path().app_data_dir() {
        let profile_dir = data_dir.join("webview_profile");
        let _ = std::fs::create_dir_all(&profile_dir);
        builder = builder.data_directory(profile_dir);
    }

    let win = builder
        .build()
        .map_err(|e| LauncherError::Launch(format!("failed to open store window: {e}")))?;

    let _ = win.show();
    let _ = win.set_focus();
    Ok(())
}

#[tauri::command]
pub async fn open_in_app_store_window(
    app: AppHandle,
    url: String,
    title: String,
) -> Result<()> {
    open_in_store_webview(app, url, title).await
}

#[tauri::command]
pub async fn open_store_link(
    app: AppHandle,
    url: String,
    mode: Option<String>,
    app_id: Option<String>,
    platform: Option<String>,
) -> std::result::Result<(), String> {
    let target_mode = mode.as_deref().unwrap_or("client").to_lowercase();
    let target_platform = platform.as_deref().unwrap_or_default().to_lowercase();

    match target_mode.as_str() {
        "client" | "native" => {
            let is_steam = target_platform == "steam"
                || app_id.is_some()
                || url.contains("store.steampowered.com")
                || url.starts_with("steam://");

            let is_epic = target_platform == "epic"
                || url.contains("store.epicgames.com")
                || url.starts_with("com.epicgames.launcher://");

            if is_steam {
                let steam_id = app_id.or_else(|| {
                    let re = regex::Regex::new(r"store\.steampowered\.com/app/(\d+)").ok()?;
                    re.captures(&url).and_then(|c| c.get(1)).map(|m| m.as_str().to_string())
                });

                let steam_uri = if let Some(id) = steam_id {
                    format!("steam://store/{id}")
                } else if url.contains("store.steampowered.com") {
                    format!("steam://openurl/{url}")
                } else if url.starts_with("steam://") {
                    url.clone()
                } else {
                    format!("steam://openurl/{url}")
                };

                let opened = tokio::task::spawn_blocking(move || {
                    #[cfg(windows)]
                    {
                        let _ = std::process::Command::new("cmd")
                            .args(["/C", "start", "", &steam_uri])
                            .spawn();
                    }
                    #[cfg(not(windows))]
                    {
                        let _ = open::that_detached(&steam_uri);
                    }
                })
                .await;

                if opened.is_ok() {
                    return Ok(());
                }
            } else if is_epic {
                let slug = if url.contains("/p/") {
                    url.split("/p/").nth(1).and_then(|s| s.split('?').next()).unwrap_or("")
                } else {
                    ""
                };

                let epic_uri = if !slug.is_empty() {
                    format!("com.epicgames.launcher://store/p/{slug}")
                } else {
                    "com.epicgames.launcher://store".to_string()
                };

                let opened = tokio::task::spawn_blocking(move || {
                    #[cfg(windows)]
                    {
                        let _ = std::process::Command::new("cmd")
                            .args(["/C", "start", "", &epic_uri])
                            .spawn();
                    }
                    #[cfg(not(windows))]
                    {
                        let _ = open::that_detached(&epic_uri);
                    }
                })
                .await;

                if opened.is_ok() {
                    return Ok(());
                }
            }

            // Fallback: open in default browser
            let url_clone = url.clone();
            tokio::task::spawn_blocking(move || {
                let _ = open::that_detached(&url_clone);
            })
            .await
            .map_err(|e| e.to_string())?;

            Ok(())
        }
        "launcher" | "webview" => {
            let title = if url.contains("steampowered.com") {
                "Steam Store".to_string()
            } else if url.contains("epicgames.com") {
                "Epic Games Store".to_string()
            } else {
                "Store".to_string()
            };
            open_in_store_webview(app, url, title)
                .await
                .map_err(|e| e.to_string())
        }
        "browser" | "external" | _ => {
            let url_clone = url.clone();
            tokio::task::spawn_blocking(move || {
                let _ = open::that_detached(&url_clone);
            })
            .await
            .map_err(|e| e.to_string())?;
            Ok(())
        }
    }
}

#[tauri::command]
pub async fn open_external_url(url: String) -> Result<()> {
    tokio::task::spawn_blocking(move || {
        let _ = open::that_detached(&url);
    })
    .await
    .map_err(|e| LauncherError::Launch(e.to_string()))?;
    Ok(())
}
