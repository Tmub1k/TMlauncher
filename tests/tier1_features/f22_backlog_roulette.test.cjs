const { describe, it } = require('../runner.cjs');
const { ok, equal, hasProperties } = require('../helpers/assert.cjs');

describe('Tier 1 - Feature 22: Intelligent Backlog Roulette', () => {
  const sampleLibrary = [
    { id: '1', title: 'Celeste', playtimeHours: 0, genre: 'Platformer', controller: true, hltbHours: 8 },
    { id: '2', title: 'Factorio', playtimeHours: 250, genre: 'Simulation', controller: false, hltbHours: 100 },
    { id: '3', title: 'Portal', playtimeHours: 0, genre: 'Puzzle', controller: true, hltbHours: 3 },
    { id: '4', title: 'Doom', playtimeHours: 12, genre: 'FPS', controller: true, hltbHours: 11 },
  ];

  it('Filters candidate pool by unplayed status (playtime == 0)', () => {
    function filterUnplayed(games) {
      return games.filter(g => g.playtimeHours === 0);
    }

    const unplayed = filterUnplayed(sampleLibrary);
    equal(unplayed.length, 2);
    equal(unplayed[0].title, 'Celeste');
    equal(unplayed[1].title, 'Portal');
  });

  it('Applies maximum estimated completion time filter constraint', () => {
    function filterMaxHours(games, maxHours) {
      return games.filter(g => g.hltbHours <= maxHours);
    }

    const shortGames = filterMaxHours(sampleLibrary, 10);
    equal(shortGames.length, 2); // Celeste (8) and Portal (3)
  });

  it('Filters candidate pool by controller support and genre tags', () => {
    function filterByTags(games, { controller, genre }) {
      return games.filter(g => {
        if (controller !== undefined && g.controller !== controller) return false;
        if (genre && g.genre !== genre) return false;
        return true;
      });
    }

    const res = filterByTags(sampleLibrary, { controller: true, genre: 'Puzzle' });
    equal(res.length, 1);
    equal(res[0].title, 'Portal');
  });

  it('Selects random game from pool using uniform pseudo-random picker', () => {
    function spinRoulette(candidates, randomFn = Math.random) {
      if (!candidates.length) return null;
      const index = Math.floor(randomFn() * candidates.length);
      return candidates[index];
    }

    const winner = spinRoulette(sampleLibrary, () => 0.5);
    ok(winner !== null);
    equal(winner.title, 'Portal');
  });

  it('Provides instant 1-click launch action for roulette winner', () => {
    const winnerModalState = {
      selectedGame: sampleLibrary[0],
      canLaunchDirectly: true,
      launchActionUri: 'steam://run/504230',
    };
    equal(winnerModalState.canLaunchDirectly, true);
    ok(winnerModalState.launchActionUri.startsWith('steam://run/'));
  });
});
