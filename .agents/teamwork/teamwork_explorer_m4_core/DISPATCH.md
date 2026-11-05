# Task Assignment: Milestone 4 Core Engine Exploration

You are an Explorer agent investigating the technical architecture and existing code for Milestone 4 Core Systems.

## Scope of Investigation
1. **Zero-Footprint Game Mode (Feature 19)**:
   - Investigate process priority adjustment and working set trimming via Win32 `SetProcessWorkingSetSize(GetCurrentProcess(), -1, -1)` when launching games.
   - Investigate process monitor to detect game termination and restore foreground priority.
2. **Save-State Time Machine (Feature 20)**:
   - Automated pre/post game session backup engine: snapshot save directories (Steam Auto-Cloud paths, `%APPDATA%`, `%LOCALAPPDATA%`), compress into timestamped zips in a local vault.
   - Restore mechanism with versioning.
3. **Shader Cache & Disk Reclaimer (Feature 23)**:
   - Scan Steam/DirectX shader caches (`%LOCALAPPDATA%\D3DSCache`, `%LOCALAPPDATA%\NVIDIA\DXCache`, `%LOCALAPPDATA%\AMD\DxCache`), orphaned Steam `shadercache` and `compatdata` folders.
   - Report sizes and safely prune upon user request.
4. **Proactive Offline Vault (Feature 28)**:
   - Resilient offline metadata & cached credentials storage.

Working directory: c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\teamwork_explorer_m4_core\
Original request: c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\ORIGINAL_REQUEST.md
PROJECT blueprint: c:\Users\Tmub\Documents\TMlauncher\PROJECT.md
Output: Write handoff.md with concrete data models, Tauri command signatures, Rust implementation strategy, and test plan.


## 2026-11-05T04:55:22Z
Read your task assignment in c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\teamwork_explorer_m4_core\DISPATCH.md.
Also read ORIGINAL_REQUEST.md at c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\ORIGINAL_REQUEST.md and PROJECT.md at c:\Users\Tmub\Documents\TMlauncher\PROJECT.md.
Investigate existing backend files in src-tauri/src/ and plan the implementation for Zero-Footprint Game Mode, Save-State Time Machine, Shader Cache Reclaimer, and Offline Vault.
Write your report to c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\teamwork_explorer_m4_core\handoff.md and notify me via send_message.
