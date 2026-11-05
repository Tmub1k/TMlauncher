# Milestone 2 Remediation Review & Adversarial Audit Report

**Reviewer Identity**: `teamwork_reviewer`  
**Role**: Reviewer & Adversarial Critic  
**Target**: Milestone 2 Remediation (Worker: `teamwork_worker_m2_fix`)  
**Verdict**: **APPROVE**  

---

## 1. Observation

Direct empirical observations from source inspection, command executions, and adversarial stress testing:

### 1.1 Remediation Fix 1: Steam Cloud Save File Snapshotting
- **File**: `src-tauri/src/platforms/steam/mod.rs`, lines 621–638 & lines 683–698.
- **Observed Code**:
  ```rust
  pub fn copy_dir_recursive(src: &Path, dst: &Path) {
      if !src.exists() {
          return;
      }
      let _ = std::fs::create_dir_all(dst);
      if let Ok(entries) = std::fs::read_dir(src) {
          for entry in entries.flatten() {
              let src_path = entry.path();
              let dest_path = dst.join(entry.file_name());
              if src_path.is_dir() {
                  Self::copy_dir_recursive(&src_path, &dest_path);
              } else if src_path.is_file() {
                  let _ = std::fs::copy(&src_path, &dest_path);
              }
          }
      }
  }
  ```
  In `sync_cloud`, discovered save directories (`remote_dirs`) are iterated and copied via `Self::copy_dir_recursive(dir, &backup_dir)`.
- **Integrity Check**: Real filesystem traversal and file copy operations. No mock bypasses or dummy stubs.

### 1.2 Remediation Fix 2: Safe Achievement Repair Cache Preservation
- **File**: `src-tauri/src/platforms/steam/mod.rs`, lines 456–518.
- **Observed Code**:
  Remote HTTP call to Steam Web API is attempted first. Only when remote response returns HTTP 200 and a valid `PlayerAchievementsEnvelope` is `achievements_{app_id}.json` updated. If offline or credentials are missing (`!remote_succeeded`), fallback executes:
  ```rust
  reconciled = self.fetch_achievements(app_id).await.unwrap_or_default();
  ```
  which reads the existing `achievements_{app_id}.json` without deleting it.
- **Integrity Check**: Tested in `m2_stress_tests.rs::area2_steam_repair_achievements_offline_wipe_bug`. Cached achievements count verified as `1` after offline repair invocation.

### 1.3 Remediation Fix 3: Roblox Lossy UTF-8 Log Decoding
- **File**: `src-tauri/src/platforms/roblox/mod.rs`, lines 553–556.
- **Observed Code**:
  ```rust
  let Ok(bytes) = std::fs::read(&path) else {
      continue;
  };
  let raw = String::from_utf8_lossy(&bytes);
  ```
- **Integrity Check**: Replaces `read_to_string` with `fs::read` and `String::from_utf8_lossy`, preventing unhandled I/O failures when reading logs containing system-encoded or malformed bytes.

### 1.4 Remediation Fix 4: Roblox Log Regex Bracket Support
- **File**: `src-tauri/src/platforms/roblox/mod.rs`, line 546.
- **Observed Code**:
  ```rust
  let re_join_full = Regex::new(r"Joining place (\d+) \[(.+)\] with universe (\d+)").expect("regex");
  ```
- **Integrity Check**: The greedy `(.+)` pattern captures titles with bracket prefixes (e.g., `[[UPDATE] Blox Fruits [RACE V4]]`), delimited by `] with universe <id>`.

### 1.5 Remediation Fix 5: Unicode Title Normalization in Frontend
- **File**: `src/lib/stores/library.ts`, lines 426–434.
- **Observed Code**:
  ```typescript
  export function normalizeGameTitle(title: string): string {
    if (!title) return '';
    return title
      .toLowerCase()
      .replace(/®|™|©/g, '')
      .replace(/\b(standard|deluxe|game of the year|goty|edition|complete|ultimate|remastered|anniversary|definitive|bundle|director's cut)\b/gu, '')
      .replace(/[^\p{L}\p{N}]/gu, '')
      .trim();
  }
  ```
- **Integrity Check**: The Unicode property escapes `[^\p{L}\p{N}]/gu` preserve Cyrillic (`ведьмак3дикаяохота` vs `герои3возрождениеэрафии`) and CJK characters (`ゼルダの伝説`) rather than collapsing them to empty strings or collisions.

### 1.6 Remediation Fix 6: Backend Edition Suffix Stripping
- **File**: `src-tauri/src/commands/platforms.rs`, lines 71–74.
- **Observed Code**:
  ```rust
  let re_edition = Regex::new(
      r"(?i)\b(standard|deluxe|game of the year|goty|edition|bundle|director's cut|complete|ultimate|remastered|anniversary|definitive)\b",
  )
  .expect("valid regex");
  ```
- **Integrity Check**: Strips `complete`, `ultimate`, `remastered`, `anniversary`, and `definitive`, allowing editions like "Control Ultimate Edition" to unify with "Control".

### 1.7 Remediation Fix 7: Canonical ID / Platform ID Interoperability
- **Files**: `src-tauri/src/core/platform.rs` (lines 35, 110–121, 510–539, 549–585), `src-tauri/src/commands/library.rs` (line 42), `src/lib/components/BentoHub.svelte` (lines 68–75).
- **Observed Code**:
  - `PlatformKind::Canonical` defined and serialized.
  - `GameId::from_str` parses `unified_<key>`, `canonical:<key>`, `canonical_<key>`, and platform-native strings into `PlatformKind::Canonical`.
  - `PlatformRegistry::resolve_canonical` resolves the canonical key against cached games, prioritizing installed games and platform hierarchy (Steam > Epic > Itch > Emulator > Roblox).
  - `PlatformRegistry::launch` and `PlatformRegistry::install` transparently resolve canonical IDs.
  - `BentoHub.svelte` matches `preferred_platform` against `sources` rather than hardcoding `sources[0]`.

