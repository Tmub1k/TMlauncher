const { describe, it } = require('../runner.cjs');
const { ok, equal } = require('../helpers/assert.cjs');

describe('Tier 2 - Boundary 28: Proactive Offline Vault Boundaries', () => {
  it('Handles expired offline tokens when no network is available with user guidance', () => {
    function verifyOfflineToken(tokenExpiry, nowTimestamp) {
      if (nowTimestamp > tokenExpiry) {
        return {
          valid: false,
          reason: 'Offline token expired. Connect to the internet once to renew license.',
        };
      }
      return { valid: true, reason: null };
    }

    const expired = verifyOfflineToken(1690000000, 1700000000);
    equal(expired.valid, false);
    ok(expired.reason.includes('renew license'));
  });

  it('Verifies HMAC cryptographic signature on offline license files', () => {
    const crypto = require('node:crypto');
    const secret = 'omnilauncher_vault_key_2026';

    function signPayload(payload) {
      return crypto.createHmac('sha256', secret).update(payload).digest('hex');
    }

    function verifyLicense(payload, sig) {
      const expected = signPayload(payload);
      return expected === sig;
    }

    const goodPayload = 'user:123|app:730|granted:true';
    const sig = signPayload(goodPayload);
    ok(verifyLicense(goodPayload, sig));
    equal(verifyLicense(goodPayload + '_tampered', sig), false);
  });

  it('Prevents offline launch of online-only multiplayer games with helpful advisory', () => {
    function canLaunchInOfflineMode(gameMeta) {
      if (gameMeta.requiresContinuousOnlineConnection) {
        return { canLaunch: false, message: `${gameMeta.title} requires continuous server connection and cannot be played offline.` };
      }
      return { canLaunch: true, message: null };
    }

    const mmo = { title: 'MMO Battle Royale', requiresContinuousOnlineConnection: true };
    const sp = { title: 'Hollow Knight', requiresContinuousOnlineConnection: false };

    equal(canLaunchInOfflineMode(mmo).canLaunch, false);
    equal(canLaunchInOfflineMode(sp).canLaunch, true);
  });

  it('Detects system clock rollback (time-travel attempt) to prevent offline license spoofing', () => {
    function detectClockRollback(lastSavedSystemTime, currentSystemTime) {
      // If current time is significantly before the last known persistent record, system time was rolled back
      return currentSystemTime < (lastSavedSystemTime - 300);
    }

    equal(detectClockRollback(1700000000, 1600000000), true, 'Clock rollback detected');
    equal(detectClockRollback(1700000000, 1700000050), false);
  });

  it('Handles disk quota exhaustion when caching offline metadata', () => {
    function cacheOfflineRecord(record, freeSpaceMB) {
      if (freeSpaceMB < 5) {
        return { cached: false, warning: 'Skipped offline cache: local disk space critically low (<5MB)' };
      }
      return { cached: true, warning: null };
    }

    const res = cacheOfflineRecord({}, 2);
    equal(res.cached, false);
    ok(res.warning.includes('critically low'));
  });
});
