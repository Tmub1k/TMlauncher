# Market Research & Ecosystem Power-Tools Analysis for OmniLauncher
**Agent**: Market Research Agent 2 (`teamwork_explorer`)  
**Mission**: Deep investigation of gaming launcher plugins, extensions, and community power-tools (Playnite, Decky Loader) to formulate flagship features for OmniLauncher ("Steam 2").  
**Date**: 2026-11-04  

---

## 1. Observation

### 1.1 Codebase & Architectural Context
From direct inspection of the workspace files:
- **`c:\Users\Tmub\Documents\TMlauncher\package.json`**:
  - Frontend framework: Svelte 5 (`svelte: ^5.16.0`), Tailwind CSS v4 (`@tailwindcss/vite: ^4.0.0`), Vite 6 (`vite: ^6.0.5`), TypeScript (`typescript: ^5.7.2`).
  - Desktop integration: Tauri v2 (`@tauri-apps/api: ^2.2.0`, `@tauri-apps/cli: ^2.2.0`).
- **`c:\Users\Tmub\Documents\TMlauncher\src-tauri\Cargo.toml`**:
  - Rust version 1.80+, Tauri 2 with `protocol-asset`, `tauri-plugin-global-shortcut`.
  - HTTP & Network: `reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }`.
  - Serialization: `serde`, `serde_json`, `toml`, `keyvalues-parser`, `keyvalues-serde`.
  - Native Windows bindings: `winreg`, `window-vibrancy`, `windows-capture`, `windows-sys`.
  - Concurrency: `tokio = { version = "1", features = ["full"] }`, `futures = "0.3"`.
- **`c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\ORIGINAL_REQUEST.md`**:
  - Mandate R5: Minimum 3 agents for deep web research to discover high-demand features and plugins missing from existing launchers.
  - Visual references: Reference 1 ("TURA" dark minimalism, Cera CY typography, slate/graphite cards) and Reference 2 (liquid chrome/silver accents, smooth glassmorphism, media cards).
  - Scope: Must transform OmniLauncher into a high-performance, flagship "Steam 2" experience.

---

### 1.2 Ecosystem Research: Top Playnite Extensions
Direct investigation of leading community extensions from the Playnite ecosystem:

#### 1. HowLongToBeat (Lacro59/playnite-howlongtobeat-plugin)
- **Observed Source**: `https://raw.githubusercontent.com/Lacro59/playnite-howlongtobeat-plugin/master/README.md`
- **Core Functionality**:
  - Looks up "Main Story", "Main + Extra", and "Completionist" average completion times from HowLongToBeat.com.
  - Automatically tags games by length (e.g. `< 5h`, `10-20h`, `50h+`).
  - Progress bar integration: visual indicator comparing actual elapsed playtime against the target completion time.
  - Global Search / QuickSearch filters: e.g. `rpg -stores=steam -time=10min<>5h`, `ttb > 2 h -np` (unplayed games).
  - Playtime upload & completion status synchronization with user HLTB profiles.
  - Alternative provider support: VNDB (Visual Novel Database) for reading time estimates.

#### 2. SuccessStory (Lacro59/playnite-successstory-plugin)
- **Observed Source**: `https://raw.githubusercontent.com/Lacro59/playnite-successstory-plugin/master/README.md`
- **Core Functionality**:
  - Aggregates and localizes achievements across multiple stores and services: Steam, GOG, Origin / EA App, Xbox Live, Epic Games, Game Jolt, Guild Wars 2, World of Warcraft, RPCS3 (PS3 emulator), and RetroAchievements.
  - Manual achievements & tracker imports for live-service Hoyoverse games (Genshin Impact, Honkai Star Rail, Zenless Zone Zero, Wuthering Waves).
  - Rarity tier calculations (common, rare, ultra-rare) and completion percentage per game.
  - In-view UI controls: achievement timeline, ribbon icons, 100% completion badge, time-to-100% estimates.

