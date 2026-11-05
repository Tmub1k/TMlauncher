//! Unified library discovery, game launching, Steam Cloud/Achievements/SteamGridDB,
//! and platform profile sync commands.

use std::sync::Arc;

use tauri::{AppHandle, Emitter, State};

use crate::core::{
    config::ConfigStore,
    platform::{
        ArtRef, Game, GameId, LaunchHandle, PlatformKind, PlatformRegistry, ProfileSync, ScanReport,
    },
    runner::{ActiveSession, ProcessRunner},
};
use crate::error::Result;
use crate::platforms::steam::{
    CloudSaveSummary, SteamCredentials, SteamGridArtwork, SteamPlatform,
};
use crate::EVT_LIBRARY_UPDATED;

#[tauri::command]
pub async fn get_library(registry: State<'_, Arc<PlatformRegistry>>) -> Result<Vec<Game>> {
    let cached = registry.cached_games().await;
    if !cached.is_empty() {
        return Ok(cached);
    }
    Ok(registry.scan_all().await.games)
}

#[tauri::command]
pub async fn refresh_library(
    app: AppHandle,
    registry: State<'_, Arc<PlatformRegistry>>,
) -> Result<ScanReport> {
    let report = registry.scan_all().await;
    let _ = app.emit(EVT_LIBRARY_UPDATED, &report);
    Ok(report)
}

#[tauri::command]
pub async fn launch_game(
    game_id: GameId,
    registry: State<'_, Arc<PlatformRegistry>>,
    runner: State<'_, Arc<ProcessRunner>>,
) -> Result<LaunchHandle> {
    let existing = registry.find(&game_id).await;
    let title = existing
        .as_ref()
        .map(|g| g.title.clone())
        .unwrap_or_else(|| game_id.native_id.clone());
    let cover_url = existing.and_then(|g| match g.art.poster {
        Some(ArtRef::Remote(url)) => Some(url),
        _ => None,
    });

    let handle = registry.launch(&game_id).await?;

    runner
        .register_launch(game_id, title, handle.pid, None, cover_url)
        .await;

    Ok(handle)
}

#[tauri::command]
pub async fn install_game(
    game_id: GameId,
    registry: State<'_, Arc<PlatformRegistry>>,
) -> Result<LaunchHandle> {
    registry.install(&game_id).await
}

#[tauri::command]
pub async fn get_active_sessions(
    runner: State<'_, Arc<ProcessRunner>>,
) -> Result<Vec<ActiveSession>> {
    Ok(runner.active_sessions().await)
}

#[tauri::command]
pub async fn sync_platform(
    platform: PlatformKind,
    registry: State<'_, Arc<PlatformRegistry>>,
) -> Result<ProfileSync> {
    let connector = registry.connector(platform)?;
    connector.sync_profile().await
}

#[tauri::command]
pub async fn set_steam_credentials(
    credentials: SteamCredentials,
    steam: State<'_, Arc<SteamPlatform>>,
    config: State<'_, ConfigStore>,
) -> Result<()> {
    steam.set_credentials(credentials.clone()).await;
    config
        .update(|cfg| {
            cfg.steam = credentials;
        })
        .await?;
    Ok(())
}

#[tauri::command]
pub async fn get_steam_credentials(
    steam: State<'_, Arc<SteamPlatform>>,
) -> Result<SteamCredentials> {
    Ok(steam.credentials().await)
}

#[tauri::command]
pub async fn get_steam_cloud_saves(app_id: u32) -> Result<Vec<CloudSaveSummary>> {
    tokio::task::spawn_blocking(move || SteamPlatform::inspect_cloud_saves(app_id)).await?
}


#[tauri::command]
pub async fn search_steamgriddb_posters(
    app_id: u32,
    steam: State<'_, Arc<SteamPlatform>>,
) -> Result<Vec<SteamGridArtwork>> {
    steam.fetch_steamgriddb_posters(app_id).await
}
