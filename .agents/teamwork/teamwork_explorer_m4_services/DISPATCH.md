# Task Assignment: Milestone 4 External Services & Integrations Exploration

You are an Explorer agent investigating the external services and data enrichment features for Milestone 4.

## Scope of Investigation
1. **HowLongToBeat (HLTB) Integration (Feature 21)**:
   - Query HLTB completion statistics (Main Story, Main + Extra, Completionist) or cached database.
   - Game card progress bar integration.
2. **PCGamingWiki (PCGW) Tweak Engine (Feature 24)**:
   - Query PCGamingWiki API / Cargo data for ultrawide support, FPS uncaps, FOV sliders, and known configuration file paths.
3. **Universal Discord Rich Presence (Feature 26)**:
   - Discord RPC client in Rust via IPC/named pipe with game name, state, elapsed timestamp, and high-res cover art.
4. **DealsRadar & Historical Lows (Feature 27)**:
   - Price tracking and deal aggregation (CheapShark / IsThereAnyDeal API or Steam Store historical low).

Working directory: c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\teamwork_explorer_m4_services\
Original request: c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\ORIGINAL_REQUEST.md
PROJECT blueprint: c:\Users\Tmub\Documents\TMlauncher\PROJECT.md
Output: Write handoff.md with concrete APIs, error fallbacks (offline/rate-limit resilience), Rust commands, and data structures.

## 2026-11-05T04:55:26Z
From: bed11d0d-d77d-423c-a811-8e81f371aec2
Priority: MESSAGE_PRIORITY_HIGH
Content: Read your task assignment in c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\teamwork_explorer_m4_services\DISPATCH.md.
Also read ORIGINAL_REQUEST.md at c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\ORIGINAL_REQUEST.md and PROJECT.md at c:\Users\Tmub\Documents\TMlauncher\PROJECT.md.
Investigate existing backend/service files and design the implementation for HLTB forecasting, PCGamingWiki tweaks, Discord Rich Presence, and DealsRadar.
Write your report to c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\teamwork_explorer_m4_services\handoff.md and notify me via send_message.
