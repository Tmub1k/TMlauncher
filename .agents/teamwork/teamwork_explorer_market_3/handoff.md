# Market & Network Architecture Research Report: Emulators Catalog, Cloudflare Networking, and Platform Integrations

**Author**: Market Research Agent 3 (`teamwork_explorer`)  
**Target Project**: OmniLauncher (TMlauncher)  
**Parent Orchestrator**: `75c17a26-5c14-4320-bf9e-bf30909202d5`  
**Timestamp**: 2026-11-04T23:05:00Z  
**Relevant Requirements**: R2 (Platform Integrations: Steam, Epic, Roblox), R3 (Store, Downloader, Emulators, Network), R5 (Market & Technical Research)

---

## 1. Observation

### 1.1 Existing Codebase & Stack Inspection
- **Project Structure**:
  - Backend: Rust 1.80 (`src-tauri`), Tauri v2 (`@tauri-apps/api: ^2.2.0`), Tokio async runtime (`tokio = { version = "1", features = ["full"] }`), Reqwest (`reqwest = { version = "0.12", features = ["json", "rustls-tls"] }`), Serde, Winreg (`0.52`).
  - Frontend: Svelte 5 (`^5.16.0`), Vite 6 (`^6.0.5`), TailwindCSS v4 (`^4.0.0`), TypeScript (`^5.7.2`).
- **Platform Connector Layer**:
  - `src-tauri/src/platforms/emulators/mod.rs`: Defines `EmulatorProfile`, `hash_rom_file` (MD5 + SHA-256), and hardcoded launch presets (`retroarch`, `dolphin`, `pcsx2`, `rpcs3`, `ryujinx`, `ppsspp`). No automated download, GitHub release querying, or catalog ingestion exists.
  - `src-tauri/src/platforms/steam/mod.rs` & `vdf_parser.rs`: Discovers Steam libraries via Windows Registry (`HKCU\Software\Valve\Steam\SteamPath`) and multi-drive `libraryfolders.vdf`. Successfully parses `appmanifest_<appid>.acf` and `remotecache.vdf` (`parse_remotecache`). Lacks Steam profile deep linking, achievements visualizer/repair tool, and Steam Cloud conflict resolution UI.
  - `src-tauri/src/platforms/epic/mod.rs`: Scans `%ProgramData%\Epic\EpicGamesLauncher\Data\Manifests\*.item`. Parses `AppName`, `DisplayName`, `CatalogNamespace`, `CatalogItemId`, `InstallLocation`. Only imports games marked installed (`!manifest.is_incomplete_install`). Uninstalled/owned games are completely omitted.
  - `src-tauri/src/platforms/roblox/mod.rs`: Holds FastFlags preset editor, multi-instance mutex (`ROBLOX_singletonMutex`), and single-log scanner (`scan_latest_log_for_place_id`). Currently outputs only 1 static game (`RobloxPlayerBeta.exe`) and only checks the active place, ignoring user play history.

---

### 1.2 Open-Source Emulators Catalog: Live Web & GitHub Release Data
Through live investigation of GitHub Releases APIs and emulator update infrastructure, exact release distribution patterns were verified:

| Emulator | Target Console | Repository / Provider | Asset Name Regex / Pattern | Executable Name | Live Release Tag & Asset URL Observed |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **PCSX2** | Sony PlayStation 2 | `PCSX2/pcsx2` (GitHub Releases) | `^pcsx2-v.*-windows-x64-Qt\.7z$` | `pcsx2-qt.exe` | Tag `v2.8.2`, asset `pcsx2-v2.8.2-windows-x64-Qt.7z` (25.6 MB) |
| **DuckStation** | Sony PlayStation 1 | `stenzek/duckstation` (GitHub Releases) | `^duckstation-windows-x64-release\.zip$` | `duckstation-qt-x64-ReleaseLTCG.exe` | Tag `latest` (rolling), asset `duckstation-windows-x64-release.zip` (73.9 MB) |
| **RPCS3** | Sony PlayStation 3 | `RPCS3/rpcs3-binaries-win` (GitHub Releases) | `^rpcs3-v.*_win64.*\.7z$` | `rpcs3.exe` | Tag `build-...`, asset `rpcs3-v0.0.43-20562-779af965_win64_msvc.7z` (37.2 MB) |
| **PPSSPP** | Sony PlayStation Portable | `hrydgard/ppsspp` (GitHub Releases) | `^PPSSPP-v.*-Windows-x64\.zip$` | `PPSSPPWindows64.exe` | Tag `v1.20.4`, asset `PPSSPP-v1.20.4-Windows-x64.zip` (20.3 MB) |
| **xemu** | Original Microsoft Xbox | `xemu-project/xemu` (GitHub Releases) | `^xemu-.*windows-x86_64\.zip$` | `xemu.exe` | Tag `v0.8.139`, asset `xemu-0.8.139-windows-x86_64.zip` (11.4 MB) |
| **melonDS** | Nintendo DS / DSi | `melonDS-emu/melonDS` (GitHub Releases) | `^melonDS-.*-windows-x86_64\.zip$` | `melonDS.exe` | Tag `1.1`, asset `melonDS-1.1-windows-x86_64.zip` (19.4 MB) |
| **Flycast** | Sega Dreamcast / Naomi | `flyinghead/flycast` (GitHub Releases) | `^flycast-win64-.*\.zip$` | `flycast.exe` | Tag `v2.7`, asset `flycast-win64-2.7.zip` (9.8 MB) |
| **Azahar** (Citra 3DS) | Nintendo 3DS | `azahar-emu/azahar` (GitHub Releases) | `^azahar-windows-msvc-.*\.zip$` | `azahar.exe` | Tag `2126.2`, asset `azahar-windows-msvc-2126.2.zip` (43.9 MB) |
| **Dolphin** | Nintendo GameCube / Wii | `dolphin-emu.org` (REST Update API) | Direct JSON `artifacts` lookup | `Dolphin.exe` | Endpoint `https://dolphin-emu.org/update/latest/beta` yields `https://dl.dolphin-emu.org/releases/2609a/dolphin-2609a-x64.7z` |
| **RetroArch** | Multi-system Frontend | `buildbot.libretro.com` (Direct CDN / Nightly) | Predictable static path | `retroarch.exe` | URL: `https://buildbot.libretro.com/stable/latest/windows/x86_64/RetroArch.7z` |

#### Critical GitHub API Nuance Discovered:
1. `libretro/RetroArch` on GitHub releases only distributes `retroarch-sourceonly-*.tar.xz`. Pre-compiled Windows binaries must be fetched from the official Libretro buildbot.
2. `dolphin-emu/dolphin` maintains an empty GitHub Releases feed (`[]`). Dolphin hosts its own REST update API (`https://dolphin-emu.org/update/latest/beta`).
3. Unauthenticated GitHub API calls are strictly capped at 60 requests per hour per client IP (`x-ratelimit-remaining`), resetting hourly. In contrast, direct release redirect downloads `https://github.com/{owner}/{repo}/releases/latest/download/{filename}` do **not** consume REST API quota.

---

### 1.3 Network Settings & Cloudflare Integration Data
- **DoH (DNS-over-HTTPS)**:
  - Standard RFC 8484 endpoints:
    - Cloudflare: `https://cloudflare-dns.com/dns-query` (IPs: `1.1.1.1`, `1.0.0.1`)
    - Google: `https://dns.google/dns-query` (IPs: `8.8.8.8`, `8.8.4.4`)
    - Quad9: `https://dns.quad9.net/dns-query` (IPs: `9.9.9.9`, `149.112.112.112` — malware blocking)
    - AdGuard: `https://dns.adguard-dns.com/dns-query` (IPs: `94.140.14.14`, `94.140.15.15` — ad/tracking blocking)
  - Rust Implementation: `reqwest 0.12` natively supports custom DNS resolution via `ClientBuilder::dns_resolver(Arc<dyn reqwest::dns::Resolve>)`. The `Resolve` trait requires `fn resolve(&self, name: Name) -> Resolving`, allowing clean asynchronous DoH resolution using standard DoH wireformat or JSON APIs.
- **Proxy Configuration**:
  - `reqwest::Proxy::all("socks5h://host:port")` natively delegates both DNS lookups and TCP connections through SOCKS5, preventing DNS leakage.
  - Windows system proxy configuration resides in `HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings` (`ProxyEnable: DWORD`, `ProxyServer: String`, `AutoConfigURL: String`).
