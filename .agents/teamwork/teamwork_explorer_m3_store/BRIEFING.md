# BRIEFING — 2026-11-05T03:30:15Z

## Mission
Explore and specify the technical architecture for Milestone 3 Store & Currency (Feature 12: Steam Store Infinite Recommendations, Feature 13: Currency Switcher USD/KZT).

## 🔒 My Identity
- Archetype: teamwork_explorer
- Roles: Store & Currency Architecture Explorer
- Working directory: c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\teamwork_explorer_m3_store\
- Original parent: f93fd91e-112f-4d7d-a956-798970f8fc58
- Milestone: Milestone 3 Store & Currency

## 🔒 Key Constraints
- Read-only investigation — do NOT implement
- Explore Feature 12 (Steam Store Infinite Recommendations) and Feature 13 (Currency Switcher USD/KZT)
- Output handoff.md following 5-component protocol

## Current Parent
- Conversation ID: f93fd91e-112f-4d7d-a956-798970f8fc58
- Updated: 2026-11-05T03:25:04Z

## Investigation State
- **Explored paths**:
  - `PROJECT.md` & `ORIGINAL_REQUEST.md` (contract specifications & acceptance criteria)
  - `tests/tier1_features/f12_infinite_store.test.cjs` & `tests/tier1_features/f13_currency_switch.test.cjs`
  - `tests/tier2_boundaries/b12_store_boundaries.test.cjs` & `tests/tier2_boundaries/b13_currency_boundaries.test.cjs`
  - `tests/tier3_interactions/i02_currency_and_deals_radar.test.cjs`
  - `tests/tier4_scenarios/s01_first_time_user_onboarding.test.cjs`
  - `tests/helpers/contracts.cjs` (`validateStorePage` schema)
  - `src-tauri/src/commands/deals.rs`, `platforms.rs`, `lib.rs`, `core/config.rs`
  - `src/lib/components/SteamStoreShowcase.svelte`, `StoreBrowserModal.svelte`, `DealsRadar.svelte`, `GameCard.svelte`, `BentoHub.svelte`, `Navigation.svelte`
- **Key findings**:
  - `src-tauri/src/commands/store.rs` does not exist yet; must be implemented with `get_store_recommendations`, `get_preferred_currency`, `set_preferred_currency`.
  - `src/lib/views/StoreView.svelte` and `src/lib/stores/store.svelte.ts` do not exist yet; must be designed for Svelte 5 runes (`$state`, `$derived`) with infinite scroll, tag filters, search, and currency switcher.
  - Test suites enforce strict schemas and boundary behaviors:
    - Base KZT exchange rate is exactly 495.0.
    - KZT formatted with `ru-RU` locale + `₸` (e.g., `29 695 ₸`); whole integer rounding, 1 KZT floor for micro-cents ($0.01 -> 5 KZT).
    - USD formatted with `$${val.toFixed(2)}`.
    - Free-to-play pricing is `"Free to Play"` in both currencies.
    - Query sanitization must strip `['";{}()$]`.
    - Rapid scroll calls must be throttled with an in-flight guard.
    - Catalog exhaustion must emit `has_more: false` and the end message `"You have caught up with all current recommendations"`.
    - Currency fallback: if conversion provider fails, fall back to `USD` with warning.
    - Stale rate notification after >7 days offline.
  - Current baseline builds and tests are 100% green (335/335 passed, cargo check passed, vite build passed, 0 tsc errors).
- **Unexplored areas**: None within the Store & Currency scope.

## Key Decisions Made
- Architecture specified with dual-layer design:
  1. Rust backend (`src-tauri/src/commands/store.rs`) handles authoritative catalog data, query sanitization, pagination, exchange rates, and persistence in `config.toml`.
  2. Svelte 5 frontend (`src/lib/stores/store.svelte.ts` and `src/lib/views/StoreView.svelte`) provides reactive state management via `$state` / `$derived`, IntersectionObserver infinite scrolling, tag/genre filtering, trailer previews, and instant currency switching without full page reloads.

## Artifact Index
- `DISPATCH.md` — Parent instructions
- `progress.md` — Liveness heartbeat and step tracking
- `BRIEFING.md` — Situational awareness
- `handoff.md` — Final comprehensive handoff report
