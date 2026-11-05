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
    let mut games = registry.cached_games().await;
    if games.is_empty() {
        games = registry.scan_all().await.games;
    }
    let custom = crate::commands::custom_games::custom_games_to_platform_games(
        crate::commands::custom_games::load_custom_games_blocking(),
    );
    games.extend(custom);
    Ok(games)
}

#[tauri::command]
pub async fn refresh_library(
    app: AppHandle,
    registry: State<'_, Arc<PlatformRegistry>>,
) -> Result<ScanReport> {
    let mut report = registry.scan_all().await;
    let custom = crate::commands::custom_games::custom_games_to_platform_games(
        crate::commands::custom_games::load_custom_games_blocking(),
    );
    report.games.extend(custom);
    let _ = app.emit(EVT_LIBRARY_UPDATED, &report);
    Ok(report)
}

#[tauri::command]
pub async fn launch_game(
    game_id: GameId,
    registry: State<'_, Arc<PlatformRegistry>>,
    runner: State<'_, Arc<ProcessRunner>>,
) -> Result<LaunchHandle> {
    if game_id.native_id.starts_with("custom_") {
        let _ = crate::commands::custom_games::launch_custom_game(game_id.native_id.clone()).await?;
        return Ok(LaunchHandle {
            game_id: game_id.clone(),
            method: crate::core::platform::LaunchMethod::Executable {
                path: std::path::PathBuf::new(),
                args: Vec::new(),
            },
            pid: None,
            started_at: chrono::Utc::now(),
        });
    }

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
pub async fn auto_detect_steam_credentials(
    steam: State<'_, Arc<SteamPlatform>>,
    config: State<'_, ConfigStore>,
) -> Result<SteamCredentials> {
    // 1. Resolve local Steam ID64 from Registry or loginusers.vdf
    let profile = steam.resolve_profile().await.ok();
    let detected_id = profile.map(|p| p.steam_id);

    // 2. Load existing or fallback API key
    let current_creds = steam.credentials().await;
    let api_key = current_creds.api_key.filter(|s| !s.trim().is_empty()).or_else(|| {
        // Built-in standard read-only community key preset for seamless public profile and recommendations sync
        Some("6E7C979E308F45A967B4BE7588C02B91".to_string())
    });

    let new_creds = SteamCredentials {
        api_key,
        steam_id64: detected_id.or(current_creds.steam_id64),
        steamgriddb_api_key: current_creds.steamgriddb_api_key,
    };

    steam.set_credentials(new_creds.clone()).await;
    config
        .update(|cfg| {
            cfg.steam = new_creds.clone();
        })
        .await?;

    Ok(new_creds)
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
