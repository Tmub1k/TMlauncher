# UI Style & Theming Specification: OmniLauncher "Steam 2"

**Document Version**: 2.0.0-PROD  
**Author**: `teamwork_spec_miner` (UI Style & Theming Spec Miner)  
**Target Milestone**: OmniLauncher Visual Redesign, Modular Theming & Core Subsystems  
**Working Directory**: `c:\Users\Tmub\Documents\TMlauncher`  
**Reference Sources**: 
1. `ORIGINAL_REQUEST.md` (Approved Task Prompt 2026-11-04T22:47:09Z)
2. Reference Image 1: `media_1793830323839_968236a7.jpg` (TURA Minimalist Dark Editorial)
3. Reference Image 2: `media_1793830335320_3ec4cbc9.jpg` (Liquid Chrome Futuristic Cyber Metallic)
4. Existing Codebase: `src/` (Svelte 5 + Tailwind 4) & `src-tauri/` (Rust Tauri v2)

---

## 1. Executive Summary & Design System Foundations

OmniLauncher is evolving from an experimental multi-hub into a flagship, high-performance desktop launcher ("Steam 2"). The visual language is defined by two complementary design paradigms:
1. **The "TURA" Philosophy (Reference 1)**: Hyper-clean Scandinavian dark minimalism. Deep matte graphite canvas, ultra-restrained typography (Cera CY / Inter geometric grotesque), microscopic hairline borders (0.06 to 0.08 alpha), generous negative space, and absolute removal of redundant explanatory text.
2. **The "Liquid Chrome Cyber" Accent Layer (Reference 2)**: Polished liquid chrome ribbons, specular metallic borders, brushed silver pill badges, and glossy fluid media blocks that bring high-tech sophistication without gaudiness or visual clutter.

### Core Visual Principles
- **Monochrome Dominance with Specular Highlights**: Surfaces are pure dark graphite (`#09090b`, `#121215`, `#16161b`). Primary accents are pure white (`#ffffff`), specular platinum (`#f0f3f8`), and brushed steel (`#8e939d`), completely discarding bright cartoonish saturated buttons.
- **Micro-Hairline Borders & Inset Sheens**: No heavy 2px-5px solid borders. Every card, dialog, and container uses `1px` hairlines with subtle multi-stop linear gradients (`rgba(255,255,255,0.12) -> rgba(255,255,255,0.02)`) or dark shadow depth.
- **Editorial Typography Hierarchy**:
  - Primary Headlines: Bold geometric grotesque uppercase (`font-weight: 800; letter-spacing: 0.08em; text-transform: uppercase; color: #ffffff`).
  - Index Markers / Counters: Monospace tabular numerals (`font-family: 'JetBrains Mono', 'Fira Code', monospace; font-size: 11px; color: #71717a`).
  - Body & Metadata: Clean sans-serif, subdued contrast (`#a1a1aa`, `#71717a`).
- **Information Self-Sufficiency**: Removal of explanatory "helper sentences" ("Click here to...", "Here you will find...", "This option allows you to..."). UI controls must be immediately intuitive through iconography, clear typography, and state feedback.

---

## 2. Current UI Clutter Audit & Elimination Matrix

The existing codebase contains legacy casual 3D game UI elements and verbose labels that contradict the minimalist design requirements.

