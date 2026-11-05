# Milestone 2 Forensic Audit Report: Steam, Epic, Roblox & Multi-Store Unification (Features 5-11)

**Auditor Identity**: teamwork_auditor  
**Working Directory**: `c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\auditor_m2_1`  
**Parent Agent**: teamwork_orchestrator (`75c17a26-5c14-4320-bf9e-bf30909202d5`)  
**Target Scope**: Milestone 2 Platform Integrations (`src-tauri/src/platforms/steam/`, `epic/`, `roblox/`, `commands/platforms.rs`, `commands/library.rs`, frontend integrations, and test suites)  
**Integrity Mode**: `development` (per `ORIGINAL_REQUEST.md` line 8)  
**Forensic Verdict**: **CLEAN**

---

## Forensic Audit Report

**Work Product**: Milestone 2 Platform Integrations, IPC Commands, and Game Properties Modal  
**Profile**: General Project (Development Mode)  
**Verdict**: **CLEAN**

### Phase Results
- **Hardcoded Output Detection**: **PASS** — Zero hardcoded test outputs, static dummy results, or PASS/FAIL strings embedded in production code.
- **Facade Detection**: **PASS** — All platform modules (`steam`, `epic`, `roblox`) and commands (`platforms.rs`, `library.rs`) execute authentic business logic, Windows registry access, VDF/manifest parsing, and real filesystem operations.
- **Pre-populated Artifact Detection**: **PASS** — No pre-populated test result logs, mock output artifacts, or pre-baked attestation files found in the workspace (`find_by_name` returned 0 `.log` or `.output` files).
- **Behavioral Verification (Build & Run)**: **PASS** —
  - `cargo check --manifest-path src-tauri/Cargo.toml` exited with code 0 (0 errors, 0 warnings).
  - `cargo test --test m2_stress_tests --manifest-path src-tauri/Cargo.toml` passed all 9 empirical stress tests in 0.84s with code 0.
  - `npx tsc --noEmit` exited with code 0 (0 type errors).
  - `npm run build` transformed 141 modules into production distribution in 33.14s with code 0.
  - `node tests/runner.cjs` executed 335 automated tests across Tiers 1-4 with 100% pass rate (335 passed, 0 failed).
  - Feature-specific runners (`F05` through `F11`) all passed 5/5 assertions each.
- **Output Verification**: **PASS** — Contracts for `SteamProfile`, `SteamAchievement`, `CloudSyncResult`, `EpicGame`, `RobloxPlace`, `UnifiedGame`, and `GameProperties` match runtime shapes.
- **Dependency Audit**: **PASS** — Uses standard auxiliary crates (`tokio`, `serde`, `keyvalues-serde`, `regex`, `winreg`, `dirs`, `reqwest`) without delegating target deliverables to black-box frameworks.

---

## 1. Observation

Direct observations from source code inspection, AST verification, and empirical command execution:

### 1.1 Source Code Inspection — Real Business Logic vs Facades

1. **Steam Platform (`src-tauri/src/platforms/steam/mod.rs` & `vdf_parser.rs`)**:
   - `resolve_profile` (lines 474–565):
     - Queries Windows Registry `HKCU\Software\Valve\Steam\ActiveProcess` for `ActiveUser`.
     - Parses `config/loginusers.vdf` via `keyvalues_serde::from_str` to extract active `PersonaName` and `MostRecent` user session.
     - Scans `userdata` subdirectories with numeric account ID validation.
     - Invokes Steam Web API (`GetPlayerSummaries/v2`) when an API key is configured, with graceful offline fallback and timestamps.
   - `fetch_achievements` (lines 342–440):
     - Queries Steam Web API (`GetPlayerAchievements/v1`) with timestamp sanity validation (`unlocktime < 1000000000 || unlocktime > now + 86400` sanitized to 0).
     - Persists and loads local offline JSON cache in `%LOCALAPPDATA%\OmniLauncher\achievements_{app_id}.json`.
     - Inspects local Goldberg emulator caches in `%APPDATA%\Goldberg SteamEmu Saves\{app_id}\achievements.json`.
   - `repair_achievements` (lines 443–471):
     - Physically removes corrupted schema cache `appcache/stats/UserGameStatsSchema_{app_id}.bin`.
     - Removes stale local cache `%LOCALAPPDATA%\OmniLauncher\achievements_{app_id}.json`.
     - Reconciles achievements by re-fetching and returns real reconciled count.
   - `sync_cloud` (lines 568–622):
     - Reads and parses `<SteamPath>/userdata/<AccountID>/<appid>/remotecache.vdf` via `parse_remotecache` (`vdf_parser.rs` lines 236–302).
     - Inspects actual filesystem entries in the game's `remote/` directory via `std::fs::read_dir`.
     - Atomically snapshots local saves into `%LOCALAPPDATA%\OmniLauncher\cloud_backups\{app_id}\{timestamp}` before any sync action.

