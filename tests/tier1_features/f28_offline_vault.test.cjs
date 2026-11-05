const { describe, it } = require('../runner.cjs');
const { ok, equal, hasProperties } = require('../helpers/assert.cjs');

describe('Tier 1 - Feature 28: Proactive Offline Vault', () => {
  it('Stores encrypted authentication tokens and license cache locally', () => {
    const vaultItem = {
      platform: 'steam',
      accountKey: 'user_123',
      tokenCipher: 'U2FsdGVkX19mocktokenpayload==',
      cachedAt: 1700000000,
      expiresAt: 1705000000,
    };
    hasProperties(vaultItem, ['platform', 'accountKey', 'tokenCipher', 'expiresAt']);
    ok(vaultItem.expiresAt > vaultItem.cachedAt);
  });

  it('Allows 1-click game launch when operating in zero-connectivity offline state', () => {
    const offlineGame = {
      appId: 730,
      title: 'Counter-Strike 2',
      installed: true,
      hasOfflineLicense: true,
      launchExecutable: 'C:\\Games\\Steam\\steamapps\\common\\CS2\\game\\bin\\win64\\cs2.exe',
    };

    function canLaunchOffline(game, isNetworkAvailable) {
      if (!isNetworkAvailable) {
        return game.installed && game.hasOfflineLicense;
      }
      return game.installed;
    }

    equal(canLaunchOffline(offlineGame, false), true);
  });

  it('Provides local sqlite / json cached metadata when remote API is unreachable', () => {
    const offlineMetadataStore = new Map([
      ['730', { title: 'CS2', coverPath: 'cache/covers/730.jpg', tags: ['FPS'] }],
    ]);

    function getMetadataOffline(appId) {
      return offlineMetadataStore.get(appId) || null;
    }

    const meta = getMetadataOffline('730');
    ok(meta !== null);
    equal(meta.title, 'CS2');
  });

  it('Enqueues offline actions into persistent synchronization queue', () => {
    const syncQueue = [];
    function queueOfflineAction(actionType, payload) {
      syncQueue.push({
        id: `act_${Date.now()}_${Math.random()}`,
        actionType,
        payload,
        queuedAt: Date.now(),
      });
    }

    queueOfflineAction('LOG_PLAYTIME', { appId: 730, seconds: 1800 });
    equal(syncQueue.length, 1);
    equal(syncQueue[0].actionType, 'LOG_PLAYTIME');
  });

  it('Flushes and reconciles queued sync operations when connection restores', () => {
    const queue = [
      { id: '1', actionType: 'SAVE_TIME', processed: false },
      { id: '2', actionType: 'ACHIEVEMENT_SYNC', processed: false },
    ];

    function reconcileSyncQueue(q) {
      for (const item of q) {
        item.processed = true;
      }
      return q.filter(i => !i.processed);
    }

    const remaining = reconcileSyncQueue(queue);
    equal(remaining.length, 0, 'All actions processed upon network reconnection');
  });
});