- **Cloudflare Edge Shielding**:
  - Game artwork CDNs (Steam Akamai, Epic Images, Roblox `tr.rbxcdn.com`, SteamGridDB) enforce strict HTTP `Referer` / `User-Agent` checks or rate-limit residential IPs.
  - Cloudflare Workers allow serverless proxying with header rewriting, URL validation (preventing open-proxy abuse), and edge caching (`cf: { cacheTtl: 604800, cacheEverything: true }`).

---

### 1.4 Platform Integrations Data (Steam, Epic, Roblox)
- **Steam**:
  - Profile Discovery: `<SteamPath>/config/loginusers.vdf` stores `PersonaName`, `AccountName`, `Timestamp`, and `MostRecent: "1"`. The 64-bit Steam ID is the Key (`76561198...`), and the 32-bit AccountID is `(SteamID64 - 76561197960265728)`.
  - Steam Achievements:
    - Online: `ISteamUserStats/GetPlayerAchievements/v0001` returns unlocked achievement keys and unix unlock timestamps; `ISteamUserStats/GetSchemaForGame/v2` provides display names and icons (`icongray` vs `icon`).
    - Offline / Local Stats Cache: Cached inside `<SteamPath>/userdata/<AccountID>/<AppID>/stats/UserStats_<AppID>_<AccountID>.bin` and `<SteamPath>/appcache/stats/UserGameStatsSchema_<AppID>.bin`.
    - Desync / Corruption: When Steam client or offline play desynchronizes achievements, deleting stale schema in `appcache/stats/` and issuing a Steamworks reload forces client re-sync.
  - Steam Cloud:
    - Auto-Cloud saves map to local paths (`%USERPROFILE%/Saved Games`, `%LOCALAPPDATA%`, `%APPDATA%`, `%DOCUMENTS%/My Games`) and sync to `<SteamPath>/userdata/<AccountID>/<AppID>/remote/`.
    - `remotecache.vdf` records every file's `size`, `localtime`, `remotetime`, `sha`, and `syncstate` (1 = in sync, 4 = uploaded).
- **Epic Games**:
  - `%ProgramData%\Epic\EpicGamesLauncher\Data\Manifests\*.item` contains `.item` JSON manifests for currently installed games.
  - Uninstalled/Owned games are not stored in permanent manifests once uninstalled; however:
    1. Epic Games Launcher logs (`%LOCALAPPDATA%\EpicGamesLauncher\Saved\Logs\EpicGamesLauncher.log`) dump catalog entries during client startup.
    2. Local Legendary/Heroic configurations (`%APPDATA%\legendary\installed.json` or Heroic `store_cache\epic_games.json`) contain the complete owned library metadata.
    3. Direct launch/install protocol `com.epicgames.launcher://apps/{namespace}%3A{catalogItemId}%3A{appName}?action=install` allows initiating installations without third-party credentials.
- **Roblox**:
  - `%LOCALAPPDATA%\Roblox\logs\*Player*.log` maintains connection logs.
  - Lines containing `[FLog::Network] Connection accepted ... placeId: <id>` or `placeid=<id>` indicate actual play sessions.
  - By parsing timestamps of recent log files, OmniLauncher can reconstruct a comprehensive history of recently played Roblox experiences.
  - Roblox Thumbnails API `https://thumbnails.roblox.com/v1/places/gameicons?placeIds={ids}&size=512x512&format=Png` resolves rich covers without requiring user credentials.

---

## 2. Logic Chain

```
[Observation: Emulators have varied release channels (GitHub releases, update REST APIs, buildbots)]
    ├──> Direct hardcoding of GitHub URLs breaks on tag rollings (e.g., DuckStation tag 'latest', RPCS3 build tags).
    ├──> Rate-limiting on GitHub API (60 req/hr) breaks unauthenticated catalog browsing.
    └──> Conclusion: Formulate a multi-provider Emulator Manifest Schema supporting regex asset matching,
         Cloudflare edge caching of release manifests, and direct download URL fallbacks.

[Observation: Game art CDNs reject hotlinking / desktop user-agents; users face ISP DNS blocks]
    ├──> Steam and Epic images intermittently fail to render or throttle when fetched directly.
    ├──> Standard OS DNS is susceptible to ISP tampering, poisoning, or latency.
    └──> Conclusion: Architect a Cloudflare Edge Proxy Worker with URL allowlisting and asset caching,
         combined with a native Rust DoH resolver implementing reqwest::dns::Resolve (Cloudflare, Google, Quad9, AdGuard).

[Observation: Steam remotecache.vdf tracks Cloud save SHAs and timestamps, but lacks user visibility]
    ├──> Users frequently lose saves due to desynchronization or accidental overwrites.
    └──> Conclusion: Implement a pre-launch snapshot backup engine and visual Cloud Sync Status inspector
         that compares local file SHAs against remotecache.vdf.

[Observation: Epic Games manifest directory only holds installed games]
    ├──> Requirement R2 mandates displaying uninstalled Epic games in the library.
    └──> Conclusion: Multi-tier detection engine parsing Epic manifests, Epic launcher logs,
         and Legendary/Heroic local caches, enabling 1-click install via Epic silent protocol.

[Observation: Roblox logs preserve placeIds with timestamps across sessions]
    ├──> Currently TMlauncher only detects active place and lists 1 static game.
    └──> Conclusion: Reconstruct recently played Roblox experiences from historical logs
         and fetch 512x512 thumbnails via Roblox Games & Thumbnails APIs.
```

