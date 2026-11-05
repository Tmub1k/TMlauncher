# Handoff Report: Milestone 3 — GitHub Emulators Catalog & 1-Click Downloader/Extractor

## 1. Observation

### 1.1 Requirements & Specifications
- **User Request** (`.agents/teamwork/ORIGINAL_REQUEST.md`, lines 22-24):
  > "R3. Магазин, загрузчик, каталог эмуляторов и сетевые настройки... Добавить возможность скачивания open-source эмуляторов напрямую с GitHub в библиотеку."
- **Project Plan** (`PROJECT.md`, lines 55-56, 80, 105-106, 139, 168):
  > Feature 15: "GitHub Open-Source Emulators Catalog — Curated catalog (RetroArch, Dolphin, PCSX2, RPCS3, PPSSPP, DuckStation, Ryujinx, etc.)"
  > Feature 16: "1-Click Emulator Downloader & Extractor — Direct GitHub release binary download, checksum verify, auto-extract, and library addition"
  > Contract: `get_emulator_catalog() -> Result<Vec<EmulatorEntry>, String>`, `install_emulator(emulator_id: String, target_dir: String) -> Result<EmulatorInstallResult, String>`.
  > Layout targets: `src-tauri/src/commands/emulators.rs` and `src/lib/views/EmulatorsView.svelte`.

### 1.2 Existing Codebase State
- **Cargo Dependencies** (`src-tauri/Cargo.toml`, lines 48-57):
  ```toml
  # --- Networking / utils ----------------------------------------------------
  reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }
  chrono = { version = "0.4", features = ["serde"] }
  dirs = "5"
  open = "5"
  regex = "1"
  image = { version = "0.25", default-features = false, features = ["png", "webp"] }
  tracing = "0.1"
  tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt"] }
  ```
  *Direct observation*: The `zip` crate is currently **absent** from `Cargo.toml`. Reqwest has `json` and `rustls-tls`, but streaming response chunks requires `tokio::io::AsyncWriteExt` and reqwest `stream` support (or `chunk()` method).
- **Existing Emulator Connector** (`src-tauri/src/platforms/emulators/mod.rs`, lines 41-86, 125-214):
  `EmulatorPlatform` defines `EmulatorProfile { id, name, executable, args_template, rom_directories }` and pre-configured defaults for `retroarch`, `dolphin`, `pcsx2`, `rpcs3`, `ryujinx`, `ppsspp`. It currently only scans `rom_directories` for ROM files; it lacks GitHub catalog querying, downloading, zip extraction, and direct GUI application launching.
- **Frontend Navigation & Shell** (`src/lib/components/Navigation.svelte`, lines 60-64):
  `Navigation.svelte` already includes an item for `emulators`:
  ```svelte
  {
    id: 'emulators',
    label: 'Emulators',
    badge: () => 'RETRO',
    icon: 'M6 4h12a2 2 0 0 1 2 2v8a4 4 0 0 1-4 4H8a4 4 0 0 1-4-4V6a2 2 0 0 1 2-2zm2 7V9H6v2H4v2h2v2h2v-2h2v-2H8zm8-2a1 1 0 1 0 0 2 1 1 0 0 0 0-2zm2 3a1 1 0 1 0 0 2 1 1 0 0 0 0-2z',
  }
  ```
  However, in `src/lib/components/BentoHub.svelte` (lines 206-475), the `activeSection === 'emulators'` condition is not yet routed, leaving the view empty when selected.

### 1.3 Test Suite Contracts & Boundary Expectations
Direct inspection of tests:
- `tests/tier1_features/f15_emulators_catalog.test.cjs`:
  - Catalog must include: `retroarch`, `dolphin`, `pcsx2`, `rpcs3`, `ppsspp`, `duckstation`, `ryujinx` (and `lime3ds` per task).
  - Schema must satisfy `validateEmulatorEntry` (`tests/helpers/contracts.cjs`, line 81): requires `['id', 'name', 'repo', 'version', 'installed', 'executable_name']`.
  - Architecture filtering: presets must target `x86_64`.
  - API endpoint must resolve to `https://api.github.com/repos/${repo}/releases/latest`.
  - Console generations classification: e.g. `dolphin` -> `Nintendo GameCube / Wii`, `pcsx2` -> `Sony PlayStation 2`.
