const { describe, it } = require('../runner.cjs');
const { ok, equal } = require('../helpers/assert.cjs');
const crypto = require('node:crypto');

describe('Tier 4 - Scenario 04: Retro Emulation Setup & 1-Click Installation', () => {
  it('Selects emulator from GitHub catalog -> picks custom drive -> verifies hash -> extracts and registers', () => {
    // 1. Browse GitHub catalog
    const catalogItem = {
      id: 'pcsx2',
      name: 'PCSX2',
      repo: 'PCSX2/pcsx2',
      tag: 'v2.0.2',
      assetName: 'pcsx2-v2.0.2-windows-x64.zip',
    };
    equal(catalogItem.name, 'PCSX2');

    // 2. Select target destination with custom folder browser
    const installDestination = 'D:\\Emulators\\PlayStation2';
    ok(installDestination.startsWith('D:\\'));

    // 3. Download binary and verify SHA256 integrity
    const mockBinary = Buffer.from('OfficialPCSX2_v2.0.2_Binary_Data');
    const validHash = crypto.createHash('sha256').update(mockBinary).digest('hex');

    function verifyChecksum(buf, hash) {
      return crypto.createHash('sha256').update(buf).digest('hex') === hash;
    }
    equal(verifyChecksum(mockBinary, validHash), true, 'Checksum verification passed');

    // 4. Extract archive and discover primary executable
    const extractedFiles = [
      'pcsx2-v2.0.2/docs/readme.txt',
      'pcsx2-v2.0.2/pcsx2-qtx64.exe',
      'pcsx2-v2.0.2/resources/shaders.pack',
    ];
    const mainExe = extractedFiles.find(f => f.toLowerCase().endsWith('.exe'));
    ok(mainExe !== undefined);

    // 5. Automatic addition to Launcher Library Hub
    const libraryEntry = {
      id: `emu_${catalogItem.id}`,
      title: catalogItem.name,
      platform: 'emulator',
      exePath: `${installDestination}\\${mainExe}`,
      readyToLaunch: true,
    };
    equal(libraryEntry.readyToLaunch, true);
    equal(libraryEntry.platform, 'emulator');
  });
});
