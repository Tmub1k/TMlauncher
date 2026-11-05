//! Tauri commands for the Roblox FastFlags editor, multi-instance mutex,
//! `roblox-player:` protocol interception, and active place detection.

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::core::platform::{GameId, LaunchHandle, PlatformKind, PlatformRegistry};
use crate::error::Result;
use crate::platforms::roblox::{FastFlagMap, FastFlagPreset, RobloxPlaceInfo, RobloxPlatform};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RobloxExperiencePreset {
    pub place_id: u64,
    pub title: String,
    pub category: String,
    pub description: String,
    pub icon_url: String,
}

#[tauri::command]
pub async fn roblox_get_fast_flags(roblox: State<'_, Arc<RobloxPlatform>>) -> Result<FastFlagMap> {
    roblox.read_fast_flags().await
}

#[tauri::command]
pub async fn roblox_set_fast_flags(
    flags: FastFlagMap,
    roblox: State<'_, Arc<RobloxPlatform>>,
) -> Result<FastFlagMap> {
    roblox.write_fast_flags(&flags).await?;
    Ok(flags)
}

#[tauri::command]
pub async fn roblox_apply_preset(
    preset: FastFlagPreset,
    roblox: State<'_, Arc<RobloxPlatform>>,
) -> Result<FastFlagMap> {
    roblox.apply_preset(preset).await
}

#[tauri::command]
pub async fn roblox_set_multi_instance(
    enabled: bool,
    roblox: State<'_, Arc<RobloxPlatform>>,
) -> Result<bool> {
    roblox.set_multi_instance(enabled)
}

#[tauri::command]
pub async fn roblox_multi_instance_status(roblox: State<'_, Arc<RobloxPlatform>>) -> Result<bool> {
    Ok(roblox.is_multi_instance_enabled())
}

#[tauri::command]
pub async fn roblox_detect_place(
    roblox: State<'_, Arc<RobloxPlatform>>,
) -> Result<Option<RobloxPlaceInfo>> {
    roblox.detect_active_place().await
}

#[tauri::command]
pub async fn roblox_register_protocol(use_omnilauncher: bool) -> Result<String> {
    tokio::task::spawn_blocking(move || RobloxPlatform::register_protocol_handler(use_omnilauncher))
        .await?
}

#[tauri::command]
pub async fn roblox_launch_place(
    place_id: u64,
    registry: State<'_, Arc<PlatformRegistry>>,
) -> Result<LaunchHandle> {
    let game_id = GameId::new(PlatformKind::Roblox, format!("place:{place_id}"));
    registry.launch(&game_id).await
}

#[tauri::command]
pub async fn roblox_search_experiences(
    http: State<'_, reqwest::Client>,
    query: String,
) -> Result<Vec<RobloxExperiencePreset>> {
    let clean = query.trim();
    if clean.is_empty() {
        return Ok(roblox_get_popular_experiences());
    }

    let url = "https://apis.roblox.com/search-api/omni-search";
    let resp = http
        .get(url)
        .query(&[
            ("searchQuery", clean),
            ("sessionId", "c0a80101-0000-0000-0000-000000000001"),
        ])
        .send()
        .await;

    let Ok(resp) = resp else {
        return Ok(roblox_get_popular_experiences());
    };

    let Ok(json) = resp.json::<serde_json::Value>().await else {
        return Ok(roblox_get_popular_experiences());
    };

    let mut presets = Vec::new();
    let mut place_ids = Vec::new();

    if let Some(results) = json.get("searchResults").and_then(|v| v.as_array()) {
        for group in results {
            if let Some(contents) = group.get("contents").and_then(|v| v.as_array()) {
                for c in contents {
                    let Some(place_id) = c.get("rootPlaceId").and_then(|v| v.as_u64()) else {
                        continue;
                    };
                    let title = c.get("name").and_then(|v| v.as_str()).unwrap_or("Roblox Experience").to_string();
                    let players = c.get("playerCount").and_then(|v| v.as_u64()).unwrap_or(0);
                    let upvotes = c.get("totalUpVotes").and_then(|v| v.as_u64()).unwrap_or(0);
                    let maturity = c.get("ageRecommendationDisplayName").and_then(|v| v.as_str()).unwrap_or("Maturity: All");
                    let category = format!("👥 {players} онлайн • {maturity}");
                    let description = format!("Рейтинг: {upvotes} лайков. Быстрый запуск в Roblox.");

                    place_ids.push(place_id);
                    presets.push(RobloxExperiencePreset {
                        place_id,
                        title,
                        category,
                        description,
                        icon_url: "https://images.rbxcdn.com/264b38d7211bf52c3c9c916fcbb228c2.png".into(),
                    });

                    if presets.len() >= 30 {
                        break;
                    }
                }
            }
            if presets.len() >= 30 {
                break;
            }
        }
    }

    if !place_ids.is_empty() {
        let ids_str = place_ids.iter().map(|id| id.to_string()).collect::<Vec<_>>().join(",");
        let thumb_url = format!(
            "https://thumbnails.roblox.com/v1/places/gameicons?placeIds={}&size=512x512&format=Png&isCircular=false",
            ids_str
        );
        if let Ok(thumb_resp) = http.get(&thumb_url).send().await {
            if let Ok(thumb_json) = thumb_resp.json::<serde_json::Value>().await {
                if let Some(data) = thumb_json.get("data").and_then(|v| v.as_array()) {
                    let mut map = std::collections::HashMap::new();
                    for d in data {
                        if let (Some(target_id), Some(img)) = (
                            d.get("targetId").and_then(|v| v.as_u64()),
                            d.get("imageUrl").and_then(|v| v.as_str()),
                        ) {
                            map.insert(target_id, img.to_string());
                        }
                    }
                    for p in &mut presets {
                        if let Some(img) = map.get(&p.place_id) {
                            p.icon_url = img.clone();
                        }
                    }
                }
            }
        }
    }

    if presets.is_empty() {
        Ok(roblox_get_popular_experiences())
    } else {
        Ok(presets)
    }
}