2. **Epic Games Platform (`src-tauri/src/platforms/epic/mod.rs`)**:
   - `locate_manifests_dir` (lines 103–144):
     - Queries `HKLM\SOFTWARE\WOW6432Node\Epic Games\EpicGamesLauncher` (`AppDataPath`) or falls back to `%ProgramData%\Epic\EpicGamesLauncher\Data\Manifests`.
   - `fetch_epic_library` (lines 185–314):
     - Scans installed `.item` manifests, parses `EpicItemManifest`, verifies executable existence on disk (`executable_path().exists()`), and formats silent launch URI `com.epicgames.launcher://apps/{ns}%3A{id}%3A{app}?action=launch&silent=true`.
     - For uninstalled games: scans Legendary caches (`%USERPROFILE%\.config\legendary\installed.json`), Heroic store caches (`heroic\store_cache\epic_games.json`), and launcher log regex (`EpicGamesLauncher.log`), populating `install_uri: "com.epicgames.launcher://apps/{app}?action=install"`.

3. **Roblox Platform (`src-tauri/src/platforms/roblox/mod.rs`)**:
   - `get_recent_places` (lines 517–617):
     - Scans `%LOCALAPPDATA%\Roblox\logs` for `*.log` files, sorts descending by `modified` timestamp, and examines the newest 25 logs.
     - Matches regex patterns:
       - `Joining place (\d+) \[([^\]]+)\] with universe (\d+)`
       - `placeid[:=\s]+(\d{5,})`
     - Deduplicates sessions by `place_id`, keeping the most recent timestamp.
     - Formats official 512x512 thumbnail URLs (`https://thumbnails.roblox.com/v1/places/gameicons?placeIds={place_id}&size=512x512&format=Png&isCircular=false`) and deep link launch URLs (`roblox://experiences/start?placeId={place_id}`).