#### 3. DuplicateHider (felixkmh/DuplicateHider)
- **Observed Source**: `https://raw.githubusercontent.com/felixkmh/DuplicateHider/master/README.md`
- **Core Functionality**:
  - Addresses multi-store duplication where players own copies of the same title across Steam, Epic, GOG, Amazon, and Xbox Game Pass.
  - Priority scoring formula:
    $$\text{Score}(game) = \text{Priority}(game.\text{Source}) - \text{priorityList}.\text{Count} \times (\text{IsInstalled}(game) ? 1 : 0)$$
  - Sorts duplicates in ascending order; hides secondary copies from the main grid.
  - Embeds an interactive "Icon Stack" (`DuplicateHider_SourceSelector`) on the primary card, allowing the user to select or launch a specific storefront version with one click.
  - Context menu integration: `{Name} [{Installed}{ on 'Source}]`.

#### 4. GameActivity & PlayState (Lacro59 & darklinkpower)
- **Observed Source**: `https://raw.githubusercontent.com/Lacro59/playnite-gameactivity-plugin/master/README.md`
- **Core Functionality**:
  - Granular gameplay session recording: exact start/stop timestamps, duration, cumulative trends by day, week, month, source, and genre.
  - Hardware performance telemetry during play: average FPS, 1% low FPS, CPU/GPU usage percentages, memory utilization, temperatures, and power draw.
  - Telemetry providers: Windows Performance Counters (PDH), WMI, LibreHardwareMonitor, HWiNFO, MSI Afterburner / RivaTuner (RTSS shared memory).
  - Idle detection & pause handling: suppresses playtime inflation when games are left running unattended.
  - PlayState process suspension: leverages Win32 API to freeze running game processes (`NtSuspendProcess`), freeing CPU resources without terminating the session ("Quick Pause").

#### 5. SteamGridDB Integration
- **Observed Source**: SteamGridDB API v2 documentation (`https://www.steamgriddb.com/api/v2/`)
- **Core Functionality**:
  - High-resolution poster grids (600x900 vertical, 920x430 horizontal, 460x215).
  - Wide hero banners (1920x620, 3840x1240).
  - Transparent logos (PNG/WebP text typography overlays).
  - Custom square icons (256x256, 512x512).
  - Native support for animated covers (APNG and animated WebP), transforming static libraries into dynamic showcases.

#### 6. IsThereAnyDeal / Deals Integrations & CheapShark
- **Observed Source**: CheapShark API (`https://www.cheapshark.com/api/1.0/deals`, `/games?id={id}`), ITAD API v2
- **Core Functionality**:
  - Real-time price aggregation across 30+ authorized stores (Steam, Epic, GOG, Humble, Fanatical, GreenManGaming).
  - Historical low tracking ("All-Time Lowest Price").
  - Free Game Radars: alerts for weekly 100% free claimable titles (Epic Free Games, Steam promotional giveaways).
  - Wishlist price drop notifications.

---

### 1.3 Ecosystem Research: Top Steam Deck / Decky Loader Plugins
Direct investigation of leading community plugins from the Steam Deck / Decky Loader ecosystem:

#### 1. CSS Loader (`suchmememanyskill/CSS-Loader`)
- Injects custom CSS rules and theme bundles into the SteamOS Chromium-based CEF client at runtime.
- Allows granular skinning: rounded corners, clean navigation bars, custom font stacks, glowing card borders, and custom background blurs without modifying core system files.
- Serves as the community standard for modular theming.

#### 2. ProtonDB Badges (`omriharel/decky-protondb-badges`)
- Displays Proton compatibility tier badges ("Platinum", "Gold", "Silver", "Bronze", "Borked", "Native") directly on game library covers.
- API Endpoint: `https://www.protondb.com/api/v1/reports/summaries/{steam_app_id}.json`.
- Yields immediate clarity on whether a game works out of the box or requires specific launch parameters/tweak scripts.

