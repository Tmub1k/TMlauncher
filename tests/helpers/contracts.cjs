/**
 * Interface Contracts & Spec Validation Engines for OmniLauncher.
 * Derives expected schemas directly from PROJECT.md § Interface Contracts.
 */

const { ok, equal, hasProperties, isValidHexColor } = require('./assert.cjs');

// 1. Theme Engine Contract Validation
function validateThemeDefinition(theme) {
  hasProperties(theme, ['id', 'name', 'author', 'version', 'colors', 'typography']);
  ok(typeof theme.id === 'string' && theme.id.length > 0, 'theme.id must be non-empty string');
  ok(typeof theme.name === 'string', 'theme.name must be string');
  ok(typeof theme.colors === 'object', 'theme.colors must be an object');

  const requiredColorVars = [
    'bg-canvas',
    'bg-surface',
    'bg-card',
    'border-subtle',
    'accent-chrome',
    'accent-glow',
    'text-primary',
    'text-muted',
  ];

  for (const c of requiredColorVars) {
    ok(c in theme.colors, `Theme colors missing required token: ${c}`);
  }

  ok(typeof theme.typography === 'object', 'theme.typography must be an object');
  hasProperties(theme.typography, ['fontFamily', 'fontSizeBase']);
}

// 2. Steam Profile Contract
function validateSteamProfile(profile) {
  hasProperties(profile, ['steam_id', 'persona_name', 'profile_url', 'avatar_url']);
  ok(/^[0-9]{17}$/.test(profile.steam_id), `steam_id must be 17-digit SteamID64, got: ${profile.steam_id}`);
}

// 3. Steam Achievement Contract
function validateSteamAchievement(ach) {
  hasProperties(ach, ['id', 'name', 'achieved', 'unlock_time']);
  ok(typeof ach.id === 'string', 'achievement id must be string');
  ok(typeof ach.achieved === 'boolean', 'achievement achieved must be boolean');
  ok(typeof ach.unlock_time === 'number', 'achievement unlock_time must be number');
}

// 4. Cloud Sync Result Contract
function validateCloudSyncResult(sync) {
  hasProperties(sync, ['app_id', 'synced', 'local_files', 'remote_files', 'last_sync_timestamp']);
  ok(typeof sync.app_id === 'number', 'app_id must be number');
  ok(typeof sync.synced === 'boolean', 'synced must be boolean');
  ok(Array.isArray(sync.local_files), 'local_files must be an array');
}

// 5. Epic Game Contract
function validateEpicGame(game) {
  hasProperties(game, ['app_name', 'display_name', 'catalog_item_id', 'installed']);
  ok(typeof game.app_name === 'string', 'app_name must be string');
  ok(typeof game.installed === 'boolean', 'installed must be boolean');
}

// 6. Roblox Place Contract
function validateRobloxPlace(place) {
  hasProperties(place, ['place_id', 'name', 'last_played_timestamp']);
  ok(typeof place.place_id === 'number' && place.place_id > 0, 'place_id must be positive number');
}

// 7. Store Recommendations Contract
function validateStorePage(page) {
  hasProperties(page, ['page', 'currency', 'items', 'has_more']);
  ok(typeof page.page === 'number', 'page must be number');
  ok(['USD', 'KZT'].includes(page.currency), `currency must be USD or KZT, got: ${page.currency}`);
  ok(Array.isArray(page.items), 'items must be array');
  for (const item of page.items) {
    hasProperties(item, ['app_id', 'title', 'price_raw', 'price_formatted', 'thumbnail_url']);
  }
}

// 8. Emulator Catalog Entry Contract
function validateEmulatorEntry(emu) {
  hasProperties(emu, ['id', 'name', 'repo', 'version', 'installed', 'executable_name']);
  ok(typeof emu.id === 'string', 'id must be string');
  ok(typeof emu.installed === 'boolean', 'installed must be boolean');
}

// 9. Network Settings Contract
function validateNetworkSettings(settings) {
  hasProperties(settings, ['dns_provider', 'doh_enabled']);
  const validDns = ['cloudflare', 'google', 'quad9', 'system', 'custom'];
  ok(validDns.includes(settings.dns_provider), `Invalid dns_provider: ${settings.dns_provider}`);
}

// 10. Save Snapshot Contract
function validateSaveSnapshot(snapshot) {
  hasProperties(snapshot, ['snapshot_id', 'game_id', 'timestamp', 'archive_size_bytes', 'sha256_checksum']);
  ok(typeof snapshot.snapshot_id === 'string', 'snapshot_id must be string');
  ok(typeof snapshot.archive_size_bytes === 'number', 'archive_size_bytes must be number');
}

// 11. HLTB Record Contract
function validateHltbRecord(hltb) {
  hasProperties(hltb, ['game_title', 'main_story_hours', 'main_extra_hours', 'completionist_hours']);
  ok(typeof hltb.main_story_hours === 'number', 'main_story_hours must be number');
}

// 12. Shader Cache Summary Contract
function validateShaderCacheSummary(summary) {
  hasProperties(summary, ['total_bytes', 'item_count', 'caches']);
  ok(typeof summary.total_bytes === 'number', 'total_bytes must be number');
  ok(Array.isArray(summary.caches), 'caches must be array');
}

// 13. PCGW Record Contract
function validatePcgwRecord(pcgw) {
  hasProperties(pcgw, ['game_title', 'ultrawide_support', 'fps_uncap_supported', 'fov_slider_supported', 'tweaks']);
  ok(typeof pcgw.ultrawide_support === 'boolean', 'ultrawide_support must be boolean');
  ok(Array.isArray(pcgw.tweaks), 'tweaks must be array');
}

module.exports = {
  validateThemeDefinition,
  validateSteamProfile,
  validateSteamAchievement,
  validateCloudSyncResult,
  validateEpicGame,
  validateRobloxPlace,
  validateStorePage,
  validateEmulatorEntry,
  validateNetworkSettings,
  validateSaveSnapshot,
  validateHltbRecord,
  validateShaderCacheSummary,
  validatePcgwRecord,
};