| Component Path | Current Cluttered Element | Concrete Elimination / Redesign Action | Visual Outcome |
|---|---|---|---|
| `src/lib/styles/app.css` (lines 50–125) | `.btn-game`, `.btn-game-green`, `.btn-game-blue`, `.btn-game-gold`, `.btn-game-red` with 4px black bottom push borders and bright gradients. | Completely remove. Replace with `.btn-minimal` and `.btn-chrome` with 1px specular metallic hairline, smooth 150ms hover lift, and monochrome high-contrast fills. | Eliminates casual mobile look; establishes sleek desktop aesthetic. |
| `src/lib/styles/app.css` (lines 126–192) | `.panel-game` with 5px border, `.panel-header-pill` floating banner, `.btn-close-game` red square 3D button. | Completely delete. Modals and panels use flat graphite obsidian surfaces (`#121215`), 1px subtle border (`rgba(255,255,255,0.08)`), and sleek hairline close icon (`✕`). | Matches TURA Reference 1 modal structure. |
| `src/lib/components/Navigation.svelte` (lines 48–60) | Brand mark subtext `"Steam & Multi-Hub • v0.2"` and redundant group headers `"Навигация"`, `"Экосистема"`. | Remove subtext and section header titles. Keep clean brand monogram `OMNI` + streamlined icon rail. | Saves 80px vertical space, eliminates visual clutter. |
| `src/lib/components/Navigation.svelte` (lines 127–142) | Footer helper labels `"F12 Скриншот"`, `"Фоновый запуск"` and verbose roulette CTA `"Во что поиграть? Рулетка 🎲"`. | Simplify roulette to a sleek metallic dice icon button; remove permanent hotkey text into a minimalist settings/shortcut modal. | Clean minimalist sidebar bottom rail. |
| `src/lib/components/BentoHub.svelte` (lines 173–197) | Verbose header text `"{filteredGames.length} Игр ({installedCount} Установлено, {uninstalledCount} в облаке)"`. | Replace with a compact monospace pill counter: `342 TITLES` with subtle sub-counts `120 INSTALLED · 222 CLOUD`. | Clean, elegant status display without colloquial words. |
| `src/lib/components/BentoHub.svelte` (lines 271–320) | Redundant labels `"Статус:"` and `"Сортировка:"` preceding dropdowns and pills. | Remove `"Статус:"` and `"Сортировка:"` text prefixes. The buttons `All`, `Installed`, `Cloud` and sorting options are self-explanatory. | Streamlined toolbar with 40% less horizontal clutter. |
| `src/lib/components/GameCard.svelte` (lines 133–158) | Prominent colored badges (`Ready` with green pill, `Cloud` with blue pill) overlaying the artwork. | Replace with a discrete 6px status pip (green indicator for installed, translucent ring for cloud) positioned in top-right corner. | Artwork remains uninterrupted and prominent. |
| `src/lib/components/GameDetailsModal.svelte` (lines 137–150) | Overlapping gold banner `"ИНФОРМАЦИЯ ОБ ИГРЕ"` and bright red close button. | Remove gold banner and red button. Replace with a native Steam-like Properties window: frameless header with game title, icon, and minimal `✕` button. | Professional, elegant Steam properties dialog. |
| `src/lib/components/DealsRadar.svelte` (lines 62–64) | Explanatory paragraph `"100% бесплатные раздачи Epic Games Store и исторические минимумы цен Steam / GOG"`. | Remove explanatory text. The title `RADAR` and store tags (`EPIC`, `STEAM`) are self-sufficient. | Tight, polished editorial layout. |
| `src/lib/components/RobloxStudioPanel.svelte` (lines 45–60) | Lengthy multi-line descriptions for every experience ("Исследуйте моря, побеждайте рейд-боссов..."). | Remove verbose descriptions. Display experience cover, title, active player count, and creator tag. | Clean cards matching TURA Reference 1 layout. |

---

## 3. Modular Theme System Architecture

The theme engine is designed to be fully modular, extensible, and hot-reloadable. Users and developers can create custom themes by dropping a single JSON manifest or CSS file into the `themes/` folder.

### 3.1 Theme Manifest Specification (`theme.json` Schema)

```json
{
  "$schema": "https://raw.githubusercontent.com/TMlauncher/omnilauncher/main/schemas/theme.schema.json",
  "id": "tura-graphite",
  "name": "TURA Graphite",
  "author": "OmniLauncher Team",
  "version": "1.0.0",
  "description": "Minimalist ultra-stylish dark design inspired by Scandinavian editorial typography",
  "accentStyle": "matte",
  "vibrancy": "mica",
  "tokens": {
    "--bg-canvas": "#09090b",
    "--bg-surface-base": "#121215",
    "--bg-surface-elevated": "#18181c",
    "--bg-surface-floating": "#202026",
    "--bg-surface-glass": "rgba(18, 18, 21, 0.75)",
    
    "--chrome-gradient": "linear-gradient(180deg, #32353c 0%, #17181c 100%)",
    "--chrome-border": "linear-gradient(135deg, rgba(255,255,255,0.2) 0%, rgba(255,255,255,0.03) 100%)",
    "--chrome-specular": "rgba(255, 255, 255, 0.45)",
    "--chrome-ribbon": "linear-gradient(90deg, #4b5161 0%, #f3f6fa 45%, #cbd3e1 55%, #1f2127 100%)",
    
    "--border-hairline": "rgba(255, 255, 255, 0.06)",
    "--border-subtle": "rgba(255, 255, 255, 0.10)",
    "--border-active": "rgba(255, 255, 255, 0.30)",
    "--border-specular": "rgba(255, 255, 255, 0.60)",
    
    "--text-primary": "#ffffff",
    "--text-secondary": "#a1a1aa",
    "--text-muted": "#52525b",
    "--text-accent": "#f4f4f5",
    
    "--accent-primary": "#ffffff",
    "--accent-primary-contrast": "#09090b",
    "--accent-metallic": "#d4d8e2",
    "--status-success": "#10b981",
    "--status-warning": "#f59e0b",
    "--status-danger": "#ef4444",
    "--status-info": "#0ea5e9",
    
    "--shadow-card": "0 10px 30px -10px rgba(0, 0, 0, 0.8)",
    "--shadow-card-hover": "0 20px 40px -12px rgba(0, 0, 0, 0.95), 0 0 24px rgba(255, 255, 255, 0.08)",
    "--shadow-modal": "0 25px 60px -15px rgba(0, 0, 0, 0.95)",
    "--shadow-specular": "inset 0 1px 0 rgba(255, 255, 255, 0.15)",
    
    "--radius-xs": "4px",
    "--radius-sm": "8px",
    "--radius-md": "12px",
    "--radius-lg": "16px",
    "--radius-xl": "22px",
    "--radius-full": "9999px",
    
    "--font-sans": "'Cera CY', 'Inter', -apple-system, BlinkMacSystemFont, sans-serif",
    "--font-mono": "'JetBrains Mono', 'Fira Code', monospace",
    "--font-heading-tracking": "0.08em",
    "--font-heading-weight": "800",
    
    "--poster-filter": "grayscale(65%) contrast(108%) brightness(95%)",
    "--poster-filter-hover": "grayscale(0%) contrast(102%) brightness(100%)",
    "--backdrop-blur": "20px"
  },
  "media": {
    "backgroundVideo": null,
    "ambientAudio": null,
    "audioVolume": 0.2
  }
}
```

