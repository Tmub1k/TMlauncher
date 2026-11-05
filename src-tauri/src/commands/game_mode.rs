//! Tauri IPC commands for Zero-Footprint Game Mode.

use std::sync::Arc;
use tauri::State;
use crate::core::game_mode::{self, GameModeStatus, MemoryTrimResult};
use crate::core::runner::ProcessRunner;

#[tauri::command]
pub async fn enter_game_mode(
    runner: State<'_, Arc<ProcessRunner>>,
    _pid: Option<u32>,
) -> Result<GameModeStatus, String> {
    let count = runner.active_sessions().await.len().max(1);
    Ok(game_mode::enter_game_mode(count))
}

#[tauri::command]
pub async fn exit_game_mode() -> Result<GameModeStatus, String> {
    Ok(game_mode::exit_game_mode())
}

#[tauri::command]
pub async fn get_game_mode_status(
    runner: State<'_, Arc<ProcessRunner>>,
) -> Result<GameModeStatus, String> {
    let count = runner.active_sessions().await.len();
    Ok(game_mode::get_game_mode_status(count))
}

#[tauri::command]
pub fn trim_process_memory() -> Result<MemoryTrimResult, String> {
    Ok(game_mode::trim_working_set())
}
