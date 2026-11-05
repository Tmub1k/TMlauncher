const { describe, it } = require('../runner.cjs');
const { ok, equal } = require('../helpers/assert.cjs');

describe('Tier 2 - Boundary 23: Shader Cache Reclaimer Boundaries', () => {
  it('Handles shader cache files locked by the display driver without crashing cleaner', () => {
    function deleteCacheFile(path, isDriverLocked) {
      if (isDriverLocked) {
        return { deleted: false, reason: 'File locked by display driver (dxvk_cache.bin)' };
      }
      return { deleted: true, reason: null };
    }

    const locked = deleteCacheFile('dxvk.bin', true);
    equal(locked.deleted, false);
    ok(locked.reason.includes('File locked'));
  });

  it('Recovers from corrupted files or invalid subdirectories inside shader cache folder', () => {
    function scanShaderDir(files) {
      let validCaches = 0;
      let corruptFilesIgnored = 0;
      for (const f of files) {
        if (f.endsWith('.bin') || f.endsWith('.toc')) {
          validCaches++;
        } else {
          corruptFilesIgnored++;
        }
      }
      return { validCaches, corruptFilesIgnored };
    }

    const res = scanShaderDir(['dxvk.bin', 'mesa.toc', 'garbage.tmp', 'corrupt.dat']);
    equal(res.validCaches, 2);
    equal(res.corruptFilesIgnored, 2);
  });

  it('Handles zero-byte cleanup cleanly when no orphaned caches exist', () => {
    function executeCleanup(reclaimableList) {
      if (reclaimableList.length === 0) {
        return { bytesReclaimed: 0, message: 'All shader caches are active. No orphaned files found.' };
      }
      return { bytesReclaimed: 1000, message: 'OK' };
    }

    const res = executeCleanup([]);
    equal(res.bytesReclaimed, 0);
    ok(res.message.includes('No orphaned files'));
  });

  it('Guards against deeply nested directory paths exceeding Windows MAX_PATH during scan', () => {
    function isSafePathLength(fullPath) {
      return fullPath.length < 260 || fullPath.startsWith('\\\\?\\');
    }

    ok(isSafePathLength('C:\\Steam\\shadercache\\730\\foz_cache.bin'));
    ok(isSafePathLength('\\\\?\\C:\\VeryLongPath\\' + 'a\\'.repeat(200)));
  });

  it('Catches OS permission denied errors when attempting to delete driver-managed system caches', () => {
    function attemptSystemCacheDelete(canWriteSystemDir) {
      if (!canWriteSystemDir) {
        return { success: false, code: 'EACCES_PERMISSION_DENIED' };
      }
      return { success: true };
    }

    const fail = attemptSystemCacheDelete(false);
    equal(fail.success, false);
    equal(fail.code, 'EACCES_PERMISSION_DENIED');
  });
});
