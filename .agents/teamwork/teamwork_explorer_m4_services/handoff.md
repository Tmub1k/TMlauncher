# Handoff Report: External Services & Integrations Architecture (Milestone 4)

**Agent**: `teamwork_explorer_m4_services`  
**Milestone**: M4 (Flagship Modern Launcher Features — External Services)  
**Parent Conversation ID**: `bed11d0d-d77d-423c-a811-8e81f371aec2`  
**Date**: 2026-11-05T05:05:00Z  

---

## 1. Observation

Direct investigation of the OmniLauncher codebase, test harness, and runtime configuration revealed the following specific observations:

### 1.1 HowLongToBeat (HLTB) Integration (Feature 21)
- **Current Backend Implementation**: In `src-tauri/src/commands/roulette.rs:122-165`, only a basic synchronous helper `estimate_hltb(title: &str) -> HltbBenchmark` exists with 9 hardcoded string checks (`cyberpunk`, `elden ring`, `witcher 3`, `hades`, `hollow knight`, `baldur`, `alan wake 2`, `celeste`, and multiplayers) followed by a deterministic FNV-1a hash formula.
- **Contract Specification**: In `PROJECT.md:116`, the declared IPC command is `get_hltb_data(game_title: String) -> Result<HltbRecord, String>`. In `tests/helpers/contracts.cjs:102-106`, `validateHltbRecord` enforces:
  ```javascript
  hasProperties(hltb, ['game_title', 'main_story_hours', 'main_extra_hours', 'completionist_hours']);
  ok(typeof hltb.main_story_hours === 'number', 'main_story_hours must be number');
  ```
- **Boundary & Interaction Tests**:
  - `tests/tier2_boundaries/b21_hltb_boundaries.test.cjs:6-12`: Requires query sanitization stripping `[™®©]` and punctuation `[:\-–—]`.
  - `tests/tier2_boundaries/b21_hltb_boundaries.test.cjs:21-30`: Requires capping calculated progress at 100% when playtime exceeds target (e.g. 12,500 hours on a 40h game).
  - `tests/tier2_boundaries/b21_hltb_boundaries.test.cjs:33-50`: Unreleased games with 0 hours must return `display: "TBD", hasData: false`.
  - `tests/tier3_interactions/i10_roulette_and_hltb_integration.test.cjs`: Backlog roulette uses HLTB estimates to filter weekend games (`playtime === 0 && hltbStoryHours <= maxHours`).
- **Frontend Deficiencies**: Neither `src/lib/components/GameCard.svelte` nor `src/lib/components/GameDetailsModal.svelte` currently renders a completion progress bar or remaining playtime indicator.

### 1.2 PCGamingWiki (PCGW) Tweak Engine (Feature 24)
- **Current Backend Implementation**: Currently, `src-tauri/src/commands/` has **no** `pcgw.rs` module. It is not declared in `src-tauri/src/commands/mod.rs` or registered in `src-tauri/src/lib.rs`.
- **Contract Specification**: In `PROJECT.md:119`, the declared IPC command is `get_pcgamingwiki_data(game_title: String) -> Result<PcgwRecord, String>`. In `tests/helpers/contracts.cjs:115-119`, `validatePcgwRecord` enforces:
  ```javascript
  hasProperties(pcgw, ['game_title', 'ultrawide_support', 'fps_uncap_supported', 'fov_slider_supported', 'tweaks']);
  ok(typeof pcgw.ultrawide_support === 'boolean', 'ultrawide_support must be boolean');
  ok(Array.isArray(pcgw.tweaks), 'tweaks must be array');
  ```
  And each tweak requires `{ id, title, configFile, key, currentVal, recommendedVal }`.
