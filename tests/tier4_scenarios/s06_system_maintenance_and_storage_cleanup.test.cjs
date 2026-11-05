const { describe, it } = require('../runner.cjs');
const { ok, equal } = require('../helpers/assert.cjs');

describe('Tier 4 - Scenario 06: System Maintenance & Storage Cleanup', () => {
  it('Scans local drives -> reclaims orphaned shader caches -> prunes old save snapshots -> updates drive capacity', () => {
    // 1. Initial drive state
    const drives = {
      'C:': { freeGB: 40, totalGB: 500 },
      'D:': { freeGB: 120, totalGB: 1000 },
    };

    // 2. Scan for orphaned shader caches
    const discoveredCaches = [
      { appId: 1091500, gameTitle: 'Cyberpunk 2077 (Uninstalled)', drive: 'C:', sizeGB: 4.5, isOrphaned: true },
      { appId: 570, gameTitle: 'Dota 2 (Active)', drive: 'C:', sizeGB: 2.0, isOrphaned: false },
      { appId: 1245620, gameTitle: 'Elden Ring (Uninstalled)', drive: 'D:', sizeGB: 8.0, isOrphaned: true },
    ];

    function cleanOrphanedCaches(cacheList, driveMap) {
      let totalFreedGB = 0;
      for (const c of cacheList) {
        if (c.isOrphaned) {
          driveMap[c.drive].freeGB += c.sizeGB;
          totalFreedGB += c.sizeGB;
        }
      }
      return totalFreedGB;
    }

    const freedGB = cleanOrphanedCaches(discoveredCaches, drives);
    equal(freedGB, 12.5, '12.5 GB of orphaned shader caches reclaimed');
    equal(drives['C:'].freeGB, 44.5);
    equal(drives['D:'].freeGB, 128.0);

    // 3. Save Vault Snapshot Rotation & Pruning
    const saveVaultHistory = [
      { id: 'snap_1', timestamp: 100 },
      { id: 'snap_2', timestamp: 200 },
      { id: 'snap_3', timestamp: 300 },
      { id: 'snap_4', timestamp: 400 },
      { id: 'snap_5', timestamp: 500 },
      { id: 'snap_6', timestamp: 600 }, // 6th snapshot
    ];

    function enforceMaxSnapshots(snaps, maxLimit = 5) {
      const sorted = [...snaps].sort((a, b) => b.timestamp - a.timestamp);
      return sorted.slice(0, maxLimit);
    }

    const keptSnapshots = enforceMaxSnapshots(saveVaultHistory, 5);
    equal(keptSnapshots.length, 5);
    equal(keptSnapshots[0].id, 'snap_6', 'Newest snapshot retained');
    equal(keptSnapshots.some(s => s.id === 'snap_1'), false, 'Oldest snapshot safely pruned');
  });
});
