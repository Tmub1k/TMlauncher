# Dispatch for Milestone 1 Sub-orchestrator (UI Redesign & Modular Theming)

- **Identity**: teamwork_orchestrator
- **Working Directory**: c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\sub_orch_milestone_1_ui\
- **Parent**: teamwork_orchestrator (75c17a26-5c14-4320-bf9e-bf30909202d5)
- **Original Request**: c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\ORIGINAL_REQUEST.md
- **Project Scope Document**: c:\Users\Tmub\Documents\TMlauncher\PROJECT.md
- **UI Spec Document**: c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\teamwork_spec_miner_survey_2\handoff.md

## Scope & Objective
Deliver Milestone 1 (Features 1-4):
1. Complete UI Redesign per TURA (Reference 1: matte deep graphite `#09090b`, clean typography, elegant cards) and Liquid Chrome (Reference 2: specular chrome borders, metallic ribbons, liquid badges).
2. Complete elimination of all legacy casual 3D skeuomorphic styling (`btn-game-*`, thick push borders, redundant Russian explanatory labels).
3. Implementation of the Modular Theme System:
   - JSON theme schema and CSS variable architecture in `src/lib/themes/`.
   - Dynamic loader supporting custom drop-in themes in `themes/` and `%APPDATA%/themes/`.
   - Built-in themes: `tura-graphite` (default), `liquid-chrome`, `obsidian-pure`, and `cyber-aurora`.
4. Frameless window setup with sleek 44px TitleBar, custom window controls (minimize, maximize, close), and minimalist navigation tabs.
5. Verification: `npm run build` and `npx tsc --noEmit` must pass with 0 errors. Run Explorer -> Worker -> Reviewer -> Challenger -> Auditor iteration cycle.