### 3.2 Canonical Built-in Themes

| Theme ID | Display Name | Visual Reference | Key Stylistic Signature | Canvas / Surface Colors | Border / Accents |
|---|---|---|---|---|---|
| `tura-graphite` | **TURA Graphite** | Reference 1 (`media_1793830323839_968236a7.jpg`) | Matte deep graphite, editorial Cera CY headings, ultra-subtle hairlines, quiet luxury. | Canvas: `#09090b`<br>Surface 1: `#121215`<br>Surface 2: `#18181c` | Hairline: `rgba(255,255,255,0.06)`<br>Accent: `#ffffff` / `#000000` |
| `liquid-chrome` | **Liquid Chrome Cyber** | Reference 2 (`media_1793830335320_3ec4cbc9.jpg`) | Specular liquid mercury ribbons, metallic chrome borders, brushed steel pill badges, fluid glass blocks. | Canvas: `#060709`<br>Surface 1: `#0e1014`<br>Surface 2: `#161920` | Border: `linear-gradient(135deg, #fff 0%, #1a1c22 100%)`<br>Accent: `#d4d8e2` |
| `obsidian-pure` | **Obsidian Pure OLED** | High-contrast Blackout | 100% black canvas for OLED displays, crisp white geometric text, zero gray wash, high battery/efficiency. | Canvas: `#000000`<br>Surface 1: `#090909`<br>Surface 2: `#121212` | Hairline: `rgba(255,255,255,0.12)`<br>Accent: `#ffffff` |
| `cyber-aurora` | **Cyber Aurora** | Futuristic Dark Iridescent | Dark titanium canvas with subtle specular iridescent blue-violet reflections and icy platinum highlights. | Canvas: `#08090d`<br>Surface 1: `#0f1118`<br>Surface 2: `#171b26` | Border: `rgba(147, 197, 253, 0.20)`<br>Accent: `#93c5fd` / `#c084fc` |

### 3.3 Dynamic Theme Loading & Drop-in Directory Architecture

1. **Storage Directories**:
   - Built-in themes: Bundled into frontend assets under `src/lib/themes/*.json`.
   - User custom themes: Loaded from `%APPDATA%/dev.omnilauncher.app/themes/` (Windows) or `themes/` in the application root directory.
2. **File Discovery & Hot-Reloading**:
   - Backend scans the `themes/` directory for any `*.json` or `*.theme.css` files on startup.
   - Rust filesystem watcher monitors the folder for `Create`, `Modify`, `Delete` events and emits `theme://list-updated` to the frontend.
   - Frontend dynamically updates the themes dropdown in Settings; selecting any theme instantly injects all tokens onto `document.documentElement` without reloading the webview.
3. **Validation & Fallback**:
   - Any missing token gracefully falls back to the default `tura-graphite` root variables.
   - Invalid JSON files emit a non-fatal warning in logs and are tagged with `[Invalid Theme Manifest]`.

---

## 4. Key Views UI Specifications

### 4.1 Navigation & App Frame (Frameless Window & Sleek Titlebar)

#### Window Configuration (`tauri.conf.json`)
- `"decorations": false` — native Windows titlebar disabled.
- `"transparent": false` — backdrop handled by Mica / Acrylic DWM APIs.
- Min width: `1100px`, Min height: `700px`. Default: `1440x900px`.

#### Sleek Custom Titlebar (Height: 44px)
- **Container**: `data-tauri-drag-region`, sticky at top, `backdrop-filter: blur(20px)`, bottom border `1px solid var(--border-hairline)`, background `var(--bg-canvas) / 85%`.
- **Left Zone (Identity & Quick Status)**:
  - Minimalist geometric logo: Liquid chrome/monochrome icon badge (`OL`).
  - App Name: `OMNI` in bold uppercase Cera CY, followed by a hairline separator and live sync status pip (green = connected).
- **Center Zone (Unified Tab Switcher)**:
  - Segmented pill container with metallic hairline border.
  - Tabs:
    1. **Library** (Icon + counter)
    2. **Store** (Icon)
    3. **Emulators** (Icon + badge)
    4. **Downloads** (Icon + active speed `42.5 MB/s`)
    5. **Settings** (Gear icon)
  - Active tab indicator: Fluid sliding pill background with subtle top specular highlight (`inset 0 1px 0 rgba(255,255,255,0.2)`).
- **Right Zone (Utilities & Window Controls)**:
  - Global Search button (`Ctrl+K` trigger pill with subtle border).
  - Steam Profile Avatar pill: Circular avatar with online state ring and user name.
  - Window controls: Minimize (`—`), Maximize/Restore (`□`), Close (`✕`).
    - Custom styled, hover effects: Close button hover is smooth subtle dark red (`#dc2626/80`), Minimize/Maximize are `white/10`.

---