#### 3. Audio Loader / Game Theme Music
- Plays game soundtrack themes when browsing or highlighting titles in the library (mirroring the PS5 console experience).
- Replaces navigation sound effects with nostalgic or high-tech sound packs (PS5, Xbox 360, Switch, Cyberpunk UI).
- Audio sources: KHInsider/VideoGameMusic, local Steam soundtrack folders, custom per-game MP3/FLAC directories.

#### 4. Metacritic & OpenCritic Scores
- Overlays verified critic ratings and recommendation percentages on game cards.
- OpenCritic API: `https://api.opencritic.com/api/game/search?criteria={query}` returning `topCriticScore`, `tier` ("Mighty", "Strong", "Fair", "Weak"), and `percentRecommended`.
- CheapShark API includes Metacritic scores directly alongside price data.

#### 5. Ludusavi (Save Game Cloud Backup by mtkennerly)
- Rust-powered cross-platform game save backup tool covering 10,000+ titles using the PCGamingWiki database.
- Provides CLI integration: `ludusavi backup --preview --api`.
- Backs up saves across Steam, Epic, GOG, Microsoft Store, and standalone emulators to local storage, NAS, or cloud storage.

---

## 2. Logic Chain

1. **The Problem of Launcher Fragmentation**:
   The modern PC gaming landscape is heavily fractured. Players own games across Steam, Epic Games, GOG, Xbox Game Pass, and emulators. Official storefront launchers are deliberately designed as walled gardens; they do not cross-reference libraries, track external playtime, or provide cross-store achievement unification.

2. **The Community Extension Revolution**:
   Gamers have turned to tools like Playnite and Decky Loader to bridge these gaps. However, relying on separate extensions leads to significant friction: plugin incompatibilities after launcher updates, memory overhead, conflicting UI hooks, and lack of visual cohesion.

3. **Strategic Opportunity for OmniLauncher ("Steam 2")**:
   OmniLauncher can leapfrog existing commercial launchers by incorporating the top 10 most demanded community power-tools directly into its native core architecture.
   Because OmniLauncher is built on Tauri v2 (Rust backend + Svelte 5 frontend), it possesses:
   - High compute efficiency: Rust can manage process monitoring, file hashing, save compression, and API caching in parallel threads with minimal RAM consumption (<50 MB idle).
   - Reactive UI performance: Svelte 5 runes (`$state`, `$derived`, `$effect`) allow instant re-renders of complex dashboards, animated artwork, and multi-source libraries with zero input lag.
   - Deep OS integration: Direct Win32 API access allows features impossible in web-only launchers (process suspension, registry scanning, hardware performance counters).

4. **Synthesis of Architectural Synergy**:
   The 10 selected flagship features reinforce each other:
   - *DuplicateHider* merges multi-store libraries $\rightarrow$ *SuccessStory* combines their achievements $\rightarrow$ *HowLongToBeat* contextualizes remaining playtime $\rightarrow$ *Ludusavi* secures save files across all variants.
   - *SteamGridDB* supplies 4K artwork and animated covers $\rightarrow$ *CSS Loader Theme Engine* formats the layout in TURA or Liquid Chrome style $\rightarrow$ *Game Theme Music* supplies acoustic ambiance on hover.
   - *ProtonDB & PCGamingWiki Badges* show tech flags $\rightarrow$ *GameActivity & Telemetry* records real performance in-game $\rightarrow$ *Deals & Free Radar* powers the discovery shop.

---

## 3. Caveats & Assumptions

1. **Third-Party API Rate Limits**:
   Public endpoints (SteamGridDB, ProtonDB, CheapShark, OpenCritic) enforce rate limits. OmniLauncher must implement an SQLite/JSON local caching layer with explicit Time-To-Live (TTL) policies (7 days for compatibility/ratings; 30 days for artwork metadata and HLTB times) to prevent IP throttling.
