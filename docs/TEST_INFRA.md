# OmniLauncher Test Infrastructure Specification (TEST_INFRA.md)

## 1. Test Philosophy & Principles

OmniLauncher ("Steam 2") is a high-performance desktop launcher built on Tauri v2 (Rust backend) and Svelte 5 (Runes frontend). Its test infrastructure enforces strict quality, safety, and requirement compliance across all 32 inventoried features in `PROJECT.md`.

### Core Tenets
1. **Opaque-Box Requirement Verification**: Tests validate system behaviors, interface contracts, data schemas, and state transitions against the specification without relying on fragile internal implementation details.
2. **Deterministic & Isolated**: Each test suite creates isolated, ephemeral sandboxes for filesystem operations, mock registries, and platform cache data. Tests clean up after themselves and have zero inter-test ordering dependencies.
3. **Four-Tier Defense-in-Depth**:
   - **Tier 1 (Feature Coverage)**: Validates primary happy paths and functional capabilities for every individual feature (Features 1–30, minimum 5 tests per feature).
   - **Tier 2 (Boundary & Corner Cases)**: Exhaustive edge condition verification (empty inputs, corrupted formats, boundary sizes, unicode, network drops, rate limits, invalid configurations).
   - **Tier 3 (Cross-Feature Pairwise Interactions)**: Validates interoperability between subsystems (e.g., Save Vault + Steam Cloud; Currency Switcher + DealsRadar; Theme Engine + UI Shaders; Game Mode + Discord RPC).
   - **Tier 4 (Real-World Workload Scenarios)**: Multi-step end-to-end user journeys (first-time onboarding, retro emulation setup, offline travel mode, system storage maintenance).
4. **Zero-Facade Guarantee**: Every test executes genuine assertions against actual contract schemas, file parsers, transformation algorithms, and state machines. Facade or trivial pass-through tests are strictly forbidden.

---

## 2. Directory Layout

The test suite is organized inside `tests/` at the project root:

```
tests/
├── runner.cjs                       # Standalone Node.js test runner & CLI reporter
├── helpers/
│   ├── assert.cjs                   # Assertion library (equal, deepEqual, match, rejects, throws)
│   ├── contracts.cjs                # Tauri IPC contract definitions & mock drivers
│   ├── fixtures.cjs                 # Test data fixtures (Steam VDFs, Epic manifests, Roblox logs)
│   └── test_context.cjs             # Ephemeral sandbox & temp directory lifecycle manager
├── tier1_features/                  # Feature Coverage Suites (Features 1–30)
│   ├── f01_tura_ui.test.cjs
│   ├── f02_liquid_chrome.test.cjs
│   ├── f03_clutter_elimination.test.cjs
│   ├── f04_modular_themes.test.cjs
│   ├── f05_steam_account.test.cjs
│   ├── f06_steam_achievements.test.cjs
│   ├── f07_steam_cloud.test.cjs
│   ├── f08_properties_modal.test.cjs
│   ├── f09_epic_uninstalled.test.cjs
│   ├── f10_roblox_recent.test.cjs
│   ├── f11_game_dedup.test.cjs
│   ├── f12_infinite_store.test.cjs
│   ├── f13_currency_switch.test.cjs
│   ├── f14_drive_selector.test.cjs
│   ├── f15_emulators_catalog.test.cjs
│   ├── f16_emulator_downloader.test.cjs
│   ├── f17_cloudflare_cdn.test.cjs
│   ├── f18_network_settings.test.cjs
│   ├── f19_zero_footprint.test.cjs
│   ├── f20_save_vault.test.cjs
│   ├── f21_hltb_forecasting.test.cjs
│   ├── f22_backlog_roulette.test.cjs
│   ├── f23_shader_reclaimer.test.cjs
│   ├── f24_pcgw_tweaks.test.cjs
│   ├── f25_session_telemetry.test.cjs
│   ├── f26_discord_rpc.test.cjs
│   ├── f27_deals_radar.test.cjs
│   ├── f28_offline_vault.test.cjs
│   ├── f29_file_structure.test.cjs
│   └── f30_clean_gui_startup.test.cjs
├── tier2_boundaries/                # Boundary & Corner Case Suites (Features 1–30)
│   ├── b01_tura_boundaries.test.cjs
│   ├── b02_chrome_boundaries.test.cjs
│   ├── b03_clutter_boundaries.test.cjs
│   ├── b04_theme_boundaries.test.cjs
│   ├── b05_steam_boundaries.test.cjs
│   ├── b06_achievements_boundaries.test.cjs
│   ├── b07_cloud_boundaries.test.cjs
│   ├── b08_properties_boundaries.test.cjs
│   ├── b09_epic_boundaries.test.cjs
│   ├── b10_roblox_boundaries.test.cjs
│   ├── b11_dedup_boundaries.test.cjs
│   ├── b12_store_boundaries.test.cjs
│   ├── b13_currency_boundaries.test.cjs
│   ├── b14_drive_boundaries.test.cjs
│   ├── b15_catalog_boundaries.test.cjs
│   ├── b16_downloader_boundaries.test.cjs
│   ├── b17_cdn_boundaries.test.cjs
│   ├── b18_network_boundaries.test.cjs
│   ├── b19_gamemode_boundaries.test.cjs
│   ├── b20_vault_boundaries.test.cjs
│   ├── b21_hltb_boundaries.test.cjs
│   ├── b22_roulette_boundaries.test.cjs
│   ├── b23_shader_boundaries.test.cjs
│   ├── b24_pcgw_boundaries.test.cjs
│   ├── b25_telemetry_boundaries.test.cjs
│   ├── b26_discord_boundaries.test.cjs
│   ├── b27_deals_boundaries.test.cjs
│   ├── b28_offline_boundaries.test.cjs
│   ├── b29_structure_boundaries.test.cjs
│   └── b30_startup_boundaries.test.cjs
├── tier3_interactions/              # Cross-Feature Integration Suites
│   ├── i01_steam_cloud_and_save_vault.test.cjs
│   ├── i02_currency_and_deals_radar.test.cjs
│   ├── i03_theme_and_ui_components.test.cjs
│   ├── i04_game_mode_and_discord_rpc.test.cjs
│   ├── i05_offline_vault_and_library_dedup.test.cjs
│   ├── i06_emulator_download_and_library.test.cjs
│   ├── i07_network_settings_and_cdn_proxy.test.cjs
│   ├── i08_properties_modal_and_pcgw_tweaks.test.cjs
│   ├── i09_shader_reclaimer_and_drive_selector.test.cjs
│   └── i10_roulette_and_hltb_integration.test.cjs
└── tier4_scenarios/                 # Real-World Workload Scenarios
    ├── s01_first_time_user_onboarding.test.cjs
    ├── s02_cross_platform_gamer_workflow.test.cjs
    ├── s03_competitive_roblox_and_steam_session.test.cjs
    ├── s04_retro_emulation_setup_and_play.test.cjs
    ├── s05_offline_travel_mode_lifecycle.test.cjs
    └── s06_system_maintenance_and_storage_cleanup.test.cjs
```

