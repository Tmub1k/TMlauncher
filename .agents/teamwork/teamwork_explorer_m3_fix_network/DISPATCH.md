# Task Assignment: Milestone 3 Network Settings & DoH Remediation Exploration

You are an Explorer agent investigating the exact remediation required for Milestone 3 Network and DNS infrastructure.

## Scope & Objective
Reviewer 2 and Challenger 2 raised:
1. `test_network_connection` in `src-tauri/src/commands/network.rs` hardcodes IP `"104.16.123.96"` instead of parsing the real DoH JSON response `Answer[0].data`.
2. Win32 `GetDiskFreeSpaceExW` in `network.rs` uses `format!("{}\\0", path)` which breaks space calculation.
3. SOCKS5 proxy regex does not support IPv6 address bracket format `[::1]`.

Investigate:
- How to parse authentic DoH response JSON (`Answer` array -> record `data`) from Cloudflare (`https://cloudflare-dns.com/dns-query?name=...&type=A`) and Google (`https://dns.google/resolve?name=...&type=A`).
- How to support IPv6 hostnames in proxy validation.
- Win32 disk space path formatting fix.

Working directory: c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\teamwork_explorer_m3_fix_network\
Project directory: c:\Users\Tmub\Documents\TMlauncher
Original request: c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\ORIGINAL_REQUEST.md
PROJECT blueprint: c:\Users\Tmub\Documents\TMlauncher\PROJECT.md
Reviewer 2 handoff: c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\reviewer_m3_2\handoff.md
Challenger 2 handoff: c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\challenger_m3_2\handoff.md
Output: Write handoff.md with clear, step-by-step fix recommendations for the Worker.

## 2026-11-05T04:10:15Z
From: bed11d0d-d77d-423c-a811-8e81f371aec2
Read your task assignment in c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\teamwork_explorer_m3_fix_network\DISPATCH.md.
Also read ORIGINAL_REQUEST.md at c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\ORIGINAL_REQUEST.md, reviewer handoff at c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\reviewer_m3_2\handoff.md, and challenger handoff at c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\challenger_m3_2\handoff.md.
Investigate authentic DoH JSON response parsing (Answer[0].data instead of hardcoded IP), IPv6 bracketed hosts in proxy validation, and Win32 disk space path formatting fix.
Write your final report to c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\teamwork_explorer_m3_fix_network\handoff.md and notify me when complete.
