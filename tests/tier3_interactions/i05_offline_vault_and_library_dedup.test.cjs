const { describe, it } = require('../runner.cjs');
const { ok, equal, deepEqual } = require('../helpers/assert.cjs');

describe('Tier 3 - Interaction 05: Proactive Offline Vault & Multi-Store Deduplication', () => {
  it('Selects offline-ready platform source on unified game card during complete disconnect', () => {
    const unifiedGame = {
      title: 'Death Stranding',
      platforms: [
        { platform: 'steam', installed: true, hasOfflineToken: true },
        { platform: 'epic', installed: true, hasOfflineToken: false }, // Epic requires online check
      ],
    };

    function resolveOfflineLaunchTarget(game, isOnline) {
      if (!isOnline) {
        const offlineReady = game.platforms.find(p => p.installed && p.hasOfflineToken);
        if (!offlineReady) throw new Error('No offline launch token available for this game');
        return offlineReady.platform;
      }
      return game.platforms[0].platform;
    }

    const targetOffline = resolveOfflineLaunchTarget(unifiedGame, false);
    equal(targetOffline, 'steam', 'Automatically picked Steam as offline-capable provider');

    const targetOnline = resolveOfflineLaunchTarget(unifiedGame, true);
    equal(targetOnline, 'steam');
  });

  it('Queues playtime recorded while playing offline and syncs back to deduplicated stats upon reconnection', () => {
    const offlineSyncQueue = [];
    const consolidatedStats = { title: 'Death Stranding', totalPlaytimeMinutes: 600 };

    // Record session while offline
    offlineSyncQueue.push({ action: 'RECORD_PLAYTIME', platform: 'steam', minutes: 90 });

    // Network returns
    function processReconnectionQueue(queue, stats) {
      for (const item of queue) {
        if (item.action === 'RECORD_PLAYTIME') {
          stats.totalPlaytimeMinutes += item.minutes;
        }
      }
      queue.length = 0; // Clear queue
    }

    processReconnectionQueue(offlineSyncQueue, consolidatedStats);
    equal(consolidatedStats.totalPlaytimeMinutes, 690, 'Playtime updated to 690 minutes (11.5h)');
    equal(offlineSyncQueue.length, 0);
  });

  it('Displays cached local artwork and metadata for deduplicated titles while offline', () => {
    function resolveOfflineArtwork(gameCard, isOnline) {
      if (!isOnline) {
        return gameCard.localCacheCoverPath || 'assets/placeholders/offline_card.png';
      }
      return gameCard.remoteCdnCoverUrl;
    }

    const card = {
      title: 'Death Stranding',
      remoteCdnCoverUrl: 'https://cdn.omnilauncher.app/ds.webp',
      localCacheCoverPath: 'cache/artwork/ds_local.webp',
    };

    equal(resolveOfflineArtwork(card, false), 'cache/artwork/ds_local.webp');
    equal(resolveOfflineArtwork(card, true), 'https://cdn.omnilauncher.app/ds.webp');
  });
});

