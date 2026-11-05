# Handoff Report: OmniLauncher Codebase Architecture & Technical Survey

- **Date**: 2026-11-04T22:58:00Z
- **Author**: Codebase Architecture Explorer (`teamwork_explorer_survey_1`)
- **Recipient**: Orchestrator (`75c17a26-5c14-4320-bf9e-bf30909202d5`)
- **Target Repository**: `c:\Users\Tmub\Documents\TMlauncher`
- **Handoff Type**: Hard (Task Complete)

---

## 1. Observation

### 1.1 Project Structure & File Layout
Direct inspection of `c:\Users\Tmub\Documents\TMlauncher` revealed the following directory layout:
```text
c:\Users\Tmub\Documents\TMlauncher
├── .agents/                      # Teamwork metadata & agent memory folders
├── OmniLauncher.exe              # 30,410,752 bytes — debug executable placed directly in root
├── Start-OmniLauncher.cmd        # 308 bytes — launch batch script
├── dist/                         # Vite frontend production build output
│   ├── assets/
│   │   ├── index-Dxoa1kCi.js     # 140.38 kB (gzip 44.76 kB)
│   │   └── index-Xddxq_gj.css    # 58.36 kB (gzip 10.13 kB)
│   └── index.html                # 0.45 kB
├── index.html                    # 351 bytes — root HTML template with data-theme="steam-material-noir"
├── node_modules/                 # Node.js dependencies
├── package.json                  # 510 bytes — package manifest
├── package-lock.json             # 76,566 bytes
├── tsconfig.json                 # 359 bytes — TypeScript compiler options
├── vite.config.ts                # 476 bytes — Vite + Svelte 5 + Tailwind v4 config
├── themes/
│   └── monochrome-material/
│       └── theme.json            # 813 bytes — Steam Material Noir theme tokens
├── src/                          # Svelte 5 frontend source code
│   ├── App.svelte                # 135 bytes — root Svelte component rendering <BentoHub />
│   ├── main.ts                   # 261 bytes — app mount & WebComponent widget host init
│   └── lib/
│       ├── components/           # 12 Svelte 5 UI components
│       ├── stores/               # 3 TypeScript store controllers (library, theme, friends)
│       ├── styles/
│       │   └── app.css           # 5,498 bytes — Tailwind v4 CSS + 3D casual game styles
│       └── widgets/
│           └── widget-host.ts    # 4,503 bytes — Shadow DOM <omni-widget-card> & iframe host
└── src-tauri/                    # Rust Tauri v2 desktop backend
    ├── Cargo.lock                # 132,951 bytes
    ├── Cargo.toml                # 2,459 bytes — Rust dependencies & release profile
    ├── build.rs                  # 39 bytes — tauri_build::build()
    ├── tauri.conf.json           # 1,300 bytes — Tauri v2 desktop window & bundle configuration
    ├── capabilities/
    │   └── default.json          # 205 bytes — Tauri permission capability
    ├── icons/                    # App icons (.ico, .png)
    ├── target/                   # Rust build outputs (debug & release artifacts)
    └── src/
        ├── main.rs               # 182 bytes — Windows subsystem configuration & entry point
        ├── lib.rs                # 7,424 bytes — Tauri app builder, services, and 30+ IPC commands
        ├── error.rs              # 4,165 bytes — LauncherError enum with thiserror
        ├── commands/             # IPC command handlers
        │   ├── deals.rs          # 18,922 bytes — Steam store, Epic freebies, CheapShark
        │   ├── library.rs        # 3,321 bytes — Library scan, launch, install, Steam stats
        │   ├── media.rs          # 1,706 bytes — Screenshots & video clip records
        │   ├── roblox.rs         # 10,919 bytes — FastFlags, mutex, experience searching
        │   ├── roulette.rs       # 5,460 bytes — Backlog roulette & HowLongToBeat
        │   ├── social.rs         # 4,471 bytes — Unified friends & Discord RPC 2.0
        │   └── window.rs         # 3,310 bytes — Windows 11 Mica/Acrylic & store iframe HTML
        ├── core/                 # Core domain services
        │   ├── config.rs         # 2,253 bytes — AppConfig & atomic config.toml store
        │   ├── discord_rpc.rs    # 5,138 bytes — Discord Rich Presence IPC bridge
        │   ├── media_capture.rs  # 27,912 bytes — WindowsGraphicsCapture & DXGI recorder
        │   ├── platform.rs       # 17,471 bytes — PlatformConnector trait, Game, Artwork
        │   └── runner.rs         # 6,575 bytes — ProcessRunner & active game tracker
        └── platforms/            # Platform connectors
            ├── steam/            # 33,645 bytes (mod.rs) + 11,697 bytes (vdf_parser.rs)
            ├── epic/             # 11,342 bytes (mod.rs)
            ├── roblox/           # 22,581 bytes (mod.rs)
            ├── emulators/        # 7,679 bytes (mod.rs)
            └── itch/             # 9,082 bytes (mod.rs)
```

