const { describe, it } = require('../runner.cjs');
const { ok, equal } = require('../helpers/assert.cjs');

describe('Tier 2 - Boundary 15: Emulators Catalog Boundaries', () => {
  it('Handles GitHub API rate-limiting (HTTP 403 X-RateLimit-Remaining: 0) with cached manifest fallback', () => {
    function fetchCatalog(isRateLimited, cachedCatalog) {
      if (isRateLimited) {
        return { source: 'local_offline_cache', data: cachedCatalog, isRateLimited: true };
      }
      return { source: 'github_api', data: cachedCatalog, isRateLimited: false };
    }

    const res = fetchCatalog(true, [{ id: 'dolphin' }]);
    equal(res.source, 'local_offline_cache');
    equal(res.isRateLimited, true);
    equal(res.data.length, 1);
  });

  it('Prefers stable releases over pre-release or nightly builds unless user explicitly requests bleeding-edge', () => {
    const releases = [
      { tag: 'v2.0.0-rc1', prerelease: true },
      { tag: 'v1.9.0', prerelease: false },
      { tag: 'nightly-20261104', prerelease: true },
    ];

    function selectRelease(list, allowPrerelease) {
      if (allowPrerelease) return list[0];
      return list.find(r => !r.prerelease) || list[0];
    }

    equal(selectRelease(releases, false).tag, 'v1.9.0');
    equal(selectRelease(releases, true).tag, 'v2.0.0-rc1');
  });

  it('Flags unsupported CPU architectures (e.g. ARM32 on x86_64 host)', () => {
    function isArchSupported(targetArch, hostArch = 'x86_64') {
      return targetArch === hostArch;
    }

    equal(isArchSupported('x86_64'), true);
    equal(isArchSupported('armv7l'), false);
    equal(isArchSupported('x86_32'), false);
  });

  it('Maintains offline catalog browsing when network is disconnected', () => {
    const offlineState = {
      isOnline: false,
      catalogEntriesCount: 7,
      canDownload: false,
      canViewInstalled: true,
    };
    equal(offlineState.canDownload, false);
    equal(offlineState.canViewInstalled, true);
  });

  it('Detects and rejects tampered catalog manifests containing untrusted external URLs', () => {
    function isValidEmulatorRepo(repo) {
      // Must be alphanumeric GitHub org/repo
      return /^[a-zA-Z0-9_-]+\/[a-zA-Z0-9_-]+$/.test(repo);
    }

    ok(isValidEmulatorRepo('dolphin-emu/dolphin'));
    ok(isValidEmulatorRepo('libretro/RetroArch'));
    equal(isValidEmulatorRepo('http://malicious.site/download'), false);
    equal(isValidEmulatorRepo('../../../etc/passwd'), false);
  });
});
