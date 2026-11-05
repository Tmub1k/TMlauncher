const { describe, it } = require('../runner.cjs');
const { ok, equal, match, hasProperties } = require('../helpers/assert.cjs');
const { validateSteamAchievement } = require('../helpers/contracts.cjs');

describe('Tier 1 - Feature 06: Steam Achievements & Desync Repair', () => {
  it('Validates achievement item schema and unlock timestamp', () => {
    const ach = {
      id: 'ACH_KILL_DRAGON',
      name: 'Dragon Slayer',
      achieved: true,
      unlock_time: 1699999999,
      icon_url: 'https://steamcdn.example.com/ach_dragon.png',
    };
    validateSteamAchievement(ach);
    equal(ach.achieved, true);
    ok(ach.unlock_time > 0);
  });

  it('Calculates total completion percentage across game achievement list', () => {
    const list = [
      { id: 'ACH_1', name: 'Start', achieved: true, unlock_time: 1700000 },
      { id: 'ACH_2', name: 'Boss 1', achieved: true, unlock_time: 1700001 },
      { id: 'ACH_3', name: 'Boss 2', achieved: false, unlock_time: 0 },
      { id: 'ACH_4', name: 'Master', achieved: false, unlock_time: 0 },
    ];

    function calculateProgress(achievements) {
      if (!achievements.length) return 0;
      const unlocked = achievements.filter(a => a.achieved).length;
      return Math.round((unlocked / achievements.length) * 100);
    }

    const pct = calculateProgress(list);
    equal(pct, 50, '2 out of 4 achievements equals 50%');
  });

  it('Detects desync between local achievements cache and Steam API', () => {
    const local = { 'ACH_1': true, 'ACH_2': false };
    const remote = { 'ACH_1': true, 'ACH_2': true };

    function findDesync(loc, rem) {
      const desynced = [];
      for (const [k, v] of Object.entries(rem)) {
        if (loc[k] !== v) desynced.push(k);
      }
      return desynced;
    }

    const diff = findDesync(local, remote);
    equal(diff.length, 1);
    equal(diff[0], 'ACH_2');
  });

  it('Local stats repair routine reconciles mismatched achievement records', () => {
    const local = { 'ACH_1': true, 'ACH_2': false };
    const remote = { 'ACH_1': true, 'ACH_2': true };

    function repairAchievements(loc, rem) {
      return { ...loc, ...rem };
    }

    const reconciled = repairAchievements(local, remote);
    equal(reconciled['ACH_2'], true, 'Desync repaired to match authoritative remote state');
  });

  it('Handles games with zero achievements without divide-by-zero errors', () => {
    const emptyAchievements = [];
    const unlockedCount = emptyAchievements.filter(a => a.achieved).length;
    const progress = emptyAchievements.length ? (unlockedCount / emptyAchievements.length) * 100 : 0;
    equal(progress, 0);
  });
});