---

### 1.2 Frontend Architecture (Svelte 5 Runes & Tailwind v4)

1. **Svelte 5 Runes**:
   The frontend code in `src/` is built on modern Svelte 5 runes rather than Svelte 4 reactivity:
   - `$state`: heavily used for reactive primitives (`games = $state<Game[]>([])`, `loading = $state(true)` in `BentoHub.svelte:38-55`).
   - `$derived` & `$derived.by`: used for reactive filtering, sorting, and game counts (`BentoHub.svelte:57-87`, `GameCard.svelte:31-44`).
   - `$effect`: used for live debounced searches (`SteamStoreShowcase.svelte:45-63`, `StoreBrowserModal.svelte:22-26`).
   - `$props()`: used for typed component interfaces (`Navigation.svelte:27`, `GameCard.svelte:25`, `GameDetailsModal.svelte:44`).

2. **Stores and Reactivity**:
   Instead of legacy Svelte stores (`writable`/`readable`), state is managed in three central TS files using class/singleton controllers with `subscribe`/`notify` or direct async functions:
   - `src/lib/stores/library.ts`: Unified game, artwork resolution (`resolveArtUrl`), playtime/byte formatting, and Tauri IPC invocations (`get_library`, `refresh_library`, `launch_game`, `install_game`).
   - `src/lib/stores/theme.ts`: `ThemeController` managing CSS variables on `document.documentElement`, `localStorage` caching, audio OST loop playback, and Windows 11 Mica vibrancy sync via `set_window_effect`.
   - `src/lib/stores/friends.ts`: Unified Steam & Epic friends list, Discord RPC 2.0 status.

3. **Routing and View Navigation**:
   There is no URL router library. `BentoHub.svelte` acts as the master container with a local `activeSection` state (`home` | `library` | `roblox` | `itch` | `deals` | `friends` | `media`). Switching tabs simply conditionally renders the corresponding subcomponent.

4. **Modal Dialogs**:
   - `GameDetailsModal.svelte`: Opens when clicking a game card.
   - `RouletteModal.svelte`: "What to Play Today?" backlog randomizer with HLTB time budget.
   - `StoreBrowserModal.svelte`: In-app store viewer that embeds external store pages inside an `<iframe>` by fetching HTML via the Rust backend (`get_store_page_html`) to bypass `X-Frame-Options`.

5. **Visual Styling & Tailwind**:
   - `vite.config.ts:8`: Uses `@tailwindcss/vite` plugin with Tailwind CSS v4.
   - `src/lib/styles/app.css`:
     * Lines 1-21 define CSS custom properties on `:root` (`--bg-canvas`, `--bg-surface-1`, `--accent`, etc.).
     * **Discrepancy with User Request**: Lines 45-192 contain extensive "3D Casual Game UI Styles" with beveled push buttons (`.btn-game-green`, `.btn-game-gold`, `.btn-game-blue`), skeuomorphic stat boxes (`.stat-box-game`), and overlapping pill banners (`.panel-header-pill`).
     * This directly conflicts with the approved task prompt (Reference 1 "TURA" minimalist dark graphite, Cera CY typography, and Reference 2 liquid chrome / silver futuristic dark interface).

---

### 1.3 Backend Architecture (`src-tauri/`)

1. **Configuration**:
   - `Cargo.toml`: Configured as a dual lib/bin package (`omnilauncher` / `omnilauncher_lib`).
   - Features: `default = ["custom-protocol"]`.
   - Windows profile: `profile.release` has `opt-level = "s"`, `lto = true`, `codegen-units = 1`, `strip = true`, `panic = "abort"`.
   - `tauri.conf.json`: Configured with window size 1440x900 (min 1100x700), `frontendDist: "../dist"`, `devUrl: "http://localhost:1420"`. Asset protocol enabled for Steam grid cache and `$PICTURE/OmniLauncher`.
   - Capabilities: `capabilities/default.json` grants `core:default`.