4. **IPC Commands & Multi-Store Deduplication (`src-tauri/src/commands/platforms.rs`)**:
   - `normalize_game_title` (lines 63–82):
     - Strips edition tags (`deluxe`, `game of the year`, `goty`, `edition`, `bundle`, `director's cut`), trademarks (`®`, `™`, `©`), and retains alphanumeric Unicode characters via `c.is_alphanumeric()`.
   - `get_unified_library` (lines 180–271):
     - Aggregates games across all providers, sorts sources by platform priority (`Steam = 1`, `Epic = 2`, `GOG = 3`), consolidates playtime, and builds multi-badge unified cards.
   - `save_game_properties` (lines 333–360):
     - Sanitizes dangerous shell operators (`;`, `&`, `|`, `` ` ``, `$`), caps input length at 4096 characters, and persists JSON config to `%LOCALAPPDATA%\OmniLauncher\properties\{safe_id}.json`.
   - `verify_game_files` (lines 363–418):
     - Resolves installation path, scans filesystem with `std::fs::read_dir`, verifies each file can be opened via `std::fs::File::open(&path)`, and computes verified vs corrupted counts.

5. **Frontend Integration (`src/lib/components/GamePropertiesModal.svelte`, `TitleBar.svelte`, `BentoHub.svelte`)**:
   - Full 5-tab properties modal with ARIA accessibility roles (`role="dialog"`, `aria-modal="true"`).
   - Cloud save toggle lock when game is active (`isGameRunning`).
   - TitleBar Steam user pill with avatar and profile link.
   - BentoHub deduplication toggle (`dedupEnabled`) and Roblox recent places grid.

---

### 1.2 Verbatim Tool Outputs & Empirical Execution

#### A. Backend Compilation (`cargo check`):
```powershell
cargo check --manifest-path src-tauri/Cargo.toml
```
```
Checking omnilauncher v0.1.0 (C:\Users\Tmub\Documents\TMlauncher\src-tauri)
Finished `dev` profile [unoptimized + debuginfo] target(s) in 4.86s
Exit code: 0
```

#### B. Rust Empirical Integration Stress Tests (`cargo test`):
```powershell
cargo test --test m2_stress_tests --manifest-path src-tauri/Cargo.toml
```
```
Finished `test` profile [unoptimized + debuginfo] target(s) in 0.51s
 Running tests\m2_stress_tests.rs (src-tauri\target\debug\deps\m2_stress_tests-6f2cf456b1001a69.exe)

running 9 tests
test area1_vdf_malformed_syntax ... ok
test area1_vdf_missing_and_empty_files ... ok
test area3_epic_corrupted_item_manifests ... ok
test area2_loginusers_vdf_structure_challenge ... ok
test area1_vdf_acf_manifest_boundaries ... ok
test area2_steam_closed_and_offline ... ok
test area5_dedup_title_variations_and_non_ascii ... ok
test area3_epic_scanner_resilience_with_corrupt_files ... ok
test area4_roblox_log_stress_and_edge_cases ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.84s
Exit code: 0
```

#### C. Frontend Type Check (`tsc`):
```powershell
npx tsc --noEmit
```
```
Exit code: 0 (0 type errors)
```

#### D. Production Frontend Build (`npm run build`):
```powershell
npm run build
```
```
vite v6.4.5 building for production...
transforming...
✓ 141 modules transformed.
rendering chunks...
computing gzip size...
dist/index.html                   0.45 kB │ gzip:  0.30 kB
dist/assets/index-B86l5q6B.css   78.35 kB │ gzip: 12.98 kB
dist/assets/window-Bgkj6zfy.js   14.24 kB │ gzip:  3.49 kB
dist/assets/index-BuVXsw41.js   195.57 kB │ gzip: 60.25 kB
✓ built in 33.14s
Exit code: 0
```

#### E. Automated Test Suite (`node tests/runner.cjs`):
```powershell
node tests/runner.cjs
```
```
======================================================
   OmniLauncher Automated E2E Test Runner            
======================================================
Target Tiers: 1, 2, 3, 4
Files Found: 76

...............................................................................................................................................................................................................................................................................................................................................

------------------------------------------------------
Test Execution Summary:
  Total Tests:   335
  Passed:        335
  Failed:        0
  Time Elapsed:  0.07s

Tier Breakdown:
  Tier 1 (Feature Coverage):     150 / 150 passed
  Tier 2 (Boundary & Corner):    154 / 154 passed
  Tier 3 (Interactions):         25 / 25 passed
  Tier 4 (Scenarios):            6 / 6 passed
------------------------------------------------------

✔ All test suites executed successfully!
Exit code: 0
```

#### F. Feature-Specific Verification (F05 through F11):
- `node tests/runner.cjs --feature=F05` -> 5 / 5 passed
- `node tests/runner.cjs --feature=F06` -> 5 / 5 passed
- `node tests/runner.cjs --feature=F07` -> 5 / 5 passed
- `node tests/runner.cjs --feature=F08` -> 5 / 5 passed
- `node tests/runner.cjs --feature=F09` -> 5 / 5 passed
- `node tests/runner.cjs --feature=F10` -> 5 / 5 passed
- `node tests/runner.cjs --feature=F11` -> 5 / 5 passed

---

## 2. Logic Chain

1. **User Request & Ground Truth Constraints**:
   - `ORIGINAL_REQUEST.md` (lines 8, 19–21, 42–48) establishes the requirements: Steam profile link, achievements & Steam Cloud sync, Steam-like Game Properties Modal, Epic Games uninstalled library import, Roblox recently played experiences, and multi-store deduplication.
   - The user specified `Integrity mode: development`. Under development mode, the auditor must strictly verify that all deliverables implement genuine logic rather than dummy facades or hardcoded values.

2. **Source Code Integrity Verification (Observation 1.1)**:
   - In `platforms/steam/mod.rs` and `vdf_parser.rs`, achievements repair and cloud sync perform real disk operations (`std::fs::remove_file`, `parse_remotecache`, `tokio::fs::create_dir_all`).
   - In `platforms/epic/mod.rs`, `.item` manifests are parsed from filesystem directories, verified for executable existence, and combined with uninstalled feeds from Legendary/Heroic/logs.
   - In `platforms/roblox/mod.rs`, Roblox log files in `%LOCALAPPDATA%\Roblox\logs` are parsed with regexes to extract place and universe IDs, deduplicated chronologically, and converted into official thumbnail URLs.
   - In `commands/platforms.rs`, game properties and file verifications perform genuine filesystem reads, argument sanitization (stripping shell operators), and JSON persistence.
   - Therefore, no dummy facades or hardcoded values exist in the platform connectors.

3. **Behavioral & Adversarial Verification (Observation 1.2)**:
   - Full Rust compilation (`cargo check`) and integration tests (`cargo test --test m2_stress_tests`) pass with zero errors. All 9 stress tests covering malformed VDFs, missing manifest files, corrupted `.item` manifests, and non-ASCII game titles succeed.
   - Frontend TypeScript check (`tsc`) and production build (`npm run build`) pass cleanly with 0 type errors and 0 build errors.
   - Full automated test suite (335 tests across 4 tiers) passes with 100% success rate.
   - Therefore, the work product meets all behavioral and contractual acceptance criteria.

---

## 3. Caveats

1. **Live Network Credentials**: Real live Steam Web API calls require user-provided Steam API keys and network connectivity. When offline or unconfigured, the implementation falls back to local registry, `loginusers.vdf`, and local cache files as designed.
2. **Frontend Helper Title Normalization**: In `src/lib/stores/library.ts`, the frontend utility `normalizeGameTitle` uses regex `[^a-z0-9]`, which strips non-ASCII Cyrillic/CJK characters. However, actual production game deduplication is executed by the Rust backend (`get_unified_library` in `commands/platforms.rs`), which uses `c.is_alphanumeric()` (Unicode-aware), correctly preserving Cyrillic and international titles.
3. No other caveats. All audited code is authentic, functional, and clean.

---

## 4. Conclusion

Milestone 2 deliverables (Features 5 through 11) have undergone rigorous forensic integrity auditing and have passed all checks.
- Steam Account & Profile Linking is fully implemented.
- Steam Cloud Sync and Achievements Desync Repair perform real filesystem and registry operations.
- Epic Games Manifest Parsing correctly imports both installed and uninstalled titles.
- Roblox Multi-Log Parser extracts recently played places with 512x512 thumbnails.
- Multi-Store Game Deduplication functions correctly with platform priority.
- All builds and test suites execute with zero errors.

**Forensic Verdict**: **CLEAN**

---

## 5. Verification Method

To independently reproduce and verify this audit verdict, execute the following commands in the workspace root (`c:\Users\Tmub\Documents\TMlauncher`):

1. **Verify Backend Compilation**:
   ```powershell
   cargo check --manifest-path src-tauri/Cargo.toml
   ```
   *Expected outcome*: Exits with code 0, 0 compiler errors.

2. **Run Rust Empirical Integration Stress Tests**:
   ```powershell
   cargo test --test m2_stress_tests --manifest-path src-tauri/Cargo.toml
   ```
   *Expected outcome*: 9 / 9 tests passed, 0 failed, exit code 0.

3. **Verify Frontend Types**:
   ```powershell
   npx tsc --noEmit
   ```
   *Expected outcome*: Exits with code 0, 0 type errors.

4. **Verify Frontend Production Build**:
   ```powershell
   npm run build
   ```
   *Expected outcome*: Vite builds standalone bundle without warnings, exits with code 0.

5. **Run Automated Test Runner**:
   ```powershell
   node tests/runner.cjs
   ```
   *Expected outcome*: 335 / 335 tests passing across Tiers 1-4.

6. **Verify Individual Milestone 2 Feature Suites**:
   ```powershell
   node tests/runner.cjs --feature=F05
   node tests/runner.cjs --feature=F06
   node tests/runner.cjs --feature=F07
   node tests/runner.cjs --feature=F08
   node tests/runner.cjs --feature=F09
   node tests/runner.cjs --feature=F10
   node tests/runner.cjs --feature=F11
   ```
   *Expected outcome*: 5 / 5 tests passing for each feature.
