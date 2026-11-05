/**
 * Authoritative Fixtures & Mock Generators for OmniLauncher Tests.
 */

const SAMPLE_STEAM_VDF = `"UserRoamingConfigStore"
{
  "Software"
  {
    "Valve"
    {
      "Steam"
      {
        "Apps"
        {
          "730"
          {
            "clouddirty" "0"
            "lastsync" "1700000000"
          }
        }
      }
    }
  }
}`;

const SAMPLE_EPIC_ITEM = JSON.stringify({
  FormatVersion: 0,
  AppVersionString: "1.0.0",
  CatalogItemId: "4a2b3c4d5e",
  DisplayName: "CyberAdventure Deluxe",
  InstallationGuid: "{12345678-ABCD-EF01-2345-6789ABCDEF01}",
  InstallLocation: "C:\\Games\\Epic\\CyberAdventure",
  MainGameAppName: "CyberAdventure",
  AppName: "CyberAdventure",
});

const SAMPLE_ROBLOX_LOG = `2026-11-04T22:00:00.123Z,1a2b,6 [DFLog::HttpTraceError]
2026-11-04T22:01:15.456Z,1a2b,6 [DFLog::GameJoin] Joining place 189707 [Natural Disaster Survival] with universe 61578
2026-11-04T22:30:00.000Z,1a2b,6 [DFLog::GameJoin] Leaving game session`;

const SAMPLE_TURA_THEME = {
  id: "tura-graphite",
  name: "TURA Graphite",
  author: "OmniLauncher Team",
  version: "1.0.0",
  colors: {
    "bg-canvas": "#09090b",
    "bg-surface": "#121215",
    "bg-card": "#18181b",
    "border-subtle": "rgba(255, 255, 255, 0.08)",
    "accent-chrome": "#e4e4e7",
    "accent-glow": "rgba(228, 228, 231, 0.15)",
    "text-primary": "#f4f4f5",
    "text-muted": "#71717a",
  },
  typography: {
    fontFamily: "Cera CY, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif",
    fontSizeBase: "14px",
    letterSpacing: "-0.015em",
  },
};

const SAMPLE_LIQUID_CHROME_THEME = {
  id: "liquid-chrome",
  name: "Liquid Chrome",
  author: "OmniLauncher Team",
  version: "1.0.0",
  colors: {
    "bg-canvas": "#050507",
    "bg-surface": "#0c0d12",
    "bg-card": "rgba(20, 22, 30, 0.75)",
    "border-subtle": "rgba(192, 210, 240, 0.18)",
    "accent-chrome": "linear-gradient(135deg, #f0f4f8 0%, #9ba1a6 50%, #dbe4ee 100%)",
    "accent-glow": "rgba(180, 200, 240, 0.35)",
    "text-primary": "#ffffff",
    "text-muted": "#94a3b8",
  },
  typography: {
    fontFamily: "Cera CY, sans-serif",
    fontSizeBase: "14px",
    letterSpacing: "-0.01em",
  },
};

const EMULATOR_PRESETS = [
  { id: "retroarch", name: "RetroArch", repo: "libretro/RetroArch", tag: "v1.19.1", arch: "x86_64" },
  { id: "dolphin", name: "Dolphin", repo: "dolphin-emu/dolphin", tag: "2407", arch: "x86_64" },
  { id: "pcsx2", name: "PCSX2", repo: "PCSX2/pcsx2", tag: "v2.0.2", arch: "x86_64" },
  { id: "rpcs3", name: "RPCS3", repo: "RPCS3/rpcs3-binaries-win", tag: "v0.0.32", arch: "x86_64" },
  { id: "ppsspp", name: "PPSSPP", repo: "hrydgard/ppsspp", tag: "v1.17.1", arch: "x86_64" },
  { id: "duckstation", name: "DuckStation", repo: "stenzek/duckstation", tag: "latest", arch: "x86_64" },
  { id: "ryujinx", name: "Ryujinx", repo: "Ryujinx/release-channel-master", tag: "1.1.1300", arch: "x86_64" },
];

module.exports = {
  SAMPLE_STEAM_VDF,
  SAMPLE_EPIC_ITEM,
  SAMPLE_ROBLOX_LOG,
  SAMPLE_TURA_THEME,
  SAMPLE_LIQUID_CHROME_THEME,
  EMULATOR_PRESETS,
};
