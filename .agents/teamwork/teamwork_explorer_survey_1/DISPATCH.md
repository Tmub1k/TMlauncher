# Dispatch for Codebase Architecture Explorer

- **Identity**: teamwork_explorer
- **Working Directory**: c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\teamwork_explorer_survey_1
- **Parent**: teamwork_orchestrator (75c17a26-5c14-4320-bf9e-bf30909202d5)
- **Original Request**: c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\ORIGINAL_REQUEST.md
- **Project Directory**: c:\Users\Tmub\Documents\TMlauncher

## Objective
Map the existing OmniLauncher codebase to understand the current architecture, frontend/backend integration, scripts, and build pipeline.
## 2026-11-04T22:50:00Z

Scope & Objectives:
Investigate the existing OmniLauncher codebase at c:\Users\Tmub\Documents\TMlauncher thoroughly:
1. Examine package.json, tsconfig.json, vite.config.ts, and frontend architecture in src/ (Svelte 5 runes, stores, components, routing, views, modal dialogs, styles, Tailwind configuration).
2. Examine src-tauri/ (Cargo.toml, tauri.conf.json, main.rs / lib.rs, commands, plugins, capabilities, window configurations).
3. Inspect how games, Steam, Epic Games, Roblox, themes, and downloads are currently handled (or stubbed).
4. Inspect the launcher executable and launch scripts: OmniLauncher.exe, Start-OmniLauncher.cmd, check why/where terminal/console windows appear on launch, and identify how to make the launcher run cleanly as a native GUI without any console/cmd popups (e.g. windows_subsystem = "windows", cmd execution flags, etc.).
5. Inspect the file system layout of c:\Users\Tmub\Documents\TMlauncher and identify unnecessary files, build clutter, and how the internal file structure should be reorganized for clarity and maintainability.
6. Check build commands (cargo check, cargo build, npm run build) and what dependencies or scripts exist. DO NOT modify any code. DO NOT run destructive commands.

Deliverable:
Write a comprehensive, structured report to c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\teamwork_explorer_survey_1\handoff.md following the Handoff Protocol (Observation, Logic Chain, Caveats, Conclusion, Verification).
Send a message to your parent when done.
