const { describe, it } = require('../runner.cjs');
const { ok, equal } = require('../helpers/assert.cjs');

describe('Tier 2 - Boundary 29: File Structure Reorganization Boundaries', () => {
  it('Detects circular symlink or junction directory loops during file scanning', () => {
    function detectSymlinkLoop(pathHistory, targetRealpath) {
      if (pathHistory.has(targetRealpath)) {
        return { isLoop: true, action: 'skip_directory' };
      }
      pathHistory.add(targetRealpath);
      return { isLoop: false, action: 'continue_scan' };
    }

    const history = new Set(['C:\\Games\\Main']);
    const loopRes = detectSymlinkLoop(history, 'C:\\Games\\Main');
    equal(loopRes.isLoop, true);
    equal(loopRes.action, 'skip_directory');
  });

  it('Ignores OS system dotfiles (.DS_Store, Thumbs.db, desktop.ini) during layout scan', () => {
    function filterSystemNoise(files) {
      const noise = new Set(['.ds_store', 'thumbs.db', 'desktop.ini']);
      return files.filter(f => !noise.has(f.toLowerCase()));
    }

    const mixed = ['App.svelte', '.DS_Store', 'Thumbs.db', 'desktop.ini', 'main.ts'];
    const clean = filterSystemNoise(mixed);
    equal(clean.length, 2);
    equal(clean[0], 'App.svelte');
    equal(clean[1], 'main.ts');
  });

  it('Handles Windows case-insensitive path comparisons vs POSIX lookups', () => {
    function pathsMatchWindows(p1, p2) {
      return p1.toLowerCase().replace(/\//g, '\\') === p2.toLowerCase().replace(/\//g, '\\');
    }

    ok(pathsMatchWindows('C:/Games/Steam', 'c:\\games\\steam'));
    ok(pathsMatchWindows('src/App.svelte', 'SRC\\APP.SVELTE'));
  });

  it('Handles filesystem paths containing spaces and unicode symbols in project tree', () => {
    function isWellFormedRelativePath(relPath) {
      return !relPath.startsWith('/') && !relPath.startsWith('\\') && !relPath.includes('..');
    }

    ok(isWellFormedRelativePath('themes/минималистичный стиль.json'));
    ok(isWellFormedRelativePath('src/lib/components/Game Details.svelte'));
    equal(isWellFormedRelativePath('../dangerous/file'), false);
  });

  it('Maintains empty required directory placeholders (.gitkeep) without treating as clutter', () => {
    function isAllowedSpecialFile(filename) {
      const allowed = new Set(['.gitkeep', '.gitignore', '.eslintrc', '.env.example']);
      return allowed.has(filename.toLowerCase());
    }

    ok(isAllowedSpecialFile('.gitkeep'));
    ok(isAllowedSpecialFile('.gitignore'));
    equal(isAllowedSpecialFile('random_file.tmp'), false);
  });
});
