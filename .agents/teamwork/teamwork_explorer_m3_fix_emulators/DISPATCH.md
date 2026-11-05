# Task Assignment: Milestone 3 Emulators & Win32 Disk Free Space Remediation Exploration

You are an Explorer agent investigating the exact remediation required for Milestone 3 Emulator downloading and Win32 disk space.

## Scope & Objective
Reviewer 2 and Challenger 2 raised:
1. `install_emulator` in `src-tauri/src/commands/emulators.rs` created a dummy local zip with a 32-byte stub instead of genuinely downloading the release from GitHub/network.
2. Rollback on extraction failure: `remove_dir_all(&dest_dir)` risks deleting an entire user-selected folder. Must only clean up files/subdirectories created during this installation.
3. Win32 `GetDiskFreeSpaceExW`: Path formatting with `format!("{}\\0", path)` before UTF-16 conversion corrupts the path. Must encode directly to wide string ending with 0.
4. Reserved device names: handling files with extensions like `CON.txt`.

Investigate:
- How to implement genuine GitHub asset download / authentic release download in `install_emulator` via `reqwest` streaming, with graceful error handling and genuine extraction.
- How to restrict rollback cleanup safely.
- Exact fix for Win32 `GetDiskFreeSpaceExW` in `emulators.rs` and `network.rs`.

Working directory: c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\teamwork_explorer_m3_fix_emulators\
Project directory: c:\Users\Tmub\Documents\TMlauncher
Original request: c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\ORIGINAL_REQUEST.md
PROJECT blueprint: c:\Users\Tmub\Documents\TMlauncher\PROJECT.md
Reviewer 2 handoff: c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\reviewer_m3_2\handoff.md
Challenger 2 handoff: c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\challenger_m3_2\handoff.md
Output: Write handoff.md with clear, step-by-step fix recommendations for the Worker.


## 2026-11-05T04:10:09Z
Read your task assignment in c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\teamwork_explorer_m3_fix_emulators\DISPATCH.md.
Also read ORIGINAL_REQUEST.md at c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\ORIGINAL_REQUEST.md, reviewer handoff at c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\reviewer_m3_2\handoff.md, and challenger handoff at c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\challenger_m3_2\handoff.md.
Investigate the genuine emulator downloader implementation via reqwest, safe directory rollback, Win32 GetDiskFreeSpaceExW null termination in emulators.rs/network.rs, and reserved names with extensions.
Write your final report to c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\teamwork_explorer_m3_fix_emulators\handoff.md and notify me when complete.
