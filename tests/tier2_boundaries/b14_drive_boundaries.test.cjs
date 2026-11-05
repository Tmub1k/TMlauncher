const { describe, it } = require('../runner.cjs');
const { ok, equal, throws } = require('../helpers/assert.cjs');

describe('Tier 2 - Boundary 14: In-Launcher Drive Selector Boundaries', () => {
  it('Handles drives with 0 bytes free space preventing selection for installation', () => {
    function canSelectForInstall(freeBytes) {
      return freeBytes > 1024 * 1024 * 100; // Require at least 100MB buffer
    }

    equal(canSelectForInstall(0), false);
    equal(canSelectForInstall(500000000), true);
  });

  it('Detects and flags read-only optical (CD/DVD) or ISO mounted drives', () => {
    const drives = [
      { drive: 'C:', isReadOnly: false, type: 'Fixed' },
      { drive: 'D:', isReadOnly: false, type: 'Fixed' },
      { drive: 'E:', isReadOnly: true, type: 'CDROM' },
    ];

    function filterWritableDrives(list) {
      return list.filter(d => !d.isReadOnly && d.type === 'Fixed');
    }

    const writable = filterWritableDrives(drives);
    equal(writable.length, 2);
    equal(writable.some(d => d.drive === 'E:'), false);
  });

  it('Validates network UNC paths (\\\\server\\share\\games) correctly', () => {
    function isValidInstallDestination(pathStr) {
      const isLocalDrive = /^[a-zA-Z]:\\/.test(pathStr);
      const isUncPath = /^\\\\[^\\/:*?"<>|\r\n]+\\[^\\/:*?"<>|\r\n]+/.test(pathStr);
      return isLocalDrive || isUncPath;
    }

    ok(isValidInstallDestination('D:\\SteamLibrary'));
    ok(isValidInstallDestination('\\\\NasServer\\Games\\Library'));
    equal(isValidInstallDestination('invalid_relative_path'), false);
  });

  it('Guards against Windows MAX_PATH length limits (260 characters)', () => {
    function validatePathLength(targetDir, gameExecutable) {
      const full = `${targetDir}\\${gameExecutable}`;
      if (full.length >= 260) {
        return { warning: 'Path approaches or exceeds Windows MAX_PATH (260 chars). Enable long paths in Windows.' };
      }
      return { warning: null };
    }

    const shortPath = validatePathLength('D:\\Games', 'game.exe');
    equal(shortPath.warning, null);

    const longDir = 'D:\\Games\\' + 'Subfolder\\'.repeat(30);
    const longPath = validatePathLength(longDir, 'deeply_nested_game_binary.exe');
    ok(longPath.warning !== null);
  });

  it('Rejects reserved Windows filenames (CON, PRN, AUX, NUL, COM1, LPT1)', () => {
    function isReservedWindowsName(folderName) {
      const reserved = new Set(['CON', 'PRN', 'AUX', 'NUL', 'COM1', 'COM2', 'LPT1', 'LPT2']);
      return reserved.has(folderName.toUpperCase().trim());
    }

    equal(isReservedWindowsName('CON'), true);
    equal(isReservedWindowsName('NUL'), true);
    equal(isReservedWindowsName('SteamLibrary'), false);
  });
});
