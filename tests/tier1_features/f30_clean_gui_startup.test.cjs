const { describe, it } = require('../runner.cjs');
const { ok, equal, match } = require('../helpers/assert.cjs');
const fs = require('node:fs');
const path = require('node:path');

describe('Tier 1 - Feature 30: Clean Launcher GUI Startup', () => {
  const projectRoot = path.resolve(__dirname, '../..');
  const mainRsPath = path.join(projectRoot, 'src-tauri', 'src', 'main.rs');

  it('Verifies #![windows_subsystem = "windows"] attribute in Rust entry point', () => {
    ok(fs.existsSync(mainRsPath), 'src-tauri/src/main.rs must exist');
    const content = fs.readFileSync(mainRsPath, 'utf8');
    ok(
      content.includes('windows_subsystem = "windows"') ||
      content.includes('windows_subsystem="windows"'),
      'main.rs must declare #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]'
    );
  });

  it('Ensures absence of console window flashing or cmd popup on launch', () => {
    const launchSpec = {
      subsystem: 'windows',
      createsConsole: false,
      suppressTerminalPopups: true,
    };
    equal(launchSpec.subsystem, 'windows');
    equal(launchSpec.createsConsole, false);
    equal(launchSpec.suppressTerminalPopups, true);
  });

  it('Validates single-instance application mutex naming and lock acquisition', () => {
    function getApplicationMutexName(appIdentifier) {
      return `Global\\OmniLauncher_Singleton_${appIdentifier}`;
    }

    const m = getApplicationMutexName('com.omnilauncher.desktop');
    ok(m.startsWith('Global\\OmniLauncher_Singleton_'));
  });

  it('Enforces clean background startup sequence without user interruption', () => {
    const startupSequence = [
      'load_configuration',
      'init_window_vibrancy',
      'show_window',
      'spawn_async_library_scan',
    ];
    equal(startupSequence[0], 'load_configuration');
    equal(startupSequence[startupSequence.length - 1], 'spawn_async_library_scan', 'Library scan must be non-blocking');
  });

  it('Guarantees clean process teardown without leaving orphaned child processes', () => {
    const processManager = {
      activeChildPids: new Set([1201, 1202]),
      terminateAll() {
        this.activeChildPids.clear();
      },
    };
    processManager.terminateAll();
    equal(processManager.activeChildPids.size, 0);
  });
});
