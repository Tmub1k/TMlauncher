# Task Assignment: Milestone 3 Quality & Integrity Remediation

You are a Worker agent assigned to implement the comprehensive remediation for Milestone 3 (Store, Emulators & Network Infrastructure) based on the detailed findings and patches provided by three independent Explorer agents.

## MANDATORY INTEGRITY WARNING
DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A teamwork_auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.

## Input Reports & Patches
1. Store Explorer Report & Patch:
   - Report: `c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\teamwork_explorer_m3_fix_store\handoff.md`
   - Patch: `c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\teamwork_explorer_m3_fix_store\store_serde_fix.patch`
2. Emulators & Win32 Explorer Report:
   - Report: `c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\teamwork_explorer_m3_fix_emulators\handoff.md`
3. Network & DoH Explorer Report:
   - Report: `c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\teamwork_explorer_m3_fix_network\handoff.md`
4. Blueprint & Requests:
   - `c:\Users\Tmub\Documents\TMlauncher\PROJECT.md`
   - `c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\ORIGINAL_REQUEST.md`

## Specific Remediation Tasks
1. **Store Serialization Parity (`src-tauri/src/commands/store.rs`)**:
   - Remove `#[serde(rename_all = "camelCase")]` from `StoreItem` and `StorePage` structs so they serialize in snake_case (`has_more`, `app_id`, `price_raw`, `price_formatted`, `discount_percent`).
   - Add the snake_case serialization unit test in `src-tauri/tests/m3_unit_tests.rs`.
2. **Win32 `GetDiskFreeSpaceExW` Fix (`src-tauri/src/commands/network.rs` & `src-tauri/src/commands/emulators.rs`)**:
   - Fix path encoding: do NOT do `format!("{}\\0", path)` before UTF-16 conversion! Use `path.to_string_lossy().encode_utf16().chain(std::iter::once(0)).collect::<Vec<u16>>()`.
   - Ensure `free_space_bytes` and `free_space_gb` return authentic positive values for valid local paths.
3. **Genuine Emulator Downloader & Safe Rollback (`src-tauri/src/commands/emulators.rs`)**:
   - Replace the 32-byte mock PE stub with authentic streaming download using `reqwest` fetching GitHub release assets or configured direct asset mirrors.
   - Fix extraction rollback: track specific created files and subdirectories so extraction failures do NOT execute `remove_dir_all(&dest_dir)` on user-selected directories.
   - Fix reserved device names: strip extensions before checking stems against `CON`, `PRN`, `AUX`, `NUL`, `COM1-9`, `LPT1-9`.
4. **Authentic DoH Response Parsing & IPv6 Proxy Regex (`src-tauri/src/commands/network.rs`)**:
   - In `test_network_connection`, parse the actual DoH JSON response `Answer[0].data` instead of returning a hardcoded IP `"104.16.123.96"`.
   - Update SOCKS5 / HTTP proxy validation to handle bracketed IPv6 hostnames like `[::1]:1080`.

## Verification Commands
After modifying the code:
1. `cargo check --manifest-path src-tauri/Cargo.toml`
2. `cargo test --manifest-path src-tauri/Cargo.toml`
3. `npx tsc --noEmit`
4. `npm run build`
5. `node tests/runner.cjs`

Write your comprehensive completion report to `c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\teamwork_worker_m3_fix\handoff.md` and notify the orchestrator.


## 2026-11-05T04:20:00Z
[Message] timestamp=2026-11-05T04:20:00Z sender=bed11d0d-d77d-423c-a811-8e81f371aec2 priority=MESSAGE_PRIORITY_HIGH content=Read your task assignment in c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\teamwork_worker_m3_fix\DISPATCH.md.
Also read the 3 explorer reports at:
- c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\teamwork_explorer_m3_fix_store\handoff.md (and store_serde_fix.patch)
- c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\teamwork_explorer_m3_fix_emulators\handoff.md
- c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\teamwork_explorer_m3_fix_network\handoff.md
Implement the remediation for Milestone 3 across store.rs, emulators.rs, network.rs, and tests.
Run the verification builds and tests (cargo check, cargo test, npx tsc, npm run build, node tests/runner.cjs).
Write your completion report to c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\teamwork_worker_m3_fix\handoff.md and notify me via send_message.

MANDATORY INTEGRITY WARNING:
DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A teamwork_auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.