2. **Reverse-Engineered Endpoints**:
   HowLongToBeat relies on an internal JSON search API that periodically updates search hash keys. OmniLauncher should maintain a scraper fallback or secondary metadata provider (such as RAWG or IGDB) to guarantee continuity.
3. **Elevated Process Handling**:
   Process suspension (`NtSuspendProcess`) and hardware telemetry reading require appropriate Windows process privileges. For games running with administrative privileges or protected by kernel anti-cheats (e.g. Riot Vanguard, BattlEye), suspension may be restricted; OmniLauncher must handle errors gracefully without crashing.
4. **Offline Resilience**:
   All features must operate gracefully in offline mode, serving cached covers, playtime logs, and compatibility data when an internet connection is unavailable.

---

## 4. Conclusion & Flagship Feature Specifications

The research concludes that OmniLauncher must integrate **10 flagship features** modeled directly on high-demand community extensions:

```
┌────────────────────────────────────────────────────────────────────────┐
│                   OmniLauncher Flagship Feature Suite                   │
├────────────────────────────┬───────────────────────────────────────────┤
│ 1. Unified Achievements    │ Cross-platform trophy matrix (SuccessStory)│
│ 2. HowLongToBeat Tracker   │ Duration estimates & backlog tags (HLTB)  │
│ 3. SteamGridDB Engine      │ 4K banners, logos & animated covers (SGDB)│
│ 4. Library Deduplication   │ Multi-store grouping & selector (DupHider)│
│ 5. Session & HW Telemetry  │ FPS/temp stats & Quick Pause (GameActivity)│
│ 6. Modular Theme Engine    │ TURA & Liquid Chrome live skins (CSSLoader│
│ 7. Compatibility Badges    │ ProtonDB & PCGamingWiki tech pills        │
│ 8. Dynamic Theme Music     │ Contextual soundtrack ambiance on hover   │
│ 9. Save Cloud Vault        │ Cross-store backup & restore (Ludusavi)   │
│ 10. Deals & Free Radar     │ Best prices, all-time lows & free alerts  │
└────────────────────────────┴───────────────────────────────────────────┘
```

Detailed architectural blueprints for each feature are provided below:

---

### Feature 1: Unified Cross-Platform Achievements Matrix & Showcase
- **Origin Ecosystem**: SuccessStory (Playnite), Steam Achievements, RetroAchievements.
- **Detailed Description**:
  Aggregates unlocked and locked achievements across all connected accounts (Steam, Epic Games, GOG, Xbox Game Pass, and RetroAchievements). Provides a unified gamer profile displaying:
  - Total Gamer Score & Global Completion Percentage.
  - Rarity Tiers: Common (>50%), Uncommon (20–50%), Rare (5–20%), Ultra-Rare (<5%) with metallic foil borders.
  - Chronological Unlock Timeline showing exact unlock timestamps.
  - Profile "Trophy Showcase": allows pinning up to 5 favorite or ultra-rare achievements to the user profile header.
  - Secret Achievement Toggle: allows revealing hidden story spoilers on demand.
- **APIs & Data Sources**:
  - Steam Web API: `ISteamUserStats/GetPlayerAchievements/v0001` and `GetSchemaForGame/v2`.
  - Epic Games Store: GraphQL achievements endpoint via OAuth token.
  - RetroAchievements: `https://retroachievements.org/API/API_GetGameInfoAndUserProgress.php`.
- **Architectural Design in OmniLauncher**:
  - Rust Backend: `src-tauri/src/services/achievements.rs` implements trait `AchievementProvider` for each store. An asynchronous background task polls for updates on game exit.
  - Data Persistence: SQLite table `achievements` storing `(game_id, achievement_id, title, description, icon_url, unlocked, unlock_time, rarity_pct)`.
  - Svelte Frontend: Component `<AchievementShowcase />` rendered in the game details view and profile page, styled with crisp typography and subtle metallic badges.

