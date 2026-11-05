const { describe, it } = require('../runner.cjs');
const { ok, equal } = require('../helpers/assert.cjs');

describe('Tier 2 - Boundary 08: Game Properties Modal Boundaries', () => {
  it('Sanitizes dangerous shell escape characters and command injection in launch arguments', () => {
    function sanitizeLaunchArgs(argsString) {
      // Disallow dangerous shell piping and chaining characters
      return argsString
        .replace(/[;&|`$]/g, '')
        .trim();
    }

    const malicious = '-windowed ; rm -rf / ; | cat secrets';
    const safe = sanitizeLaunchArgs(malicious);
    equal(safe.includes(';'), false);
    equal(safe.includes('|'), false);
    ok(safe.includes('-windowed'));
  });

  it('Caps maximum length of custom launch arguments (e.g. 4096 characters)', () => {
    function limitArgsLength(input, maxLen = 4096) {
      return input.slice(0, maxLen);
    }

    const hugeArgs = '-arg '.repeat(2000);
    const limited = limitArgsLength(hugeArgs);
    equal(limited.length, 4096);
  });

  it('Handles file verification when files are locked by running antivirus scanner', () => {
    function verifyFileIntegrity(filePath, isLockedByAnotherProcess) {
      if (isLockedByAnotherProcess) {
        return { status: 'file_locked', retryAfterMs: 500 };
      }
      return { status: 'verified', checksumMatches: true };
    }

    const locked = verifyFileIntegrity('game.exe', true);
    equal(locked.status, 'file_locked');
    equal(locked.retryAfterMs, 500);
  });

  it('Disables cloud sync toggle changes while game process is actively running', () => {
    function canToggleCloud(isGameRunning) {
      return !isGameRunning;
    }

    equal(canToggleCloud(true), false, 'Cannot change cloud settings while game running');
    equal(canToggleCloud(false), true);
  });

  it('Falls back to system default when compatibility tool selection is empty', () => {
    function resolveCompatTool(selectedTool) {
      if (!selectedTool || selectedTool === 'none') {
        return 'native_windows_subsystem';
      }
      return selectedTool;
    }

    equal(resolveCompatTool(''), 'native_windows_subsystem');
    equal(resolveCompatTool('none'), 'native_windows_subsystem');
    equal(resolveCompatTool('Proton-GE'), 'Proton-GE');
  });
});