2. **IPC Commands & System Services**:
   - `src-tauri/src/lib.rs` registers 38 Tauri commands spanning library scanning, process running with Discord RPC, DXGI media capture (`windows-capture`), Roblox FastFlags and multi-instance mutex, deal radars, and DWM window vibrancy (`window-vibrancy`).

---

### 1.4 Detailed Platform Inspection

#### Steam (`src-tauri/src/platforms/steam/mod.rs`)
- **Discovery**: Reads Windows Registry `HKCU\Software\Valve\Steam\SteamPath` and WOW6432Node fallbacks, successfully locating `libraryfolders.vdf` and `appmanifest_*.acf`.
- **Artwork**: Resolves in priority order: user grid overrides (`userdata/<id>/config/grid/`), local `appcache/librarycache/<appid>/`, and Akamai Steam CDN fallback.
- **Uninstalled Games**: Discovered via `HKCU\Software\Valve\Steam\Apps` registry subkeys and `userdata/<id>/config/localconfig.vdf` playtimes.
- **Steam Achievements (`fetch_achievements`)**:
  * Lines 305-335: Calls `https://api.steampowered.com/ISteamUserStats/GetPlayerAchievements/v1/`.
  * **Failure point**: If the user has not configured `steam.apiKey` or `steam.steamId64` in settings, it returns `LauncherError::AuthRequired("Steam Web API key not set")`. There is no local Steam achievement cache fallback or automated Steam profile discovery from registry.
- **Steam Cloud Saves (`inspect_cloud_saves`)**:
  * Inspects local `remotecache.vdf` files in `<SteamPath>/userdata/<AccountID>/<AppID>/`. It does not perform active two-way cloud sync or recovery.
- **Game Launching & Installation**:
  * Launch: `steam://rungameid/<appid>`.
  * Install: Lines 744-753 execute `steam://install/<appid>`. This pops up Steam's native drive selection and installation dialog rather than allowing the user to pick drives inside OmniLauncher.
- **Steam Store Showcase**:
  * `commands/deals.rs:173`: Queries `https://store.steampowered.com/api/featuredcategories/?l=russian`.
  * Hardcoded to Russian locale and RUB currency. No infinite recommendation feed. No KZT (тенге) support.

#### Epic Games (`src-tauri/src/platforms/epic/mod.rs`)
- **Discovery**: Scans `C:\ProgramData\Epic\EpicGamesLauncher\Data\Manifests\*.item`.
- **Limitation**: Only detects games that have already been installed (or whose `.item` manifests exist).
- **Uninstalled Games Missing**: Uninstalled games owned by the user on Epic Games are NOT present in `.item` manifests and therefore completely disappear from the library catalog.

#### Roblox (`src-tauri/src/platforms/roblox/mod.rs`)
- **Capabilities**:
  * FastFlags: Modifies `ClientSettings/ClientAppSettings.json` (FPS unlock, Vulkan, Future lighting).
  * Multi-Instance: Successfully holds the Win32 named mutex `ROBLOX_singletonMutex` in-process.
  * Active Place Detection: Parses the latest `%LOCALAPPDATA%\Roblox\logs\*.log` for `placeId` and queries Roblox API.
- **Limitation**:
  * Lines 513-532: `detect_installed_games` returns only a single fixed entry: `Game::new(PlatformKind::Roblox, "player", "Roblox")`.
  * It does **not** populate the library with recently launched Roblox games/experiences, despite having place ID log parsing logic.

#### Emulators (`src-tauri/src/platforms/emulators/mod.rs`)
- Pre-configured CLI profiles exist for RetroArch, Dolphin, PCSX2, RPCS3, Ryujinx, and PPSSPP.
- **Missing**: No direct download or installation of open-source emulators from GitHub releases. If the emulator binary is not pre-installed by the user, the platform is inactive.

#### Network, Cloudflare & Proxies
- Current networking uses standard `reqwest::Client` with `rustls-tls` (`Cargo.toml:49`).
- There is currently **no DNS selector** (Cloudflare 1.1.1.1, Google 8.8.8.8), no custom proxy settings, and no image proxying through Cloudflare CDN servers.

---

### 1.5 Launcher Executables, Launch Scripts & The Console Window Issue

