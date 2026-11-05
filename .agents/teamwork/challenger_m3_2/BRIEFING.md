# BRIEFING — 2026-11-05T04:05:00Z

## Mission
Adversarially challenge the Emulators, Directory Picker, and Network/Proxy implementations (Features 14, 15, 16, 17, 18) by writing and running empirical stress tests.

## 🔒 My Identity
- Archetype: empirical challenger
- Roles: critic, specialist
- Working directory: c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\challenger_m3_2\
- Original parent: f93fd91e-112f-4d7d-a956-798970f8fc58
- Milestone: M3 (Store, Emulators & Network Infrastructure)
- Instance: 2 of 2 (Emulators & Filesystem Challenger)

## 🔒 Key Constraints
- Review-only — do NOT modify implementation code directly; write adversarial test scripts and report findings.
- Empirical verification required: every claim and bug must be reproduced by executing code.
- Write report to c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\challenger_m3_2\handoff.md following 5-Component protocol.
- Conclusion MUST state an explicit verdict: APPROVE or REQUEST_CHANGES.
- Report all results back via send_message to f93fd91e-112f-4d7d-a956-798970f8fc58.
- Agent directory (.agents/teamwork/) contains only metadata; tests go into tests/ or standard project test locations.

## Current Parent
- Conversation ID: f93fd91e-112f-4d7d-a956-798970f8fc58
- Updated: 2026-11-05T04:05:00Z

## Review Scope
- **Files reviewed**:
  - `src-tauri/src/commands/emulators.rs`
  - `src-tauri/src/commands/network.rs`
  - `src-tauri/src/core/config.rs`
  - `src-tauri/src/commands/store.rs`
  - `src/lib/stores/network.svelte.ts` / `network.ts`
  - `src/lib/stores/filesystem.svelte.ts` / `filesystem.ts`
  - `src/lib/stores/emulators.svelte.ts` / `emulators.ts`
  - `src/lib/components/DirectoryPickerModal.svelte`
  - `tests/runner.cjs` & Tier 1-4 suites
- **Interface contracts**: PROJECT.md, acceptance criteria R3
- **Review criteria**:
  - Zip-Slip path traversal vulnerability (`../../evil.txt`, Windows backslashes, absolute paths).
  - Emulator installation with nonexistent IDs, corrupted/empty archives.
  - Directory picker path traversal, folder creation with Windows reserved names (`CON`, `PRN`, `AUX`, `NUL`, etc.), invalid characters (`*`, `?`, `:`, `<>`), deep paths (> 260 chars).
  - Network settings validation: malformed proxy URLs, port ranges (1-65535), hostile scheme injection, DoH URL validation.

## Key Decisions Made
- Implemented empirical Rust stress test suite `src-tauri/tests/m3_adversarial_tests.rs` (15 tests covering zip-slip, corrupted archives, reserved names, proxy/DoH).
- Implemented empirical JavaScript stress suite `tests/stress/m3_adversarial_suite.cjs` (22 tests covering frontend regex, DoH RFC 8484, reserved names, path traversal).
- Verified full test suite (`cargo test` -> 51 passed, `node tests/runner.cjs` -> 335 passed, `node tests/stress/m3_adversarial_suite.cjs` -> 22 passed).
- Determined verdict: APPROVE with documented hardening recommendations.

## Artifact Index
- `BRIEFING.md` — persistent situational awareness
- `progress.md` — heartbeat and task log
- `handoff.md` — final 5-component report
- `src-tauri/tests/m3_adversarial_tests.rs` — Rust adversarial test suite
- `tests/stress/m3_adversarial_suite.cjs` — JS adversarial test suite

## Attack Surface
- **Hypotheses tested**:
  - Zip-Slip path traversal via forward slash (`../../`), backslash (`..\..\`), absolute Unix (`/Windows/`), absolute Windows (`C:/Windows/`), and nested traversal (`foo/../../`).
  - Corrupted archive inputs: 0-byte file, random garbage bytes, truncated zip streams.
  - Blind rollback deletion blast radius in `install_emulator`.
  - Windows reserved device names in directory creation (`CON`, `PRN`, `AUX`, `NUL`, `COM1`..`9`, `LPT1`..`9`) and extensions (`CON.txt`, `aux.json`).
  - Invalid character filtering in folder names (`<`, `>`, `:`, `"`, `|`, `?`, `*`).
  - Deep path creation (> 260 characters).
  - Network proxy validation: schemes (`ftp://`, `javascript://`, `file://`), missing ports, port 0, port 65536, CRLF injection, IPv6 addresses (`[::1]`).
  - Custom DoH validation: plain HTTP rejection, empty domains, root paths.
- **Vulnerabilities found**:
  - Rollback Blast Radius (Low/Medium): In `commands/emulators.rs` line 396, if an extraction error occurs, `std::fs::remove_dir_all(&dest_dir)` is executed. If a user provided a custom `target_dir` containing pre-existing files, those files would be deleted on rollback.
  - Windows Device Extension Bypass (Low): Folder creation check matches exact `name_upper` against `CON`, `PRN`, etc., allowing `CON.txt` or `aux.json` to bypass the check.
  - Frontend Path Traversal Match (Low): Frontend regex `/^[a-zA-Z]:\\(?:[^\\/:*?"<>|\r\n]+\\)*[^\\/:*?"<>|\r\n]*$/` permits `..` segments.
  - IPv6 Proxy Incompatibility (Low): Proxy regex does not parse bracketed IPv6 addresses (`http://[::1]:8080`).
- **Untested angles**:
  - Physical network disconnect mid-download of multi-gigabyte real emulator archives (mocked in tests).
