//! Custom / Non-Steam third-party application management.
//!
//! Stores user-added third-party games and software in `%APPDATA%\OmniLauncher\custom_games.json`.

use std::path::PathBuf;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use crate::core::platform::{Artwork, ArtRef, Game, PlatformKind};
use crate::error::{LauncherError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomGameEntry {
    pub id: String,
    pub name: String,
    pub executable_path: String,
    pub launch_arguments: Option<String>,
    pub working_dir: Option<String>,
    pub icon_url: Option<String>,
    pub cover_url: Option<String>,
    pub added_at: i64,
}

fn get_storage_path() -> PathBuf {
    let base = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("OmniLauncher");
    let _ = std::fs::create_dir_all(&base);
    base.join("custom_games.json")
}

pub fn load_custom_games_blocking() -> Vec<CustomGameEntry> {
    let path = get_storage_path();
    if !path.exists() {
        return Vec::new();
    }
    let Ok(raw) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };
    serde_json::from_str(&raw).unwrap_or_default()
}

pub fn save_custom_games_blocking(list: &[CustomGameEntry]) -> Result<()> {
    let path = get_storage_path();
    let serialized = serde_json::to_string_pretty(list)?;
    std::fs::write(&path, serialized)
        .map_err(|e| LauncherError::Io { path, source: e })?;
    Ok(())
}

#[tauri::command]
pub async fn add_custom_game(
    name: String,
    executable_path: String,
    launch_arguments: Option<String>,
    cover_url: Option<String>,
) -> Result<CustomGameEntry> {
    let exe = PathBuf::from(&executable_path);
    if !exe.exists() {
        return Err(LauncherError::InvalidInput(format!(
            "Файл не найден по пути: {}",
            executable_path
        )));
    }

    let id = format!("custom_{}", Utc::now().timestamp_millis());
    let working_dir = exe.parent().map(|p| p.to_string_lossy().to_string());

    let entry = CustomGameEntry {
        id: id.clone(),
        name: if name.trim().is_empty() {
            exe.file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "Custom Game".into())
        } else {
            name.trim().to_string()
        },
        executable_path,
        launch_arguments,
        working_dir,
        icon_url: None,
        cover_url,
        added_at: Utc::now().timestamp(),
    };

    let mut list = load_custom_games_blocking();
    list.push(entry.clone());
    save_custom_games_blocking(&list)?;

    Ok(entry)
}

#[tauri::command]
pub async fn get_custom_games() -> Result<Vec<CustomGameEntry>> {
    Ok(load_custom_games_blocking())
}

#[tauri::command]
pub async fn remove_custom_game(id: String) -> Result<bool> {
    let mut list = load_custom_games_blocking();
    let initial_len = list.len();
    list.retain(|g| g.id != id);
    if list.len() != initial_len {
        save_custom_games_blocking(&list)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

#[tauri::command]
pub async fn launch_custom_game(id: String) -> Result<bool> {
    let list = load_custom_games_blocking();
    let game = list
        .into_iter()
        .find(|g| g.id == id)
        .ok_or_else(|| LauncherError::GameNotFound(format!("custom game {id}")))?;

    let mut cmd = std::process::Command::new(&game.executable_path);
    if let Some(args_str) = &game.launch_arguments {
        for arg in args_str.split_whitespace() {
            cmd.arg(arg);
        }
    }
    if let Some(wd) = &game.working_dir {
        cmd.current_dir(wd);
    }

    let child = cmd.spawn().map_err(|e| {
        LauncherError::Launch(format!("Не удалось запустить стороннее приложение: {e}"))
    })?;

    tracing::info!(pid = child.id(), title = %game.name, "launched custom game");
    Ok(true)
}

pub fn custom_games_to_platform_games(entries: Vec<CustomGameEntry>) -> Vec<Game> {
    entries
        .into_iter()
        .map(|entry| {
            let mut game = Game::new(
                PlatformKind::Steam, // Unified display in UI
                entry.id.clone(),
                entry.name,
            );
            game.installed = true;
            game.executable = Some(PathBuf::from(&entry.executable_path));
            if let Some(wd) = entry.working_dir {
                game.install_dir = Some(PathBuf::from(wd));
            }
            if let Some(cover) = entry.cover_url {
                game.art = Artwork {
                    poster: Some(ArtRef::Remote(cover)),
                    hero: None,
                    logo: None,
                    icon: None,
                };
            }
            game.tags = vec!["Сторонняя игра".into(), "Custom App".into()];
            game
        })
        .collect()
}
