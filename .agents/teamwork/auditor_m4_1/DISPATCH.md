# Task Assignment: Milestone 4 Forensic Integrity Audit

You are a Forensic Auditor agent (`teamwork_auditor`) tasked with performing a forensic integrity audit of Milestone 4 deliverables.

## Audit Mandate & Integrity Forensics
1. Verify authentic implementations of Zero-Footprint Game Mode, Save-State Time Machine (safe extraction without Zip-Slip), Shader Cache Reclaimer (scanning authentic directories, safe deletion), HowLongToBeat (authentic query/cache fallback cascade), and PCGW tweak engine.
2. Verify NO hardcoded dummy mocks, dummy stubs, or self-certifying mock tests.
3. Verify all builds and tests pass cleanly:
   - `cargo test --manifest-path src-tauri/Cargo.toml`
   - `npx tsc --noEmit`
   - `npm run build`
   - `node tests/runner.cjs`
4. Formulate an explicit verdict: CLEAN or INTEGRITY VIOLATION.

Working directory: c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\auditor_m4_1\
Worker handoff: c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\teamwork_worker_m4\handoff.md
Original request: c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\ORIGINAL_REQUEST.md
PROJECT blueprint: c:\Users\Tmub\Documents\TMlauncher\PROJECT.md
Output: Write handoff.md with comprehensive audit evidence and explicit verdict: CLEAN or INTEGRITY VIOLATION.
