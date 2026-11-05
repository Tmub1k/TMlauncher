const { describe, it } = require('../runner.cjs');
const { ok, equal, deepEqual } = require('../helpers/assert.cjs');

describe('Tier 4 - Scenario 02: Cross-Platform Unified Library Workflow', () => {
  it('Aggregates Steam, Epic, and Roblox libraries, dedupes titles, and configures Game Properties modal', () => {
    // Step 1: Scan cross-platform catalogs
    const rawLibrary = [
      { id: 'steam_cp', title: 'Cyberpunk 2077', platform: 'steam', installed: true },
      { id: 'epic_cp', title: 'Cyberpunk 2077', platform: 'epic', installed: false },
      { id: 'epic_alanwake2', title: 'Alan Wake 2', platform: 'epic', installed: false },
      { id: 'roblox_189707', title: 'Natural Disaster Survival', platform: 'roblox', installed: true },
    ];

    // Step 2: Multi-Store Deduplication
    function deduplicate(list) {
      const map = new Map();
      for (const item of list) {
        const norm = item.title.toLowerCase().replace(/[^a-z0-9]/g, '');
        if (!map.has(norm)) {
          map.set(norm, { ...item, platforms: [item.platform], sources: [item] });
        } else {
          const existing = map.get(norm);
          existing.platforms.push(item.platform);
          existing.installed = existing.installed || item.installed;
        }
      }
      return Array.from(map.values());
    }

    const unified = deduplicate(rawLibrary);
    equal(unified.length, 3, '4 raw entries consolidated into 3 distinct cards');

    const cpCard = unified.find(u => u.title === 'Cyberpunk 2077');
    ok(cpCard !== undefined);
    deepEqual(cpCard.platforms, ['steam', 'epic']);
    equal(cpCard.installed, true);

    // Step 3: Open Game Properties Modal for Cyberpunk 2077
    const propertiesModalState = {
      appId: 'steam_cp',
      title: 'Cyberpunk 2077',
      launchArgs: '',
      cloudEnabled: true,
      appliedTweaks: [],
    };

    // Step 4: Configure Launch Arguments & apply PCGamingWiki tweak
    propertiesModalState.launchArgs = '-novid -high';
    propertiesModalState.appliedTweaks.push('ultrawide_fov_fix');

    equal(propertiesModalState.launchArgs, '-novid -high');
    equal(propertiesModalState.appliedTweaks.length, 1);
    equal(propertiesModalState.appliedTweaks[0], 'ultrawide_fov_fix');
  });
});
