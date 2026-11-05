//! Native Windows 11 Mica / Tabbed (Mica Alt) / Acrylic window vibrancy control.

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

    let base_tag = format!("<base href=\"{url}\" target=\"_blank\">");
    let html = if let Some(idx) = text.find("<head>") {
        let (head, rest) = text.split_at(idx + 6);
        format!("{head}\n{base_tag}\n{rest}")
    } else {
        format!("{base_tag}\n{text}")
    };
    Ok(html)
}

#[tauri::command]
pub async fn open_in_app_store_window(
    app: AppHandle,
    url: String,
    title: String,
) -> Result<()> {
    let parsed: tauri::Url = url.parse().map_err(|e| LauncherError::InvalidInput(format!("{e}")))?;

    if let Some(w) = app.get_webview_window("store_window") {
        let _ = w.navigate(parsed);
        let _ = w.set_title(&title);
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
        return Ok(());
    }

    let win = tauri::WebviewWindowBuilder::new(
        &app,
        "store_window",
        tauri::WebviewUrl::External(parsed),
    )
    .title(&title)
    .inner_size(1220.0, 820.0)
    .center()
    .build()
    .map_err(|e| LauncherError::Launch(format!("failed to open store window: {e}")))?;

    let _ = win.show();
    let _ = win.set_focus();
    Ok(())
}