### 1.8 Remediation Fix 8: Interactive Platform Switcher UI
- **Files**: `src/lib/components/GameCard.svelte` (lines 33–39, 67–88, 185–199), `src/lib/components/GamePropertiesModal.svelte` (lines 37–41, 43–65, 375–397).
- **Observed Code**:
  - `GameCard.svelte`: `switchPlatform(targetPlatform, event)` stops event propagation, finds matching source by platform or `PLATFORM_LABELS` (mapping `'emulator'` to `'RETRO'`), updates `game.preferred_platform`, `game.platform`, `game.nativeId`, `game.id`, `game.installed`, `game.installDir`, `game.executable`, `game.sizeBytes`, and highlights active platform with an amber ring.
  - `GamePropertiesModal.svelte`: Active Platform button group rendered in Tab 1 (General), interactive switching implemented, and interval timer cleanup registered in `$effect(() => { return () => clearInterval(verifyInterval!); })`.

### 1.9 Remediation Fix 9: Quote-Aware Launch Argument Parser
- **File**: `src/lib/stores/library.ts`, lines 386–424.
- **Observed Code**:
  - Sanitizes shell metacharacters: `rawInput.replace(/[;&|`$><^%()\r\n\0]/g, '')`.
  - State machine tokenizer iterates characters, tracking `inQuote` (`"` or `'`), preserving spaces inside quotes as single argv tokens, and splitting on unquoted whitespace.

### 1.10 Remediation Fix 10: UTF-8 Character Boundary Safety in Rust
- **File**: `src-tauri/src/commands/platforms.rs`, lines 86–97 & 360–362.
- **Observed Code**:
  ```rust
  pub fn safe_truncate_utf8(s: &mut String, max_bytes: usize) {
      if s.len() > max_bytes {
          let boundary = s
              .char_indices()
              .map(|(idx, _)| idx)
              .take_while(|&idx| idx <= max_bytes)
              .last()
              .unwrap_or(0);
          s.truncate(boundary);
      }
  }
  ```
  Invoked in `save_game_properties` to cap launch arguments at 4096 bytes without panicking on multi-byte UTF-8 boundaries.

### 1.11 Command Verification Results (Directly Executed)
- `cargo check --manifest-path src-tauri/Cargo.toml` -> Exited 0 (0 errors, 0 warnings).
- `cargo test --manifest-path src-tauri/Cargo.toml` -> Exited 0 (17 tests passed: 7 unit tests + 10 stress tests in `m2_stress_tests.rs`).
- `npx tsc --noEmit` -> Exited 0 (0 type errors).
- `npm run build` -> Exited 0 (141 modules transformed, `dist/` bundle created).
- `node tests/runner.cjs` -> Exited 0 (335 tests passed across Tiers 1-4, 0 failed).
- `npx tsx tests/stress/adversarial_challenger_m2_2.ts` -> Exited 0 (14 checks passed, 0 vulnerabilities).
- `node tests/stress/m2_frontend_challenge.cjs` -> Exited 0 (All Cyrillic, CJK, and launch arg checks passed).

---

## 2. Logic Chain

1. **Integrity Audit**:
   - Inspected source code for hardcoded test fixtures, dummy implementations, facade classes, or bypassed logic. None found. All 10 remediation areas implement genuine algorithmic solutions (`copy_dir_recursive`, `from_utf8_lossy`, `safe_truncate_utf8`, state-machine tokenizer, `\p{L}\p{N}` regex, `resolve_canonical`).
2. **Correctness & Safety**:
   - Steam save files are physically copied into local backup directories before cloud sync.
   - Achievement repair validates remote connectivity before touching local caches; offline users retain their progress.
   - Non-UTF-8 bytes in Roblox logs are safely replaced with lossy characters rather than crashing the scanner.
   - String truncation avoids slicing inside multi-byte characters, preventing panic crashes.
   - Launch argument parser allows quoted arguments with spaces while stripping all shell command injection operators.
3. **Multi-Store Interoperability**:
   - Non-Latin game titles are properly deduplicated without collapsing.
   - Unified IDs (`unified_<key>`) seamlessly parse and route through `PlatformRegistry` to concrete installations.
   - Interactive platform switching is wired both on library cards and in game properties.
4. **Independent Verification**:
   - All backend and frontend compilation, test runners, and stress suites were run directly and passed with zero errors.

---

## 3. Caveats

No caveats. All 10 remediation items have been verified with complete, production-ready implementations and empirical test execution.

---

## 4. Conclusion

**Verdict: APPROVE**

Milestone 2 remediation satisfies all requirements, resolves all defects raised by Challenger 1 and Challenger 2, introduces no regressions, passes all integrity checks, and executes cleanly across all verification commands.

---

## 5. Verification Method

To reproduce and verify these findings independently:

```powershell
# 1. Rust backend check & tests
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml

# 2. Frontend typecheck & build
npx tsc --noEmit
npm run build

# 3. Automated E2E test runner
node tests/runner.cjs

# 4. Adversarial challenger suites
npx tsx tests/stress/adversarial_challenger_m2_2.ts
node tests/stress/m2_frontend_challenge.cjs
```
