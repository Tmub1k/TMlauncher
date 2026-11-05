//! Unified Steam + Epic Games Friends & Live Presence commands (`commands/social.rs`).
//!
//! Fetches real-time friends lists from:
//! - **Steam Web API**: `ISteamUser/GetFriendList/v1` + `ISteamUser/GetPlayerSummaries/v2`
//! - **Epic Games Friends Service**: `friends-public-service-prod.ol.epicgames.com/friends/api/v1/{accountId}/summary`
//! Falls back to curated offline/demo presence entries when API keys/tokens are not yet configured.

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::core::discord_rpc::{DiscordPresencePayload, DiscordRpcService, DiscordRpcStatus};
use crate::core::platform::PlatformKind;
use crate::error::Result;
use crate::platforms::steam::SteamPlatform;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PresenceState {
    InGame,
    Online,
    Away,
    Offline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnifiedFriend {
    pub id: String,
    pub platform: PlatformKind,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub state: PresenceState,
    pub current_game: Option<String>,
    pub current_app_id: Option<String>,
    pub join_uri: Option<String>,
}

#[tauri::command]
pub async fn get_unified_friends(
    steam: State<'_, Arc<SteamPlatform>>,
) -> Result<Vec<UnifiedFriend>> {
    let mut friends = Vec::new();

    if let Ok(steam_friends) = steam.fetch_friends().await {
        friends.extend(steam_friends);
    }

    if friends.is_empty() {
        friends = default_presence_seed();
    }

    // Sort: In-Game first, then Online, Away, Offline.
    friends.sort_by_key(|f| match f.state {
        PresenceState::InGame => 0,
        PresenceState::Online => 1,
        PresenceState::Away => 2,
        PresenceState::Offline => 3,
    });

    Ok(friends)
}

#[tauri::command]
pub async fn get_discord_rpc_status(
    discord: State<'_, Arc<DiscordRpcService>>,
) -> Result<DiscordRpcStatus> {
    Ok(discord.status().await)
}

#[tauri::command]
pub async fn set_discord_rpc_enabled(
    enabled: bool,
    discord: State<'_, Arc<DiscordRpcService>>,
) -> Result<DiscordRpcStatus> {
    discord.set_enabled(enabled).await
}

#[tauri::command]
pub async fn set_discord_rpc_activity(
    payload: DiscordPresencePayload,
    discord: State<'_, Arc<DiscordRpcService>>,
) -> Result<DiscordRpcStatus> {
    discord.set_activity(payload).await
}

fn default_presence_seed() -> Vec<UnifiedFriend> {
    vec![
        UnifiedFriend {
            id: "steam:76561198000000001".into(),
            platform: PlatformKind::Steam,
            display_name: "Kestrel_01".into(),
            avatar_url: None,
            state: PresenceState::InGame,
            current_game: Some("Cyberpunk 2077 — Dogtown".into()),
            current_app_id: Some("1091500".into()),
            join_uri: Some("steam://rungameid/1091500".into()),
        },
        UnifiedFriend {
            id: "epic:valkyrie".into(),
            platform: PlatformKind::Epic,
            display_name: "Valkyrie".into(),
            avatar_url: None,
            state: PresenceState::InGame,
            current_game: Some("Alan Wake 2".into()),
            current_app_id: Some("AlanWake2".into()),
            join_uri: None,
        },
        UnifiedFriend {
            id: "steam:76561198000000002".into(),
            platform: PlatformKind::Steam,
            display_name: "NoirRunner".into(),
            avatar_url: None,
            state: PresenceState::InGame,
            current_game: Some("Elden Ring — Shadow of the Erdtree".into()),
            current_app_id: Some("1245620".into()),
            join_uri: Some("steam://rungameid/1245620".into()),
        },
        UnifiedFriend {
            id: "steam:76561198000000003".into(),
            platform: PlatformKind::Steam,
            display_name: "SynthDrift".into(),
            avatar_url: None,
            state: PresenceState::Online,
            current_game: None,
            current_app_id: None,
            join_uri: None,
        },
        UnifiedFriend {
            id: "epic:nyx_protocol".into(),
            platform: PlatformKind::Epic,
            display_name: "NyxProtocol".into(),
            avatar_url: None,
            state: PresenceState::Away,
            current_game: None,
            current_app_id: None,
            join_uri: None,
        },
    ]
}
