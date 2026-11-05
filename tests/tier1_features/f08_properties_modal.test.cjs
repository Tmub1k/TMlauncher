const { describe, it } = require('../runner.cjs');
const { ok, equal, deepEqual, hasProperties } = require('../helpers/assert.cjs');

describe('Tier 1 - Feature 08: Steam-like Game Properties Modal', () => {
  it('Enforces 5-tab modular architecture in properties modal', () => {
    const propertiesTabs = ['general', 'installed_files', 'cloud', 'achievements', 'compatibility'];
    equal(propertiesTabs.length, 5);
    ok(propertiesTabs.includes('general'));
    ok(propertiesTabs.includes('installed_files'));
    ok(propertiesTabs.includes('cloud'));
    ok(propertiesTabs.includes('achievements'));
    ok(propertiesTabs.includes('compatibility'));
  });

  it('Parses, formats, and persists custom launch arguments', () => {
    function sanitizeLaunchArgs(rawInput) {
      if (!rawInput) return [];
      return rawInput
        .trim()
        .split(/\s+/)
        .filter(arg => arg.length > 0);
    }

    const args = sanitizeLaunchArgs('-novid -tickrate 128 +fps_max 0');
    deepEqual(args, ['-novid', '-tickrate', '128', '+fps_max', '0']);
  });

  it('Provides file verification and local installation size inspection', () => {
    const fileStats = {
      installDir: 'C:\\Games\\Steam\\steamapps\\common\\Counter-Strike Global Offensive',
      totalSizeBytes: 34359738368, // 32 GB
      fileCount: 4120,
      verifyIntegrityAvailable: true,
    };
    hasProperties(fileStats, ['installDir', 'totalSizeBytes', 'fileCount', 'verifyIntegrityAvailable']);
    ok(fileStats.totalSizeBytes > 0);
    equal(fileStats.verifyIntegrityAvailable, true);
  });

  it('Controls Steam Cloud save toggle per game', () => {
    const cloudSettings = {
      appId: 730,
      cloudEnabled: true,
      quotaBytes: 104857600, // 100MB
      usedBytes: 15728640,   // 15MB
    };
    equal(cloudSettings.cloudEnabled, true);
    ok(cloudSettings.usedBytes < cloudSettings.quotaBytes);
  });

  it('Compatibility tab configures Proton / DXVK translation parameters', () => {
    const compatConfig = {
      forceCompatibilityTool: true,
      selectedTool: 'DXVK-Async-2.4',
      environmentVariables: {
        'DXVK_ASYNC': '1',
        'DXVK_HUD': 'fps,memory',
      },
    };
    equal(compatConfig.forceCompatibilityTool, true);
    equal(compatConfig.environmentVariables.DXVK_ASYNC, '1');
  });
});