---

## 3. Test Runner Invocation

The test suite runs using standard Node.js without requiring third-party testing framework installations:

```powershell
# Run the complete test suite across all 4 tiers
node tests/runner.cjs

# Filter execution by test tier
node tests/runner.cjs --tier=1
node tests/runner.cjs --tier=2
node tests/runner.cjs --tier=3
node tests/runner.cjs --tier=4

# Filter execution by specific feature ID
node tests/runner.cjs --feature=F04
node tests/runner.cjs --feature=F20

# Run with verbose diagnostic logs
node tests/runner.cjs --verbose

# Run with fail-fast (exit on first error)
node tests/runner.cjs --bail
```

---

## 4. Test Tier Specification

| Tier | Name | Target | Min Test Cases | Description |
|------|------|--------|----------------|-------------|
| **Tier 1** | Feature Coverage | Features 1–30 | 150 (≥5 per feature) | Verifies standard operations, expected outputs, schema validity, and nominal contract states. |
| **Tier 2** | Boundary & Corner Cases | Features 1–30 | 150 (≥5 per feature) | Validates limits, null/empty payloads, malformed data, network disconnects, corrupted files, and error handling. |
| **Tier 3** | Cross-Feature Interactions | Major Subsystem Pairs | 25 | Validates multi-module state synchronization, transaction rollback, and cross-platform unified workflows. |
| **Tier 4** | Real-World Workload Scenarios | End-to-End User Journeys | 6 scenarios | Simulates complete user sessions spanning startup, library sync, game launch, session backup, and shutdown. |

**Total Suite Target**: ≥ 330 rigorously verified test cases.

---

## 5. Authoritative Expected Output Derivation

All expected values are derived from:
1. `PROJECT.md § Interface Contracts` for IPC command request/response structures.
2. `ORIGINAL_REQUEST.md` for visual specifications (TURA palette `#09090b`, Liquid Chrome gradients, Cera CY font family).
3. Valve VDF standard grammar for Steam `remotecache.vdf` and `appmanifest_<appid>.acf`.
4. Epic Games Launcher `.item` and `Manifests/` directory structure.
5. Roblox Windows Client log formats (`%LOCALAPPDATA%\Roblox\logs`).
6. GitHub Releases REST API v3 schema for emulator downloads.
7. RFC 8484 (DNS over HTTPS) and RFC 1928 (SOCKS5 proxy) standards.
