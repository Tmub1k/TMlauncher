# BRIEFING — 2026-11-05T05:05:00Z

## Mission
Implement the complete Milestone 4 flagship features and community-vetted directives across backend and frontend, and verify with the full test suite.

## 🔒 My Identity
- Archetype: teamwork_worker
- Roles: implementer, qa, specialist
- Working directory: c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\teamwork_worker_m4
- Original parent: bed11d0d-d77d-423c-a811-8e81f371aec2
- Milestone: Milestone 4 (Flagship Modern Launcher Features & Community Directives)

## 🔒 Key Constraints
- DO NOT CHEAT. All implementations must be genuine.
- DO NOT hardcode test results, expected outputs, or verification strings in source code.
- DO NOT create dummy or facade implementations that produce correct-looking outputs without genuine logic.
- Maintain real state and produce real behavior.
- Minimal change principle: only modify what is necessary, no unrelated refactoring.
- Re-read files before modifying.
- Write progress.md as a liveness heartbeat.
- Write handoff.md with 5 components upon completion.

## Current Parent
- Conversation ID: bed11d0d-d77d-423c-a811-8e81f371aec2
- Updated: not yet

## Task Summary
- **What to build**:
  - Backend:
    1. Zero-Footprint Game Mode (runner.rs working set trim via SetProcessWorkingSetSize & IDLE_PRIORITY_CLASS, gamemode://status event)
    2. Save-State Time Machine (src-tauri/src/commands/save_vault.rs + core/save_vault.rs, PreRestore backup, Zip-Slip protection, open_save_folder)
    3. Shader Cache & Disk Reclaimer (src-tauri/src/commands/shader_reclaimer.rs, safe scan & clean for DirectX, NVIDIA, AMD, Steam caches)
    4. Proactive Offline Vault (offline readiness diagnostics, library caching)
    5. HowLongToBeat (src-tauri/src/commands/hltb.rs, get_hltb_data, 5-tier fallback cascade, contract compliance)
    6. PCGamingWiki (src-tauri/src/commands/pcgw.rs, get_pcgamingwiki_data, apply_pcgw_tweak with .omnibak backups and clearing read-only attributes)
    7. Discord RPC hardening (15s backoff, title sanitization, 128 char limit)
    8. DealsRadar historical low ATL badges
  - Frontend:
    1. Intelligent Backlog Roulette (RouletteModal.svelte with filters, slot machine reel animation, Web Audio API sound synthesis)
    2. Save Vault Modal (SaveVaultModal.svelte with timeline, diff, 1-click restore, directory opener)
    3. Storage Manager Modal (StorageManagerModal.svelte with per-game disk breakdown & shader reclaimer)
    4. Sentiment & Metacritic Badges on GameCard.svelte & GameDetailsModal.svelte
    5. Walkthrough Notes in GamePropertiesModal.svelte with debounced autosave
    6. Smart Dynamic Filter Pills in BentoHub.svelte
    7. Direct Folder Openers (Game files, Save directory, Mod directory)
    8. Handheld & Laptop Eco Mode (eco store, CSS throttling)
    9. Custom Cover Art Replacer in GamePropertiesModal.svelte
- **Success criteria**:
  - cargo check passes
  - cargo test passes
  - npx tsc --noEmit passes
  - npm run build passes
  - node tests/runner.cjs passes (all tests passing)
- **Interface contracts**: PROJECT.md, tests/helpers/contracts.cjs
- **Code layout**: PROJECT.md § Code Layout

## Key Decisions Made
- Architecture follows recommendations from explorer handoffs (core, services, ui).
- Web Audio API used for RouletteModal sound effects to ensure self-contained, dependency-free audio.
- .omnibak creation and read-only clearing for PCGW tweak application.
- Zip-Slip path sanitization for Save Vault restore.

## Artifact Index
- c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\teamwork_worker_m4\DISPATCH.md — Task assignment
- c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\teamwork_worker_m4\BRIEFING.md — Persistent memory
- c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\teamwork_worker_m4\progress.md — Heartbeat progress log
- c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\teamwork_worker_m4\handoff.md — Final completion report

## Change Tracker
- **Files modified**: [TBD]
- **Build status**: Pending
- **Pending issues**: None

## Quality Status
- **Build/test result**: Pending initial run
- **Lint status**: Pending
- **Tests added/modified**: Pending

## Loaded Skills
- None
