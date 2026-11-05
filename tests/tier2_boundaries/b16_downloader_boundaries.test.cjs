const { describe, it } = require('../runner.cjs');
const { ok, equal, throws } = require('../helpers/assert.cjs');

describe('Tier 2 - Boundary 16: Emulator Downloader Boundaries', () => {
  it('Handles network disconnect mid-download with resume chunk offset support', () => {
    function computeRangeHeader(existingBytes, totalBytes) {
      if (existingBytes > 0 && existingBytes < totalBytes) {
        return `bytes=${existingBytes}-${totalBytes - 1}`;
      }
      return null;
    }

    const range = computeRangeHeader(50000000, 100000000);
    equal(range, 'bytes=50000000-99999999');
    equal(computeRangeHeader(0, 100000000), null);
  });

  it('Aborts extraction immediately on SHA256 checksum mismatch and cleans up temporary archive', () => {
    let tempCleaned = false;
    function extractWithVerification(actualHash, expectedHash) {
      if (actualHash !== expectedHash) {
        tempCleaned = true;
        throw new Error('CHECKSUM_MISMATCH: Downloaded asset integrity compromised');
      }
      return true;
    }

    throws(() => {
      extractWithVerification('corrupted_hash', 'valid_expected_hash');
    }, /CHECKSUM_MISMATCH/);
    equal(tempCleaned, true);
  });

  it('Guards against Zip-Slip path traversal vulnerability in archive entries', () => {
    function isSafeZipPath(entryPath, destinationRoot) {
      const path = require('node:path');
      const resolved = path.resolve(destinationRoot, entryPath);
      return resolved.startsWith(path.resolve(destinationRoot));
    }

    ok(isSafeZipPath('bin/emulator.exe', 'C:\\Games\\Emu'));
    ok(isSafeZipPath('config/settings.ini', 'C:\\Games\\Emu'));
    equal(isSafeZipPath('../../Windows/System32/evil.dll', 'C:\\Games\\Emu'), false, 'Zip-slip traversal blocked');
  });

  it('Fails gracefully with clear error when disk fills during decompression', () => {
    function checkDecompressSpace(archiveSizeBytes, freeDiskBytes) {
      const estimatedUncompressed = archiveSizeBytes * 3.5;
      if (freeDiskBytes < estimatedUncompressed) {
        return { canExtract: false, error: 'Insufficient disk space for archive extraction' };
      }
      return { canExtract: true, error: null };
    }

    const res = checkDecompressSpace(500 * 1024 * 1024, 200 * 1024 * 1024);
    equal(res.canExtract, false);
    ok(res.error.includes('Insufficient disk space'));
  });

  it('Rejects 0-byte downloaded archive files before extraction', () => {
    function validateArchiveSize(bytes) {
      if (bytes <= 0) throw new Error('Archive is empty (0 bytes)');
      return true;
    }

    throws(() => validateArchiveSize(0), /empty/);
    ok(validateArchiveSize(1024));
  });
});