---

### Feature 2: HowLongToBeat (HLTB) Estimator & Backlog Planner
- **Origin Ecosystem**: HowLongToBeat plugin by Lacro59, HLTB for Decky Loader.
- **Detailed Description**:
  Embeds verified completion times directly onto game library cards and game details headers:
  - Three standardized metrics: **Main Story**, **Main + Extras**, and **Completionist**.
  - Dynamic Progress Gauge: compares user's actual recorded playtime against the target completion time (e.g. "18.5h / 24.0h — 77% Story Completed").
  - Backlog Status Tags: allows tagging games as *Backlog*, *In Progress*, *Beaten*, *Abandoned*, or *Endless*.
  - Smart Library Sorting: sort library by "Shortest to Beat" or "Time Left to Complete".
- **APIs & Data Sources**:
  - HowLongToBeat search API: POST to `https://howlongtobeat.com/api/search/{key}` with JSON payload `{"searchTerms": [game_title], "searchType": "games"}`.
  - Fallback: Local lookup cache and RAWG API metadata.
- **Architectural Design in OmniLauncher**:
  - Rust Backend: `src-tauri/src/services/hltb.rs` manages search query normalization, caching results in `hltb_cache.json` with a 30-day TTL.
  - Svelte Frontend: Compact `<HltbBadge />` component on game cards showing `[ ⏱️ 12h Story ]`, and an interactive `<HltbProgressCard />` in the game details view.

---

### Feature 3: SteamGridDB High-Resolution Artwork & Animated Cover Engine
- **Origin Ecosystem**: SteamGridDB Playnite extension, Decky SGDB plugin.
- **Detailed Description**:
  Provides a native asset management engine to replace missing or low-resolution artwork with premium community assets:
  - Vertical poster covers (600x900 standard).
  - Wide cinematic hero banners (1920x620).
  - Transparent logos (PNG/WebP typography overlays positioned over the banner).
  - Square icons (256x256).
  - **Animated Covers Support**: Native playback of animated WebP and APNG covers on hover or game focus, bringing the library to life.
  - Interactive In-Launcher Art Picker: right-click game $\rightarrow$ "Change Artwork" opens a visual modal browsing community submissions with 1-click apply.
- **APIs & Data Sources**:
  - SteamGridDB REST API v2 (`https://www.steamgriddb.com/api/v2/`):
    - Search: `GET /search/autocomplete/{query}`
    - By Steam ID: `GET /games/steam/{app_id}`
    - Grids: `GET /grids/game/{game_id}?dimensions=600x900&types=static,animated`
    - Heroes: `GET /heroes/game/{game_id}`
    - Logos: `GET /logos/game/{game_id}`
    - Icons: `GET /icons/game/{game_id}`
- **Architectural Design in OmniLauncher**:
  - Rust Backend: `src-tauri/src/services/steamgriddb.rs` handles asset downloading, caching to `%APPDATA%/OmniLauncher/artwork_cache/{game_id}/`, and generating blurred thumbnail placeholders.
  - Svelte Frontend: `<ArtworkPickerModal />` with tabs for Grid, Hero, Logo, and Icon; `<AnimatedCover />` component utilizing CSS hardware acceleration and image lazy-loading.

---

### Feature 4: Multi-Store Library Deduplication & Unified Game Hub (DuplicateHider)
- **Origin Ecosystem**: DuplicateHider by felixkmh.
- **Detailed Description**:
  Automatically identifies and collapses multiple ownership records of the same game across Steam, Epic Games, GOG, Amazon, and Xbox Game Pass into a single unified card in the library.
  - Priority Scoring: automatically chooses the primary display version based on user rules (e.g. *Installed First* > *Steam* > *GOG* > *Epic*).
  - Storefront Pill Selector: the game card and overview display discreet store icons (Steam, Epic, GOG). Clicking any icon switches the active version; clicking "Play" launches the installed version directly.
  - Library Decluttering: eliminates double and triple entries while retaining individual store achievements and play records.
