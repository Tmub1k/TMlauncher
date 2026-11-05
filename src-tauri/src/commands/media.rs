//! Screenshot, hardware video clip recording, and gallery Tauri commands.

use std::sync::Arc;

use tauri::State;

use crate::core::{
    config::ConfigStore,
    media_capture::{CaptureConfig, MediaCapture, MediaItem, RecordingStatus, RecordingToggle},
};
use crate::error::Result;

#[tauri::command]
pub async fn take_screenshot(media: State<'_, Arc<MediaCapture>>) -> Result<MediaItem> {
    media.take_screenshot().await
}

#[tauri::command]
pub async fn toggle_recording(media: State<'_, Arc<MediaCapture>>) -> Result<RecordingToggle> {
    media.toggle_recording().await
}

#[tauri::command]
pub async fn recording_status(media: State<'_, Arc<MediaCapture>>) -> Result<RecordingStatus> {
    Ok(media.status().await)
}

#[tauri::command]
pub async fn list_media(media: State<'_, Arc<MediaCapture>>) -> Result<Vec<MediaItem>> {
    media.list_media().await
}

#[tauri::command]
pub async fn delete_media(path: String) -> Result<bool> {
    let p = std::path::PathBuf::from(&path);
    if p.exists() {
        tokio::fs::remove_file(&p).await.map_err(|source| crate::error::LauncherError::Io {
            path: p,
            source,
        })?;
        return Ok(true);
    }
    Ok(false)
}

#[tauri::command]
pub async fn get_capture_config(media: State<'_, Arc<MediaCapture>>) -> Result<CaptureConfig> {
    Ok(media.config().await)
}

#[tauri::command]
pub async fn set_capture_config(
    next: CaptureConfig,
    media: State<'_, Arc<MediaCapture>>,
    config: State<'_, ConfigStore>,
) -> Result<CaptureConfig> {
    media.update_config(next.clone()).await?;
    let saved = config
        .update(|cfg| {
            cfg.capture = next;
        })
        .await?;
    Ok(saved.capture)
}