#[tauri::command]
pub fn roblox_get_popular_experiences() -> Vec<RobloxExperiencePreset> {
    vec![
        RobloxExperiencePreset {
            place_id: 2753915549,
            title: "Blox Fruits".into(),
            category: "Anime RPG".into(),
            description: "Станьте мастером меча или используйте силу фруктов. Исследуйте океаны и побеждайте боссов.".into(),
            icon_url: "https://tr.rbxcdn.com/180DAY-b4d24a9e525166299b9ba89953ad7ff3/150/150/Image/Webp/noFilter".into(),
        },
        RobloxExperiencePreset {
            place_id: 4924922222,
            title: "Brookhaven 🏡 RP".into(),
            category: "Roleplay & Life".into(),
            description: "Живите в городе, покупайте дома, водите крутые автомобили и заводите новых друзей.".into(),
            icon_url: "https://tr.rbxcdn.com/180DAY-39d5622345eebfe97bfabdf0fb2cbf93/150/150/Image/Webp/noFilter".into(),
        },
        RobloxExperiencePreset {
            place_id: 17625359962,
            title: "RIVALS ⚔️".into(),
            category: "Competitive FPS".into(),
            description: "Динамичный дуэльный и командный шутер от первого лица с кастомизацией оружия.".into(),
            icon_url: "https://tr.rbxcdn.com/180DAY-923cb9f6cb7beaa4ff499c7ea0e92750/150/150/Image/Webp/noFilter".into(),
        },
        RobloxExperiencePreset {
            place_id: 920587237,
            title: "Adopt Me!".into(),
            category: "Pets & Simulation".into(),
            description: "Выращивайте милых питомцев, украшайте свой дом и исследуйте остров Адопт Ми.".into(),
            icon_url: "https://tr.rbxcdn.com/180DAY-5ae32b262a67e54c75ca78e826b5c00e/150/150/Image/Webp/noFilter".into(),
        },
        RobloxExperiencePreset {
            place_id: 16732694052,
            title: "Fisch 🎣".into(),
            category: "Adventure RPG".into(),
            description: "Исследуйте архипелаг, ловите редких морских чудовищ и прокачивайте удочки.".into(),
            icon_url: "https://tr.rbxcdn.com/180DAY-5ca5989eef1ff51e51b1406e23635a96/150/150/Image/Webp/noFilter".into(),
        },
        RobloxExperiencePreset {
            place_id: 142823291,
            title: "Murder Mystery 2".into(),
            category: "Survival Mystery".into(),
            description: "Выживайте в роли невиновного, раскройте убийцу в роли шерифа или устройте хаос.".into(),
            icon_url: "https://tr.rbxcdn.com/180DAY-f0e74f1b5394ef338f97bbcb17ca2c68/150/150/Image/Webp/noFilter".into(),
        },
        RobloxExperiencePreset {
            place_id: 13772394625,
            title: "Blade Ball ⚔️".into(),
            category: "Arcade Combat".into(),
            description: "Отбивайте смертоносный мяч мечами и способностями. Последний выживший побеждает.".into(),
            icon_url: "https://tr.rbxcdn.com/180DAY-fdbdcf8362678da47f8ba7febeba74c2/150/150/Image/Webp/noFilter".into(),
        },
        RobloxExperiencePreset {
            place_id: 10449761463,
            title: "The Strongest Battlegrounds".into(),
            category: "Anime Fighting".into(),
            description: "Брутальные арены, разрушаемое окружение и комбо величайших аниме героев.".into(),
            icon_url: "https://tr.rbxcdn.com/180DAY-218228591f4861b53bb26fe0b6667ff4/150/150/Image/Webp/noFilter".into(),
        },
        RobloxExperiencePreset {
            place_id: 6516141723,
            title: "DOORS 👁️".into(),
            category: "Horror Roguelite".into(),
            description: "Пройдите 100 дверей отеля, разгадывая загадки и прячась от сущностей во тьме.".into(),
            icon_url: "https://tr.rbxcdn.com/180DAY-27f91a27e7ca4a243c49e772274d817f/150/150/Image/Webp/noFilter".into(),
        },
        RobloxExperiencePreset {
            place_id: 1962086868,
            title: "Tower of Hell".into(),
            category: "Hardcore Obby".into(),
            description: "Случайно генерируемая башня паркура без чекпоинтов. Успей дойти до вершины!".into(),
            icon_url: "https://tr.rbxcdn.com/180DAY-79b8f15858cf3e271eb1b4b7c126ec6e/150/150/Image/Webp/noFilter".into(),
        },
    ]
}
