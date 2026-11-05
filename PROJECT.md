# Project: OmniLauncher ("Steam 2") Flagship Overhaul

## Architecture Overview
OmniLauncher is a high-performance, next-generation desktop gaming launcher built with:
- **Frontend**: Svelte 5 (Runes architecture: `$state`, `$derived`, `$effect`, `$props`), Vite 6, Tailwind CSS v4, TypeScript 5.7.
- **Backend**: Tauri v2 in Rust, Tokio async runtime, `reqwest` with rustls, `windows-sys` Win32 API, `keyvalues-parser` for Valve VDF, `discord-rich-presence`, `dirs`.
- **Styling Architecture**: Modular theme engine driven by CSS Custom Properties, JSON theme descriptors, and two core visual languages:
  - **TURA Style**: Matte deep graphite (`#09090b`), Cera CY grotesque typography, ultra-clean cards, zero redundant explanatory text.
  - **Liquid Chrome Style**: Specular chrome ribbons, metallic badges, fluid glass panels.

```
┌────────────────────────────────────────────────────────────────────────┐
│                        OmniLauncher Frontend (Svelte 5)                │
├──────────────┬──────────────┬──────────────┬─────────────┬─────────────┤
│ Library View │  Store Feed  │  Emulators   │  Downloads  │  Settings   │
│ (TURA Cards, │ (Infinite,   │  (GitHub     │  (Drive/Dir │  (DNS, DoH, │
│ Dedupe, HLTB)│  USD / KZT)  │   Catalog)   │   Picker)   │   Proxy)    │
├──────────────┴──────────────┴──────────────┴─────────────┴─────────────┤
│  Modular Theme Engine (JSON schema, CSS vars, runtime hot reload)      │
│  Steam Properties Modal (Launch args, Files, Cloud, Achievements, Compat)│
│  Flagship Features: Save Vault, Roulette, Shader Reclaimer, PCGW Tweak │
└────────────────────────────────────────────────────────────────────────┘
                                   │  Tauri IPC Commands
┌──────────────────────────────────▼─────────────────────────────────────┐
│                         Rust Backend (src-tauri)                       │
├────────────────────────────────────────────────────────────────────────┤
│ • Windowing & Lifecycle: Clean GUI launch (#![windows_subsystem])      │
│ • Zero-Footprint Game Mode: Process priority drop & memory trimming    │
│ • Platform Parsers: Steam VDF/Cloud, Epic Manifests, Roblox Logs       │
│ • Emulators Engine: GitHub Releases API, stream downloader, zip extract │
│ • Network & CDN: Cloudflare media proxy, DoH (1.1.1.1, 8.8.8.8), SOCKS5│
│ • Local Storage & Vault: SQLite / JSON cache, save-game backup engine  │
└────────────────────────────────────────────────────────────────────────┘
```

## Feature Inventory
Every feature identified during the Survey and 3-agent Deep Market Research is inventoried below and mapped to a specific milestone.

