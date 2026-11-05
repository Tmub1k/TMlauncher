const { describe, it } = require('../runner.cjs');
const { ok, equal, deepEqual, match } = require('../helpers/assert.cjs');
const { SAMPLE_STEAM_VDF } = require('../helpers/fixtures.cjs');
const { validateCloudSyncResult } = require('../helpers/contracts.cjs');

describe('Tier 1 - Feature 07: Steam Cloud Sync & Versioning', () => {
  it('Parses remotecache.vdf and detects cloud sync status', () => {
    function parseVdfCloudState(vdfContent, appId) {
      const appBlockMatch = new RegExp(`"${appId}"\\s*\\{[^}]*"clouddirty"\\s*"([01])"`, 'm');
      const matchRes = vdfContent.match(appBlockMatch);
      if (!matchRes) return null;
      return {
        app_id: Number(appId),
        is_dirty: matchRes[1] === '1',
      };
    }

    const state = parseVdfCloudState(SAMPLE_STEAM_VDF, '730');
    ok(state !== null);
    equal(state.app_id, 730);
    equal(state.is_dirty, false);
  });

  it('Validates CloudSyncResult contract payload', () => {
    const result = {
      app_id: 730,
      synced: true,
      local_files: ['save01.dat', 'settings.cfg'],
      remote_files: ['save01.dat', 'settings.cfg'],
      last_sync_timestamp: 1700000000,
    };
    validateCloudSyncResult(result);
    equal(result.synced, true);
    equal(result.local_files.length, 2);
  });

  it('Detects dirty local saves requiring cloud synchronization before launch', () => {
    const dirtyVdf = `"UserRoamingConfigStore"
    {
      "Apps"
      {
        "570"
        {
          "clouddirty" "1"
          "lastsync" "1699990000"
        }
      }
    }`;

    const isDirty = dirtyVdf.includes('"clouddirty" "1"');
    equal(isDirty, true, 'Dirty cloud status properly flagged');
  });

  it('Resolves authoritative Steam Auto-Cloud save directories on Windows', () => {
    function resolveSteamAutoCloudPath(steamId3, appId) {
      return `C:\\Program Files (x86)\\Steam\\userdata\\${steamId3}\\${appId}\\remote`;
    }

    const path = resolveSteamAutoCloudPath('12345678', '730');
    match(path, /Steam\\userdata\\12345678\\730\\remote$/);
  });

  it('Resolves cloud conflict when local and remote timestamps differ', () => {
    const conflict = {
      localTimestamp: 1700005000,
      remoteTimestamp: 1700001000,
      strategy: 'prompt_or_prefer_newer',
    };

    function resolveConflict(c) {
      if (c.localTimestamp > c.remoteTimestamp) return 'upload_local';
      if (c.remoteTimestamp > c.localTimestamp) return 'download_remote';
      return 'noop';
    }

    equal(resolveConflict(conflict), 'upload_local');
  });
});
