//! Universal Discord Rich Presence 2.0 (`core/discord_rpc.rs`).
//!
//! Broadcasts Rich Presence over the local Discord IPC socket (`discord-ipc-0..9`)
//! for ANY game launched through OmniLauncher — Steam, Epic Games, Roblox places,
//! itch.io indie builds, or RetroArch / Dolphin / PCSX2 / RPCS3 / Ryujinx ROMs.

use std::sync::Mutex as StdMutex;

use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
use serde::{Deserialize, Serialize};

use crate::error::Result;

/// Default OmniLauncher Discord Application ID (can be overridden by user config).
pub const DEFAULT_DISCORD_CLIENT_ID: &str = "1214992487129481296";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscordPresencePayload {
    pub game_id: String,
    pub title: String,
    pub state: String,
    pub large_image: Option<String>,
    pub started_unix: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscordRpcStatus {
    pub enabled: bool,
    pub connected: bool,
    pub client_id: String,
    pub current: Option<DiscordPresencePayload>,
}

struct InnerState {
    enabled: bool,
    client_id: String,
    client: Option<DiscordIpcClient>,
    current: Option<DiscordPresencePayload>,
}

pub struct DiscordRpcService {
    inner: StdMutex<InnerState>,
}

impl DiscordRpcService {
    pub fn new(client_id: Option<String>) -> Self {
        let id = client_id
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_DISCORD_CLIENT_ID.to_string());
        Self {
            inner: StdMutex::new(InnerState {
                enabled: true,
                client_id: id,
                client: None,
                current: None,
            }),
        }
    }

    pub async fn status(&self) -> DiscordRpcStatus {
        let guard = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        DiscordRpcStatus {
            enabled: guard.enabled,
            connected: guard.client.is_some(),
            client_id: guard.client_id.clone(),
            current: guard.current.clone(),
        }
    }

    pub async fn set_enabled(&self, enabled: bool) -> Result<DiscordRpcStatus> {
        {
            let mut guard = self.inner.lock().unwrap_or_else(|p| p.into_inner());
            guard.enabled = enabled;
            if !enabled {
                if let Some(mut client) = guard.client.take() {
                    let _ = client.clear_activity();
                    let _ = client.close();
                }
                guard.current = None;
            }
        }
        Ok(self.status().await)
    }

    pub async fn set_activity(&self, payload: DiscordPresencePayload) -> Result<DiscordRpcStatus> {
        let (enabled, client_id) = {
            let mut guard = self.inner.lock().unwrap_or_else(|p| p.into_inner());
            guard.current = Some(payload.clone());
            (guard.enabled, guard.client_id.clone())
        };

        if !enabled {
            return Ok(self.status().await);
        }

        // Run blocking local named-pipe IPC on the blocking pool so we never stall Tokio.
        let payload_clone = payload.clone();
        let connected_client = tokio::task::spawn_blocking(move || -> Option<DiscordIpcClient> {
            let mut client = DiscordIpcClient::new(&client_id).ok()?;
            if client.connect().is_err() {
                return None;
            }

            let details = format!("Playing {}", payload_clone.title);
            let mut act = activity::Activity::new()
                .details(&details)
                .state(&payload_clone.state);

            if let Some(ts) = payload_clone.started_unix {
                act = act.timestamps(activity::Timestamps::new().start(ts));
            }

            let large_img = payload_clone
                .large_image
                .as_deref()
                .filter(|u| u.starts_with("http"))
                .unwrap_or("omnilauncher_logo");

            act = act.assets(
                activity::Assets::new()
                    .large_image(large_img)
                    .large_text(&payload_clone.title),
            );

            let _ = client.set_activity(act);
            Some(client)
        })
        .await?;

        {
            let mut guard = self.inner.lock().unwrap_or_else(|p| p.into_inner());
            if let Some(mut prev) = guard.client.take() {
                let _ = prev.close();
            }
            guard.client = connected_client;
        }

        Ok(self.status().await)
    }

    pub async fn clear_activity(&self) -> Result<DiscordRpcStatus> {
        let prev_client = {
            let mut guard = self.inner.lock().unwrap_or_else(|p| p.into_inner());
            guard.current = None;
            guard.client.take()
        };

        if let Some(mut client) = prev_client {
            let _ = tokio::task::spawn_blocking(move || {
                let _ = client.clear_activity();
                let _ = client.close();
            })
            .await;
        }

        Ok(self.status().await)
    }
}