- **APIs & Integration Mechanics**:
  - Title normalization algorithm in Rust: removes editions ("Game of the Year", "Definitive Edition", "Director's Cut"), punctuation, and roman numerals (`IV` $\leftrightarrow$ `4`).
  - Cross-reference mapping via Steam AppID and IGDB slugs.
- **Architectural Design in OmniLauncher**:
  - Rust Backend: `src-tauri/src/services/deduplication.rs` runs on library sync, generating a unified `master_game_id` linking constituent store entries in the database.
  - Svelte Frontend: `<StorefrontSelector />` component embedded directly on the game hero banner, letting users toggle active copies effortlessly.

---

### Feature 5: Hardware Telemetry & Deep Session Playtime Analytics (GameActivity / PlayState)
- **Origin Ecosystem**: GameActivity by Lacro59 & PlayState by darklinkpower.
- **Detailed Description**:
  Provides in-depth analytics and system monitoring for every gaming session:
  - Session Logging: tracks exact session start time, duration, and date.
  - Activity Heatmap: displays a GitHub-style annual activity matrix showing days and hours played.
  - In-Game Hardware Telemetry: captures average FPS, 1% lows, GPU temperature, CPU utilization, and RAM usage throughout the session without needing an external overlay.
  - Smart Idle Detection: detects inactivity (>10 minutes of zero mouse/keyboard/gamepad input) or window defocus, pausing the playtime counter to prevent stat inflation.
  - Win32 Process Suspension ("Quick Pause"): global hotkey (e.g. `Ctrl+Alt+P`) freezes the game process (`NtSuspendProcess`), dropping CPU/GPU usage to 0% so the player can step away without losing progress or burning power.
- **APIs & Integration Mechanics**:
  - Win32 API: `OpenProcess`, `NtSuspendProcess`, `NtResumeProcess` via `windows-sys`.
  - Telemetry: Windows Performance Data Helper (`pdh.dll`) or RTSS shared memory interface.
- **Architectural Design in OmniLauncher**:
  - Rust Backend: `src-tauri/src/services/telemetry.rs` spawns a dedicated worker thread monitoring the active game PID every 5 seconds.
  - Svelte Frontend: `<PlaytimeHeatmap />` and `<SessionHistoryChart />` rendered with lightweight SVGs, matching the dark minimalist dashboard theme.

---

### Feature 6: Modular Theme Engine & Liquid Accent Customizer (CSS Loader Model)
- **Origin Ecosystem**: CSS Loader (Steam Deck / Decky Loader).
- **Detailed Description**:
  A modular theming architecture allowing live styling changes with zero application reloads.
  - Built-In Flagship Themes:
    - **Theme A ("TURA Graphite")**: Reference 1 aesthetic. Deep graphite/onyx surfaces (`#0d0e12`), Cera CY typography, subtle 1px border lines (`#1f242d`), ultra-minimal card margins.
    - **Theme B ("Liquid Chrome")**: Reference 2 aesthetic. Cyber-metallic dark palette, liquid chrome/brushed silver gradient accents, translucent glassmorphism (`backdrop-filter: blur(16px)`), dynamic glows.
  - Theme Customizer Controls: user sliders for accent color, corner radius (from sharp 0px to smooth 16px), background blur intensity, and card layout density.
  - Community Theme Import: supports importing `.theme.json` + `theme.css` packages.
- **APIs & Integration Mechanics**:
  - CSS Custom Properties (`--bg-primary`, `--accent-chrome`, `--border-subtle`, `--card-radius`).
  - Dynamic stylesheet injection via Tauri DOM access.
