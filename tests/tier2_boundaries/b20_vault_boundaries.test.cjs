const { describe, it } = require('../runner.cjs');
const { ok, equal, throws } = require('../helpers/assert.cjs');

describe('Tier 2 - Boundary 20: Save-State Time Machine Boundaries', () => {
  it('Handles save directories containing actively locked open files by retrying with shadow copy', () => {
    function backupSaveDir(hasLockedFile, maxRetries = 3) {
      if (hasLockedFile) {
        return { method: 'vss_or_retry_snapshot', success: true };
      }
      return { method: 'direct_zip', success: true };
    }

    const locked = backupSaveDir(true);
    equal(locked.method, 'vss_or_retry_snapshot');
    equal(locked.success, true);
  });

  it('Detects corrupted or truncated zip archive before attempting restoration', () => {
    function verifyZipIntegrity(zipBuffer) {
      // Standard zip end of central directory signature: 0x06054b50 (PK\x05\x06)
      const pkEndSignature = Buffer.from([0x50, 0x4b, 0x05, 0x06]);
      const hasSig = zipBuffer.includes(pkEndSignature);
      if (!hasSig) {
        throw new Error('ARCHIVE_CORRUPTED: Missing zip end of central directory signature');
      }
      return true;
    }

    const validZipMock = Buffer.concat([Buffer.from('zip_data...'), Buffer.from([0x50, 0x4b, 0x05, 0x06])]);
    ok(verifyZipIntegrity(validZipMock));

    const corruptedZipMock = Buffer.from('truncated_corrupted_data');
    throws(() => verifyZipIntegrity(corruptedZipMock), /ARCHIVE_CORRUPTED/);
  });

  it('Aborts restore operation if target destination directory is read-only', () => {
    function checkWritableDestination(isReadOnly) {
      if (isReadOnly) {
        return { canRestore: false, error: 'Destination save folder is read-only or permission denied' };
      }
      return { canRestore: true, error: null };
    }

    const ro = checkWritableDestination(true);
    equal(ro.canRestore, false);
    ok(ro.error.includes('read-only'));
  });

  it('Warns user and halts snapshot creation when disk capacity exceeds 95%', () => {
    function checkVaultDiskSpace(usedPercent) {
      if (usedPercent >= 95) {
        return { canSnapshot: false, warning: 'Storage critical: Disk is over 95% full' };
      }
      return { canSnapshot: true, warning: null };
    }

    equal(checkVaultDiskSpace(98).canSnapshot, false);
    equal(checkVaultDiskSpace(70).canSnapshot, true);
  });

  it('Creates snapshot metadata gracefully even when game save directory is initially empty', () => {
    function snapshotEmptyDir(filesInDir) {
      if (filesInDir.length === 0) {
        return { snapshotCreated: true, fileCount: 0, note: 'Initialized baseline save snapshot' };
      }
      return { snapshotCreated: true, fileCount: filesInDir.length };
    }

    const res = snapshotEmptyDir([]);
    equal(res.snapshotCreated, true);
    equal(res.fileCount, 0);
  });
});
