# Task Assignment: Milestone 4 Flagship Modern Launcher Features Implementation

You are a Worker agent assigned to implement the complete Milestone 4 feature suite for OmniLauncher, combining flagship capabilities and the community-vetted user directives.

## MANDATORY INTEGRITY WARNING
DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A teamwork_auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.

## Input Explorer Reports
1. Core Systems Report:
   `c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\teamwork_explorer_m4_core\handoff.md`
2. External Services Report:
   `c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\teamwork_explorer_m4_services\handoff.md`
3. Flagship UI & Community Directives Report:
   `c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\teamwork_explorer_m4_ui\handoff.md`
4. Master Blueprint:
   `c:\Users\Tmub\Documents\TMlauncher\PROJECT.md`
5. Original Request:
   `c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\ORIGINAL_REQUEST.md`

## Scope of Implementation

### Part A: Core Systems Backend (`src-tauri/`)
1. **Zero-Footprint Game Mode (Feature 19)**:
   - In `src-tauri/src/core/runner.rs`, add working set trimming via `SetProcessWorkingSetSize` and priority lowering to `IDLE_PRIORITY_CLASS` during game execution, restoring `NORMAL_PRIORITY_CLASS` on exit. Emit `gamemode://status` event.
2. **Save-State Time Machine (Feature 20)**:
   - Implement `src-tauri/src/commands/save_vault.rs` (and register in `main.rs`/`lib.rs`).
   - Commands: `create_save_snapshot`, `list_save_snapshots`, `restore_save_snapshot` (with mandatory PreRestore backup and Zip-Slip protection), `open_save_folder`.
3. **Shader Cache & Disk Reclaimer (Feature 23)**:
   - Implement `src-tauri/src/commands/shader_reclaimer.rs`.
   - Commands: `scan_shader_caches`, `clean_shader_caches` (graceful skip for locked files, scanning DirectX, NVIDIA, AMD, Steam caches).
4. **Proactive Offline Vault (Feature 28)**:
   - Local caching of library metadata, manifests, and offline readiness diagnostics.
5. **HowLongToBeat Integration (Feature 21)**:
   - Implement `src-tauri/src/commands/hltb.rs`. Command: `get_hltb_data` (title normalizer, disk/memory cache, 5-tier fallback cascade).
6. **PCGamingWiki Tweak Engine (Feature 24)**:
   - Implement `src-tauri/src/commands/pcgw.rs`. Command: `get_pcgamingwiki_data`, `apply_pcgw_tweak` (handling `.omnibak` backups, clearing read-only attributes).
7. **Universal Discord Rich Presence (Feature 26)**:
   - Integrate `src-tauri/src/core/discord_rpc.rs` with title sanitization, 15s reconnection backoff, multi-platform assets.
8. **DealsRadar & Historical Lows (Feature 27)**:
   - Implement `src-tauri/src/commands/deals.rs` with All-Time Low (ATL) price milestone badges and multi-store comparisons.

### Part B: Frontend UI & Community Vetted Features (`src/`)
1. **Intelligent Backlog Roulette (Feature 22)**:
   - Implement `src/lib/components/RouletteModal.svelte` (constraint picker, slot reel animation, Web Audio API mechanical tick and victory chime).
2. **Save Vault UI (Feature 20 UI)**:
   - Implement `src/lib/components/SaveVaultModal.svelte` (timeline, file diffing, 1-click restore, direct save folder opener).
3. **Storage & Shader Manager UI (Feature 23 UI)**:
   - Implement `src/lib/components/StorageManagerModal.svelte` (disk breakdown per game, shader cache cleanup button).
4. **Community Vetted Directives**:
   - Steam Review Sentiment Badges & Metacritic score pills on `GameCard.svelte` and `GameDetailsModal.svelte`.
   - Personal RPG / Walkthrough Notes tab with debounced localStorage autosave in `GamePropertiesModal.svelte`.
   - Smart Dynamic Filter Pills (Installed, Unplayed Backlog, Favorites, Steam, Epic, Roblox, Emulators) in `BentoHub.svelte` or `LibraryView.svelte`.
   - Direct Folder Openers (Game files, Save directory, Mod directory).
   - Handheld & Laptop Eco / Battery Saver Mode (`ecoStore.svelte.ts`) with animation throttling.
   - Custom Cover Art / Capsule replacer in `GamePropertiesModal.svelte`.

## Verification Commands
After implementation, you must execute:
1. `cargo check --manifest-path src-tauri/Cargo.toml`
2. `cargo test --manifest-path src-tauri/Cargo.toml`
3. `npx tsc --noEmit`
4. `npm run build`
5. `node tests/runner.cjs`

Write your comprehensive completion report to:
`c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\teamwork_worker_m4\handoff.md`
and notify me via `send_message`.

## 2026-11-05T05:02:15Z
[Message] timestamp=2026-11-05T05:02:15Z sender=bed11d0d-d77d-423c-a811-8e81f371aec2 priority=MESSAGE_PRIORITY_HIGH content=Read your task assignment in c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\teamwork_worker_m4\DISPATCH.md.
Also read the 3 explorer reports at:
- c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\teamwork_explorer_m4_core\handoff.md
- c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\teamwork_explorer_m4_services\handoff.md
- c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\teamwork_explorer_m4_ui\handoff.md
Implement the full Milestone 4 features across backend (save_vault.rs, shader_reclaimer.rs, hltb.rs, pcgw.rs, runner.rs) and frontend (RouletteModal.svelte, SaveVaultModal.svelte, StorageManagerModal.svelte, sentiment badges, walkthrough notes, eco mode, filter pills, cover replacer).
Execute verification builds and test suites:
- cargo check --manifest-path src-tauri/Cargo.toml
- cargo test --manifest-path src-tauri/Cargo.toml
- npx tsc --noEmit
- npm run build
- node tests/runner.cjs
Write your completion report to c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\teamwork_worker_m4\handoff.md and notify me via send_message.

MANDATORY INTEGRITY WARNING:
DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A teamwork_auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.