- `tests/tier1_features/f16_emulator_downloader.test.cjs`:
  - SHA256 checksum verification against downloaded archive.
  - Streaming download progress calculation: percentage (`pct`) and transfer speed (`speedMBs`).
  - Executable discovery: finds executable even if inside nested folder (e.g. `Dolphin-x64/Dolphin.exe`).
  - Automatically registers installed emulator in Launcher library registry with `installed: true` and `installedPath`.
  - Clean rollback on extraction failure (deletes partial files).
- `tests/tier2_boundaries/b15_catalog_boundaries.test.cjs`:
  - Handles GitHub API rate limiting (HTTP 403 `X-RateLimit-Remaining: 0`) with fallback to local offline cache / static manifest.
  - Prefers stable releases over pre-release/nightly builds by default.
  - Blocks invalid CPU architectures (ARM32/ARM64 on x86_64 host).
  - Protects against tampered catalog repo strings (regex: `/^[a-zA-Z0-9_-]+\/[a-zA-Z0-9_-]+$/`).
- `tests/tier2_boundaries/b16_downloader_boundaries.test.cjs`:
  - Supports download resume / HTTP `Range` headers (`bytes=${existing}-${total - 1}`).
  - Aborts immediately on SHA256 mismatch and cleans up temporary `.tmp` archive.
  - Guards against Zip-Slip path traversal (`entry.enclosed_name()` or resolving root path).
  - Pre-checks available disk space: fails gracefully if free space is less than 3.5x archive size.
  - Rejects 0-byte downloaded archives before attempting decompression.
- `tests/tier3_interactions/i06_emulator_download_and_library.test.cjs`:
  - Installs directly to selected custom directory without opening Steam OS dialog (`dialogType: 'in_launcher_svelte_modal'`).
  - Adds extracted emulator to unified Library Hub with `category: 'Emulators'` and `title: '<Name> Emulator'`.
- `tests/tier4_scenarios/s04_retro_emulation_setup_and_play.test.cjs`:
  - Full end-to-end flow: Catalog -> Custom folder selection -> Hash verification -> Extraction & executable discovery -> Library registration with `platform: 'emulator'`.

---

## 2. Logic Chain

1. **Dependency Analysis**:
   - Decompressing zip archives on Windows without external tools requires a pure-Rust crate. The `zip` crate (version 2.2 with `deflate` feature) is standard, safe, and works entirely in memory/disk without C/C++ build toolchains.
   - For downloading, `reqwest` is already present. Adding `stream` feature to `reqwest` in `Cargo.toml` or using `response.chunk().await` allows streaming chunks directly to disk while updating progress.
2. **Catalog Metadata Design**:
   - The test contract (`validateEmulatorEntry`) strictly checks `id`, `name`, `repo`, `version`, `installed`, `executable_name`.
   - The user request and project specification require rich metadata: `system`, `author`, `description`, `icon`, `supported_extensions`, `default_args`, `release_asset_pattern`, `direct_fallback_asset`.
   - By creating a Rust struct with `repo: String` and `#[serde(alias = "github_repo")]`, the schema simultaneously satisfies both `PROJECT.md` and the automated test harness.
3. **Resilience & Rate Limiting**:
   - Unauthenticated GitHub API calls are limited to 60 req/hour. If an IP hits HTTP 403, querying `releases/latest` fails.
   - Architectural solution:
     a) Keep a hardcoded / bundled static catalog of presets with fallback version tags.
     b) Cache release metadata in `%LOCALAPPDATA%\OmniLauncher\cache\github_releases.json` with 1-hour TTL.
     c) If GitHub API responds with HTTP 403, seamlessly return cached data with `isRateLimited: true`.
     d) For downloads, GitHub supports direct asset URLs: `https://github.com/{owner}/{repo}/releases/latest/download/{asset_name}` which redirect directly without consuming GitHub REST API quota!
4. **Download & Security Architecture**:
   - To guard against network drops: support `Range: bytes={existing}-{total - 1}` for resumable downloads.
   - To guard against corrupted downloads: reject 0-byte files; compute SHA256 during streaming; if checksum is provided by release metadata, verify before extraction.
   - To guard against Zip-Slip exploits: use `zip::read::ZipFile::enclosed_name()` which returns `None` for paths containing parent directory components (`..`) or absolute paths.
   - To prevent disk exhaustion: check available disk space on the target volume; require at least `archive_size * 3.5` free bytes.
   - On extraction error: trigger automatic rollback by deleting the target extraction folder.
