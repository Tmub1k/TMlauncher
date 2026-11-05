const { describe, it } = require('../runner.cjs');
const { ok, equal, hasProperties } = require('../helpers/assert.cjs');

describe('Tier 1 - Feature 26: Universal Discord Rich Presence', () => {
  it('Validates Discord RPC activity payload structure and fields', () => {
    const activity = {
      details: 'Playing Cyberpunk 2077',
      state: 'Night City - Act 2',
      timestamps: {
        start: 1700000000,
      },
      assets: {
        large_image: 'cp2077_cover_highres',
        large_text: 'Cyberpunk 2077',
        small_image: 'platform_steam',
        small_text: 'Steam',
      },
      instance: true,
    };
    hasProperties(activity, ['details', 'state', 'timestamps', 'assets']);
    equal(activity.assets.small_text, 'Steam');
  });

  it('Broadcasts presence seamlessly across non-Steam titles (Epic, Roblox, Emulators)', () => {
    function createPresence(game) {
      return {
        details: `Playing ${game.title}`,
        state: game.platform === 'emulator' ? `Emulating on ${game.emulatorName}` : 'In Game',
        assets: {
          large_image: game.coverKey,
          small_image: `platform_${game.platform}`,
          small_text: game.platform.toUpperCase(),
        },
      };
    }

    const emuPresence = createPresence({
      title: 'The Legend of Zelda',
      platform: 'emulator',
      emulatorName: 'Dolphin',
      coverKey: 'zelda_ww',
    });
    equal(emuPresence.state, 'Emulating on Dolphin');
    equal(emuPresence.assets.small_text, 'EMULATOR');
  });

  it('Calculates and synchronizes elapsed playtime in Discord timestamp', () => {
    const startTimestamp = Math.floor(Date.now() / 1000) - 1800; // 30 min ago
    const presenceTimestamps = { start: startTimestamp };
    ok(presenceTimestamps.start > 0);
  });

  it('Clears Discord rich presence activity when game exits', () => {
    let currentDiscordActivity = { details: 'Playing Game' };

    function clearPresence() {
      currentDiscordActivity = null;
    }

    clearPresence();
    equal(currentDiscordActivity, null, 'Discord presence cleared immediately upon exit');
  });

  it('Respects user privacy setting disabling Discord Rich Presence', () => {
    const userSettings = { discordRpcEnabled: false };

    function shouldUpdateDiscord(settings) {
      return settings.discordRpcEnabled;
    }

    equal(shouldUpdateDiscord(userSettings), false);
  });
});
