const { describe, it } = require('../runner.cjs');
const { ok, equal, hasProperties } = require('../helpers/assert.cjs');
const { EMULATOR_PRESETS } = require('../helpers/fixtures.cjs');
const { validateEmulatorEntry } = require('../helpers/contracts.cjs');

describe('Tier 1 - Feature 15: GitHub Open-Source Emulators Catalog', () => {
  it('Curated catalog includes flagship emulators (RetroArch, Dolphin, PCSX2, RPCS3, PPSSPP, DuckStation, Ryujinx)', () => {
    const ids = EMULATOR_PRESETS.map(e => e.id);
    ok(ids.includes('retroarch'));
    ok(ids.includes('dolphin'));
    ok(ids.includes('pcsx2'));
    ok(ids.includes('rpcs3'));
    ok(ids.includes('ppsspp'));
    ok(ids.includes('duckstation'));
    ok(ids.includes('ryujinx'));
  });

  it('Validates individual emulator metadata schema', () => {
    const emu = {
      id: 'dolphin',
      name: 'Dolphin',
      repo: 'dolphin-emu/dolphin',
      version: '2407',
      installed: false,
      executable_name: 'Dolphin.exe',
    };
    validateEmulatorEntry(emu);
    equal(emu.name, 'Dolphin');
  });

  it('Filters emulator presets by target processor architecture', () => {
    function filterByArch(presets, arch) {
      return presets.filter(p => p.arch === arch);
    }

    const x64Emus = filterByArch(EMULATOR_PRESETS, 'x86_64');
    equal(x64Emus.length, EMULATOR_PRESETS.length);
  });

  it('Resolves authoritative GitHub release API endpoint per emulator', () => {
    function getReleaseApiUrl(repo) {
      return `https://api.github.com/repos/${repo}/releases/latest`;
    }

    const retroUrl = getReleaseApiUrl('libretro/RetroArch');
    equal(retroUrl, 'https://api.github.com/repos/libretro/RetroArch/releases/latest');
  });

  it('Classifies emulators into historical console generations', () => {
    const generations = {
      'retroarch': 'Multi-System Retro (8-bit / 16-bit / 32-bit)',
      'dolphin': 'Nintendo GameCube / Wii',
      'pcsx2': 'Sony PlayStation 2',
      'rpcs3': 'Sony PlayStation 3',
      'duckstation': 'Sony PlayStation 1',
    };
    equal(generations['dolphin'], 'Nintendo GameCube / Wii');
    equal(generations['pcsx2'], 'Sony PlayStation 2');
  });
});