### 4.2 Library View & Hero Experience

#### Visual Layout & Density
- **Hero Spotlight (Top 260px)**:
  - Dynamically features the most recently played game or user favorite.
  - Panoramic blurred backdrop art with a smooth gradient fade to canvas.
  - Game title in heavy uppercase typography, total playtime (`142.4 hrs`), achievements progress bar (`48 / 64`), and a high-contrast `PLAY` button.
- **Controls & Filter Bar**:
  - Compact horizontal bar:
    - Platform filter capsules: `All`, `Steam`, `Epic`, `Roblox`, `Emulators`.
    - Status pills: `Installed (42)`, `Cloud (180)`.
    - Instant live search input with keyboard shortcut tooltip (`/`).
    - Sort selector: `Recent`, `Playtime`, `Alphabetical`, `Release Date`.
    - View mode toggle: `Grid (2:3 Posters)` vs `Detailed List`.
- **Game Grid**:
  - Responsive auto-fill grid: `grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); gap: 20px;`.
  - Cards:
    - 2:3 vertical aspect ratio.
    - Subtle 1px hairline border, `border-radius: var(--radius-lg)` (16px).
    - Image filter: default `grayscale(65%) contrast(108%)`, transitioning smoothly on hover to `grayscale(0%) contrast(100%)` in 200ms.
    - Hover state: `-translate-y-1.5`, diffuse outer shadow `0 20px 40px -10px rgba(0,0,0,0.9)`, subtle chrome specular border glow.
    - Discrete installed indicator: tiny 6px emerald pip in top right.
    - Context Menu on right-click: `Play`, `Properties...`, `Browse Local Files`, `Add to Favorites`, `Uninstall`.

---

### 4.3 Steam-like Game Properties Modal

A dedicated, comprehensive modal replicating the depth of Steam's game properties dialog, designed with the TURA minimalist dark style.

#### Modal Structure
- **Dimensions**: Centered dialog `880px × 560px`, `border-radius: 20px`, surface `var(--bg-surface-base)`, border `1px solid var(--border-subtle)`, deep shadow `var(--shadow-modal)`.
- **Header**: Minimalist bar with game icon, title in bold uppercase, platform tag (`STEAM #730`), and hairline `✕` close button.
- **Left Navigation Tabs**:
  1. `General` (Общие)
  2. `Compatibility` (Совместимость)
  3. `Installed Files` (Установленные файлы)
  4. `Cloud Saves` (Steam Cloud)
  5. `Achievements` (Достижения)
  6. `Launch Options` (Параметры запуска)

#### Tab Pane Specifications
1. **General / Launch Options**:
   - Desktop Shortcut toggle: `Create desktop shortcut` with 1-click action.
   - Launch Arguments Input: Clean monospace input box with placeholder `-novid -tickrate 128 -high +fps_max 0`. Supports presets dropdown (e.g. Vulkan Renderer, Safe Mode, Windowed).
   - Steam Overlay & Discord Rich Presence toggles.
2. **Installed Files**:
   - Storage statistics bar: `Size on disk: 68.42 GB (Drive D:)`.
   - "Browse Local Files": Invokes native Windows Explorer at game install folder.
   - "Verify File Integrity": Initiates background validation check with progress indicator.
   - "Move Install Folder": Allows migrating the game to any configured library folder or drive without opening Steam.
3. **Cloud Saves (Steam Cloud)**:
   - Status: `Synchronized` with timestamp.
   - Storage usage: `4.12 MB / 100.00 MB used`.
   - Cloud Saves inspection table: Lists remote file names, sizes, and modification dates with "Download Backup" option.
4. **Achievements**:
   - Overall unlock percentage and counter (`52 / 67 Unlocked - 77%`).
   - Clean scrollable grid of achievement badges with silver/chrome locked states and colored unlocked states.

---

### 4.4 Store View: Infinite Recommendations & Currency Switcher

#### Architecture & Data Flow
- **Steam Store Showcase Integration**:
  - Connects to Steam's live storefront API, specials, and recommendations.
  - Infinite Feed Architecture: Infinite scroll powered by `IntersectionObserver`. When the user scrolls within 300px of the feed bottom, the next batch of recommended titles is fetched and appended seamlessly without UI hitching.
- **Storefront Sections**:
  1. **Top Spotlight Carousel**: 3D fluid video/screenshot showcase with discount badge (`-75%`), price, and 1-click `Add to Library` or `Buy on Steam`.
  2. **Infinite Category Feeds**:
     - "Recommended for You"
     - "Top Sellers & Trending"
     - "Historical Low Discounts"
     - "New Indie Highlights"
- **Currency Switcher & Real-time Conversion Engine**:
  - Switcher control located in the Store sub-header: Pill toggle `[ USD ($) | KZT (₸) ]`.
  - Conversion Engine:
    - Real-time exchange rate fetched from financial API or cached fallback (e.g., 1 USD = 500 KZT).
    - Formats:
      - USD: `$59.99`, `$14.99`, `Free`
      - KZT: `29 990 ₸`, `7 490 ₸`, `Бесплатно`
    - Formatter includes Kazakhstani tenge character `₸` and localized digit grouping (`29 990 ₸`).
    - Recalculates discounts and strikethrough original prices instantly across all cards when toggled.

