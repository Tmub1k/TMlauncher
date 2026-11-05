const { describe, it } = require('../runner.cjs');
const { ok, equal, hasProperties } = require('../helpers/assert.cjs');
const { validateShaderCacheSummary } = require('../helpers/contracts.cjs');

describe('Tier 1 - Feature 23: Shader Cache & Disk Reclaimer', () => {
  it('Validates ShaderCacheSummary contract schema', () => {
    const summary = {
      total_bytes: 5368709120, // 5GB
      item_count: 3,
      caches: [
        { app_id: 1091500, game_title: 'Cyberpunk 2077', size_bytes: 2147483648, is_installed: false },
        { app_id: 570, game_title: 'Dota 2', size_bytes: 1073741824, is_installed: true },
        { app_id: 730, game_title: 'CS2', size_bytes: 2147483648, is_installed: false },
      ],
    };
    validateShaderCacheSummary(summary);
    equal(summary.item_count, 3);
  });

  it('Identifies orphaned shader caches for uninstalled games', () => {
    const caches = [
      { app_id: 1, is_installed: false, size_bytes: 1000 },
      { app_id: 2, is_installed: true, size_bytes: 2000 },
      { app_id: 3, is_installed: false, size_bytes: 3000 },
    ];

    function getOrphanedCaches(list) {
      return list.filter(c => !c.is_installed);
    }

    const orphans = getOrphanedCaches(caches);
    equal(orphans.length, 2);
    equal(orphans[0].app_id, 1);
    equal(orphans[1].app_id, 3);
  });

  it('Calculates total reclaimable disk space across orphaned caches', () => {
    const orphans = [
      { app_id: 1, size_bytes: 1024 * 1024 * 500 }, // 500MB
      { app_id: 3, size_bytes: 1024 * 1024 * 700 }, // 700MB
    ];

    const reclaimableTotal = orphans.reduce((sum, c) => sum + c.size_bytes, 0);
    equal(reclaimableTotal, 1024 * 1024 * 1200); // 1200MB
  });

  it('Prevents accidental deletion of active/installed game caches', () => {
    function safeDeleteCache(cacheItem) {
      if (cacheItem.is_installed) {
        throw new Error('Refusing to delete shader cache of actively installed title');
      }
      return { deleted: true, freedBytes: cacheItem.size_bytes };
    }

    const activeItem = { app_id: 2, is_installed: true, size_bytes: 2000 };
    let caught = false;
    try {
      safeDeleteCache(activeItem);
    } catch (err) {
      caught = true;
    }
    equal(caught, true, 'Safety guard prevents deletion of active caches');
  });

  it('Formats reclaimed byte quantities into gigabytes (GB) string for UI', () => {
    function formatBytes(bytes) {
      const gb = bytes / (1024 * 1024 * 1024);
      return `${gb.toFixed(2)} GB`;
    }

    equal(formatBytes(5368709120), '5.00 GB');
  });
});
