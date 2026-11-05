## 2026-11-05T03:35:20Z
You are a teamwork_worker.
Your working directory is: c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\teamwork_worker_m3\
Your role: Store, Emulators & Network Implementation Worker
Your parent is: f93fd91e-112f-4d7d-a956-798970f8fc58

MANDATORY FIRST STEP:
Read ORIGINAL_REQUEST.md at c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\ORIGINAL_REQUEST.md and PROJECT.md at c:\Users\Tmub\Documents\TMlauncher\PROJECT.md.

Next, read the 3 Explorer handoff reports which contain the complete architectural blueprints and code designs:
1. c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\teamwork_explorer_m3_store\handoff.md
2. c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\teamwork_explorer_m3_emulators\handoff.md
3. c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\teamwork_explorer_m3_system\handoff.md

MANDATORY INTEGRITY WARNING:
DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A teamwork_auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.

SCOPE OF WORK — MILESTONE 3 (Features 12 to 18):
1. Feature 12 & 13: Store Infinite Feed & USD/KZT Currency Switcher:
   - Implement `src-tauri/src/commands/store.rs`:
     - `get_store_recommendations(page: usize, currency: String) -> Result<StorePage, String>` with tag filtering, search sanitization, discount calculations, in-flight debounce lock, catalog exhaustion handling, currency fallback.
     - Currency conversion between USD and KZT (fixed base exchange rate 495.0, integer rounding for KZT, 1 KZT floor for micro-prices, 64-bit integer overflow protection).
   - Implement `src/lib/stores/store.svelte.ts` (Svelte 5 Runes state).
   - Implement `src/lib/views/StoreView.svelte` with infinite scroll, tag pills, search bar, TURA/Liquid Chrome cards, media preview, USD/KZT currency switcher toggle with instant price recalculation.
2. Feature 15 & 16: GitHub Open-Source Emulators Catalog & 1-Click Downloader/Extractor:
   - Implement `src-tauri/src/commands/emulators.rs`:
     - Curated 8 emulators catalog (RetroArch, Dolphin, PCSX2, RPCS3, PPSSPP, DuckStation, Ryujinx, Lime3DS).
     - Commands: `get_emulator_catalog() -> Result<Vec<EmulatorEntry>, String>`, `install_emulator(emulator_id: String, target_dir: Option<String>) -> Result<EmulatorInstallResult, String>`.
     - GitHub Releases API query with HTTP 403 fallback to cached releases.
     - Streaming download with `emulator://progress` event emission.
     - Zip extraction (using `zip` crate) with Zip-Slip path sanitization, 3.5x uncompressed space check, directory rollback on failure.
     - Auto-registration into library (`PlatformKind::Emulator`, category: 'Emulators').
   - Implement `src/lib/stores/emulators.svelte.ts` and `src/lib/views/EmulatorsView.svelte` with system filters, description, GitHub badges, 1-Click Install button, live progress bar, Launch button.
3. Feature 14, 17 & 18: In-Launcher Directory Picker & Network/Cloudflare Settings:
   - Implement `src-tauri/src/commands/network.rs` (and filesystem commands):
     - `get_system_drives()`: Windows logical drive enumeration, total & free bytes (`GetDiskFreeSpaceExW`).
     - `list_directory_contents(path: String)`: Folder listing, parent path, navigation.
     - `create_directory(path: String)`: Folder creation.
     - `update_network_settings(dns_provider: String, proxy_url: Option<String>)`: Persist DNS & Proxy settings.
   - Implement `src/lib/components/DirectoryPickerModal.svelte`: In-launcher folder browser, drive space visualizer, breadcrumbs, folder creation, no OS dialogs.
   - Implement/update `src/lib/views/SettingsView.svelte` with Network tab: DNS provider selector (Cloudflare 1.1.1.1, Google 8.8.8.8, Quad9 9.9.9.9, Custom DoH), Proxy URL input, test connection button.
   - Cloudflare CDN media routing helper in frontend and backend.
4. Wiring & Navigation:
   - Register all commands in `src-tauri/src/commands/mod.rs` and `src-tauri/src/lib.rs`.
   - Connect views in `src/lib/components/Navigation.svelte` and `src/lib/components/BentoHub.svelte` (so navigating to Store, Emulators, Settings switches views seamlessly).

VERIFICATION REQUIREMENTS:
Run and verify:
1. `cargo check --manifest-path src-tauri/Cargo.toml`
2. `cargo test --manifest-path src-tauri/Cargo.toml`
3. `npx tsc --noEmit`
4. `npm run build`
5. `node tests/runner.cjs` (all 335+ tests must pass)
