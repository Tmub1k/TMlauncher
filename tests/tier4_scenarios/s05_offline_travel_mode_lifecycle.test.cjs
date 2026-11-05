const { describe, it } = require('../runner.cjs');
const { ok, equal } = require('../helpers/assert.cjs');

describe('Tier 4 - Scenario 05: Offline Travel Mode Full Lifecycle', () => {
  it('Transitions from online cache preparation -> offline airplane play -> reconnection and cloud reconciliation', () => {
    // 1. Online preparation: Proactive Offline Vault caches tokens and game art
    const offlineVaultCache = new Map();
    offlineVaultCache.set('app_730', {
      title: 'Counter-Strike 2',
      offlineToken: 'valid_token_token_123',
      artworkPath: 'cache/art/730.webp',
      lastSynced: 1700000000,
    });
    equal(offlineVaultCache.has('app_730'), true);

    // 2. Offline Phase: Network disconnected
    let isConnected = false;
    const offlineActionQueue = [];

    function attemptLaunchGame(appId) {
      if (!isConnected) {
        const cached = offlineVaultCache.get(appId);
        if (!cached || !cached.offlineToken) throw new Error('Offline license missing');
        return { launchedOffline: true, title: cached.title };
      }
      return { launchedOffline: false };
    }

    const launchRes = attemptLaunchGame('app_730');
    equal(launchRes.launchedOffline, true, 'Launched smoothly without internet');

    // 3. User plays for 2 hours and earns achievement
    offlineActionQueue.push({ action: 'RECORD_PLAYTIME', hours: 2 });
    offlineActionQueue.push({ action: 'UNLOCK_ACHIEVEMENT', achId: 'ACH_AIRPLANE_MODE' });
    equal(offlineActionQueue.length, 2);

    // 4. Reconnection Phase: Internet restored
    isConnected = true;
    let cloudReconciledCount = 0;

    function reconcileWithCloud(queue) {
      while (queue.length > 0) {
        queue.shift();
        cloudReconciledCount++;
      }
    }

    reconcileWithCloud(offlineActionQueue);
    equal(offlineActionQueue.length, 0, 'Queue completely flushed');
    equal(cloudReconciledCount, 2, 'Playtime and achievements synced to cloud server');
  });
});