| # | Feature | Description | Milestone | Source |
|---|---------|-------------|-----------|--------|
| 1 | TURA Minimalist UI Redesign | Deep graphite palette (`#09090b`), clean typography, elegant cards, borderless frameless window | M1 | R1, Ref 1 |
| 2 | Liquid Chrome Visual Accents | Specular chrome ribbons, metallic gradients, liquid badges, fluid media blocks | M1 | R1, Ref 2 |
| 3 | Legacy Clutter Elimination | Strip casual 3D skeuomorphic styling (`btn-game-*`, thick push borders, redundant labels) | M1 | R1, Survey |
| 4 | Modular Theme System | JSON theme manifests, CSS variables, drop-in `themes/` folder, runtime hot reload | M1 | R1, SpecMiner |
| 5 | Steam Account & Profile Link | SteamID64 resolution, profile web navigation, avatar display | M2 | R2, Survey |
| 6 | Steam Achievements & Desync Repair | Achievements grid, unlocking status, Web API key fallback & local stats repair | M2 | R2, Agent 3 |
| 7 | Steam Cloud Sync & Versioning | Cloud status detection (`remotecache.vdf`), Auto-Cloud path sync, pre-launch snapshot | M2 | R2, Agent 3 |
| 8 | Steam-like Game Properties Modal | 5-tab modal: General (launch args), Installed files, Cloud saves, Achievements, Compat | M2 | R2, SpecMiner |
| 9 | Epic Games Uninstalled Library Import | Scans `.item` manifests + uninstalled catalog import to display complete Epic library | M2 | R2, Agent 3 |
| 10 | Roblox Recently Played Games | Multi-log parsing in `%LOCALAPPDATA%\Roblox\logs` + high-res thumbnails API | M2 | R2, Agent 3 |
| 11 | Multi-Store Game Deduplication | DuplicateHider-style consolidation of games owned across Steam/Epic/GOG into 1 card | M2 | R5, Agent 1 & 2 |
| 12 | Steam Store Infinite Recommendations | Dynamic infinite scroll feed with recommendations, tag filters, and game previews | M3 | R3, SpecMiner |
| 13 | Currency Switcher (USD / KZT) | Real-time currency toggle between USD ($) and Kazakhstani Tenge (₸) with price conversion | M3 | R3, Survey |
| 14 | In-Launcher Drive & Directory Selector | Native custom drive space visualizer and folder browser without opening Steam OS dialog | M3 | R3, Agent 3 |
| 15 | GitHub Open-Source Emulators Catalog | Curated catalog (RetroArch, Dolphin, PCSX2, RPCS3, PPSSPP, DuckStation, Ryujinx, etc.) | M3 | R3, Agent 3 |
| 16 | 1-Click Emulator Downloader & Extractor | Direct GitHub release binary download, checksum verify, auto-extract, and library addition | M3 | R3, Agent 3 |
| 17 | Cloudflare CDN Media Routing | Route game banners/covers through Cloudflare CDN caching & anti-hotlinking proxy | M3 | R3, Agent 3 |
| 18 | Network Settings (DNS & Proxy) | Choice of Cloudflare 1.1.1.1, Google 8.8.8.8, Quad9, custom DoH, and SOCKS5/HTTP proxy | M3 | R3, Agent 3 |
| 19 | Zero-Footprint Game Mode | Process hibernation, RAM flush (`SetProcessWorkingSetSize`), 0% CPU idle during gameplay | M4 | R5, Agent 1 |
| 20 | Save-State Time Machine | Automated pre/post game session zip backups, timeline restore modal (Ludusavi model) | M4 | R5, Agent 1 & 2 |
| 21 | HowLongToBeat (HLTB) Forecasting | Estimated completion hours (Story, Extra, 100%) and dynamic progress bar on game cards | M4 | R5, Agent 1 & 2 |
| 22 | Intelligent Backlog Roulette | Constraint-based game picker (playtime, genre, controller support, completion status) | M4 | R5, Agent 1 |
| 23 | Shader Cache & Disk Reclaimer | Scans and deletes orphaned DXVK/Vulkan shader caches and compatdata prefixes | M4 | R5, Agent 1 |
| 24 | PCGamingWiki 1-Click Tweak Engine | Displays ultrawide, FPS uncap, FOV info + 1-click config file fixes | M4 | R5, Agent 1 & 2 |
| 25 | Session Diary & Hardware Telemetry | Tracks play sessions, logs playtime history, and provides hardware health insights | M4 | R5, Agent 1 & 2 |
| 26 | Universal Discord Rich Presence | Live Discord RPC with high-res artwork, game title, and elapsed time for all platforms | M4 | R5, Agent 1 |
| 27 | DealsRadar & Historical Low Price | Cross-store deal comparisons and all-time low discount indicator | M4 | R5, Agent 1 & 2 |
| 28 | Proactive Offline Vault | Resilient token & metadata caching, 1-click launch during complete offline state | M4 | R5, Agent 1 |
| 29 | Project File Structure Reorganization | Clean root directory, move scripts and docs to appropriate subfolders (`scripts/`, `docs/`) | M5 | R4, Survey |
| 30 | Clean Launcher GUI Startup | Apply `#![windows_subsystem = "windows"]`, compile release binary, eliminate CMD popups | M5 | R4, Survey |
| 31 | Full E2E Test Suite (Tiers 1-4) | Comprehensive opaque-box test runner validating all features against acceptance criteria | M6 | Acceptance |
| 32 | Adversarial Hardening & Forensic Audit | White-box stress testing (Tier 5) and mandatory Forensic Integrity Audit | M6 | Acceptance |