- **Architectural Design in OmniLauncher**:
  - Rust Backend: `src-tauri/src/services/theme_manager.rs` discovers themes in `%APPDATA%/OmniLauncher/themes/` and handles file validation.
  - Svelte Frontend: Svelte 5 reactive store `themeStore` dynamically updates `:root` style declarations upon selection.

---

### Feature 7: ProtonDB & PCGamingWiki Compatibility & Feature Flags Badges
- **Origin Ecosystem**: ProtonDB Badges (Decky Loader), PCGamingWiki extension.
- **Detailed Description**:
  Equips each game page with instant technical capability badges before the player hits Launch:
  - **ProtonDB Badge**: Platinum / Gold / Silver / Bronze / Borked / Native rating (critical for handhelds, ROG Ally, Steam Deck, and Windows Linux subsystem users).
  - **PCGamingWiki Tech Flags**:
    - Aspect Ratio: Ultrawide 21:9 and 32:9 support status.
    - Refresh Rate: Uncapped FPS / >120Hz support.
    - Controller Support: DualSense haptics, Xbox controller, Gyro aim.
    - Video Features: HDR10, Ray Tracing, DLSS / FSR / XeSS.
    - FOV Slider: Supported / Not Supported.
  - Community Fixes Drawer: clicking any badge slides out recommended launch arguments, configuration file tweaks, and crash fixes.
- **APIs & Data Sources**:
  - ProtonDB API: `https://www.protondb.com/api/v1/reports/summaries/{steam_app_id}.json`.
  - PCGamingWiki API: MediaWiki API `https://www.pcgamingwiki.com/w/api.php?action=cargoquery&tables=Infobox_game,Video`.
- **Architectural Design in OmniLauncher**:
  - Rust Backend: `src-tauri/src/services/compatibility.rs` queries and caches technical flags for each detected AppID.
  - Svelte Frontend: `<TechBadgesRow />` positioned below the game title in the game details view and properties dialog.

---

### Feature 8: Dynamic Game Theme Music & Audio Ambiance Engine
- **Origin Ecosystem**: Game Theme Music (Playnite by Lacro59), Audio Loader (Decky Loader).
- **Detailed Description**:
  Elevates the launcher into a premium cinematic console experience.
  - Soundtrack Ambiance: highlighting or opening a game smoothly fades in the game's official theme music in the background.
  - Smooth Crossfading: 400ms exponential audio fade-in/fade-out when navigating between games to avoid sudden audio cuts.
  - Auto-Ducking: automatically silences or ducks launcher music when a video trailer plays or when a game executable starts.
  - Custom UI Audio Schemes: customizable navigation sound effects (subtle clicks, hover hums, launch chimes) modeled after modern console UIs.
- **APIs & Data Sources**:
  - Local Steam Soundtrack directory scanner (`steamapps/common/.../Soundtrack`).
  - Online stream fallback: VideoGameMusic (KHInsider API) or user-provided MP3/FLAC/OGG files stored in `%APPDATA%/OmniLauncher/audio/{game_id}/theme.mp3`.
- **Architectural Design in OmniLauncher**:
  - Svelte Frontend: Audio service `src/lib/services/audioAmbiance.ts` using Web Audio API (`GainNode`, `AudioContext`) for crossfading.
  - Rust Backend: Audio scanner finding audio assets in local game directories.

---

### Feature 9: Unified Cross-Platform Save Game Backup & Cloud Vault (Ludusavi Integration)
- **Origin Ecosystem**: Ludusavi by mtkennerly.
- **Detailed Description**:
  Comprehensive, automated save game protection for all games—especially vital for storefronts lacking robust cloud saves (Epic Games, non-Steam games, standalone emulators):
  - Auto-Backup on Exit: takes an incremental snapshot of save files immediately when a game session terminates.
  - Versioned Restore Points: allows rolling back saves to previous points in time (protects against corrupted saves or accidental overwrites).
  - Differential Compression: uses Zstandard (`zstd`) compression to minimize disk footprint.
  - Cloud Sync Support: syncs backups to a user-specified cloud storage folder (Google Drive, OneDrive, Nextcloud, or local NAS).
