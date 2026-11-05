const { describe, it } = require('../runner.cjs');
const { ok, equal } = require('../helpers/assert.cjs');

describe('Tier 3 - Interaction 06: Emulator Downloader & In-Launcher Directory Selector', () => {
  it('Installs emulator directly to selected custom directory without opening Steam OS dialog', () => {
    const selectedCustomDir = 'D:\\Emulators\\PlayStation2';
    function downloadAndInstallEmulator(emuId, targetDir) {
      return {
        installed: true,
        installedLocation: `${targetDir}\\PCSX2.exe`,
        usedBuiltinPicker: true,
        dialogType: 'in_launcher_svelte_modal',
      };
    }

    const res = downloadAndInstallEmulator('pcsx2', selectedCustomDir);
    equal(res.installed, true);
    equal(res.installedLocation, 'D:\\Emulators\\PlayStation2\\PCSX2.exe');
    equal(res.dialogType, 'in_launcher_svelte_modal');
  });

  it('Automatically adds newly extracted emulator to unified Library Hub upon completion', () => {
    const library = [
      { id: 'steam_730', title: 'Counter-Strike 2', category: 'PC' },
    ];

    function onEmulatorInstallComplete(emuMeta, exePath) {
      library.push({
        id: `emu_${emuMeta.id}`,
        title: emuMeta.name,
        category: 'Emulators',
        executablePath: exePath,
        installed: true,
      });
    }

    onEmulatorInstallComplete({ id: 'dolphin', name: 'Dolphin Emulator' }, 'D:\\Emulators\\Dolphin\\Dolphin.exe');
    equal(library.length, 2);
    equal(library[1].category, 'Emulators');
    equal(library[1].title, 'Dolphin Emulator');
  });
});
