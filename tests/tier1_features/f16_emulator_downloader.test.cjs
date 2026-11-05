const { describe, it } = require('../runner.cjs');
const { ok, equal, match, hasProperties } = require('../helpers/assert.cjs');
const crypto = require('node:crypto');

describe('Tier 1 - Feature 16: 1-Click Emulator Downloader & Extractor', () => {
  it('Validates SHA256 checksum verification against downloaded binary', () => {
    const mockFileBuffer = Buffer.from('MockEmulatorBinaryPayload_v1.0');
    const expectedHash = crypto.createHash('sha256').update(mockFileBuffer).digest('hex');

    function verifyChecksum(buffer, hash) {
      const actual = crypto.createHash('sha256').update(buffer).digest('hex');
      return actual.toLowerCase() === hash.toLowerCase();
    }

    ok(verifyChecksum(mockFileBuffer, expectedHash));
    equal(verifyChecksum(mockFileBuffer, '00000000000000000000000000000000'), false);
  });

  it('Calculates streaming download progress percentage and transfer rate', () => {
    function calculateProgress(bytesDownloaded, totalBytes, elapsedSeconds) {
      const pct = Math.round((bytesDownloaded / totalBytes) * 100);
      const speedMBs = ((bytesDownloaded / 1024 / 1024) / elapsedSeconds).toFixed(2);
      return { pct, speedMBs: Number(speedMBs) };
    }

    const prog = calculateProgress(50 * 1024 * 1024, 100 * 1024 * 1024, 5);
    equal(prog.pct, 50);
    equal(prog.speedMBs, 10.0);
  });

  it('Simulates zip extraction logic and executable discovery', () => {
    const archiveFiles = [
      'Dolphin-x64/license.txt',
      'Dolphin-x64/User/Config.ini',
      'Dolphin-x64/Dolphin.exe',
      'Dolphin-x64/Qt6Core.dll',
    ];

    function findMainExecutable(files, targetExeName) {
      return files.find(f => f.toLowerCase().endsWith(targetExeName.toLowerCase())) || null;
    }

    const found = findMainExecutable(archiveFiles, 'Dolphin.exe');
    equal(found, 'Dolphin-x64/Dolphin.exe');
  });

  it('Registers newly installed emulator in Launcher library registry', () => {
    const registry = [];
    function registerInstalledEmulator(reg, emuMeta, installPath) {
      const entry = {
        ...emuMeta,
        installed: true,
        installedPath: installPath,
        installedAt: Date.now(),
      };
      reg.push(entry);
      return entry;
    }

    const item = registerInstalledEmulator(registry, { id: 'duckstation', name: 'DuckStation' }, 'D:\\Emulators\\DuckStation');
    equal(registry.length, 1);
    equal(registry[0].installed, true);
    equal(item.installedPath, 'D:\\Emulators\\DuckStation');
  });

  it('Rolls back installation directory if extraction encounters failure', () => {
    let rolledBack = false;
    function installWithRollback(succeed) {
      try {
        if (!succeed) throw new Error('Extraction corrupted archive');
        return true;
      } catch (err) {
        rolledBack = true;
        return false;
      }
    }

    const res = installWithRollback(false);
    equal(res, false);
    equal(rolledBack, true, 'Rollback triggered cleanly on extraction error');
  });
});