1. **PE Subsystem Inspection**:
   Direct binary inspection of the PE Optional Header at offset `e_lfanew + 0x5C`:
   - `OmniLauncher.exe` in root: `Subsystem = 3` (`IMAGE_SUBSYSTEM_WINDOWS_CUI`).
   - `src-tauri\target\debug\omnilauncher.exe`: `Subsystem = 3` (`IMAGE_SUBSYSTEM_WINDOWS_CUI`).
   - In Windows, any executable with PE Subsystem = 3 is designated a Console application; Windows automatically creates, allocates, and attaches a visible console window (cmd/terminal) when launched from Explorer or desktop!

2. **Root Cause in Source Code**:
   In `src-tauri/src/main.rs`:
   ```rust
   // Prevents an additional console window on Windows in release builds.
   #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

   fn main() {
       omnilauncher_lib::run();
   }
   ```
   Because `windows_subsystem = "windows"` is gated by `not(debug_assertions)`, whenever debug assertions are enabled (any debug build, `cargo build`, or dev mode), the binary is linked as a console subsystem application!
   Furthermore, the loose `OmniLauncher.exe` in the root folder is a 30.4 MB debug build with Subsystem 3.

3. **`Start-OmniLauncher.cmd`**:
   ```cmd
   @echo off
   setlocal
   set "PATH=C:\Program Files\nodejs;%USERPROFILE%\.cargo\bin;%PATH%"
   cd /d "%~dp0"
   if "%~1"=="--dev" (
       npm.cmd run tauri dev
   ) else if exist "%~dp0src-tauri\target\debug\omnilauncher.exe" (
       start "" "%~dp0src-tauri\target\debug\omnilauncher.exe"
   ) else (
       npm.cmd run tauri dev
   )
   ```
   When a user double clicks a `.cmd` batch file:
   - Windows Explorer spawns `cmd.exe` (visible black terminal window).
   - The script launches `src-tauri\target\debug\omnilauncher.exe` which (due to Subsystem = 3) spawns a second console window, or runs `npm.cmd run tauri dev` which runs Node.js and Cargo in the console.

---

### 1.6 Verification of Build Commands & Pipeline

1. **Frontend Production Build (`npm run build`)**:
   - Command: `npm run build`
   - Result: Exit code 0.
   - Built in 28.15s:
     * `dist/index.html` (0.45 kB)
     * `dist/assets/index-Xddxq_gj.css` (58.36 kB)
     * `dist/assets/index-Dxoa1kCi.js` (140.38 kB)
2. **TypeScript Compilation Check (`npx tsc --noEmit`)**:
   - Command: `npx tsc --noEmit`
   - Result: Exit code 0, 0 type errors.
3. **Rust Backend Debug Check (`cargo check`)**:
   - Command: `cd src-tauri; cargo check`
   - Result: Exit code 0 in 10.18s, 0 errors, 0 warnings.
4. **Rust Backend Release Check (`cargo check --release`)**:
   - Command: `cd src-tauri; cargo check --release`
   - Result: Exit code 0 in 3m 38s, all 130+ crates compiled and verified cleanly with 0 errors.

---

## 2. Logic Chain

```
[Observation: src-tauri/src/main.rs line 2: #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]]
    │
    ▼
[Deduction: In all non-release compiles, Windows links with Subsystem 3 (Console)]
    │
    ▼
[Observation: OmniLauncher.exe in root has PE Subsystem 3; Start-OmniLauncher.cmd is a .cmd script]
    │
    ▼
[Conclusion 1: Launcher opens console windows because (a) the root binary was built in debug with Subsystem 3, and (b) launching via .cmd forces Windows to spawn cmd.exe. To run completely cleanly without any console popups: (1) set `#![windows_subsystem = "windows"]` unconditionally in main.rs, and (2) produce the release binary with embedded assets as the main launcher executable.]

[Observation: app.css lines 45-192 define .btn-game-green, .panel-game, .panel-header-pill, etc.]
    │
    ▼
[Comparison: User prompt mandates Minimalist Dark "TURA" + liquid chrome/silver, clean typography, removal of superfluous text]
    │
    ▼
[Conclusion 2: The entire casual 3D skeuomorphic styling layer must be replaced by a modern modular theme engine and clean, minimalist dark components.]

[Observation: Roblox detect_installed_games returns vec![player], while detect_active_place has log parser]
    │
    ▼