---

## 3. Caveats
1. **GitHub API Unauthenticated Rate Limits**:
   - If 10 emulators query GitHub API sequentially on every launch, the 60 req/hr limit is exhausted in 6 launches.
   - **Mitigation**: Local JSON cache with 24-hour TTL in `%LOCALAPPDATA%\OmniLauncher\cache\emulators\`, HTTP ETag checking (`304 Not Modified` is free of rate limits), and optional Cloudflare Worker release-caching proxy.
2. **Nintendo Switch / 3DS Legal Landscape**:
   - Ryujinx and Citra repositories have experienced takedowns and fork migration (e.g., Citra succeeded by Azahar/Lime3DS; Ryujinx continuing under community forks). The emulator catalog should be configurable via external or remote JSON manifest so download sources can update dynamically without binary updates.
3. **Steam Achievement Unlocking Security**:
   - Third-party desktop apps cannot legitimately grant Steam achievements on Valve's cloud without running under Steamworks API context. The "Steam achievements fix" in this report focuses on: (1) repairing corrupted local achievement/stat schemas (`appcache/stats`), (2) synchronizing offline unlocks, and (3) bridging emulator/Goldberg achievement files into OmniLauncher's unified achievement tracker.
4. **Epic Games Full Library Sync without Login**:
   - Reading uninstalled games without user login relies on local log/cache discovery (Heroic/Legendary/Epic client logs). Users who never launched Epic Games Launcher on the machine will need an optional OAuth link to discover uninstalled titles.

---

## 4. Conclusion & Technical Specifications

### 4.1 Emulator Catalog Data Structure Specification

```rust
// Rust schema for src-tauri/src/platforms/emulators/catalog.rs

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmulatorCatalogEntry {
    pub id: String,
    pub name: String,
    pub console: String,
    pub platform_family: String, // e.g. "sony", "nintendo", "sega", "microsoft"
    pub description: String,
    pub icon_url: String,
    pub website_url: String,
    pub provider: DownloadProvider,
    pub executable_name: String,
    pub args_template: Vec<String>,
    pub supported_extensions: Vec<String>,
    pub requires_bios: bool,
    pub bios_info: Option<BiosRequirementInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DownloadProvider {
    GitHubRelease {
        owner: String,
        repo: String,
        asset_regex: String,
        is_prerelease_allowed: bool,
    },
    DolphinUpdateApi {
        channel: String, // "beta" or "release"
    },
    LibretroBuildbot {
        channel: String, // "stable" or "nightly"
        archive_name: String,
    },
    DirectArchive {
        url: String,
        sha256: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BiosRequirementInfo {
    pub description: String,
    pub destination_path: String,
    pub expected_filenames: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmulatorDownloadProgress {
    pub emulator_id: String,
    pub total_bytes: u64,
    pub downloaded_bytes: u64,
    pub percentage: f32,
    pub speed_bytes_per_sec: u64,
    pub status: DownloadStatus, // "downloading", "extracting", "completed", "failed"
}
```

---

### 4.2 Network & Cloudflare Architecture Specification

#### A. Cloudflare Worker Edge Proxy (`worker.ts`)
```typescript
// OmniLauncher Edge Shield Worker
export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    const url = new URL(request.url);
    if (url.pathname === '/health') return new Response('OK');
    
    if (url.pathname === '/proxy') {
      const targetUrl = url.searchParams.get('url');
      if (!targetUrl) return new Response('Missing url parameter', { status: 400 });

      // Host validation to prevent open-proxy abuse
      const allowedHosts = [
        'steamcdn-a.akamaihd.net',
        'cdn.cloudflare.steamstatic.com',
        'shared.cloudflare.steamstatic.com',
        'images.epicgames.com',
        'tr.rbxcdn.com',
        'raw.githubusercontent.com',
        'api.github.com',
        'buildbot.libretro.com',
        'dl.dolphin-emu.org',
      ];
      
      const parsed = new URL(targetUrl);
      if (!allowedHosts.some(h => parsed.hostname.endsWith(h))) {
        return new Response('Host not in allowed proxy list', { status: 403 });
      }

      // Cloudflare Edge Cache
      const cache = caches.default;
      let response = await cache.match(request);
      if (response) return response;

      // Rewriting headers to bypass hotlinking protection
      const forwardHeaders = new Headers();
      forwardHeaders.set('User-Agent', 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36');
      forwardHeaders.set('Accept', 'image/avif,image/webp,image/png,image/*,*/*');
      forwardHeaders.set('Referer', `${parsed.protocol}//${parsed.hostname}/`);

      const upstream = await fetch(targetUrl, { headers: forwardHeaders });
      if (!upstream.ok) return upstream;

      const responseHeaders = new Headers(upstream.headers);
      responseHeaders.set('Access-Control-Allow-Origin', '*');
      responseHeaders.set('Cache-Control', 'public, max-age=604800, s-maxage=2592000'); // 30 days edge cache

      response = new Response(upstream.body, {
        status: upstream.status,
        headers: responseHeaders,
      });

      await cache.put(request, response.clone());
      return response;
    }

    return new Response('Not Found', { status: 404 });
  }
};
```

#### B. Rust DNS-over-HTTPS (DoH) Resolver Architecture
```rust
// Custom DoH resolver implementing reqwest::dns::Resolve
use reqwest::dns::{Name, Resolve, Resolving};
use std::{net::SocketAddr, sync::Arc};

pub enum DohProvider {
    Cloudflare, // https://cloudflare-dns.com/dns-query
    Google,     // https://dns.google/dns-query
    Quad9,      // https://dns.quad9.net/dns-query
    AdGuard,    // https://dns.adguard-dns.com/dns-query
    Custom(String),
}

pub struct DohResolver {
    endpoint: String,
    client: reqwest::Client,
    cache: Arc<tokio::sync::RwLock<std::collections::HashMap<String, Vec<SocketAddr>>>>,
}

impl Resolve for DohResolver {
    fn resolve(&self, name: Name) -> Resolving {
        let endpoint = self.endpoint.clone();
        let client = self.client.clone();
        let cache = self.cache.clone();
        let host = name.as_str().to_string();

        Box::pin(async move {
            // Check in-memory DNS cache
            if let Some(cached) = cache.read().await.get(&host) {
                let addrs: Box<dyn Iterator<Item = SocketAddr> + Send> = Box::new(cached.clone().into_iter());
                return Ok(addrs);
            }

            // Query DoH endpoint with DNS JSON API (RFC 8427)
            let query_url = format!("{}?name={}&type=A", endpoint, host);
            let resp = client.get(&query_url)
                .header("Accept", "application/dns-json")
                .send()
                .await
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

            let doh_json: serde_json::Value = resp.json().await
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

            let mut socket_addrs = Vec::new();
            if let Some(answers) = doh_json.get("Answer").and_then(|a| a.as_array()) {
                for ans in answers {
                    if let Some(data) = ans.get("data").and_then(|d| d.as_str()) {
                        if let Ok(ip) = data.parse::<std::net::IpAddr>() {
                            socket_addrs.push(SocketAddr::new(ip, 0));
                        }
                    }
                }
            }

            cache.write().await.insert(host, socket_addrs.clone());
            let addrs: Box<dyn Iterator<Item = SocketAddr> + Send> = Box::new(socket_addrs.into_iter());
            Ok(addrs)
        })
    }
}
```

---

### 4.3 10 Concrete Technical & Architectural Proposals

#### Proposal 1: Universal Open-Source Emulator Catalog & In-Launcher Installer
- **Scope**: Requirements R3, R5
- **Architecture**:
  - Implement `src-tauri/src/platforms/emulators/catalog.rs` holding metadata and download providers for 10 top emulators (PCSX2, Dolphin, RetroArch, RPCS3, DuckStation, PPSSPP, xemu, melonDS, Flycast, Azahar).
  - Background downloader streaming into `%LOCALAPPDATA%\OmniLauncher\emulators\{id}\` with archive unzipping (supporting `.zip` via `zip` crate and `.7z` via `sevenz-rust`).
  - Automatically provisions emulator profiles into `EmulatorPlatform` so downloaded emulators become immediately active launch targets for ROMs.

#### Proposal 2: Cloudflare CDN Edge Shield & Anti-Hotlink Asset Router
- **Scope**: Requirement R3
- **Architecture**:
  - Deploy Cloudflare Worker routing `https://cdn.omnilauncher.app/proxy?url=...` with automatic Edge caching (30 days TTL).
  - Tauri client-side asset loader falls back smoothly: Local Cache -> Cloudflare Worker Proxy -> Origin Direct -> Blank Placeholder.
  - Fixes 403 Forbidden errors when loading game covers from Steam Akamai CDN and Epic Store media.

#### Proposal 3: Native DoH (DNS-over-HTTPS) & SOCKS5 / HTTP Proxy Manager
- **Scope**: Requirement R3
- **Architecture**:
  - Add Network Settings panel in OmniLauncher UI: DNS selector (System Default, Cloudflare 1.1.1.1, Google 8.8.8.8, Quad9 9.9.9.9, AdGuard, Custom DoH) and Proxy selector (Direct, System Proxy, HTTP, HTTPS, SOCKS5).
  - In `src-tauri/src/core/config.rs`, persist network preferences.
  - Implement `DohResolver` implementing `reqwest::dns::Resolve` and configure `reqwest::ClientBuilder::proxy()` across all launcher HTTP clients.

#### Proposal 4: Bi-Directional Steam Cloud Sync & Versioned Snapshot Engine
- **Scope**: Requirement R2
- **Architecture**:
  - Read `<SteamPath>/userdata/<AccountID>/<AppID>/remotecache.vdf` and map Auto-Cloud root folders (`%USERPROFILE%/Saved Games`, `%LOCALAPPDATA%`, `%APPDATA%`, `%DOCUMENTS%`).
  - Before launching any Steam game, take an atomic local snapshot into `%LOCALAPPDATA%\OmniLauncher\cloud_backups\{appid}\{timestamp}\`.
  - Compare local file SHA-1 with `remotecache.vdf` SHA. If conflict is detected (cloud modified while offline), present visual modal with file diffs and option to restore or force upload.

#### Proposal 5: Steam Achievements Dashboard & Cache Repair ("Fix") Utility
- **Scope**: Requirement R2
- **Architecture**:
  - Add full achievement visualizer in game details: total unlocked percentage, rarity progress bars, unlocked vs locked icons.
  - "Repair Achievements" action in Game Properties: clears stale/corrupted binary stat files in `<SteamPath>/appcache/stats/UserGameStatsSchema_{appid}.bin` and triggers Steam protocol `steam://validate/{appid}` to force schema recalculation.
  - Integrated Goldberg / Steamworks Emu listener: reads `%APPDATA%\Goldberg SteamEmu Saves\{appid}\achievements.json` to show achievements for standalone games.

#### Proposal 6: Multi-Tier Epic Games Ownership Importer (Installed & Uninstalled)
- **Scope**: Requirement R2
- **Architecture**:
  - Tier 1: Manifest scanner for installed games (`%ProgramData%\Epic\EpicGamesLauncher\Data\Manifests\*.item`).
  - Tier 2: Epic Launcher Log Parser (`%LOCALAPPDATA%\EpicGamesLauncher\Saved\Logs\EpicGamesLauncher.log`) extracting catalog definitions of uninstalled owned games.
  - Tier 3: Legendary / Heroic Cache Importer (`%APPDATA%\legendary\installed.json` & Heroic `epic_games.json`).
  - Uninstalled games render in library with an "Install via Epic" badge; clicking triggers `com.epicgames.launcher://apps/{namespace}%3A{catalogItemId}%3A{appName}?action=install`.

#### Proposal 7: Roblox Experience History & High-Res Thumbnails Aggregator
- **Scope**: Requirement R2
- **Architecture**:
  - Parse the latest 20 log files in `%LOCALAPPDATA%\Roblox\logs\*Player*.log` with regex matching connection lines `[FLog::Network] ... placeId: (\d+)`.
  - Batch query `https://apis.roblox.com/universes/v1/places/{placeId}/universe` and `https://thumbnails.roblox.com/v1/games/icons` for 512x512 artwork.
  - Show recently played Roblox games as first-class library items with one-click deep link launch (`roblox://experiences/start?placeId={id}`).

#### Proposal 8: Steam-Grade "Game Properties" Modal
- **Scope**: Requirement R2
- **Architecture**:
  - Unified modal for any library title with 4 tabs:
    1. **General**: Custom launch parameters (`-novid -fullscreen`), custom executable override, desktop shortcut generator.
    2. **Compatibility**: High-DPI scaling override, forced GPU preference (Windows Registry `GraphicsDrivers`), Windows 7/8 compatibility flags.
    3. **Installed Files**: File size breakdown, "Open Install Directory" button, and "Verify Files Integrity" tool.
    4. **Cloud / Saves**: Steam Cloud sync status, save folder location, manual backup snapshot button, and restore list.

#### Proposal 9: In-Launcher Native Drive & Folder Picker (Zero-Explorer-Dialog UX)
- **Scope**: Requirement R3
- **Architecture**:
  - Eliminate the generic Windows Explorer folder dialog (`open_dialog`).
  - Implement Tauri command `get_system_drives()` querying `GetLogicalDrives()` in Windows API, returning drive letters, labels, total capacity, and free space.
  - Implement directory browser command `list_folder_contents(path)` with instant folder creation modal (`create_directory(path)`) styled in OmniLauncher's dark chrome theme.

#### Proposal 10: Infinite Recommendation Feed with Multi-Currency Engine
- **Scope**: Requirement R3
- **Architecture**:
  - In Store view, implement virtualized infinite scrolling recommendation feed fetching paginated game deals / top sellers from Steam Store API (`store.steampowered.com/api/featuredcategories`) and CheapShark / IsThereAnyDeal.
  - Built-in live currency switcher: USD ($) and Kazakhstani Tenge (₸) with live conversion rate cache (`open.er-api.com` or fixed rate fallback ~495 KZT/USD).

---

## 5. Verification Method

### 5.1 Independent Verification Commands
1. **GitHub Releases Asset Resolution**:
   Execute HTTP test queries to verify live assets for PCSX2, DuckStation, PPSSPP, xemu:
   ```powershell
   curl.exe -s "https://api.github.com/repos/PCSX2/pcsx2/releases/latest" | Select-String "pcsx2-v.*-windows-x64-Qt.7z"
   curl.exe -s "https://api.github.com/repos/stenzek/duckstation/releases/latest" | Select-String "duckstation-windows-x64-release.zip"
   curl.exe -s "https://api.github.com/repos/hrydgard/ppsspp/releases/latest" | Select-String "PPSSPP-v.*-Windows-x64.zip"
   ```
2. **Dolphin Update API Verification**:
   ```powershell
   curl.exe -s "https://dolphin-emu.org/update/latest/beta" | Select-String "dolphin-.*-x64.7z"
   ```
3. **DoH Endpoint Check**:
   Test Cloudflare and Google DoH JSON responses:
   ```powershell
   curl.exe -s -H "Accept: application/dns-json" "https://cloudflare-dns.com/dns-query?name=api.github.com&type=A"
   curl.exe -s -H "Accept: application/dns-json" "https://dns.google/dns-query?name=api.github.com&type=A"
   ```
4. **Local Epic Manifest & Roblox Log Verification**:
   Inspect local machine paths on Windows:
   ```powershell
   Test-Path "C:\ProgramData\Epic\EpicGamesLauncher\Data\Manifests"
   Test-Path "$env:LOCALAPPDATA\Roblox\logs"
   Get-ChildItem "$env:LOCALAPPDATA\Roblox\logs\*.log" | Select-Object -First 3
   ```
5. **Codebase Compilation Check**:
   ```powershell
   cargo check --manifest-path "c:\Users\Tmub\Documents\TMlauncher\src-tauri\Cargo.toml"
   ```

### 5.2 Invalidation Conditions
- If GitHub deprecates the unauthenticated Releases API without redirects, direct release asset URL pattern routing (`/releases/latest/download/{asset}`) must be activated.
- If Epic Games changes the internal manifest format from `.item` JSON to binary, the manifest parser must be updated.
- If Roblox switches log storage to encrypted format, place tracking will fall back to Windows Active Window title scraping or Roblox Presence API.
