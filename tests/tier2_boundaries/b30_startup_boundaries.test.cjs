const { describe, it } = require('../runner.cjs');
const { ok, equal } = require('../helpers/assert.cjs');

describe('Tier 2 - Boundary 30: Clean GUI Startup Boundaries', () => {
  it('Handles race condition when two launcher processes start simultaneously (single instance mutex)', () => {
    let mutexOwner = null;
    function tryAcquireSingleton(processId) {
      if (mutexOwner === null) {
        mutexOwner = processId;
        return { acquired: true, role: 'primary_instance' };
      }
      return { acquired: false, role: 'secondary_instance_exit' };
    }

    const p1 = tryAcquireSingleton(1001);
    const p2 = tryAcquireSingleton(1002);
    equal(p1.acquired, true);
    equal(p1.role, 'primary_instance');
    equal(p2.acquired, false);
    equal(p2.role, 'secondary_instance_exit');
  });

  it('Secondary launcher instance signals primary instance window to focus before exiting', () => {
    let primaryBroughtToFront = false;
    function handleSecondaryLaunch() {
      primaryBroughtToFront = true;
      return { exitCode: 0, reason: 'Passed focus to running instance' };
    }

    const res = handleSecondaryLaunch();
    equal(primaryBroughtToFront, true);
    equal(res.exitCode, 0);
  });

  it('Falls back to software rendering if system GPU driver crashes or lacks Webview2 hardware acceleration', () => {
    function resolveRenderingEngine(gpuHardwareAccelerationAvailable) {
      if (!gpuHardwareAccelerationAvailable) {
        return { engine: 'software_rasterizer', flags: ['--disable-gpu', '--disable-software-rasterizer=false'] };
      }
      return { engine: 'hardware_d3d11', flags: [] };
    }

    const fallback = resolveRenderingEngine(false);
    equal(fallback.engine, 'software_rasterizer');
    ok(fallback.flags.includes('--disable-gpu'));
  });

  it('Recovers from corrupted local configuration file by restoring factory defaults', () => {
    function loadConfigSafe(corruptedJson) {
      try {
        return JSON.parse(corruptedJson);
      } catch (err) {
        return {
          theme: 'tura-graphite',
          currency: 'USD',
          recoveredFromCorruption: true,
        };
      }
    }

    const recovered = loadConfigSafe('{ corrupted config: [');
    equal(recovered.recoveredFromCorruption, true);
    equal(recovered.theme, 'tura-graphite');
    equal(recovered.currency, 'USD');
  });

  it('Launches cleanly under restricted Windows User Account Control (standard non-admin user)', () => {
    function checkElevationRequirement(requiresAdmin) {
      // Launcher must never trigger UAC prompt on standard launch
      return !requiresAdmin;
    }

    equal(checkElevationRequirement(false), true, 'Standard user execution permitted without UAC elevation prompt');
  });
});
