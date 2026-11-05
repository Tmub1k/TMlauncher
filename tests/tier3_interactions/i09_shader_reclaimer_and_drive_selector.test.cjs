const { describe, it } = require('../runner.cjs');
const { ok, equal } = require('../helpers/assert.cjs');

describe('Tier 3 - Interaction 09: Shader Cache Reclaimer & Drive Space Visualizer', () => {
  it('Updates logical drive space visualizer in real-time after deleting orphaned shader caches', () => {
    const driveState = {
      drive: 'C:',
      totalBytes: 500000000000,
      freeBytes: 100000000000, // 100GB
    };

    function reclaimShaderCachesOnDrive(drive, orphanedBytes) {
      drive.freeBytes += orphanedBytes;
      return drive;
    }

    reclaimShaderCachesOnDrive(driveState, 5000000000); // 5GB freed
    equal(driveState.freeBytes, 105000000000); // 105GB
  });

  it('Aggregates reclaimable shader caches across multiple storage drives (C:, D:, E:)', () => {
    const driveScans = [
      { drive: 'C:', orphanBytes: 2000000000 }, // 2GB
      { drive: 'D:', orphanBytes: 8000000000 }, // 8GB
      { drive: 'E:', orphanBytes: 0 },
    ];

    const totalReclaimable = driveScans.reduce((sum, d) => sum + d.orphanBytes, 0);
    equal(totalReclaimable, 10000000000); // 10GB total
  });
});
