# Milestone 4 Frontend & Flagship UI Architecture Handoff

## 1. Observation

Direct inspection of the OmniLauncher frontend codebase (`c:\Users\Tmub\Documents\TMlauncher\src\`) revealed the following components, architecture, and current status:

### Codebase Status & File Locations
1. **Frontend Architecture**:
   - Svelte 5 with native Runes (`$state`, `$derived`, `$derived.by`, `$effect`, `$props`).
   - Vite 6 bundler, Tailwind CSS v4, TypeScript 5.7.
   - Master shell: `src/App.svelte` mounting `TitleBar.svelte` (44px frameless drag region) and `BentoHub.svelte` (central hub for all views, modals, filters, and cards).
   - Core Styling tokens in `src/lib/styles/app.css` and `src/lib/styles/chrome.css`:
     - TURA Style: Matte deep graphite (`--bg-canvas: #09090b`, `--bg-surface: #121215`, `--border-subtle: rgba(255,255,255,0.10)`), clean typography (`Cera CY`, `JetBrains Mono`).
     - Liquid Chrome Style: Specular ribbons (`--chrome-ribbon`), metallic badges (`.chrome-badge`), fluid glass panels (`.chrome-glass`), sleek buttons (`.btn-chrome`, `.btn-minimal`).

2. **Existing UI Components & Gaps**:
   - **`RouletteModal.svelte`** (`src/lib/components/RouletteModal.svelte`):
     - Currently has minimal session time buttons (30m, 120m, unlimited) and an "Unplayed / Backlog" checkbox.
     - **Gaps**: Lacks completion status filters (Unplayed, In-Progress, Completed), genre/tag filters (Action, RPG, Strategy, Indie, etc.), controller support filter (Full Controller, Keyboard/Mouse), physical slot-machine reel strip animation with easing, and synthesized audio tick/triumph feedback.
   - **`SaveVaultModal.svelte`**:
     - Does **not** exist yet (`src/lib/components/SaveVaultModal.svelte` is absent).
     - **Gaps**: Need dedicated UI for browsing save timestamps (pre/post-session snapshots), viewing file changes/diffs between save versions, 1-click restore with auto-safety rollback, manual bookmark creation, and save directory opening in Windows Explorer.
   - **`StorageManagerModal.svelte`**:
     - Does **not** exist yet (`src/lib/components/StorageManagerModal.svelte` is absent).
     - **Gaps**: Need dedicated UI for visual breakdown of disk usage per game (base install, shaders, saves, media), shader cache & orphaned prefix scanner (`%LOCALAPPDATA%\D3DSCache`, `DXCache`, Steam `shadercache`), and 1-click cleanup button.
   - **`GameCard.svelte`** (`src/lib/components/GameCard.svelte`):
     - Renders 2:3 TURA card with poster artwork, platform badge, playtime, and play button.
     - **Gaps**: Missing Steam Review Sentiment badge (% positive / "Overwhelmingly Positive" pill) and Metacritic critic rating pill.
   - **`GameDetailsModal.svelte`** (`src/lib/components/GameDetailsModal.svelte`):
     - Displays banner, achievements, and cloud saves.
     - **Gaps**: Missing detailed sentiment breakdown, Metacritic score pill, and Direct Folder Openers (Game files, Save directory, Mod folder).
   - **`GamePropertiesModal.svelte`** (`src/lib/components/GamePropertiesModal.svelte`):
     - Steam-grade 5-tab modal (`general`, `installed_files`, `cloud`, `achievements`, `compatibility`).
     - **Gaps**:
       1. Missing Personal RPG / Walkthrough Notes tab with local storage autosave.
       2. Missing Direct Folder Openers (open game install, open save directory, open mod directory in Explorer).
       3. Missing Custom Cover Art / Capsule replacer in General or Artwork tab.
   - **Dynamic Filter Pills in `BentoHub.svelte`**:
     - Currently has platform tabs (All, Steam, Epic, Roblox, itch, Emulator) and basic status radio buttons.
     - **Gaps**: Missing Smart Dynamic Filter Pills (Installed, Unplayed Backlog, Favorites, Controller Ready, Short Games) as cohesive chip filters.
   - **Handheld & Laptop Eco / Battery Saver Mode**:
     - Currently absent from `SettingsView.svelte` and `TitleBar.svelte`.

---

## 2. Logic Chain

From the observations above and the user directives in `ORIGINAL_REQUEST.md` (lines 62-81), the architectural reasoning is structured as follows:

1. **Intelligent Backlog Roulette (`RouletteModal.svelte` - Feature 22)**:
   - *Observation*: Users suffer from library choice paralysis across Steam, Epic, Roblox, itch.io, and emulators.
   - *Deduction*: A picker modal must let users express realistic intent: "I have 45 minutes, I want an RPG with full controller support that I haven't finished."
   - *Audio Synthesis Design*: Rather than loading external `.wav`/`.mp3` files (which risk 404s, missing asset errors, or lag), we implement a 100% self-contained Web Audio API synthesizer (`new AudioContext()`) producing mechanical reel clicks that decelerate and a dual-frequency chime (`C5` + `E5`) when locking the winner.
   - *Visual Easing*: A CSS/Svelte vertical slot-machine reel strip with cubic-bezier easing `cubic-bezier(0.12, 0.8, 0.32, 1)` provides tactile gratification matching the Liquid Chrome specular border.

2. **Save-State Time Machine UI (`SaveVaultModal.svelte` - Feature 20 UI)**:
   - *Observation*: Save loss from Steam Cloud desync, mod corruption, or bad branching decisions in RPGs causes major user frustration.
   - *Deduction*: The UI must present a chronological timeline of snapshots (Pre-session, Post-session, Manual bookmarks), display file manifests with delta flags (added/modified/deleted), and provide 1-click restore.
   - *Safety Rule*: To eliminate any risk of data loss, invoking "Restore" must automatically capture a `pre_restore_safeguard_<timestamp>` snapshot before replacing live files.

3. **Storage & Shader Manager (`StorageManagerModal.svelte` - Feature 23 UI)**:
   - *Observation*: PC games leave gigabytes of DirectX/Vulkan shader caches and orphaned Steam prefixes long after uninstallation.
   - *Deduction*: Users require visual transparency. A visual bar/treemap showing space consumption per game alongside an explicit shader reclaimer tab with checkboxes and a 1-click "Purge Selected (X.X GB)" button delivers immediate storage recovery.

4. **Community Vetted Additions (User Directives)**:
   - *Sentiment & Metacritic Badges*: Steam review percentages and Metacritic scores provide immediate quality indicators on `GameCard.svelte` and `GameDetailsModal.svelte`.
   - *Walkthrough & RPG Notes Tab*: Players constantly juggle quest steps, boss weaknesses, and door codes. Adding a 6th `'notes'` tab in `GamePropertiesModal.svelte` with debounced `localStorage` autosave gives users a built-in game scratchpad.
   - *Direct Folder Openers*: Gamers frequently need to reach game directories, save folders, and mod locations. Providing 1-click triggers calling Windows `explorer.exe` solves this natively.
   - *Handheld & Laptop Eco Mode*: On devices like ROG Ally, Legion Go, or laptops on battery, high-framerate CSS blurs and frequent background polling drain battery. Throttling animation frame rates and polling intervals extends playtime significantly.
   - *Custom Cover Art Replacer*: Broken or missing posters on non-Steam/Epic titles are an eyesore; providing a 1-click image picker / URL input in `GamePropertiesModal.svelte` allows total user customization.

---

## 3. Caveats

1. **Web Audio Autoplay Policy**: Modern browser engines (including Edge WebView2) restrict `AudioContext` until the user interacts with the page. The audio engine must be lazily initialized upon the first user interaction (clicking "Spin").
2. **Directory Openers Security**: When opening Windows Explorer, file paths must be validated and sanitized to prevent path traversal or execution of arbitrary arguments.
3. **Battery API Availability**: `navigator.getBattery()` is supported in Chromium-based WebView2 on Windows, but may return rejected promises on desktop PCs without battery drivers. The UI must gracefully fall back to a manual toggle.
4. **Shader Cache Pruning Safety**: Shader caches currently in use by a running game process must not be locked or corrupted; the UI must disable pruning for currently running games.

---

## 4. Conclusion & Complete Component Blueprint

The following complete architectural designs, TypeScript models, Svelte 5 runes implementations, and styling specifications must be implemented for Milestone 4.

### 4.1. Intelligent Backlog Roulette (`RouletteModal.svelte`)

#### TypeScript Data Contracts:
```typescript
export type CompletionFilter = 'all' | 'unplayed' | 'in_progress' | 'completed';
export type SessionWindowFilter = 'quick' | 'standard' | 'marathon'; // <30m, 1-2h, unlimited
export type HltbTierFilter = 'all' | 'short' | 'medium' | 'epic'; // <10h, 10-30h, 30h+
export type ControllerFilter = 'all' | 'full' | 'keyboard_mouse';

export interface RouletteConstraints {
  completion: CompletionFilter;
  sessionWindow: SessionWindowFilter;
  hltbTier: HltbTierFilter;
  genres: string[];
  controller: ControllerFilter;
  platform: 'all' | PlatformKind;
}

export interface RouletteResult {
  winner: Game;
  hltb: HltbBenchmark;
  reelCandidates: Game[];
  reason: string;
}
```

#### Web Audio API Sound Synthesizer:
```typescript
class RouletteSoundEngine {
  private ctx: AudioContext | null = null;

  private init() {
    if (!this.ctx && typeof window !== 'undefined') {
      const AudioCtx = window.AudioContext || (window as any).webkitAudioContext;
      if (AudioCtx) this.ctx = new AudioCtx();
    }
    if (this.ctx && this.ctx.state === 'suspended') {
      void this.ctx.resume();
    }
  }

  playTick(frequency = 750, volume = 0.08) {
    try {
      this.init();
      if (!this.ctx) return;
      const osc = this.ctx.createOscillator();
      const gain = this.ctx.createGain();
      osc.type = 'triangle';
      osc.frequency.setValueAtTime(frequency, this.ctx.currentTime);
      osc.frequency.exponentialRampToValueAtTime(150, this.ctx.currentTime + 0.025);
      gain.gain.setValueAtTime(volume, this.ctx.currentTime);
      gain.gain.exponentialRampToValueAtTime(0.001, this.ctx.currentTime + 0.025);
      osc.connect(gain);
      gain.connect(this.ctx.destination);
      osc.start();
      osc.stop(this.ctx.currentTime + 0.03);
    } catch {
      // Audio context unavailable
    }
  }

  playWinnerChime() {
    try {
      this.init();
      if (!this.ctx) return;
      const notes = [523.25, 659.25, 783.99]; // C5, E5, G5
      notes.forEach((freq, i) => {
        const osc = this.ctx!.createOscillator();
        const gain = this.ctx!.createGain();
        osc.type = 'sine';
        osc.frequency.setValueAtTime(freq, this.ctx!.currentTime + i * 0.06);
        gain.gain.setValueAtTime(0.12, this.ctx!.currentTime + i * 0.06);
        gain.gain.exponentialRampToValueAtTime(0.001, this.ctx!.currentTime + i * 0.06 + 0.4);
        osc.connect(gain);
        gain.connect(this.ctx!.destination);
        osc.start(this.ctx!.currentTime + i * 0.06);
        osc.stop(this.ctx!.currentTime + i * 0.06 + 0.45);
      });
    } catch {
      // Audio context unavailable
    }
  }
}
```

#### Svelte 5 Component Structure:
- **Filter Row**:
  - Completion Status Pills: `All`, `Unplayed (< 30m)`, `In Progress`, `Completed`.
  - Session Time Pills: `30m (Quick Burst)`, `2h (Standard)`, `Unlimited (Marathon)`.
  - Genre Chips: `Action`, `RPG`, `Strategy`, `Indie`, `Adventure`, `Shooter`, `Puzzle`.
  - Controller Support: `Any`, `Full Controller Ready 🎮`.
- **Reel Strip Visualizer**:
  - Viewport: Height 180px, overflow hidden, framed with `.chrome-border` and specular chrome sheen.
  - Reel Track: Flexbox column or row of cards translated with CSS transform `translateY(-${offset}px)`.
  - During spin: `spinning = true`, ticks play at interval that slows down from 40ms to 250ms using a timer loop.
- **Winner Presentation Card**:
  - High-res poster + Glowing title.
  - HLTB pill badges: `Main: X.Xh`, `+Extras: X.Xh`, `100%: X.Xh`.
  - Reason badge: `Matches your 45m window • Action RPG • 94% Positive`.
  - Buttons: `[ Launch Game 🚀 ]` (`.btn-chrome`), `[ Spin Again 🎲 ]` (`.btn-minimal`).

---

### 4.2. Save Vault Modal (`SaveVaultModal.svelte` - Feature 20 UI)

#### TypeScript Data Contracts:
```typescript
export interface SaveSnapshot {
  id: string;
  gameId: string;
  gameTitle: string;
  timestamp: string; // ISO 8601
  label: string; // e.g. "Pre-Session Auto", "Manual: Before Boss"
  trigger: 'pre_session' | 'post_session' | 'manual' | 'cloud_sync' | 'safeguard';
  totalSizeBytes: number;
  fileCount: number;
  files: SaveFileInfo[];
}

export interface SaveFileInfo {
  relativePath: string;
  sizeBytes: number;
  modifiedTime: string;
  status?: 'added' | 'modified' | 'deleted' | 'identical';
}

export interface SaveDiff {
  snapshotId: string;
  baseTimestamp: string;
  addedFiles: string[];
  modifiedFiles: { path: string; deltaBytes: number }[];
  deletedFiles: string[];
}
```

#### Component Layout & Features:
1. **Header Zone**:
   - Game poster thumbnail, game title, and save directory location (`%USERPROFILE%\Saved Games\...`).
   - Action buttons:
     - `[ Create Snapshot Now 💾 ]` with optional custom label modal/input.
     - `[ Open Save Folder 📂 ]` (calls `open_game_directory(gameId, 'save')`).
2. **Timeline View**:
   - Left-hand rail showing chronological nodes connected by a subtle 1px metallic timeline trace.
   - Each node displays:
     - Badge by trigger type:
       - `Pre-Session Auto`: Cyan badge
       - `Post-Session Auto`: Emerald badge
       - `Manual Bookmark`: Amber badge
       - `Cloud Sync`: Purple badge
       - `Safeguard`: Gold badge
     - Timestamp (`new Date(snapshot.timestamp).toLocaleString()`).
     - Archive size and file count (`formatBytes(totalSizeBytes)`, `${fileCount} files`).
     - Label / note.
     - Action buttons:
       - `[ Restore ]`: Opens confirmation popover ("Restore save to this point in time? A safety backup will be created automatically.").
       - `[ View Files ]`: Toggles file list drawer.
       - `[ Delete ]`: Removes snapshot archive.
3. **File Manifest & Diff Viewer**:
   - Tree list of all saved files in the snapshot with color-coded diff markers (`+` Added in green, `~` Modified in blue, `-` Deleted in red).
4. **Safety Rollback Engine**:
   - When user clicks "Restore", backend executes `create_save_snapshot(gameId, 'Pre-Restore Safety Rollback', 'safeguard')` before writing older files to disk.

---

### 4.3. Storage & Shader Manager Modal (`StorageManagerModal.svelte` - Feature 23 UI)

#### TypeScript Data Contracts:
```typescript
export interface GameStorageUsage {
  gameId: string;
  title: string;
  platform: PlatformKind;
  installSizeBytes: number;
  shaderCacheSizeBytes: number;
  saveDataSizeBytes: number;
  mediaSizeBytes: number;
  totalSizeBytes: number;
  installDir: string | null;
}

export interface ShaderCacheEntry {
  id: string;
  appId: number;
  title: string;
  path: string;
  sizeBytes: number;
  apiType: 'directx' | 'vulkan' | 'dxvk';
  isOrphaned: boolean; // Game is uninstalled but cache remains
  selected: boolean;
}

export interface ShaderCacheSummary {
  entries: ShaderCacheEntry[];
  totalSizeBytes: number;
  orphanedSizeBytes: number;
  scannedAt: string;
}
```

#### Component Layout & Features:
1. **Header Overview Card**:
   - Reclaimable space banner: `X.X GB Orphaned & Shader Waste Detectable`.
   - Drive capacity bar for C: and secondary game drives.
2. **Two Master Tabs**:
   - **Tab 1: Games Footprint Visualizer**:
     - Visual proportional bar of top installed games by size.
     - Game cards with 4-way breakdown chips:
       - Base Game (`68.4 GB`)
       - Shader Cache (`2.1 GB`)
       - Save Data (`14 MB`)
       - Screenshots/Clips (`180 MB`)
     - Actions: `[ Open Folder ]`, `[ Purge Game Shaders ]`.
   - **Tab 2: Shader Cache Reclaimer**:
     - Scans and displays:
       - NVIDIA DXCache (`%LOCALAPPDATA%\NVIDIA\DXCache`)
       - AMD DxCache (`%LOCALAPPDATA%\AMD\DxCache`)
       - DirectX D3DSCache (`%LOCALAPPDATA%\D3DSCache`)
       - Steam `steamapps/shadercache/<appId>`
     - Table with column headers: Checkbox, Game Title, AppID, Cache Type, Size, Status (`Active` vs `⚠ Orphaned`).
     - Action Bar:
       - `[ Select All Orphaned ]` button.
       - `[ Purge Selected (X.X GB) 🧹 ]` action button with live reclaimed counter and progress toast.

---

### 4.4. Community Vetted Features (User Directives)

#### 1. Steam Review Sentiment Badges & Metacritic Pills
- **Data Model**:
  ```typescript
  export interface GameSentiment {
    steamScoreDesc?: 'Overwhelmingly Positive' | 'Very Positive' | 'Positive' | 'Mixed' | 'Mostly Negative';
    steamPercent?: number; // e.g. 96
    steamReviewCount?: number; // e.g. 142500
    metacriticScore?: number; // 0 - 100
  }
  ```
- **Styling & Presentation**:
  - `GameCard.svelte`:
    - Overlaid in top right / bottom scrim:
      - Sentiment Pill: Emerald rounded pill with thumbs up icon: `👍 96%` (or amber `⚡ 62%`, rose `👎 28%`).
      - Metacritic Pill: Rounded square badge with bold critic score: `88` (green `border-emerald-500/40 text-emerald-300`, amber `border-amber-500/40 text-amber-300`, rose `border-rose-500/40 text-rose-300`).
  - `GameDetailsModal.svelte`:
    - Hero section banner displays full text:
      `👍 Overwhelmingly Positive (96% of 142,500 user reviews) • Metascore: 88/100 Critic Rating`.

#### 2. Personal RPG / Walkthrough Notes Tab (`GamePropertiesModal.svelte`)
- **Tab Placement**:
  - Add 6th tab: `'notes'` (`{ id: 'notes', label: 'Walkthrough Notes 📝' }`).
- **Features**:
  - Monospace rich text scratchpad for builds, quest lines, door codes, and boss tactics.
  - Character and word counter (`1,420 characters • 240 words`).
  - "Autosaved" status pip (turns emerald when saved, amber when typing).
  - Quick action toolbar:
    - Insert Heading (`# `)
    - Checklist Item (`- [ ] `)
    - Code Block (```)
    - Templates: "Boss Strategies", "Build Tracker", "Door Codes & Secrets".
    - `[ Export as .md 📄 ]` button.
  - Storage:
    - Auto-saves to `localStorage.getItem('omnilauncher.notes.' + gameId)` with a 300ms debounce.
    - Persists permanently across sessions.

#### 3. Smart Dynamic Filter Pills (`BentoHub.svelte` / `LibraryView.svelte`)
- **Pill Row UI**:
  - Placed directly above the card grid in `BentoHub.svelte`:
    1. `All` (`{displayGames.length}`)
    2. `Installed` (`{installedCount}`) with emerald pip.
    3. `Unplayed Backlog` (`{unplayedCount}`) with purple pip (playtime < 30m).
    4. `Favorites` (`{favoritesCount}`) with gold star.
    5. `Steam` (`{steamCount}`)
    6. `Epic Games` (`{epicCount}`)
    7. `Roblox` (`{robloxCount}`)
    8. `Emulators` (`{emulatorCount}`)
    9. `Controller Ready 🎮`
- **Behavior**:
  - Keyboard focusable (`Enter` / `Space` toggle), active pills have specular chrome border and bold typography.
  - Real-time reactive filtering using Svelte 5 `$derived.by`.

#### 4. Direct Folder Openers
- **Integration Points**:
  - In `GamePropertiesModal.svelte` under "Installed Files" tab:
    ```svelte
    <div class="grid grid-cols-3 gap-2.5">
      <button onclick={() => openGameFolder(gameId, 'install')} class="btn-minimal">
        📁 Game Files
      </button>
      <button onclick={() => openGameFolder(gameId, 'save')} class="btn-minimal">
        💾 Save Directory
      </button>
      <button onclick={() => openGameFolder(gameId, 'mods')} class="btn-minimal">
        🧩 Mod / Config
      </button>
    </div>
    ```
  - In `GameDetailsModal.svelte` under "Overview" tab.
  - In `GameCard.svelte` right-click context menu: "Open Game Directory", "Open Save Location".
- **Backend IPC**:
  - `invoke('open_game_directory', { gameId, kind: 'install' | 'save' | 'mods' })`.

#### 5. Handheld & Laptop Eco / Battery Saver Mode
- **Svelte 5 State Store (`src/lib/stores/eco.svelte.ts`)**:
  ```typescript
  class EcoStore {
    enabled = $state(false);
    oledBlack = $state(true);
    reduceMotion = $state(true);
    pollingIntervalSecs = $state(300);
    batteryLevel = $state<number | null>(null);
    isCharging = $state(true);

    init() {
      if (typeof navigator !== 'undefined' && 'getBattery' in navigator) {
        (navigator as any).getBattery().then((battery: any) => {
          this.batteryLevel = Math.round(battery.level * 100);
          this.isCharging = battery.charging;
          battery.addEventListener('chargingchange', () => {
            this.isCharging = battery.charging;
            if (!battery.charging && this.batteryLevel! < 30) this.enabled = true;
          });
          battery.addEventListener('levelchange', () => {
            this.batteryLevel = Math.round(battery.level * 100);
          });
        }).catch(() => {});
      }
    }

    toggle() {
      this.enabled = !this.enabled;
      document.documentElement.dataset.eco = this.enabled ? 'true' : 'false';
    }
  }
  export const ecoStore = new EcoStore();
  ```
- **CSS Throttle Rules (when `data-eco="true"` on `:root`)**:
  - `--bg-canvas: #000000; --bg-surface: #040404;` (True OLED pitch black).
  - `* { animation-duration: 0.001ms !important; transition-duration: 0.001ms !important; }` (Eliminates continuous GPU repaint overhead).
  - Disables ambient blur and background video loops.
- **UI Trigger**:
  - Leaf / battery pill icon in `TitleBar.svelte` next to live status pip (`🍃 ECO`).
  - Toggle in `SettingsView.svelte` with full handheld battery optimization options.

#### 6. Custom Cover Art / Capsule Replacer
- **Integration in `GamePropertiesModal.svelte`**:
  - Under "General" tab or dedicated artwork section:
    - Live 2:3 vertical capsule preview and 16:9 hero preview.
    - Two input choices:
      1. `[ Choose Local File 🖼️ ]`: In-launcher file dialog or native picker for `.png`, `.jpg`, `.webp`.
      2. Direct URL input: Paste any image link (e.g. from SteamGridDB).
    - `[ Reset to Official Art ↺ ]` button.
    - Live update: immediately assigns `game.art.poster = { kind: 'remote' | 'local', value: customUrl }` and updates the reactive library store so `GameCard.svelte` updates without a restart!

---

## 5. Verification Method

To independently verify the implementation:

1. **Static Analysis & Type Checking**:
   - Execute `npm run check` or `npx svelte-check --tsconfig ./tsconfig.json`.
   - Confirm zero TypeScript type errors in Svelte 5 runes (`$state`, `$derived`, `$props`).
2. **Frontend Production Build**:
   - Execute `npm run build`.
   - Confirm Vite compiles `dist/` cleanly without bundle warnings or broken imports.
3. **UI Functional Verification**:
   - **RouletteModal**:
     - Click "Backlog Spin 🎲" in sidebar. Verify modal opens with TURA/Chrome styling.
     - Toggle completion status, session length, and controller filters. Click "Spin Slot Machine".
     - Observe the slot reel animation, hear the synthesized Web Audio clicks and winner chime, and verify the winner card displays HLTB hours and launch button.
   - **SaveVaultModal**:
     - Open Save Vault for an installed game. Verify timeline nodes, archive sizes, and file list.
     - Click "Create Snapshot Now" with a custom label; verify the new node appears.
     - Click "Restore" and verify pre-restore safeguard snapshot is taken before rollback.
   - **StorageManagerModal**:
     - Open Storage Manager. Verify drive space gauges and per-game footprint visualizer.
     - Switch to "Shader Cache Reclaimer" tab. Verify detected caches and orphaned status pills.
     - Click "Purge Selected" and verify disk space update toast.
   - **Community Features**:
     - Inspect `GameCard.svelte` and verify Steam sentiment badge (`👍 XX%`) and Metacritic score pill (`XX`).
     - Open `GamePropertiesModal.svelte`. Switch to 6th "Walkthrough Notes" tab. Enter text and templates, reload modal, and verify instant local storage recovery.
     - In `GamePropertiesModal.svelte` and `GameDetailsModal.svelte`, click "Game Files", "Save Directory", "Mod Directory" and verify Windows Explorer opens to the exact folders.
     - In `TitleBar.svelte`, click `🍃 ECO` and verify true OLED pitch black theme and animation throttling take effect.
     - In `GamePropertiesModal.svelte`, paste a custom cover URL or select an image file; verify `GameCard.svelte` poster updates immediately.
