# BRIEFING — 2026-11-05T05:01:00Z

## Mission
Investigate existing backend/service files and design the implementation for HLTB forecasting, PCGamingWiki tweaks, Discord Rich Presence, and DealsRadar for Milestone 4.

## 🔒 My Identity
- Archetype: Explorer
- Roles: Read-only investigator, External Integrations & Services architect
- Working directory: c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\teamwork_explorer_m4_services
- Original parent: bed11d0d-d77d-423c-a811-8e81f371aec2
- Milestone: M4 (Flagship Modern Launcher Features - External Services)

## 🔒 Key Constraints
- Read-only investigation — do NOT implement
- Focus on HLTB forecasting, PCGamingWiki tweaks, Universal Discord Rich Presence, DealsRadar
- Concrete APIs, error fallbacks (offline/rate-limit resilience), Rust commands, and data structures
- Write handoff.md in your working directory and notify via send_message

## Current Parent
- Conversation ID: bed11d0d-d77d-423c-a811-8e81f371aec2
- Updated: 2026-11-05T05:01:00Z

## Investigation State
- **Explored paths**:
  - `src-tauri/Cargo.toml`, `src-tauri/src/lib.rs`, `src-tauri/src/commands/deals.rs`, `src-tauri/src/core/discord_rpc.rs`, `src-tauri/src/commands/social.rs`, `src-tauri/src/commands/roulette.rs`, `src-tauri/src/core/runner.rs`, `src-tauri/src/core/config.rs`, `src-tauri/src/error.rs`
  - `src/lib/components/GameCard.svelte`, `GameDetailsModal.svelte`, `GamePropertiesModal.svelte`, `DealsRadar.svelte`, `FriendsList.svelte`, `RouletteModal.svelte`, `BentoHub.svelte`
  - `src/lib/stores/library.ts`, `friends.ts`, `store.ts`
  - `tests/runner.cjs`, `tests/helpers/contracts.cjs`, `tier1_features/f21_hltb_forecasting.test.cjs`, `f24_pcgw_tweaks.test.cjs`, `f26_discord_rpc.test.cjs`, `f27_deals_radar.test.cjs`
  - `tests/tier2_boundaries/b21_hltb_boundaries.test.cjs`, `b24_pcgw_boundaries.test.cjs`, `b26_discord_boundaries.test.cjs`, `b27_deals_boundaries.test.cjs`
  - `tests/tier3_interactions/i02_currency_and_deals_radar.test.cjs`, `i04_game_mode_and_discord_rpc.test.cjs`, `i08_properties_modal_and_pcgw_tweaks.test.cjs`, `i10_roulette_and_hltb_integration.test.cjs`
- **Key findings**:
  - HLTB: Basic hash fallback exists in `roulette.rs`. Needs standalone `commands/hltb.rs` with `get_hltb_data` and batch support, query sanitization (foreign symbols/trademarks), multi-tier cache (memory/disk/curated/heuristic), and progress bar integration into `GameCard.svelte` and `GameDetailsModal.svelte`.
  - PCGW: Completely missing backend `commands/pcgw.rs`. Needs MediaWiki Cargo API integration (`action=cargoquery`), config file path resolution, Windows read-only flag removal and automatic `.omnibak` backups, clamping out-of-range FOV/FPS values, and integration into `GamePropertiesModal.svelte` (compatibility tab).
  - Discord RPC: `core/discord_rpc.rs` exists with named pipes, but lacks reconnection backoff (15s) when Discord starts after the game, markdown escaping, 128-char limit truncation, and rate-limit guard.
  - DealsRadar: `commands/deals.rs` queries Epic and CheapShark, but lacks All-Time Low (ATL) price flag (`is_historical_low`), cross-store side-by-side deal comparisons, and persistent offline deals cache.
- **Unexplored areas**: None within the scope of external services.

## Key Decisions Made
- Designed unified multi-tier resilience architecture (Memory Cache -> Disk Cache -> Live API with 3.5s timeout -> Curated Dataset -> Heuristic Fallback) for both HLTB and PCGW to guarantee 0% failure in offline mode or during rate limiting.
- Aligned data schemas strictly with `tests/helpers/contracts.cjs` (`validateHltbRecord`, `validatePcgwRecord`) and tier 2 boundary test rules.

## Artifact Index
- handoff.md — Complete 5-component architectural handoff report with exact data structures, APIs, error fallbacks, and UI contracts.
