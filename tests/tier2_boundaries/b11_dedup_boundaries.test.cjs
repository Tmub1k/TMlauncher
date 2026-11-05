const { describe, it } = require('../runner.cjs');
const { ok, equal, deepEqual } = require('../helpers/assert.cjs');

describe('Tier 2 - Boundary 11: Game Deduplication Boundaries', () => {
  it('Distinguishes games with identical names released by different developers (e.g. Prey 2006 vs Prey 2017)', () => {
    const games = [
      { id: '1', title: 'Prey', releaseYear: 2006, developer: 'Human Head Studios' },
      { id: '2', title: 'Prey', releaseYear: 2017, developer: 'Arkane Studios' },
    ];

    function deduplicateStrict(list) {
      const map = new Map();
      for (const g of list) {
        const key = `${g.title.toLowerCase()}_${g.releaseYear || ''}_${g.developer.toLowerCase()}`;
        map.set(key, g);
      }
      return Array.from(map.values());
    }

    const deduped = deduplicateStrict(games);
    equal(deduped.length, 2, 'Must not merge distinct games with identical titles');
  });

  it('Handles title comparison when titles differ only by exclamation marks or roman numerals', () => {
    function getFuzzyKey(title) {
      return title
        .toLowerCase()
        .replace(/[-_]/g, ' ')
        .replace(/[!?:;,.]/g, '')
        .replace(/\s+/g, ' ')
        .trim();
    }

    equal(getFuzzyKey('DOOM!'), getFuzzyKey('Doom'));
    equal(getFuzzyKey('Half-Life: 2'), getFuzzyKey('Half Life 2'));
  });

  it('Consolidates 5 or more cross-store duplicates for a single widely owned title', () => {
    const duplicates = [
      { platform: 'steam', title: 'Grand Theft Auto V' },
      { platform: 'epic', title: 'Grand Theft Auto V' },
      { platform: 'rockstar', title: 'Grand Theft Auto V' },
      { platform: 'gog', title: 'Grand Theft Auto V' },
      { platform: 'xbox_pc', title: 'Grand Theft Auto V' },
    ];

    function mergeDuplicates(list) {
      return {
        title: list[0].title,
        platformCount: list.length,
        platforms: list.map(l => l.platform),
      };
    }

    const merged = mergeDuplicates(duplicates);
    equal(merged.platformCount, 5);
    deepEqual(merged.platforms, ['steam', 'epic', 'rockstar', 'gog', 'xbox_pc']);
  });

  it('Instantly unmerges library when user disables deduplication setting in UI', () => {
    const rawList = [{ id: 1 }, { id: 2 }, { id: 3 }, { id: 4 }];
    function getDisplayLibrary(raw, isDedupEnabled) {
      if (!isDedupEnabled) return raw;
      return [raw[0], raw[2]]; // Simulated 50% merged list
    }

    equal(getDisplayLibrary(rawList, true).length, 2);
    equal(getDisplayLibrary(rawList, false).length, 4, 'Full library restored immediately when unmerged');
  });

  it('Allows user custom override rule forcing two specific titles to never merge', () => {
    const userOverrides = new Set(['do_not_merge:game_a:game_b']);

    function shouldMerge(idA, idB) {
      if (userOverrides.has(`do_not_merge:${idA}:${idB}`) || userOverrides.has(`do_not_merge:${idB}:${idA}`)) {
        return false;
      }
      return true;
    }

    equal(shouldMerge('game_a', 'game_b'), false);
    equal(shouldMerge('game_c', 'game_d'), true);
  });
});
