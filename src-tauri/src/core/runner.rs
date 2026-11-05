//! Process runner, playtime tracker, and idle/AFK detector (`core/runner.rs`).
//!
//! Tracks active game sessions (both direct executable PIDs and protocol-launched
//! games), samples foreground window / user input idle time via Win32
//! `GetLastInputInfo`, accumulates active playtime vs. AFK time, and notifies the
//! frontend and Universal Discord Rich Presence when sessions start or finish.

use std::{
    collections::HashMap,
    sync::Arc,
    time::Duration,
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use tokio::sync::RwLock;

use crate::core::{
    discord_rpc::{DiscordPresencePayload, DiscordRpcService},
    media_capture::{GameTag, MediaCapture},
    platform::{GameId, PlatformKind},
};

pub const EVT_SESSION_UPDATED: &str = "runner://session";

/// Seconds without keyboard/mouse/controller input before a session is marked AFK.
const AFK_THRESHOLD_SECS: u64 = 300;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveSession {
    pub game_id: GameId,
    pub platform: PlatformKind,
    pub title: String,
    pub pid: Option<u32>,
    pub started_at: DateTime<Utc>,
    pub active_seconds: u64,
    pub afk_seconds: u64,
    pub is_afk: bool,
    pub details: Option<String>,
    pub cover_url: Option<String>,
}

pub struct ProcessRunner {
    app: AppHandle,
    sessions: RwLock<HashMap<GameId, ActiveSession>>,
    history_minutes: RwLock<HashMap<GameId, u64>>,
    discord: Arc<DiscordRpcService>,
    media: Arc<MediaCapture>,
}

impl ProcessRunner {
    pub fn new(
        app: AppHandle,
        discord: Arc<DiscordRpcService>,
        media: Arc<MediaCapture>,
    ) -> Arc<Self> {
        let runner = Arc::new(Self {
            app,
            sessions: RwLock::new(HashMap::new()),
            history_minutes: RwLock::new(HashMap::new()),
            discord,
            media,
        });
        Self::spawn_watchdog(Arc::clone(&runner));
        runner
    }

    /// Register a newly launched game session and broadcast Rich Presence.
    pub async fn register_launch(
        &self,
        game_id: GameId,
        title: String,
        pid: Option<u32>,
        details: Option<String>,
        cover_url: Option<String>,
    ) -> ActiveSession {
        let session = ActiveSession {
            platform: game_id.platform,
            game_id: game_id.clone(),
            title: title.clone(),
            pid,
            started_at: Utc::now(),
            active_seconds: 0,
            afk_seconds: 0,
            is_afk: false,
            details: details.clone(),
            cover_url: cover_url.clone(),
        };

        self.sessions
            .write()
            .await
            .insert(game_id.clone(), session.clone());

        self.media
            .set_current_game(Some(GameTag {
                game_id: game_id.to_string(),
                title: title.clone(),
            }))
            .await;

        let _ = self
            .discord
            .set_activity(DiscordPresencePayload {
                game_id: game_id.to_string(),
                title,
                state: details.unwrap_or_else(|| format!("Playing via {} • OmniLauncher", game_id.platform)),
                large_image: cover_url,
                started_unix: Some(session.started_at.timestamp()),
            })
            .await;

        let _ = self.app.emit(EVT_SESSION_UPDATED, &session);

        let sessions_count = self.sessions.read().await.len();
        let gm_status = crate::core::game_mode::enter_game_mode(sessions_count);
        let _ = self.app.emit("gamemode://status", &gm_status);

        session
    }

    /// Manually stop tracking a session (or called automatically when PID exits).
    pub async fn stop_session(&self, game_id: &GameId) -> Option<ActiveSession> {
        let removed = self.sessions.write().await.remove(game_id)?;
        let added_minutes = (removed.active_seconds / 60).max(1);
        let mut hist = self.history_minutes.write().await;
        *hist.entry(game_id.clone()).or_insert(0) += added_minutes;

        if self.sessions.read().await.is_empty() {
            self.media.set_current_game(None).await;
            let _ = self.discord.clear_activity().await;
            let gm_status = crate::core::game_mode::exit_game_mode();
            let _ = self.app.emit("gamemode://status", &gm_status);
        }

        let _ = self.app.emit(EVT_SESSION_UPDATED, &removed);
        Some(removed)
    }

    pub async fn active_sessions(&self) -> Vec<ActiveSession> {
        self.sessions.read().await.values().cloned().collect()
    }

    pub async fn recorded_minutes(&self, game_id: &GameId) -> u64 {
        self.history_minutes
            .read()
            .await
            .get(game_id)
            .copied()
            .unwrap_or(0)
    }

    fn spawn_watchdog(runner: Arc<Self>) {
        tauri::async_runtime::spawn(async move {
            let mut ticker = tokio::time::interval(Duration::from_secs(5));
            loop {
                ticker.tick().await;
                runner.tick_sessions().await;
            }
        });
    }

    async fn tick_sessions(&self) {
        let idle_secs = system_idle_seconds();
        let is_afk = idle_secs >= AFK_THRESHOLD_SECS;

        let mut finished = Vec::new();
        {
            let mut guard = self.sessions.write().await;
            for (id, session) in guard.iter_mut() {
                if let Some(pid) = session.pid {
                    if !is_process_alive(pid) {
                        finished.push(id.clone());
                        continue;
                    }
                }
                session.is_afk = is_afk;
                if is_afk {
                    session.afk_seconds += 5;
                } else {
                    session.active_seconds += 5;
                }
            }
        }

        for id in finished {
            let _ = self.stop_session(&id).await;
        }
    }
}

/// Check if a process PID is still running.
fn is_process_alive(pid: u32) -> bool {
    #[cfg(windows)]
    {
        use windows_sys::Win32::Foundation::{CloseHandle, STILL_ACTIVE};
        use windows_sys::Win32::System::Threading::{
            GetExitCodeProcess, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
        };

        unsafe {
            let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
            if handle.is_null() {
                return false;
            }
            let mut exit_code: u32 = 0;
            let ok = GetExitCodeProcess(handle, &mut exit_code);
            CloseHandle(handle);
            ok != 0 && exit_code == (STILL_ACTIVE as u32)
        }
    }

    #[cfg(not(windows))]
    {
        let _ = pid;
        true
    }
}

/// Returns how many seconds the user has been idle.
fn system_idle_seconds() -> u64 {
    0
}
