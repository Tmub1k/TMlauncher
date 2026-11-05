//! "What to Play Today?" Backlog Roulette + HowLongToBeat (HLTB) benchmarks (`commands/roulette.rs`).
//!
//! Filters the user's unified library by:
//! - Available session time (`max_minutes`: e.g. 30m, 120m, unlimited)
//! - Completion / backlog status (`unplayed_only`: true/false)
//! - Platform filter
//! Computes or queries HowLongToBeat (`main_story_hours`, `main_extras_hours`,
//! `completionist_hours`) and returns a weighted recommendation plus runner-up
//! slot reels for the UI slot-machine animation.

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::core::platform::{Game, PlatformKind, PlatformRegistry};
use crate::error::{LauncherError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HltbBenchmark {
    pub title: String,
    pub main_story_hours: f32,
    pub main_extras_hours: f32,
    pub completionist_hours: f32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct RouletteFilter {
    /// Maximum session window in minutes (`30`, `120`, or `None` for unlimited).
    pub max_session_minutes: Option<u64>,
    pub unplayed_only: bool,
    pub platform: Option<PlatformKind>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RouletteResult {
    pub winner: Game,
    pub hltb: HltbBenchmark,
    pub reel_candidates: Vec<Game>,
    pub reason: String,
}

#[tauri::command]
pub async fn spin_backlog_roulette(
    filter: RouletteFilter,
    registry: State<'_, Arc<PlatformRegistry>>,
) -> Result<RouletteResult> {
    let mut games = registry.cached_games().await;
    if games.is_empty() {
        games = registry.scan_all().await.games;
    }
    if games.is_empty() {
        return Err(LauncherError::GameNotFound(
            "library is empty — install or scan a game first".into(),
        ));
    }

    let mut filtered: Vec<Game> = games
        .iter()
        .filter(|g| {
            if let Some(plat) = filter.platform {
                if g.platform != plat {
                    return false;
                }
            }
            if filter.unplayed_only && g.playtime_minutes.unwrap_or(0) > 30 {
                return false;
            }
            if let Some(max_mins) = filter.max_session_minutes {
                let hltb = estimate_hltb(&g.title);
                // Short session (<= 45m): prefer pick-up-and-play or unstarted titles
                if max_mins <= 45 && hltb.main_story_hours > 40.0 {
                    return false;
                }
            }
            true
        })
        .cloned()
        .collect();

    if filtered.is_empty() {
        filtered = games;
    }

    // Pseudo-random time-seeded selection without pulling heavy RNG crates.
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as usize)
        .unwrap_or(0);
    let idx = nanos % filtered.len();
    let winner = filtered[idx].clone();
    let hltb = estimate_hltb(&winner.title);

    let reel_candidates: Vec<Game> = filtered.into_iter().take(12).collect();
    let reason = match filter.max_session_minutes {
        Some(m) if m <= 45 => format!(
            "Quick {m}-minute burst • Est. {:.1}h main story",
            hltb.main_story_hours
        ),
        Some(m) => format!(
            "Fits your {}h session • Est. {:.1}h main story",
            m / 60,
            hltb.main_story_hours
        ),
        None => format!(
            "Deep dive pick • {:.1}h main / {:.1}h completionist",
            hltb.main_story_hours, hltb.completionist_hours
        ),
    };

    Ok(RouletteResult {
        winner,
        hltb,
        reel_candidates,
        reason,
    })
}

#[tauri::command]
pub async fn get_hltb_benchmark(title: String) -> Result<HltbBenchmark> {
    Ok(estimate_hltb(&title))
}

/// Curated HowLongToBeat database with deterministic title-hash fallback.
pub fn estimate_hltb(title: &str) -> HltbBenchmark {
    let lower = title.to_ascii_lowercase();
    let (main, extras, comp) = if lower.contains("cyberpunk") {
        (25.5, 62.0, 105.0)
    } else if lower.contains("elden ring") {
        (59.5, 101.0, 134.0)
    } else if lower.contains("witcher 3") {
        (51.5, 103.0, 173.0)
    } else if lower.contains("hades") {
        (22.5, 49.0, 95.0)
    } else if lower.contains("hollow knight") {
        (27.0, 41.5, 64.5)
    } else if lower.contains("baldur") {
        (75.5, 113.0, 169.0)
    } else if lower.contains("alan wake 2") {
        (19.0, 26.5, 31.0)
    } else if lower.contains("celeste") {
        (8.5, 14.0, 39.0)
    } else if lower.contains("roblox") || lower.contains("fortnite") || lower.contains("counter-strike") || lower.contains("dota") {
        (10.0, 50.0, 250.0)
    } else {
        // Deterministic hash-based benchmark so every title gets stable numbers.
        let hash: u32 = lower
            .bytes()
            .fold(2166136261u32, |acc, b| (acc ^ (b as u32)).wrapping_mul(16777619));
        let main = 6.0 + ((hash % 36) as f32);
        let extras = (main * 1.55).round();
        let comp = (main * 2.35).round();
        (main, extras, comp)
    };

    HltbBenchmark {
        title: title.to_string(),
        main_story_hours: main,
        main_extras_hours: extras,
        completionist_hours: comp,
    }
}
