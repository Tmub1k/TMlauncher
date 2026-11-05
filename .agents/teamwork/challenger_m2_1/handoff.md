# Milestone 2 Empirical Challenge Report: Platform Integrations & Unification

**Empirical Verdict**: **REJECT**

---

## 1. Observation

Direct empirical observations from source code inspection and test harness execution (`src-tauri/tests/m2_stress_tests.rs` and `tests/stress/m2_frontend_challenge.cjs`):

### 1.1 Scope Item 1: Steam VDF Parsing Robustness
- **Observation 1.1.1 (Safe VDF Error Handling)**:
  In `src-tauri/src/platforms/steam/vdf_parser.rs`:
  - `parse_library_folders_file`, `parse_app_manifest_file`, and `parse_remotecache` return `Err(LauncherError::Io)` on missing files and `Err(LauncherError::VdfParse)` on empty strings, unclosed braces (`"libraryfolders" { "0" { "path" "C:\\Steam"`), binary byte sequences, and invalid JSON.
  - In `parse_app_manifest_str` (lines 176-198), missing `name` defaults to `format!("Steam App {app_id}")`, invalid `StateFlags` defaults to `0`, and negative/non-numeric `appid` fails cleanly with `LauncherError::VdfParse`.
- **Observation 1.1.2 (Scanning Abort on Corrupted libraryfolders.vdf)**:
  In `src-tauri/src/platforms/steam/mod.rs` (lines 871-875):
  ```rust
  let libraries = if vdf_path.exists() {
      parse_library_folders_file(&vdf_path, &steam_root)?
  } else {
      vec![steam_root.clone()]
  };
  ```
  The `?` operator aborts the entire `scan_installed_blocking()` if `libraryfolders.vdf` is corrupt, rather than falling back to `vec![steam_root.clone()]`.

### 1.2 Scope Item 2: Steam Achievements and Cloud Offline / Closed State
- **Observation 1.2.1 (Steam Closed & Offline Profile / Achievements Fallback)**:
  In `src-tauri/src/platforms/steam/mod.rs` (lines 473-565):
  - When Steam is closed (`ActiveUser == 0`), `resolve_profile` falls back to `loginusers.vdf`. `keyvalues-serde` unwraps the root `"users"` object, allowing `table.keys()` to match the 17-digit SteamID.
  - `fetch_achievements` (lines 399-409) returns cached `achievements_{app_id}.json` or `Ok([])` when offline.
- **Observation 1.2.2 (CRITICAL FLAW — Permanent Offline Cache Deletion)**:
  In `src-tauri/src/platforms/steam/mod.rs` (lines 443-471):
  ```rust
  pub async fn repair_achievements(&self, app_id: u32) -> Result<SteamRepairResult> {
      ...
      if let Some(local_app_data) = dirs::data_local_dir() {
          let cache_file = local_app_data.join("OmniLauncher").join(format!("achievements_{app_id}.json"));
          if cache_file.exists() {
              let _ = std::fs::remove_file(&cache_file);
              cache_cleared = true;
          }
      }

      let reconciled = self.fetch_achievements(app_id).await.unwrap_or_default();
  ```
  When `repair_achievements` is called while offline or without a Steam Web API key, it deletes `achievements_{app_id}.json`. `fetch_achievements` cannot reach Steam servers and finds no local cache file, returning `reconciled: 0`. The user's cached achievements are permanently erased.
  - Verified empirically in test `area2_steam_repair_achievements_offline_wipe_bug`:
    ```
    Repair achievements result while offline: SteamRepairResult { app_id: 777777, cache_cleared: true, achievements_reconciled: 0, message: "Reconciled 0 achievements after cleaning cache" }
    Post-repair achievements count: 0
    ```
- **Observation 1.2.3 (CRITICAL FLAW — Phantom Cloud Snapshots / 0 Bytes Copied)**:
  In worker handoff report `teamwork_worker_m2/handoff.md` (lines 13 & 106), the worker claimed:
  > "pre-launch snapshotting makes a copy of files found in remotecache.vdf into %LOCALAPPDATA%\OmniLauncher\cloud_backups\ before any overwrite occurs, fulfilling the data safety requirement."
  However, in `src-tauri/src/platforms/steam/mod.rs` (lines 598-608):
  ```rust
  if let Some(data_dir) = dirs::data_local_dir() {
      let ts = Utc::now().timestamp();
      let backup_dir = data_dir
          .join("OmniLauncher")
          .join("cloud_backups")
          .join(app_id.to_string())
          .join(ts.to_string());
      let _ = tokio::fs::create_dir_all(&backup_dir).await;
      snapshot_id = Some(format!("cloud_snapshot_{app_id}_{ts}"));
  }
  ```
  No files are ever copied into `backup_dir`. Zero file-copy operations exist anywhere in `sync_cloud`.
  - Verified empirically in test `area2_steam_cloud_empty_snapshot_bug_reproduction`:
    ```
    Cloud backup snapshot dir: "C:\Users\Tmub\AppData\Local\OmniLauncher\cloud_backups\888888\1793838231", file count: 0
    ```

