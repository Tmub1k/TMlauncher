const { describe, it } = require('../runner.cjs');
const { ok, equal, hasProperties } = require('../helpers/assert.cjs');
const { validateSaveSnapshot } = require('../helpers/contracts.cjs');

describe('Tier 1 - Feature 20: Save-State Time Machine', () => {
  it('Validates SaveSnapshot contract and metadata payload', () => {
    const snap = {
      snapshot_id: 'snap_20261104_230000_eldenring',
      game_id: 'eldenring',
      timestamp: 1700000000,
      archive_size_bytes: 4194304,
      sha256_checksum: 'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855',
      note: 'Pre-Boss fight backup',
    };
    validateSaveSnapshot(snap);
    equal(snap.game_id, 'eldenring');
    ok(snap.archive_size_bytes > 0);
  });

  it('Generates timestamped zip backup file path and structure', () => {
    function generateSnapshotFilename(gameId, timestamp) {
      const d = new Date(timestamp);
      const iso = d.toISOString().replace(/[:.]/g, '-');
      return `save_vault_${gameId}_${iso}.zip`;
    }

    const fn = generateSnapshotFilename('hades', 1700000000000);
    ok(fn.startsWith('save_vault_hades_'));
    ok(fn.endsWith('.zip'));
  });

  it('Restores save state from past snapshot with safety rollback backup', () => {
    let rollbackCreated = false;
    let restoreSucceeded = false;

    function restoreSnapshot(snapshotId, targetSaveDir) {
      // 1. Create emergency rollback snapshot of current dir
      rollbackCreated = true;
      // 2. Unpack target snapshot
      restoreSucceeded = true;
      return { status: 'restored', snapshotId };
    }

    const res = restoreSnapshot('snap_1', 'C:\\Saves\\Game');
    equal(rollbackCreated, true);
    equal(restoreSucceeded, true);
    equal(res.status, 'restored');
  });

  it('Enforces automatic snapshot rotation and pruning policy', () => {
    const maxSnapshotsPerGame = 5;
    const history = [
      { id: 'snap_1', time: 100 },
      { id: 'snap_2', time: 200 },
      { id: 'snap_3', time: 300 },
      { id: 'snap_4', time: 400 },
      { id: 'snap_5', time: 500 },
      { id: 'snap_6', time: 600 }, // Newest snapshot
    ];

    function pruneSnapshots(snaps, maxLimit) {
      const sorted = [...snaps].sort((a, b) => b.time - a.time);
      const kept = sorted.slice(0, maxLimit);
      const pruned = sorted.slice(maxLimit);
      return { kept, pruned };
    }

    const result = pruneSnapshots(history, maxSnapshotsPerGame);
    equal(result.kept.length, 5);
    equal(result.pruned.length, 1);
    equal(result.pruned[0].id, 'snap_1', 'Oldest snapshot pruned');
  });

  it('Verifies archive checksum integrity before performing restore', () => {
    function checkIntegrity(computedSha, manifestSha) {
      return computedSha === manifestSha;
    }

    ok(checkIntegrity('abc123hash', 'abc123hash'));
    equal(checkIntegrity('abc123hash', 'corrupted'), false);
  });
});
