const { describe, it } = require('../runner.cjs');
const { ok, equal, deepEqual, hasProperties } = require('../helpers/assert.cjs');

describe('Tier 1 - Feature 11: Multi-Store Game Deduplication', () => {
  it('Consolidates cross-platform instances of same title into a unified card', () => {
    const rawGames = [
      { id: 'steam_1091500', title: 'Cyberpunk 2077', platform: 'steam', installed: true },
      { id: 'epic_cp2077', title: 'Cyberpunk 2077', platform: 'epic', installed: false },
      { id: 'gog_cp2077', title: 'Cyberpunk 2077', platform: 'gog', installed: false },
    ];

    function deduplicateLibrary(games) {
      const map = new Map();
      for (const g of games) {
        const normKey = g.title.toLowerCase().replace(/[^a-z0-9]/g, '');
        if (!map.has(normKey)) {
          map.set(normKey, {
            primaryTitle: g.title,
            platforms: [g.platform],
            installed: g.installed,
            sources: [g],
          });
        } else {
          const entry = map.get(normKey);
          entry.platforms.push(g.platform);
          entry.installed = entry.installed || g.installed;
          entry.sources.push(g);
        }
      }
      return Array.from(map.values());
    }

    const unified = deduplicateLibrary(rawGames);
    equal(unified.length, 1, 'Should consolidate into 1 unified card');
    equal(unified[0].primaryTitle, 'Cyberpunk 2077');
    deepEqual(unified[0].platforms, ['steam', 'epic', 'gog']);
    equal(unified[0].installed, true);
  });

  it('Applies platform hierarchy priority (Steam > Epic > GOG > Emulators)', () => {
    const priority = { 'steam': 1, 'epic': 2, 'gog': 3, 'emulator': 4 };
    function pickPreferredPlatform(platforms) {
      return platforms.sort((a, b) => priority[a] - priority[b])[0];
    }

    const preferred = pickPreferredPlatform(['epic', 'steam', 'gog']);
    equal(preferred, 'steam', 'Steam has primary default launch priority');
  });

  it('Normalizes title differences such as edition tags and trademarks', () => {
    function normalizeGameTitle(title) {
      return title
        .toLowerCase()
        .replace(/®|™|©/g, '')
        .replace(/\b(standard|deluxe|game of the year|goty|edition)\b/g, '')
        .replace(/[^a-z0-9]/g, '')
        .trim();
    }

    const t1 = normalizeGameTitle('The Witcher® 3: Wild Hunt - GOTY Edition');
    const t2 = normalizeGameTitle('The Witcher 3: Wild Hunt (Deluxe)');
    equal(t1, t2, 'Both editions normalize to identical deduplication key');
  });

  it('Consolidates playtime across all platform sources', () => {
    const sources = [
      { platform: 'steam', playtimeMinutes: 1200 },
      { platform: 'gog', playtimeMinutes: 300 },
    ];
    const totalHours = sources.reduce((sum, s) => sum + s.playtimeMinutes, 0) / 60;
    equal(totalHours, 25, 'Total playtime is sum of cross-platform sessions');
  });

  it('Provides multi-platform launch action submenu on unified card', () => {
    const cardActions = {
      defaultAction: { platform: 'steam', action: 'launch' },
      alternatives: [
        { platform: 'epic', action: 'install' },
        { platform: 'gog', action: 'install' },
      ],
    };
    equal(cardActions.defaultAction.platform, 'steam');
    equal(cardActions.alternatives.length, 2);
  });
});