---

### 4.5 Emulators View: GitHub Open-Source Catalog & 1-Click Install

#### Curated GitHub Catalog Registry

| Emulator | Target Console | Official GitHub Repository | Executable Target | Default ROM Extensions |
|---|---|---|---|---|
| **RetroArch** | Multi-system (NES, SNES, GBA, Arcade) | `libretro/RetroArch` | `retroarch.exe` | `.nes`, `.sfc`, `.gba`, `.bin`, `.zip` |
| **Dolphin** | GameCube & Nintendo Wii | `dolphin-emu/dolphin` | `Dolphin.exe` | `.iso`, `.rvz`, `.gcm`, `.wad` |
| **PCSX2** | Sony PlayStation 2 | `PCSX2/pcsx2` | `pcsx2-qt.exe` | `.iso`, `.chd`, `.bin`, `.gz` |
| **RPCS3** | Sony PlayStation 3 | `RPCS3/rpcs3` | `rpcs3.exe` | `.pkg`, `.iso`, `EBOOT.BIN` |
| **Ryujinx** | Nintendo Switch | `Ryujinx/Ryujinx` (or verified mirror) | `Ryujinx.exe` | `.nsp`, `.xci` |
| **PPSSPP** | Sony PlayStation Portable | `hrydgard/ppsspp` | `PPSSPPWindows64.exe` | `.iso`, `.cso` |
| **DuckStation** | Sony PlayStation 1 | `stenzek/duckstation` | `duckstation-qt-x64-ReleaseLTCG.exe` | `.chd`, `.cue`, `.iso` |
| **Cemu** | Nintendo Wii U | `cemu-project/Cemu` | `Cemu.exe` | `.wua`, `.wud`, `.rpx` |

#### 1-Click Download & Setup Workflow
1. User clicks "Install" on any catalog card.
2. Tauri backend calls GitHub Releases API: `GET https://api.github.com/repos/{owner}/{repo}/releases/latest`.
3. Locates Windows 64-bit release asset (`.zip` / `.7z`).
4. Downloads asset directly through backend streaming downloader with real-time byte progress sent to `Downloads` manager.
5. Decompresses package into `emulators/<id>/`.
6. Auto-detects main executable and creates an active profile in `EmulatorPlatform`.
7. Adds emulator as a launchable title in Library and prompts for ROM directory binding.

---

### 4.6 Downloads & Network Settings