### 1.3 Scope Item 3: Epic Games Manifest Scanner Corrupted Files
- **Observation 1.3.1 (Scanner Error Resilience)**:
  In `src-tauri/src/platforms/epic/mod.rs` (lines 161-182):
  - `scan_manifests_blocking` processes `.item` files inside a `match Self::parse_item_file(&path)` loop.
  - Corrupted, truncated, empty, or schema-invalid `.item` files trigger `Err(err)` which is logged as a warning (`skipping malformed Epic .item manifest`), allowing remaining valid manifests to load cleanly.
  - Verified empirically in test `area3_epic_scanner_resilience_with_corrupt_files`: with 4 corrupted files and 1 valid file, scanner yielded exactly `1` game ("Fortnite") with zero crashes.
- **Observation 1.3.2 (Uninstalled Filter Ineffective on Installed Manifests Directory)**:
  In `src-tauri/src/platforms/epic/mod.rs` (lines 185-314), `include_uninstalled` controls whether secondary sources (Legendary, Heroic, logs) are inspected, but does not filter manifests with `installed == false` already parsed from the manifests folder.

### 1.4 Scope Item 4: Roblox Log Parser Stress & Boundaries
- **Observation 1.4.1 (CRITICAL FLAW — Non-UTF8 Log Files Silently Dropped)**:
  In `src-tauri/src/platforms/roblox/mod.rs` (lines 552-555):
  ```rust
  for (mod_time, path) in log_files {
      let Ok(raw) = std::fs::read_to_string(&path) else {
          continue;
      };
  ```
  Roblox logs frequently contain foreign characters, emojis, or Windows system encoding bytes. `std::fs::read_to_string` requires valid UTF-8 and fails with `ErrorKind::InvalidData`. When it fails, `let Ok(raw) ... else { continue; }` silently skips the entire log file, losing all places joined in that session.
  - Verified empirically in test `area4_roblox_log_stress_and_edge_cases`:
    ```
    std::fs::read_to_string on non-utf8 log result: is_err=true
    ```
- **Observation 1.4.2 (CRITICAL FLAW — Title Bracket Truncation Bug)**:
  In `src-tauri/src/platforms/roblox/mod.rs` (line 546):
  ```rust
  let re_join_full = Regex::new(r"Joining place (\d+) \[([^\]]+)\] with universe (\d+)").expect("regex");
  ```
  Many popular Roblox game titles contain square brackets (e.g. `"[UPDATE] Blox Fruits [RACE V4]"` or `"[RP] Brookhaven"`). The regex `[^\]]+` terminates at the first closing bracket `]`, causing the subsequent literal `" with universe"` to fail matching. The title and universe ID are lost, and the game is renamed to `"Roblox Experience #<place_id>"`.
  - Verified empirically in test `area4_roblox_log_stress_and_edge_cases`:
    ```
    Does regex match title with brackets? false
    ```
- **Observation 1.4.3 (Giant Log Performance)**:
  A 10MB log with 100,000 lines was parsed in 1.40s without OOM.

### 1.5 Scope Item 5: Deduplication with Variations, Punctuation & Non-ASCII
- **Observation 1.5.1 (CRITICAL FLAW — Frontend Non-ASCII Collapse & Collisions)**:
  In `src/lib/stores/library.ts` (lines 395-402):
  ```ts
  export function normalizeGameTitle(title: string): string {
    return title
      .toLowerCase()
      .replace(/®|™|©/g, '')
      .replace(/\b(standard|deluxe|game of the year|goty|edition)\b/g, '')
      .replace(/[^a-z0-9]/g, '')
      .trim();
  }
  ```
  `replace(/[^a-z0-9]/g, '')` strips all non-ASCII unicode characters:
  - "Ведьмак 3: Дикая Охота" normalizes to `"3"`.
  - "Герои 3: Возрождение Эрафии" normalizes to `"3"`.
  - Result: completely unrelated Russian/Cyrillic games collide into a single card `"3"`.
  - "Сталкер: Тень Чернобыля" and "ゼルダの伝説" normalize to `""` (empty string).
  - Verified empirically in `tests/stress/m2_frontend_challenge.cjs`:
    ```
    Frontend normalize('Ведьмак 3: Дикая Охота') => '3'
    Frontend normalize('Герои 3: Возрождение Эрафии') => '3'
    [CRITICAL FLAW] Collision: 'Ведьмак 3: Дикая Охота' and 'Герои 3: Возрождение Эрафии' BOTH normalize to '3' in frontend!
    [CRITICAL FLAW] Data Loss: 'Сталкер: Тень Чернобыля' normalizes to EMPTY STRING in frontend!
    ```
- **Observation 1.5.2 (Backend Edition Keyword Gaps & Roman Numerals)**:
  In `src-tauri/src/commands/platforms.rs` (line 71):
  ```rust
  let re_edition = Regex::new(r"(?i)\b(standard|deluxe|game of the year|goty|edition|bundle|director's cut)\b")
  ```
  The regex lacks: `complete`, `ultimate`, `remastered`, `anniversary`, `definitive`, `special`.
  - "The Witcher 3: Wild Hunt" (`thewitcher3wildhunt`) vs "The Witcher 3: Wild Hunt - Complete Edition" (`thewitcher3wildhuntcomplete`) do NOT deduplicate.
  - "Control" (`control`) vs "Control Ultimate Edition" (`controlultimate`) do NOT deduplicate.
  - Roman numerals ("Final Fantasy VII" vs "Final Fantasy 7") do NOT deduplicate.

