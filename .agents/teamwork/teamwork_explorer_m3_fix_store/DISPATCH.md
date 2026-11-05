# Task Assignment: Milestone 3 Store Serde & Contract Remediation Exploration

You are an Explorer agent investigating the exact remediation required for Milestone 3 Store contracts and serialization.

## Scope & Objective
Reviewer 1 issued REQUEST_CHANGES on Milestone 3:
1. `StoreItem` and `StorePage` in `src-tauri/src/commands/store.rs` have `#[serde(rename_all = "camelCase")]`, which breaks snake_case expectations in `tests/helpers/contracts.cjs:71`, `store.svelte.ts`, and `StoreView.svelte`.
2. `tests/tier1_features/f12_infinite_store.test.cjs` used self-certifying JS mock instead of exercising Rust IPC serialization.

Investigate:
- How removing `#[serde(rename_all = "camelCase")]` aligns with `StorePage` and `StoreItem` contracts.
- Frontend usage in `src/lib/views/StoreView.svelte` and `src/lib/stores/store.svelte.ts`.
- Exact test to add to `src-tauri/tests/m3_unit_tests.rs` to verify snake_case serialization (`has_more`, `app_id`, `price_raw`, `price_formatted`).

Working directory: c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\teamwork_explorer_m3_fix_store\
Project directory: c:\Users\Tmub\Documents\TMlauncher
Original request: c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\ORIGINAL_REQUEST.md
PROJECT blueprint: c:\Users\Tmub\Documents\TMlauncher\PROJECT.md
Reviewer 1 handoff: c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\reviewer_m3_1\handoff.md
Output: Write handoff.md with clear, step-by-step fix recommendations for the Worker.


## 2026-11-05T04:10:05Z
Read your task assignment in c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\teamwork_explorer_m3_fix_store\DISPATCH.md.
Also read ORIGINAL_REQUEST.md at c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\ORIGINAL_REQUEST.md and reviewer handoff at c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\reviewer_m3_1\handoff.md.
Investigate the serde rename_all camelCase issue in src-tauri/src/commands/store.rs, frontend and test impacts, and formulate the exact remediation steps.
Write your final report to c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\teamwork_explorer_m3_fix_store\handoff.md and notify me when complete.