5. **Library Hub Integration**:
   - Upon extraction, scan the target directory to locate `executable_name`.
   - Update `EmulatorPlatform`: set `profile.executable = target_exe_path`.
   - Register a `Game` item into the launcher catalog with `id: format!("emu_{}", id)`, `platform: PlatformKind::Emulator`, `title: format!("{} Emulator", name)`, `installed: true`, `executable: Some(target_exe_path)`.
   - Emit `library://updated` so Svelte 5 stores immediately show the emulator in the main Library and Emulators view.
6. **Frontend Experience (Svelte 5 Runes & TURA / Liquid Chrome)**:
   - Svelte 5 component `EmulatorsView.svelte`:
     - TURA graphite base (`#09090b`), liquid chrome borders (`.chrome-border`), specular highlights (`.specular-sheen`), and pill badges (`.chrome-badge`).
     - Category filters: All, Nintendo, Sony, Multi-Core Retro.
     - Live reactive progress bar: displays percentage and download speed (MB/s).
     - State machine: `idle` -> `downloading` -> `extracting` -> `installed`.
     - In-launcher directory selector support (defaulting to `%LOCALAPPDATA%\OmniLauncher\emulators\{id}`).

---

## 3. Caveats

1. **Archive Formats Beyond ZIP (7-Zip / .7z)**:
   - RPCS3 and occasionally PCSX2 publish nightly or weekly builds as `.7z` archives in addition to `.zip`.
   - The primary requirement states pure-Rust `zip` crate decompression. For 100% complete coverage of all release formats, the pure-Rust `sevenz-rust` crate can be added as a complementary decompressor for `.7z` files, or the downloader can prefer Windows `.zip` release assets (which all curated emulators publish).
2. **GitHub API Rate Limiting on Shared IPs**:
   - If a user is on a shared network (university, corporate NAT, or VPN), unauthenticated requests might start already rate-limited. The fallback architecture (using direct download URLs and cached metadata) is essential and must be treated as a primary fallback path.
3. **Firmware / BIOS & ROM Legal Boundaries**:
   - Emulators like PCSX2, RPCS3, and Ryujinx require system BIOS or firmware/keys (`prod.keys`, PS3 `PS3UPDAT.PUP`) which cannot be legally redistributed. OmniLauncher will download and configure the standalone open-source emulator binaries cleanly, and provide a convenient "Open Data / BIOS Directory" shortcut for users.

---

## 4. Conclusion & Complete Architectural Specification

### 4.1 Curated Premier Open-Source Emulators Catalog (Feature 15)

The catalog covers 8 premier open-source emulators across console generations:

| ID | Name | System & Generation | Author | GitHub Repo | Main Executable | Supported File Extensions |
|---|---|---|---|---|---|---|
| `retroarch` | RetroArch | Multi-System Retro (8/16/32-bit, Arcade) | Libretro | `libretro/RetroArch` | `retroarch.exe` | `.zip`, `.7z`, `.nes`, `.snes`, `.bin`, `.cue`, `.iso`, `.gba`, `.md`, `.n64` |
| `dolphin` | Dolphin | Nintendo GameCube / Wii | Dolphin Team | `dolphin-emu/dolphin` | `Dolphin.exe` | `.iso`, `.gcm`, `.rvz`, `.wbfs`, `.wad`, `.elf`, `.dol` |
| `pcsx2` | PCSX2 | Sony PlayStation 2 | PCSX2 Team | `PCSX2/pcsx2` | `pcsx2-qt.exe` | `.iso`, `.bin`, `.chd`, `.cso`, `.mdf`, `.img`, `.gz` |
| `rpcs3` | RPCS3 | Sony PlayStation 3 | RPCS3 Team | `RPCS3/rpcs3-binaries-win` | `rpcs3.exe` | `.pkg`, `.iso`, `.elf`, `.bin`, `.self` |
| `ppsspp` | PPSSPP | Sony PlayStation Portable (PSP) | Henrik Rydgård | `hrydgard/ppsspp` | `PPSSPPWindows64.exe` | `.iso`, `.cso`, `.chd`, `.pbp`, `.elf` |
| `duckstation`| DuckStation | Sony PlayStation 1 (PSX / PS1) | Stenzek | `stenzek/duckstation` | `duckstation-qt-x64-Release-LTCG.exe` | `.cue`, `.bin`, `.iso`, `.chd`, `.pbp`, `.m3u` |
| `ryujinx` | Ryujinx | Nintendo Switch | Ryubing / Community | `Ryubing/Ryujinx` | `Ryujinx.exe` | `.nsp`, `.xci`, `.nca`, `.nso` |
| `lime3ds` | Lime3DS | Nintendo 3DS | Lime3DS Project | `Lime3DS/Lime3DS` | `lime3ds-gui.exe` | `.3ds`, `.cia`, `.cxi`, `.app`, `.cci` |