[Observation: Epic detect_installed_games only parses Manifests/*.item, omitting uninstalled games]
    │
    ▼
[Observation: Steam install_game delegates to steam://install/<id> opening Steam's system dialog]
    │
    ▼
[Observation: Emulators platform has CLI templates but zero GitHub release download code]
    │
    ▼
[Observation: DealsRadar/Steam store has fixed 4 categories, RUB/USD currency, no KZT, no Cloudflare/DNS/proxy settings]
    │
    ▼
[Conclusion 3: Core functional gaps exist across all 5 requested domains:
  1. Steam: Need in-launcher directory picker, achievement fallback without API key, and user profile linking.
  2. Epic: Need uninstalled library games retrieval (e.g., from local Epic launcher cache/logs/configs).
  3. Roblox: Need recently played places added to library via log scanner.
  4. Emulators: Need GitHub releases API download & extraction.
  5. Store & Network: Need infinite scroll, KZT currency conversion, Cloudflare/DNS/proxy configuration.]
```

---

## 3. Caveats

1. **Epic Games Library for Uninstalled Games**:
   Epic Games does not provide a public unauthenticated Web API for library retrieval. On Windows, Epic Games Launcher stores user credentials and entitlement cache in `%LOCALAPPDATA%\EpicGamesLauncher\Saved\Config` and local SQLite/JSON logs. Parsing uninstalled entitlements must be done via these local caches or known entitlement lists.
2. **Steam Cloud Direct File Upload**:
   True remote Steam Cloud synchronization without Steamworks SDK requires either official Steam Web API or interacting through the running Steam client service. OmniLauncher currently parses local `remotecache.vdf`.
3. **Windows 11 DWM Mica Vibrancy**:
   `window-vibrancy` (`apply_mica`, `apply_tabbed`) requires Windows 11 build 22000+. On older Windows 10 versions, the backend gracefully falls back to solid dark noir CSS backgrounds.

---

## 4. Conclusion

OmniLauncher has a robust, clean foundation (Tauri v2 + Rust 1.80 + Svelte 5 runes + Tailwind v4 + Vite 6), with all existing build commands passing with 0 errors.

To satisfy all user requirements (`ORIGINAL_REQUEST.md`), the implementation plan must execute the following structured modules:

| Target Area | Current State | Required Architecture Change |
|---|---|---|
| **Clean GUI Launch** | Debug binary in root with PE Subsystem 3; `.cmd` script spawns cmd console | Set `#![windows_subsystem = "windows"]` unconditionally in `main.rs`. Compile release binary; replace root executable with release GUI binary. Remove cmd console spawns. |
| **Visual Design (R1)** | 3D casual game buttons (`.btn-game`), skeuomorphic panels | Redesign components to "TURA" minimalist dark graphite, liquid chrome/silver accents, clean Cera CY/Inter typography, and extensible modular theme engine. |
| **Platform Integrations (R2)** | Steam achievements fail without API key; Steam install delegates to external dialog; Epic misses uninstalled games; Roblox only shows player | 1. Add Steam profile transition & link.<br>2. Local achievement parsing & fallback.<br>3. Built-in Steam drive/directory selector.<br>4. Epic uninstalled game discovery.<br>5. Roblox recently played places added to library. |
| **Store, Network & Emulators (R3)** | 4 static tabs, RUB only, no KZT; no emulators downloader; no Cloudflare/DNS/proxy settings | 1. Infinite recommendation feed.<br>2. USD & KZT (тенге) switcher with real-time conversion.<br>3. Open-source emulator downloader via GitHub API.<br>4. Cloudflare CDN routing, custom DNS (Cloudflare 1.1.1.1, Google 8.8.8.8), and proxy settings in launcher. |
| **Repository Layout (R4)** | 30 MB loose debug executable in root, loose .cmd | Reorganize internal project files: remove debug clutter from root, place build tools/launchers in clean structure. |

---

## 5. Verification Method

To independently verify the observations made in this report:

1. **Verify Frontend Build & Types**:
   ```powershell
   npm run build
   npx tsc --noEmit
   ```
   Both must finish with exit code 0.

2. **Verify Backend Rust Compilation**:
   ```powershell
   cd src-tauri
   cargo check
   cargo check --release
   ```
   Both must succeed with exit code 0.

3. **Verify PE Subsystem of Executables**:
   ```powershell
   powershell -Command "[BitConverter]::ToUInt16([IO.File]::ReadAllBytes('OmniLauncher.exe'), [BitConverter]::ToInt32([IO.File]::ReadAllBytes('OmniLauncher.exe'), 60) + 92)"
   ```
   Output `3` confirms Console Subsystem (CUI); output `2` confirms Native GUI Subsystem (no console window).

4. **Verify CSS Discrepancy**:
   Inspect `src/lib/styles/app.css` at line 50 for `.btn-game` and `.panel-game`.