---

## 2. Logic Chain

1. **Safety Contract Invalidation (Observation 1.2.3)**:
   - Feature 7 acceptance criteria and worker handoff specifically claim pre-launch Steam Cloud snapshotting to protect local save data before overwriting.
   - Empirical inspection shows `sync_cloud` creates an empty folder via `create_dir_all` and copies 0 bytes of data.
   - Relying on this gives users a false sense of save data security; restoring from this snapshot restores nothing.

2. **Offline Resilience Invalidation (Observation 1.2.2)**:
   - Feature 6 specifies achievement repair when desyncs occur.
   - Deleting the local cache prior to verifying network connectivity causes permanent achievement loss for offline users.

3. **Data Loss in Roblox Experiences (Observations 1.4.1 & 1.4.2)**:
   - Roblox log files frequently contain UTF-8 invalid bytes. Skipping entire files on `std::fs::read_to_string` error silently hides recent games from the user's library.
   - Regular expression `re_join_full` fails to handle titles with brackets, which is standard naming convention for Roblox experiences (e.g. `[UPDATE]`, `[BETA]`, `[RP]`).

4. **Multi-Store Unification Defect (Observation 1.5.1)**:
   - The frontend title normalization function strips all non-ASCII characters, creating cross-title collisions on non-English game libraries.

---

## 3. Caveats

- The backend Rust codebase compiles without warnings, and happy-path operations pass the existing 335 opaque-box assertions in `tests/runner.cjs`.
- Epic Games `.item` parser and Steam VDF parser core logic are resilient against malformed files and syntax errors.
- The issues identified are specific to edge cases, offline handling, data-copying omissions, and regex boundary conditions.

---

## 4. Conclusion

**Verdict: REJECT Milestone 2.**

The milestone cannot be approved in its current state due to two critical data-integrity flaws and three high-severity parsing/normalization defects:
1. **Critical Flaw 1**: Steam Cloud pre-launch snapshots are phantom — empty directories created with 0 files copied.
2. **Critical Flaw 2**: Steam achievement repair deletes the user's cached achievements without verifying remote connectivity, causing permanent data wipe when offline.
3. **High Flaw 3**: Roblox log parser drops entire log files when encountering non-UTF-8 bytes instead of lossy decoding.
4. **High Flaw 4**: Roblox log regex drops titles containing square brackets (e.g. `[UPDATE] Game`).
5. **High Flaw 5**: Frontend `normalizeGameTitle` strips non-ASCII characters, causing Cyrillic and CJK titles to collide or become empty.
6. **Medium Flaw 6**: Deduplication misses common edition keywords (`complete`, `ultimate`, `remastered`).

### Remediation Requirements for Approval:
1. In `src-tauri/src/platforms/steam/mod.rs::sync_cloud`, actually copy the files in `summary.remote_dir` (or `local_files`) into `backup_dir`.
2. In `src-tauri/src/platforms/steam/mod.rs::repair_achievements`, fetch remote achievements first before unlinking local cache, or keep a fallback copy if remote fetch fails.
3. In `src-tauri/src/platforms/roblox/mod.rs::get_recent_places`, use `std::fs::read` + `String::from_utf8_lossy` instead of `std::fs::read_to_string`.
4. In `src-tauri/src/platforms/roblox/mod.rs`, update `re_join_full` regex to support titles with brackets (e.g. `r"Joining place (\d+) \[(.+)\] with universe (\d+)"`).
5. In `src/lib/stores/library.ts::normalizeGameTitle`, use Unicode-aware character matching `\p{L}\p{N}` instead of `[^a-z0-9]`.
6. In `src-tauri/src/commands/platforms.rs::normalize_game_title`, add `complete`, `ultimate`, `remastered`, `anniversary`, `definitive` to `re_edition`.

---

## 5. Verification Method

To independently verify all findings and reproduced bugs, execute:

1. **Run the Empirical Challenge Test Suite**:
   ```powershell
   cargo test --test m2_stress_tests --manifest-path src-tauri/Cargo.toml -- --nocapture
   ```
   *Expected outcome*: 10 tests run and pass, reproducing the empty cloud backup dir, the offline achievement wipeout, the Roblox bracket failure, the non-UTF8 read failure, and the edition mismatch.

2. **Run Frontend Normalization Collision Script**:
   ```powershell
   node tests/stress/m2_frontend_challenge.cjs
   ```
   *Expected outcome*: Exposes Cyrillic game collisions ("Ведьмак 3" and "Герои 3" -> "3") and data loss ("Сталкер" -> "").

3. **Verify Existing Regression Suites Remain Functional**:
   ```powershell
   node tests/runner.cjs
   ```