## Milestones Summary

| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| M1 | UI Redesign & Modular Theme Engine | Features 1-4: TURA & Liquid Chrome styling, clutter removal, theme system, frameless titlebar | none | DONE |
| M2 | Platform Integrations & Library Hub | Features 5-11: Steam profile/achievements/cloud, properties modal, Epic uninstalled, Roblox, Deduplication | M1 | DONE |
| M3 | Store, Emulators & Network Infrastructure | Features 12-18: Infinite store, USD/KZT currency, internal folder picker, GitHub emulators, Cloudflare/DNS/proxy | M1 | DONE |
| M4 | Flagship Modern Launcher Features | Features 19-28 + Community Additions (Sentiment badges, Walkthrough notes, Smart filter pills, Direct folder openers, Eco mode, Cover art replacer, Storage breakdown) | M1-M3 | DONE |
| M5 | File Structure Reorganization & Clean Launch | Features 29-30: Project root cleanup, Windows GUI subsystem flags, release binary packaging, no CMD popups | M1-M4 | DONE |
| M6 | E2E Test Pass & Adversarial Hardening | Features 31-32: 100% E2E test verification, Tier 5 hardening, Forensic Integrity Audit | M1-M5, Test Track | DONE |

## Interface Contracts

### 1. Theme Engine Contract (`src/lib/themes/`)
- Descriptors: `ThemeDefinition { id: string, name: string, author: string, version: string, colors: Record<string, string>, typography: TypographyConfig }`
- CSS Variables: `--bg-canvas`, `--bg-surface`, `--bg-card`, `--border-subtle`, `--accent-chrome`, `--accent-glow`, `--text-primary`, `--text-muted`.
- Method: `applyTheme(themeId: string): void`, `getAvailableThemes(): ThemeDefinition[]`.

### 2. Platform Integration Contract (`src-tauri/src/commands/platforms.rs`)
- Commands:
  - `get_steam_profile() -> Result<SteamProfile, String>`
  - `get_steam_achievements(app_id: u32) -> Result<Vec<SteamAchievement>, String>`
  - `sync_steam_cloud(app_id: u32) -> Result<CloudSyncResult, String>`
  - `get_epic_library(include_uninstalled: bool) -> Result<Vec<EpicGame>, String>`
  - `get_roblox_recent_places() -> Result<Vec<RobloxPlace>, String>`
  - `get_unified_library() -> Result<Vec<UnifiedGame>, String>`

### 3. Store, Emulators & Network Contract (`src-tauri/src/commands/`)
- Commands:
  - `get_store_recommendations(page: usize, currency: String) -> Result<StorePage, String>`
  - `browse_in_app_directory(initial_path: Option<String>) -> Result<DirectoryListing, String>`
  - `get_emulator_catalog() -> Result<Vec<EmulatorEntry>, String>`
  - `install_emulator(emulator_id: String, target_dir: String) -> Result<EmulatorInstallResult, String>`
  - `update_network_settings(dns_provider: String, proxy_url: Option<String>) -> Result<(), String>`