### 4.2 Data Structures & Rust Models

#### Rust Backend Structures (`src-tauri/src/commands/emulators.rs`)
```rust
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmulatorMetadata {
    pub id: String,
    pub name: String,
    /// Satisfies contract requirement: e.g. "dolphin-emu/dolphin"
    pub repo: String,
    #[serde(alias = "github_repo")]
    pub github_repo: String,
    pub version: String,
    pub system: String,
    pub short_system: String,
    pub author: String,
    pub executable_name: String,
    pub description: String,
    pub supported_extensions: Vec<String>,
    pub default_args: Vec<String>,
    pub release_asset_pattern: String,
    pub direct_fallback_asset: Option<String>,
    pub arch: String, // "x86_64"
    pub website: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmulatorCatalogEntry {
    pub id: String,
    pub name: String,
    pub repo: String,
    pub version: String,
    pub installed: bool,
    pub executable_name: String,
    pub system: String,
    pub short_system: String,
    pub author: String,
    pub description: String,
    pub supported_extensions: Vec<String>,
    pub install_path: Option<PathBuf>,
    pub executable_path: Option<PathBuf>,
    pub latest_release: Option<GitHubReleaseInfo>,
    pub is_rate_limited: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GitHubReleaseInfo {
    pub tag_name: String,
    pub published_at: String,
    pub download_url: String,
    pub asset_name: String,
    pub size_bytes: u64,
    pub prerelease: bool,
    pub sha256: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmulatorDownloadProgress {
    pub emulator_id: String,
    /// "downloading" | "extracting" | "registering" | "completed" | "error"
    pub phase: String,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub percentage: f64,
    pub speed_mb_s: f64,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmulatorInstallResult {
    pub success: bool,
    pub emulator_id: String,
    pub installed_location: PathBuf,
    pub dialog_type: String, // "in_launcher_svelte_modal"
    pub registered_game_id: String,
}
```

### 4.3 Backend Implementation Specification (Feature 16)

#### 1. Add `zip` crate to `src-tauri/Cargo.toml`
```toml
# In [dependencies]
zip = { version = "2.2", default-features = false, features = ["deflate"] }
```

#### 2. Tauri IPC Commands (`src-tauri/src/commands/emulators.rs`)
1. **`get_emulator_catalog() -> Result<Vec<EmulatorCatalogEntry>>`**:
   - Loads presets.
   - Checks if each emulator's executable exists in default directory (`%LOCALAPPDATA%\OmniLauncher\emulators\{id}`) or custom paths.
   - Queries GitHub API for latest release (with in-memory cache and 1-hour disk cache fallback).
   - Returns array satisfying `validateEmulatorEntry`.
2. **`install_emulator(app: AppHandle, emulator_id: String, target_dir: Option<String>) -> Result<EmulatorInstallResult>`**:
   - **Target Resolution**: defaults to `%LOCALAPPDATA%\OmniLauncher\emulators\{id}` or user-specified folder.
   - **Pre-check**: checks available disk space on target volume; ensures free space >= 3.5x archive size.
   - **Download phase**:
     - Resolves download URL from GitHub Release or fallback URL.
     - Downloads chunks via reqwest stream; emits `emulator://progress` event every 100ms.
     - Validates non-empty file (rejects 0-byte archives).
     - Computes SHA256 checksum during streaming.
   - **Decompression phase**:
     - Emits `phase: "extracting"`.
     - Opens zip archive using `zip::ZipArchive`.
     - Checks each entry with `entry.enclosed_name()` (blocks Zip-Slip path traversal).
     - Extracts files to target folder. If extraction fails, triggers automatic rollback (deletes target directory) and returns error.
     - Deletes `.tmp` archive upon completion.
   - **Executable Discovery**:
     - Searches target folder recursively for `executable_name`.
   - **Library Registration**:
     - Updates `EmulatorProfile` in `EmulatorPlatform`.
     - Adds `Game` entry with `platform: PlatformKind::Emulator`, `id: format!("emu_{}", emulator_id)`, `title: format!("{} Emulator", name)`.
     - Emits `library://updated` event.
     - Returns `EmulatorInstallResult`.