- **Boundary & Interaction Tests**:
  - `tests/tier2_boundaries/b24_pcgw_boundaries.test.cjs:5-16`: Configuration files marked Windows `FILE_ATTRIBUTE_READONLY` must have their read-only flag cleared prior to editing.
  - `tests/tier2_boundaries/b24_pcgw_boundaries.test.cjs:18-31`: Syntax validation is mandatory before writing to prevent corrupting unbalanced XML/INI configs.
  - `tests/tier2_boundaries/b24_pcgw_boundaries.test.cjs:46-56`: User parameters must be clamped (`FOV: 60..130`, `FPS: 30..360`).
  - `tests/tier1_features/f24_pcgw_tweaks.test.cjs:43-50`: Mandatory creation of `.omnibak` configuration backups before modifying any configuration file, with clean 1-click revert capability.
  - `tests/tier3_interactions/i08_properties_modal_and_pcgw_tweaks.test.cjs:5-22`: PCGW quick tweaks must integrate into the `GamePropertiesModal.svelte` Compatibility tab.

### 1.3 Universal Discord Rich Presence (Feature 26)
- **Current Backend Implementation**: `src-tauri/src/core/discord_rpc.rs` uses `discord-rich-presence = "0.2.5"` and named pipe IPC (`\\.\pipe\discord-ipc-0..9`). In `src-tauri/src/core/runner.rs:104-114`, `ProcessRunner::register_launch` sets activity, and `stop_session` clears it.
- **Deficiencies & Edge Cases**:
  - `tests/tier2_boundaries/b26_discord_boundaries.test.cjs:5-16`: If the Discord client is not running during launch, the launcher must not block, returning `DISCORD_NOT_RUNNING_IDLE`.
  - `tests/tier2_boundaries/b26_discord_boundaries.test.cjs:18-28`: If Discord starts after the game or disconnects, an automatic 15-second backoff reconnect schedule is required.
  - `tests/tier2_boundaries/b26_discord_boundaries.test.cjs:30-38`: Special markdown characters (`*`, `_`, `~`, `` ` ``, `>`) must be stripped from activity strings.
  - `tests/tier2_boundaries/b26_discord_boundaries.test.cjs:40-48`: Strings must be hard-truncated to 128 characters (Discord IPC limit).
  - `tests/tier2_boundaries/b26_discord_boundaries.test.cjs:50-63`: Enforce a maximum rate of 1 presence update per 15 seconds.
  - `tests/tier3_interactions/i04_game_mode_and_discord_rpc.test.cjs:5-48`: Discord presence must start atomically when the game launches and clear atomically when the process terminates.

### 1.4 DealsRadar & Historical Lows (Feature 27)
- **Current Backend Implementation**: In `src-tauri/src/commands/deals.rs:40-60`, `get_deals_radar` queries Epic freebies and CheapShark store 1 deals.
- **Deficiencies & Edge Cases**:
  - `tests/tier1_features/f27_deals_radar.test.cjs:21-29`: Does not currently query or flag All-Time Low (ATL) prices (`is_historical_low: bool`, `badge: "-[X]% [ATL!]"`).
  - `tests/tier2_boundaries/b27_deals_boundaries.test.cjs:20-31`: Must handle uncatalogued titles with missing historical records cleanly (`hasHistoricalData: false`).
  - `tests/tier2_boundaries/b27_deals_boundaries.test.cjs:33-46`: Price hikes (`currentPrice >= regularPrice`) must be discarded.
  - `tests/tier3_interactions/i02_currency_and_deals_radar.test.cjs:5-48`: When currency switches to KZT (`rate = 495.0`), ATL calculation must preserve fidelity and render localized regional savings (`Вы экономите X ₸`).

---

## 2. Logic Chain

1. **Failure Mode of Live Web APIs**: Both HLTB and PCGamingWiki are community web properties subject to Cloudflare WAF anti-bot measures, MediaWiki rate limits, and endpoint URL rotations. Relying solely on live network scraping would make OmniLauncher brittle in offline environments or when firewalled.
2. **Multi-Tier Resilience Pattern**: To ensure 100% uptime and 0ms UI stalling:
   - **Tier 1 (Memory Cache)**: Fast in-memory lookup via `RwLock<HashMap<...>>`.
   - **Tier 2 (Persistent Disk Cache)**: JSON cache in `$APP_DATA/cache/` with time-to-live (TTL).
   - **Tier 3 (Live Network API with strict 3.5s timeout)**: Asynchronous non-blocking HTTP request with polite headers.
   - **Tier 4 (Curated Static Database)**: Top 100 PC titles hardcoded in the binary for zero-network environments.
   - **Tier 5 (Deterministic Heuristic Fallback)**: Genre-aware / title-hash estimator ensuring every game always renders valid visual data without throwing unhandled exceptions.
3. **Contract Adherence**: The test harness in `tests/runner.cjs` executes 336 tests across Tiers 1-4. The data structures returned by the Rust commands must strictly conform to `validateHltbRecord` and `validatePcgwRecord` in `tests/helpers/contracts.cjs`.
4. **IPC Named Pipe Reliability**: Discord RPC over Windows named pipes requires isolation inside `tokio::task::spawn_blocking` with reconnect state machines so that process crashes, admin privilege boundaries, or Discord launches do not freeze the Tauri main thread.

---

## 3. Detailed Architectural Design & Specifications

### 3.1 HowLongToBeat (HLTB) Integration (Feature 21)

#### 3.1.1 Network Protocol & Query Sanitization
- **Primary Live Endpoint**: `POST https://howlongtobeat.com/api/search`
- **Request Headers**:
  ```http
  POST /api/search HTTP/1.1
  Host: howlongtobeat.com
  User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36
  Referer: https://howlongtobeat.com/
  Origin: https://howlongtobeat.com
  Content-Type: application/json
  ```
- **Request Payload**:
  ```json
  {
    "searchType": "games",
    "searchTerms": ["Cyberpunk 2077"],
    "searchPage": 1,
    "size": 10,
    "searchOptions": {
      "games": {
        "userId": 0,
        "platform": "",
        "sortCategory": "popular",
        "rangeCategory": "main"
      }
    }
  }
  ```
- **Query Normalization Engine**:
  ```rust
  pub fn sanitize_hltb_query(title: &str) -> String {
      let no_trademark = regex::Regex::new(r"[™®©]").unwrap().replace_all(title, "");
      let no_punct = regex::Regex::new(r"[:\-–—]").unwrap().replace_all(&no_trademark, " ");
      let clean = regex::Regex::new(r"\s+").unwrap().replace_all(&no_punct, " ");
      clean.trim().to_string()
  }
  ```

#### 3.1.2 Rust Data Structures (`src-tauri/src/commands/hltb.rs`)
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HltbRecord {
    pub game_title: String,
    pub main_story_hours: f32,
    pub main_extra_hours: f32,
    pub completionist_hours: f32,
    pub confidence_score: f32,
    pub is_estimated_fallback: bool,
    pub source: String,             // "live_api" | "cache" | "curated" | "heuristic"
    pub formatted_story: String,     // e.g., "51h 30m"
    pub formatted_extra: String,     // e.g., "102h"
    pub formatted_completionist: String,
    pub display: String,            // "51.5h" or "TBD" if 0h
    pub has_data: bool,             // false for unreleased titles
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HltbCardProgress {
    pub game_id: String,
    pub played_hours: f32,
    pub target_story_hours: f32,
    pub progress_percent: u32,       // 0..100 clamped
    pub remaining_story_hours: f32,
    pub badge_label: String,         // "58% Main" or "Main Done (100%)"
}
```

#### 3.1.3 Rust IPC Commands
- `get_hltb_data(game_title: String) -> Result<HltbRecord>`
- `get_batch_hltb_data(game_titles: Vec<String>) -> Result<HashMap<String, HltbRecord>>`
- `calculate_hltb_progress(played_minutes: u64, target_story_hours: f32) -> HltbCardProgress`

#### 3.1.4 Frontend Integration Contract (`GameCard.svelte` & `GameDetailsModal.svelte`)
- **Card Progress Bar (`GameCard.svelte`)**:
  ```svelte
  {#if hltbProgress && hltbProgress.target_story_hours > 0}
    <div class="mt-2 w-full space-y-1">
      <div class="flex items-center justify-between font-mono text-[9px] text-zinc-400">
        <span>{hltbProgress.badge_label}</span>
        <span>{formatPlaytime(game.playtimeMinutes)} / {hltbProgress.target_story_hours}h</span>
      </div>
      <div class="h-1 w-full overflow-hidden rounded-full bg-white/10">
        <div
          class="h-full rounded-full bg-gradient-to-r from-emerald-500 to-teal-400 transition-all duration-300"
          style="width: {hltbProgress.progress_percent}%"
        ></div>
      </div>
    </div>
  {/if}
  ```

---

### 3.2 PCGamingWiki (PCGW) Tweak Engine (Feature 24)

#### 3.2.1 MediaWiki Cargo Query Protocol
- **Endpoint**: `https://www.pcgamingwiki.com/w/api.php`
- **Steam App ID Lookup Query**:
  `action=cargoquery&tables=Steam,Game,Video,Save_game_data&fields=Game._pageName=Page,Video.ultrawide,Video.ultrawide_notes,Video.fov_slider,Video.fov_slider_notes,Video.fps_60,Video.fps_120,Video.fps_60_notes,Save_game_data.config_file_path,Save_game_data.save_game_path&where=Steam.steam_app_id='{app_id}'&format=json`
- **Title Fallback Query**:
  `action=cargoquery&tables=Game,Video,Save_game_data&fields=Game._pageName=Page,Video.ultrawide,Video.ultrawide_notes,Video.fov_slider,Video.fov_slider_notes,Video.fps_60,Video.fps_120,Video.fps_60_notes,Save_game_data.config_file_path,Save_game_data.save_game_path&where=Game._pageName%20LIKE%20'{sanitized_title}%'&format=json`
- **User-Agent Header**: `User-Agent: OmniLauncher/0.1.0 (https://github.com/omnilauncher; client-app)`

#### 3.2.2 Rust Data Structures (`src-tauri/src/commands/pcgw.rs`)
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PcgwTweak {
    pub id: String,
    pub title: String,
    pub config_file: String,
    pub section: Option<String>,
    pub key: String,
    pub current_val: String,
    pub recommended_val: String,
    pub applied: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PcgwRecord {
    pub game_title: String,
    pub ultrawide_support: bool,
    pub fps_uncap_supported: bool,
    pub fov_slider_supported: bool,
    pub ultrawide_notes: Option<String>,
    pub fps_notes: Option<String>,
    pub fov_notes: Option<String>,
    pub config_paths: Vec<String>,
    pub tweaks: Vec<PcgwTweak>,
    pub source: String,             // "live_cargo" | "cache" | "curated" | "fallback"
    pub page_url: Option<String>,
}
```

#### 3.2.3 Safe Configuration File Modifier (`core/tweak_engine.rs`)
1. **Read-Only Attribute Unsetting**:
   ```rust
   #[cfg(windows)]
   pub fn ensure_file_writable(path: &Path) -> std::io::Result<bool> {
       use std::os::windows::fs::MetadataExt;
       let metadata = std::fs::metadata(path)?;
       let attrs = metadata.file_attributes();
       const FILE_ATTRIBUTE_READONLY: u32 = 0x1;
       if attrs & FILE_ATTRIBUTE_READONLY != 0 {
           let new_attrs = attrs & !FILE_ATTRIBUTE_READONLY;
           // Clear read-only flag via Windows API
           // ...
           return Ok(true);
       }
       Ok(false)
   }
   ```
2. **Mandatory `.omnibak` Creation**:
   Before modifying any file (e.g. `Engine.ini`), copy to `Engine.ini.omnibak`. Reverting copies `Engine.ini.omnibak` back over `Engine.ini`.
3. **Parameter Clamping**:
   - FOV: clamped to `[60, 130]`
   - MaxFPS: clamped to `[30, 360]`

#### 3.2.4 Rust Tauri Commands
- `get_pcgamingwiki_data(game_title: String, steam_app_id: Option<u32>) -> Result<PcgwRecord>`
- `apply_pcgw_tweak(config_path: String, section: Option<String>, key: String, value: String) -> Result<()>`
- `revert_pcgw_tweak(config_path: String) -> Result<()>`
- `open_game_config_directory(config_path: String) -> Result<()>`

---

### 3.3 Universal Discord Rich Presence (Feature 26)

#### 3.3.1 Connection & State Machine
In `src-tauri/src/core/discord_rpc.rs`:
- Named Pipe Path: `\\.\pipe\discord-ipc-0` through `discord-ipc-9`.
- Connection Lifecycle:
  1. `connect()` attempts IPC handshake.
  2. If Discord is not running (`ENOENT`), transitions to `State::IdleNotRunning`. Does not crash or block.
  3. Spawns background reconnect watchdog (`tokio::time::interval(Duration::from_secs(15))`). If a game session is active and Discord opens, connects and sends presence immediately.
  4. On process exit: `clear_activity()` clears presence atomically.

#### 3.3.2 Sanitization, Truncation & Rate Limiting
- **Markdown Stripping**:
  ```rust
  pub fn sanitize_discord_string(text: &str) -> String {
      let re = regex::Regex::new(r"[*_~`>]").unwrap();
      re.replace_all(text, "").to_string()
  }
  ```
- **128-Character Hard Limit**: Truncate strings to `str.chars().take(128).collect()`.
- **15-Second Rate Limiter**: Maintain `last_sent: Instant`. If called within 15 seconds, queue the latest payload and debounce.

#### 3.3.3 Multi-Platform Mapping
| Platform | `details` | `state` | `large_image` | `small_image` |
|---|---|---|---|---|
| **Steam** | `Playing {game.title}` | `Steam • {status}` | CDN Header URL | `platform_steam` |
| **Epic Games** | `Playing {game.title}` | `Epic Games • In Game` | KeyImage URL | `platform_epic` |
| **Roblox** | `Playing Roblox` | `Place: {place.name}` | Thumbnail URL | `platform_roblox` |
| **Retro Emulators**| `Playing {rom.title}` | `Emulating on {emulator.name}` | Boxart / Logo | `platform_emulator` |

---

### 3.4 DealsRadar & Historical Lows (Feature 27)

#### 3.4.1 CheapShark All-Time Low (ATL) Protocol
- Query `https://www.cheapshark.com/api/1.0/deals?storeID=1&pageSize=20&onSale=1&AAA=1`
- Query historical price milestone:
  `https://www.cheapshark.com/api/1.0/games?id={gameId}` or `https://www.cheapshark.com/api/1.0/games?steamAppID={app_id}`
- Response contains:
  ```json
  {
    "info": { "title": "Cyberpunk 2077" },
    "cheapestPriceEver": {
      "price": "29.99",
      "date": 1698710400
    },
    "deals": [ ... ]
  }
  ```
- ATL Rule:
  ```rust
  let is_historical_low = sale_price <= cheapest_price_ever + 0.01;
  let badge = if is_historical_low {
      format!("-{savings}% [ATL!]")
  } else {
      format!("-{savings}%")
  };
  ```

#### 3.4.2 Cross-Store Deal Comparison Command
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoreDealComparison {
    pub title: String,
    pub steam_app_id: Option<u32>,
    pub cheapest_ever_usd: f64,
    pub cheapest_ever_date: Option<String>,
    pub deals: Vec<StoreDealItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoreDealItem {
    pub store_name: String,      // "Steam", "GOG", "Epic Games Store", "Humble Store"
    pub price_usd: f64,
    pub normal_price_usd: f64,
    pub savings_percent: u32,
    pub is_historical_low: bool,
    pub deal_url: String,
}
```

#### 3.4.3 Multi-Store Currency Switcher Integration
- Rate constant: `KZT_RATE = 495.0`.
- Mathematical rule: Convert display price `amount_kzt = round(price_usd * 495)`.
- Regional formatting: Russian locale number formatting (`24 745 ₸`).
- Savings banner: `"Вы экономите {saved_kzt} ₸"`.

---

## 4. Caveats

- **HLTB Scraping Fragility**: HowLongToBeat changes client-side JavaScript bundle hashes periodically. The implementation must never panic if the endpoint returns HTTP 403 or non-JSON responses; it must fall back silently to the curated dataset or heuristic estimator.
- **PCGamingWiki Cargo Outages**: The MediaWiki Cargo database may occasionally return API errors or slow queries. Disk caching with a 7-day TTL is necessary to eliminate repeated calls for static game data.
- **Discord Elevated Privileges**: If Discord is running as Administrator (e.g., when launched by a game) and OmniLauncher is running as a standard user, named pipe connection may return `AccessDenied` (Win32 Error 5). The watchdog must log a debug message and gracefully enter idle without blocking the GUI.
- **CheapestPriceEver Staleness**: CheapShark updates its historical price index periodically. Historical lows reflect tracked digital storefronts (Steam, GOG, Epic, Humble, GreenManGaming), not grey-market keys.

---

## 5. Conclusion

Milestone 4's external service integration is architecturally ready for implementation:
1. **HLTB (`commands/hltb.rs`)**: Complete with query normalizer, 5-tier resilience fallback, `validateHltbRecord` schema compliance, and Svelte 5 card progress bar contract.
2. **PCGamingWiki (`commands/pcgw.rs`)**: Cargo API integration, Windows read-only attribute unsetting, `.omnibak` automated backups, and 1-click tweak integration into `GamePropertiesModal.svelte`.
3. **Discord RPC (`core/discord_rpc.rs`)**: Hardened with 15-second reconnection watchdog, markdown cleaner, 128-char truncation, and multi-platform asset resolution.
4. **DealsRadar (`commands/deals.rs`)**: Augmented with CheapShark All-Time Low (ATL) milestones, cross-store comparison, and dual USD/KZT currency formatting.

---

## 6. Verification Method

To independently verify the implementation against existing automated test specifications:

1. **Run Full Automated Test Suite**:
   ```powershell
   node tests/runner.cjs
   ```
   *Expected Result*: All 336 tests across Tiers 1-4 must continue passing with 0 failures.

2. **Verify Specific Feature & Boundary Test Targets**:
   ```powershell
   node -e "require('./tests/tier1_features/f21_hltb_forecasting.test.cjs')"
   node -e "require('./tests/tier1_features/f24_pcgw_tweaks.test.cjs')"
   node -e "require('./tests/tier1_features/f26_discord_rpc.test.cjs')"
   node -e "require('./tests/tier1_features/f27_deals_radar.test.cjs')"
   node -e "require('./tests/tier2_boundaries/b21_hltb_boundaries.test.cjs')"
   node -e "require('./tests/tier2_boundaries/b24_pcgw_boundaries.test.cjs')"
   node -e "require('./tests/tier2_boundaries/b26_discord_boundaries.test.cjs')"
   node -e "require('./tests/tier2_boundaries/b27_deals_boundaries.test.cjs')"
   node -e "require('./tests/tier3_interactions/i02_currency_and_deals_radar.test.cjs')"
   node -e "require('./tests/tier3_interactions/i04_game_mode_and_discord_rpc.test.cjs')"
   node -e "require('./tests/tier3_interactions/i08_properties_modal_and_pcgw_tweaks.test.cjs')"
   node -e "require('./tests/tier3_interactions/i10_roulette_and_hltb_integration.test.cjs')"
   ```

3. **Verify Rust Backend Compiles**:
   ```powershell
   cd src-tauri
   cargo check
   ```

4. **Verify Frontend Types**:
   ```powershell
   npm run check
   ```