### 4. Flagship Features Contract (`src-tauri/src/commands/`)
- Commands:
  - `enter_game_mode(process_id: u32) -> Result<(), String>`
  - `exit_game_mode() -> Result<(), String>`
  - `create_save_snapshot(game_id: String) -> Result<SaveSnapshot, String>`
  - `list_save_snapshots(game_id: String) -> Result<Vec<SaveSnapshot>, String>`
  - `restore_save_snapshot(snapshot_id: String) -> Result<(), String>`
  - `get_hltb_data(game_title: String) -> Result<HltbRecord, String>`
  - `scan_shader_caches() -> Result<ShaderCacheSummary, String>`
  - `clean_shader_caches(app_ids: Vec<u32>) -> Result<u64, String>`
  - `get_pcgamingwiki_data(game_title: String) -> Result<PcgwRecord, String>`

## Code Layout
```
c:\Users\Tmub\Documents\TMlauncher\
├── src/                               # Svelte 5 frontend
│   ├── lib/
│   │   ├── components/                # Reusable UI widgets
│   │   │   ├── TitleBar.svelte        # Sleek frameless titlebar
│   │   │   ├── Navigation.svelte      # Minimalist tabs
│   │   │   ├── GameCard.svelte        # TURA-style 2:3 cards + badges
│   │   │   ├── GameDetailsModal.svelte # Details + HLTB + PCGW
│   │   │   ├── GamePropertiesModal.svelte # Steam-like properties
│   │   │   ├── RouletteModal.svelte   # Backlog spinner
│   │   │   ├── SaveVaultModal.svelte  # Save Time Machine
│   │   │   ├── StorageManagerModal.svelte # Shader reclaimer
│   │   │   └── DirectoryPickerModal.svelte # In-launcher folder selector
│   │   ├── views/                     # Main view pages
│   │   │   ├── LibraryView.svelte
│   │   │   ├── StoreView.svelte       # Infinite feed + USD/KZT
│   │   │   ├── EmulatorsView.svelte   # GitHub catalog
│   │   │   ├── DownloadsView.svelte   # Drive manager
│   │   │   └── SettingsView.svelte    # DNS, DoH, Proxy, Themes
│   │   ├── themes/                    # Modular theme system
│   │   │   ├── theme-engine.ts        # Dynamic theme loader & watcher
│   │   │   ├── tura-graphite.json
│   │   │   ├── liquid-chrome.json
│   │   │   └── obsidian.json
│   │   ├── stores/                    # Svelte 5 runes state stores
│   │   │   ├── library.svelte.ts
│   │   │   ├── theme.svelte.ts
│   │   │   ├── store.svelte.ts
│   │   │   └── network.svelte.ts
│   │   └── styles/
│   │       ├── app.css                # Base & theme CSS variables
│   │       └── chrome.css             # Liquid chrome metallic shaders
│   ├── App.svelte                     # Main root application shell
│   └── main.ts
├── src-tauri/                         # Rust Tauri v2 backend
│   ├── src/
│   │   ├── main.rs                    # GUI entrypoint (#![windows_subsystem = "windows"])
│   │   ├── lib.rs                     # Tauri plugin initialization & command registration
│   │   ├── core/                      # Core runtime modules
│   │   │   ├── runner.rs              # Process execution & Game Mode
│   │   │   ├── discord_rpc.rs         # Universal Discord presence
│   │   │   └── storage.rs             # Local database & config
│   │   ├── commands/                  # Tauri IPC commands
│   │   │   ├── library.rs
│   │   │   ├── platforms.rs
│   │   │   ├── emulators.rs
│   │   │   ├── store.rs
│   │   │   ├── network.rs
│   │   │   ├── save_vault.rs
│   │   │   ├── hltb.rs
│   │   │   └── pcgw.rs
│   │   └── platforms/                 # Platform providers
│   │       ├── steam/
│   │       ├── epic/
│   │       └── roblox/
│   ├── Cargo.toml
│   └── tauri.conf.json
├── tests/                             # E2E Test Suite (Tiers 1-4)
│   ├── runner.cjs                     # Opaque-box test runner
│   ├── tier1_features/
│   ├── tier2_boundaries/
│   ├── tier3_interactions/
│   └── tier4_scenarios/
├── themes/                            # User drop-in theme directory
├── scripts/                           # Reorganized build & maintenance scripts
└── docs/                              # Project documentation
```
