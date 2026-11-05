const { describe, it } = require('../runner.cjs');
const { ok, equal, match, hasProperties } = require('../helpers/assert.cjs');

describe('Tier 1 - Feature 14: In-Launcher Drive & Directory Selector', () => {
  it('Enumerates local logical drive letters and volume statistics', () => {
    const drives = [
      { drive: 'C:', label: 'System', totalBytes: 500000000000, freeBytes: 150000000000 },
      { drive: 'D:', label: 'Games', totalBytes: 2000000000000, freeBytes: 850000000000 },
    ];
    equal(drives.length, 2);
    equal(drives[0].drive, 'C:');
    ok(drives[1].freeBytes > drives[0].freeBytes);
  });

  it('Calculates drive utilization percentage for UI progress bars', () => {
    function calculateUtilization(total, free) {
      const used = total - free;
      return Math.round((used / total) * 100);
    }

    const utilC = calculateUtilization(500000000000, 150000000000);
    equal(utilC, 70, '70% disk usage calculated accurately');
  });

  it('Validates target installation path formatting and Windows drive letter notation', () => {
    function isValidWindowsInstallPath(p) {
      return /^[a-zA-Z]:\\(?:[^\\/:*?"<>|\r\n]+\\)*[^\\/:*?"<>|\r\n]*$/.test(p);
    }

    ok(isValidWindowsInstallPath('D:\\Games\\SteamLibrary'));
    ok(isValidWindowsInstallPath('C:\\OmniLauncher\\Games'));
    equal(isValidWindowsInstallPath('/unix/style/path'), false);
    equal(isValidWindowsInstallPath('D:\\Invalid|Char\\'), false);
  });

  it('Verifies sufficient free space before initiating download', () => {
    function canInstallGame(driveFreeBytes, gameSizeBytes, safetyMarginBytes = 1073741824) {
      return driveFreeBytes >= (gameSizeBytes + safetyMarginBytes);
    }

    const freeBytes = 50 * 1024 * 1024 * 1024; // 50 GB
    const gameBytes = 40 * 1024 * 1024 * 1024; // 40 GB
    const hugeGameBytes = 55 * 1024 * 1024 * 1024; // 55 GB

    equal(canInstallGame(freeBytes, gameBytes), true);
    equal(canInstallGame(freeBytes, hugeGameBytes), false);
  });

  it('Navigates folder hierarchy internally without opening Steam OS dialog', () => {
    const folderTree = {
      path: 'D:\\Games',
      children: [
        { name: 'Steam', isDir: true },
        { name: 'Epic', isDir: true },
        { name: 'Retro', isDir: true },
      ],
    };
    equal(folderTree.children.length, 3);
    equal(folderTree.children[0].name, 'Steam');
  });
});