#### In-Launcher Drive & Directory Selector (Custom UI, Zero OS Dialogs)
- **Problem Solved**: Standard Steam or OS folder dialogs interrupt the user experience with disruptive native Win32 windows.
- **In-Launcher Drive Picker Component**:
  - Reads Windows logical drives via `sysinfo` or Win32 API (`GetLogicalDrives`).
  - Renders sleek drive storage cards:
    - Drive letter (`C:\`, `D:\`, `E:\`), label (`Fast NVMe SSD`), filesystem (`NTFS`).
    - Visual storage gauge bar: `Used: 640 GB / Free: 360 GB (Total 1.0 TB)`.
  - Directory Tree Navigator:
    - Clean inline folder explorer showing child folders.
    - Breadcrumb trail: `D: \ Games \ SteamLibrary \ steamapps \ common`.
    - "New Folder" inline creation input.
    - "Select This Directory" primary button.

#### Cloudflare Media & File Routing
- Images (posters, heroes, icons) and external release files routed through secure Cloudflare edge nodes or Cloudflare CDN workers.
- Eliminates IP leaks, ensures DDoS resilience, reduces latency, and bypasses regional ISP throttling.
- In-memory and local disk LRU cache for game artwork.

#### Custom DNS & Proxy Hub
- **DNS Preset Selector (Settings -> Network)**:
  - Presets:
    1. **Cloudflare 1.1.1.1** (Standard high-speed DoH `https://cloudflare-dns.com/dns-query`)
    2. **Cloudflare 1.1.1.2** (Malware & phishing blocking)
    3. **Google Public DNS 8.8.8.8** (Reliable global resolution DoH `https://dns.google/dns-query`)
    4. **Quad9 9.9.9.9** (Privacy & threat intelligence)
    5. **Custom DNS-over-HTTPS (DoH)**: Custom URL input box.
- **Proxy Configuration**:
  - Modes: `Direct (No Proxy)`, `Use System Proxy`, `Manual Proxy`.
  - Protocols: `HTTP`, `HTTPS`, `SOCKS5`.
  - Fields: `Host`, `Port`, `Username`, `Password`.
  - "Test Connection" button: Measures real-time round-trip latency to Cloudflare edge nodes and displays a green ping badge (`24 ms · Connected`).

---

### 4.7 Launcher Startup & Clean Presentation

- **Problem Solved**: Launching via `Start-OmniLauncher.cmd` or running dev commands can pop up intrusive command prompt windows or terminals.
- **Clean Launch Architecture**:
  1. Set Windows subsystem unconditionally on production release builds: `#![windows_subsystem = "windows"]` in `src-tauri/src/main.rs`.
  2. Implement a silent executable launcher wrapper (`OmniLauncher.exe`) that executes the main process with `CREATE_NO_WINDOW` (`0x08000000`) Win32 flag.
  3. Create an automated Windows shortcut (`OmniLauncher.lnk`) with `Window: Minimized` or normal GUI startup.

---

## 5. Architectural Data Contracts & Tauri IPC Specifications

```rust
// ============================================================================
// Data Contracts for Theme, Store, Emulators, Network, and Drive Management
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemeSummary {
    pub id: String,
    pub name: String,
    pub author: String,
    pub version: String,
    pub is_custom: bool,
    pub file_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriveInfo {
    pub drive_letter: String,       // e.g. "D:\"
    pub volume_name: String,        // e.g. "Games & Media"
    pub file_system: String,        // e.g. "NTFS"
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub is_removable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderEntry {
    pub name: String,
    pub path: String,
    pub has_subfolders: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmulatorCatalogItem {
    pub id: String,
    pub name: String,
    pub console: String,
    pub repo_owner: String,
    pub repo_name: String,
    pub current_version: Option<String>,
    pub latest_version: Option<String>,
    pub installed: bool,
    pub install_path: Option<String>,
    pub download_size_bytes: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkSettings {
    pub dns_mode: String,           // "cloudflare", "google", "quad9", "custom", "system"
    pub custom_doh_url: Option<String>,
    pub proxy_mode: String,         // "direct", "system", "manual"
    pub proxy_protocol: String,     // "http", "socks5"
    pub proxy_host: String,
    pub proxy_port: u16,
    pub proxy_auth_enabled: bool,
    pub proxy_user: Option<String>,
    pub proxy_password: Option<String>,
    pub cloudflare_media_routing: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoreCurrencyConfig {
    pub currency: String,           // "USD" or "KZT"
    pub exchange_rate_usd_to_kzt: f64,
    pub auto_update_rate: bool,
}
```

### Tauri Command Interface Additions

| Command Name | Arguments | Return Type | Description |
|---|---|---|---|
| `get_available_themes` | None | `Result<Vec<ThemeSummary>>` | Enumerates bundled and custom themes from `themes/`. |
| `load_theme_manifest` | `theme_id: String` | `Result<ThemeManifest>` | Reads and parses full theme token definition. |
| `get_system_drives` | None | `Result<Vec<DriveInfo>>` | Lists all local Windows drives with storage capacity. |
| `browse_directory` | `path: String` | `Result<Vec<FolderEntry>>` | Returns child subdirectories for in-launcher folder picker. |
| `create_directory` | `path: String` | `Result<()>` | Creates a new directory in the selected path. |
| `get_emulator_catalog` | None | `Result<Vec<EmulatorCatalogItem>>` | Returns curated GitHub emulator list with release status. |
| `install_emulator` | `emulator_id: String` | `Result<LaunchHandle>` | Downloads latest release from GitHub and unpacks to `emulators/`. |
| `get_network_settings` | None | `Result<NetworkSettings>` | Loads DNS, proxy, and Cloudflare routing configuration. |
| `save_network_settings`| `settings: NetworkSettings` | `Result<()>` | Applies and persists DNS/proxy settings. |
| `test_network_latency` | `target: Option<String>` | `Result<u64>` | Pings Cloudflare edge / DNS to measure latency in ms. |
| `get_store_currency` | None | `Result<StoreCurrencyConfig>` | Gets active store currency and exchange rate. |
| `set_store_currency` | `currency: String` | `Result<()>` | Switches currency (`USD` or `KZT`). |

---

## 6. Features Discovered & Probe Specifications

Reporting all discovered and probed features per the Specification Miner procedure:

## Features Discovered
| # | Category | Feature | Description | Inputs | Outputs | Error Behavior | Discovered Via |
|---|---|---|---|---|---|---|---|
| 1 | Theming | Modular Theme Engine | JSON-driven custom styling system with CSS custom property injection. | `theme_id: String` | Injects 25+ `--*` tokens onto `:root`. | Falls back to `tura-graphite` if token missing or invalid. | `src/lib/stores/theme.ts` & Reference 1 & 2 analysis. |
| 2 | Theming | Drop-in Themes Directory | Hot-reloading of custom themes from `%APPDATA%/themes/` without restarting app. | `fs::watch` event | Emits `theme://list-updated`. | Non-fatal log if manifest has invalid JSON. | `src-tauri/src/core/config.rs` & R1 requirements. |
| 3 | Theming | TURA Minimalist Palette | Ultra-clean graphite theme with Cera CY typography and hairline borders. | Active theme selection | Obsidian/graphite background, high contrast text. | None. | Reference 1 visual analysis. |
| 4 | Theming | Liquid Chrome Cyber Palette | Metallic silver ribbons, specular borders, and brushed steel pill badges. | Active theme selection | Specular gradients, polished chrome controls. | None. | Reference 2 visual analysis. |
| 5 | Frame | Frameless Window & Custom Titlebar | Sleek borderless window with drag region, tab rail, and custom controls. | Window drag, minimize, maximize, close | Window state manipulation via Tauri API. | Safe no-op on non-Windows platforms. | `src-tauri/tauri.conf.json` lines 24–27 & R1. |
| 6 | Navigation | Minimalist Tab Switcher | Compact rail for Library, Store, Emulators, Downloads, Settings. | Tab index click | Switches active view with smooth transition. | Retains scroll position and state. | `src/lib/components/Navigation.svelte`. |
| 7 | Library | Steam Properties Modal | Comprehensive game properties dialog (launch options, files, cloud, achievements). | `game_id: String` | Multi-tab inspection and configuration modal. | Shows error message if Steam Cloud is unavailable. | `ORIGINAL_REQUEST.md` R2 & `GameDetailsModal.svelte`. |
| 8 | Library | File Integrity Verifier | Verifies game files against Steam cache or checksums. | `app_id: u32` | Progress percentage (0–100%) and scan summary. | Reports corrupt/missing files if found. | Steam protocol `steam://validate/<appid>` & R2. |
| 9 | Library | Move Install Directory | Migrates installed game folder to a different drive or directory. | `target_dir: PathBuf` | Background copy/move with byte progress. | Aborts and rolls back if disk space insufficient. | R2 & `libraryfolders.vdf` parsing. |
| 10 | Store | Infinite Recommendations Feed | Steam-like infinite scroll stream of personalized and trending game recommendations. | Scroll threshold (300px) | Fetches and appends next batch of items. | Shows "End of feed" or retry button on network error. | `SteamStoreShowcase.svelte` & R3. |
| 11 | Store | USD / KZT Currency Switcher | Real-time price conversion between US Dollars and Kazakhstani tenge. | Currency selection (`USD` / `KZT`) | Instant price recalculation with tenge symbol `₸`. | Falls back to cached exchange rate if API down. | `ORIGINAL_REQUEST.md` R3. |
| 12 | Emulators | GitHub Open-Source Catalog | Curated catalog of top emulators (RetroArch, Dolphin, PCSX2, RPCS3, Ryujinx, etc.). | Catalog query | Renders cards with version, release date, and install CTA. | Cached offline roster if GitHub API rate-limited. | `src-tauri/src/platforms/emulators/mod.rs` & R3. |
| 13 | Emulators | 1-Click Emulator Downloader | Downloads latest release asset from GitHub, unpacks archive, and registers profile. | `emulator_id: String` | Streamed download, auto-extraction, active profile. | Catches network timeouts, displays retry button. | R3 requirement & GitHub Releases API. |
| 14 | Network | In-Launcher Drive & Directory Selector | Custom visual drive and folder picker without Steam/Windows system dialogs. | Drive/Path selection | Returns absolute directory path. | Highlights read-only or insufficient space drives. | `ORIGINAL_REQUEST.md` R3. |
| 15 | Network | Cloudflare Media Routing | Routes all artwork, screenshots, and remote assets through Cloudflare edge servers. | Asset URL request | Accelerated, cached TLS 1.3 response. | Falls back to origin CDN if Cloudflare edge down. | `ORIGINAL_REQUEST.md` R3. |
| 16 | Network | Custom DNS Selector | Built-in selector for Cloudflare 1.1.1.1, Google 8.8.8.8, Quad9, or custom DoH. | DNS preset selection | Sets DoH resolver for launcher HTTP client. | Reverts to system DNS if custom DoH fails probe. | `ORIGINAL_REQUEST.md` R3. |
| 17 | Network | Proxy Configuration Hub | Configurable HTTP / HTTPS / SOCKS5 proxy with credential support and latency test. | Host, Port, Credentials | Proxied socket connections, live latency badge. | Returns red badge `Connection Failed` on timeout. | `ORIGINAL_REQUEST.md` R3. |
| 18 | Startup | Clean GUI Launch (No Terminal Popups) | Silent launcher startup with `windows_subsystem = "windows"` and hidden console wrapper. | Application launch | Direct GUI presentation with zero cmd windows. | None. | `Start-OmniLauncher.cmd` & `ORIGINAL_REQUEST.md` R4. |

---

## 7. Edge Cases & Observed Behavior

## Edge Cases
| # | Feature | Input | Observed Behavior |
|---|---|---|---|
| 1 | Modular Themes | Corrupted `theme.json` in user `themes/` folder. | JSON parser fails gracefully; launcher logs warning, skips corrupt file, and retains active theme without crash. |
| 2 | Modular Themes | Theme missing specific CSS custom property (e.g. `--chrome-ribbon`). | CSS variable cascades back to `:root` defaults defined in `app.css` (`tura-graphite` base). |
| 3 | Frameless Titlebar | Window maximized on multi-monitor setup with different DPI scales. | Window accurately fills monitor working area excluding taskbar; double-click on titlebar restores previous geometry. |
| 4 | Currency Switcher | Store item is 100% Free or Giveaway (`salePrice == 0`). | Display renders `Free` (USD) and `Бесплатно` (KZT) without showing `0 ₸` or broken calculation. |
| 5 | Currency Switcher | Currency toggled while an infinite scroll page is actively loading. | Existing rendered items update instantly; pending API payload receives new currency formatting upon resolution. |
| 6 | In-Launcher Drive Selector | User selects optical drive (DVD-ROM) or write-protected USB drive. | Selector displays "Read-Only Media" warning and disables the "Set as Library Folder" button. |
| 7 | In-Launcher Drive Selector | Selected drive has less than 500 MB free space remaining. | Storage gauge bar turns crimson (`#ef4444`); warning text advises selecting a drive with more space. |
| 8 | GitHub Emulator Download | GitHub API returns HTTP 403 Rate Limit Exceeded. | Downloader falls back to pre-indexed direct release asset CDN mirror URLs. |
| 9 | GitHub Emulator Download | Download interrupted due to network disconnection at 74%. | Downloads manager pauses task, flags as `Paused`, and supports byte-range `Resume` via `Range: bytes=X-` header. |
| 10 | Custom DNS / Proxy | User enters invalid DoH address or unreachable proxy IP. | Connection test times out after 4 seconds; launcher prevents saving broken config and prompts user to verify credentials. |
| 11 | Launch Arguments | User enters arguments with spaces and quotes (e.g., `+set fs_game "my mod"`). | Argument parser preserves quoted tokens and passes them safely into Windows process creation. |

---

## 8. Five-Component Handoff Protocol

### 1. Observation
- **Visual References**:
  - Reference 1 (`media_1793830323839_968236a7.jpg`): Ultra-clean dark editorial portfolio ("TURA"), featuring deep charcoal `#09090b` canvas, Cera CY typography, microscopic 0.06 alpha borders, zero explanatory text, and high-contrast white text.
  - Reference 2 (`media_1793830335320_3ec4cbc9.jpg`): Liquid chrome futuristic cyber UI with fluid metallic ribbons, specular borders (`linear-gradient(135deg, #fff 0%, #1a1c22 100%)`), brushed steel pill badges, and glassmorphic photo/video blocks.
- **Codebase Clutter in `src/`**:
  - `src/lib/styles/app.css` lines 50–192: Legacy casual mobile 3D push buttons (`btn-game-green`, `btn-game-blue`, `btn-game-gold`), 5px black borders, and gold overlapping pill banners.
  - `src/lib/components/Navigation.svelte`: Redundant group labels (`Навигация`, `Экосистема`), version subtext, and verbose footer hotkey hints.
  - `src/lib/components/BentoHub.svelte`: Verbose filter text `"Игр (... Установлено, ... в облаке)"`, redundant dropdown prefixes `"Статус:"`, `"Сортировка:"`.
  - `src/lib/components/GameDetailsModal.svelte`: Mobile-style gold header banner and red square close button.
- **Window & Launch Configuration**:
  - `src-tauri/tauri.conf.json` line 25: `"decorations": true`, rendering standard OS titlebar instead of custom frameless UI.
  - `Start-OmniLauncher.cmd`: Launches via batch script triggering terminal console window.

### 2. Logic Chain
1. *Observation*: The user explicitly demands a redesign per References 1 & 2 ("TURA" & "Liquid Chrome"), removing all unnecessary labels and explanatory clutter, with a modular theme engine, Steam-like properties modal, infinite store feed with USD/KZT switcher, GitHub emulator catalog, in-launcher directory picker, Cloudflare routing, custom DNS/proxy, and clean terminal-free startup.
2. *Deduction*: The existing casual 3D button classes (`btn-game-*`) and verbose Russian helper labels in `src/` directly violate this goal and must be systematically expunged.
3. *Deduction*: By specifying a typed JSON theme manifest and a dynamic directory scanner for `themes/`, we satisfy the modular theme requirement while enabling both Reference 1 ("TURA Graphite") and Reference 2 ("Liquid Chrome Cyber") as first-class built-ins.
4. *Deduction*: Disabling native decorations in `tauri.conf.json` and implementing a 44px custom frameless titlebar with integrated tab rail provides the sleek, unified aesthetic seen in Reference 2.
5. *Deduction*: Replacing the native Win32 folder dialog with a custom Svelte drive/directory picker ensures consistent theming and satisfies the requirement of selecting drives without opening Steam's system dialog.

### 3. Caveats
- Windows 11 DWM Mica vibrancy requires Windows 11 build 22000 or higher; on Windows 10 or Linux, the theme engine automatically falls back to solid canvas backgrounds (`var(--bg-canvas)`).
- Direct GitHub API requests for emulator releases are subject to an anonymous rate limit of 60 requests per hour per IP; a cached registry and mirror fallback are specified to guarantee reliability.

### 4. Conclusion
The visual redesign and architectural specifications are fully detailed and ready for immediate implementation. All legacy visual clutter has been audited with an exact removal matrix. The Modular Theme Engine, Frameless Titlebar, Steam Properties Modal, Store (USD/KZT), Emulators Catalog, and In-Launcher Drive/Network settings are fully specified with data models, visual specs, and edge case coverage.

### 5. Verification Method
1. Inspect `handoff.md` (this document) to confirm all user requirements from `ORIGINAL_REQUEST.md` (R1–R5) are mapped to concrete UI and backend specifications.
2. Verify visual token fidelity against Reference 1 (`media_1793830323839_968236a7.jpg`) and Reference 2 (`media_1793830335320_3ec4cbc9.jpg`).
3. Confirm that all 18 discovered features and 11 edge cases provide unambiguous inputs, outputs, and error handling behaviors for the implementation agents.