- **APIs & Integration Mechanics**:
  - Integration of Ludusavi database manifests (`manifest.yaml`) mapping 10,000+ PC games to save file paths across Windows registries, `%APPDATA%`, `%LOCALAPPDATA%`, and `Documents/My Games`.
- **Architectural Design in OmniLauncher**:
  - Rust Backend: `src-tauri/src/services/save_vault.rs` performs path resolution, file copying, and compression asynchronously.
  - Svelte Frontend: `<SaveVaultManager />` accessible via the Game Properties modal ("Saves" tab), displaying snapshots with dates, sizes, and 1-click restore.

---

### Feature 10: Live Deals, Historical Price Lows & Free Games Radar (ITAD & CheapShark)
- **Origin Ecosystem**: IsThereAnyDeal (ITAD) Playnite extension, CheapShark.
- **Detailed Description**:
  Turns the Store and Recommendations feed into a high-value shopping intelligence hub:
  - Real-Time Price Comparison: compares current prices across 30+ official digital stores (Steam, Epic, GOG, Humble, Fanatical).
  - "All-Time Low" Badge: informs players if a sale is the best price ever recorded.
  - Currency Conversion (R3 compliance): live currency toggle between USD ($) and Kazakhstani Tenge (₸ / KZT) with real-time conversion rates.
  - Free Games Radar: banner at the top of the Store feed highlighting currently free claimable games (e.g. Epic weekly giveaways, Steam free weekends, GOG promotions) with direct claim links.
- **APIs & Data Sources**:
  - CheapShark Public API:
    - Current Deals: `https://www.cheapshark.com/api/1.0/deals?storeID=1,2,7&pageSize=30`
    - Game Lookup: `https://www.cheapshark.com/api/1.0/games?title={title}`
  - Exchange Rate API / Cached Forex for USD $\rightarrow$ KZT conversion.
- **Architectural Design in OmniLauncher**:
  - Rust Backend: `src-tauri/src/services/deals.rs` polls deals periodically and caches JSON responses; provides IPC command `get_store_recommendations(currency)`.
  - Svelte Frontend: `<DealsFeed />` with discount percentage pills (`-75%`), all-time low badges, and currency selector.

---

## 5. Verification Method

To independently verify the facts, APIs, and recommendations in this research report:

1. **Verify Project Stack & Dependencies**:
   Inspect `package.json` and `src-tauri/Cargo.toml`:
   - Check Svelte 5, Tailwind v4, Vite 6 in `package.json`.
   - Check Tauri 2, reqwest, tokio, windows-sys in `src-tauri/Cargo.toml`.
2. **Verify Public API Availability**:
   Execute curl/HTTP checks against the researched public endpoints:
   ```bash
   # ProtonDB API test (e.g. Cyberpunk 2077 AppID 1091500)
   curl -s "https://www.protondb.com/api/v1/reports/summaries/1091500.json" | grep "tier"

   # CheapShark Deals API test (free endpoint, no key required)
   curl -s "https://www.cheapshark.com/api/1.0/deals?storeID=1&pageSize=3"

   # OpenCritic Game Search API test
   curl -s "https://api.opencritic.com/api/game/search?criteria=Elden%20Ring"
   ```
3. **Verify Playnite & Decky Source Code References**:
   - Inspect `Lacro59/playnite-howlongtobeat-plugin` repository.
   - Inspect `Lacro59/playnite-successstory-plugin` repository.
   - Inspect `felixkmh/DuplicateHider` repository.
   - Inspect `mtkennerly/ludusavi` repository.
4. **Invalidation Conditions**:
   - If a storefront or API revokes public access, the fallback architecture (local caching + secondary provider) outlined in Section 4 must be verified to maintain system stability.