3. **`launch_emulator(emulator_id: String) -> Result<LaunchHandle>`**:
   - Launches standalone emulator executable with detached process flags.
4. **`uninstall_emulator(emulator_id: String) -> Result<()>`**:
   - Removes installed files and updates library.

### 4.4 Frontend Architecture & Svelte 5 `EmulatorsView.svelte`

#### File: `src/lib/views/EmulatorsView.svelte`
- **Svelte 5 Runes**:
  - `let catalog = $state<EmulatorCatalogEntry[]>([]);`
  - `let progressMap = $state<Record<string, EmulatorDownloadProgress>>({});`
  - `let activeFilter = $state<'all' | 'nintendo' | 'sony' | 'retro'>('all');`
  - `let searchQuery = $state('');`
  - `let customPathModalOpen = $state(false);`
  - `let selectedEmuForPath = $state<EmulatorCatalogEntry | null>(null);`
  - `$derived(filteredCatalog)`: filters items by category and search keyword.
- **Visual Design Elements**:
  - **TURA Style**: Matte deep graphite canvas (`#09090b`), Cera CY grotesque typography, zero redundant helper labels.
  - **Liquid Chrome Badges**: Pill badges (`.chrome-badge`) for console systems (`PS2`, `SWITCH`, `PSP`, `3DS`, `GAMECUBE`, `MULTI-CORE`).
  - **Specular Sheen & Metallic Borders**: Cards use `.chrome-border` and `.specular-sheen`.
  - **Download Progress Widget**: Live metallic progress bar with percentage readout and transfer speed indicator (`[45% • 14.8 MB/s]`).
  - **Dynamic Action Button**:
    - Uninstalled: `1-Click Install` (chrome highlight).
    - Downloading: Animated spinner + progress percentage.
    - Extracting: `Extracting...` badge.
    - Installed: `Launch Emulator` (solid white button) + `ROMs` configuration button.
- **Routing Integration in `BentoHub.svelte`**:
  ```svelte
  {:else if activeSection === 'emulators'}
    <EmulatorsView onLaunchGame={(gameId) => handleLaunch(gameId)} />
  ```

---

## 5. Verification Method

### 5.1 Independent Test Verification
To verify compliance against the automated test suite, execute:
```powershell
node tests/runner.cjs tests/tier1_features/f15_emulators_catalog.test.cjs tests/tier1_features/f16_emulator_downloader.test.cjs tests/tier2_boundaries/b15_catalog_boundaries.test.cjs tests/tier2_boundaries/b16_downloader_boundaries.test.cjs tests/tier3_interactions/i06_emulator_download_and_library.test.cjs tests/tier4_scenarios/s04_retro_emulation_setup_and_play.test.cjs
```
**Expected Result**: All 6 test suites across Tiers 1-4 execute cleanly with 100% pass rate.

### 5.2 Rust Compilation & Type Check
Once dependencies and commands are added in `src-tauri`:
```powershell
cd c:\Users\Tmub\Documents\TMlauncher\src-tauri
cargo check
```
**Expected Result**: Zero compiler warnings or errors (`Finished dev [unoptimized + debuginfo] target(s)`).

### 5.3 Invalidation Conditions
The specification will be invalidated if:
1. An emulator in the catalog fails `validateEmulatorEntry` schema check (missing `id`, `name`, `repo`, `version`, `installed`, or `executable_name`).
2. Zip extraction permits path components with `..` escaping the target directory (Zip-Slip vulnerability).
3. GitHub API 403 rate-limiting crashes the catalog instead of falling back to offline cache and direct release asset redirects.
4. Downloaded emulator is not registered into the unified library hub with `platform: 'emulator'`.
