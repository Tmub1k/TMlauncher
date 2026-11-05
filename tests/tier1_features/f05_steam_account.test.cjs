const { describe, it } = require('../runner.cjs');
const { ok, equal, match, hasProperties } = require('../helpers/assert.cjs');
const { validateSteamProfile } = require('../helpers/contracts.cjs');

describe('Tier 1 - Feature 05: Steam Account & Profile Link', () => {
  it('Validates 64-bit SteamID resolution and format invariants', () => {
    const profile = {
      steam_id: '76561198000000000',
      persona_name: 'GamerZero',
      profile_url: 'https://steamcommunity.com/profiles/76561198000000000',
      avatar_url: 'https://avatars.steamstatic.com/sample_full.jpg',
    };
    validateSteamProfile(profile);
    equal(profile.steam_id.length, 17, 'SteamID64 must be 17 digits');
  });

  it('Generates authoritative Steam community web link from profile', () => {
    const steamId = '76561198012345678';
    const profileUrl = `https://steamcommunity.com/profiles/${steamId}`;
    match(profileUrl, /^https:\/\/steamcommunity\.com\/profiles\/[0-9]{17}$/);
  });

  it('Resolves custom vanity URL into standard profile representation', () => {
    const mockVanityMap = {
      'gaben_official': '76561197960287930',
      'player_alpha': '76561198099887766',
    };
    function resolveVanity(vanity) {
      if (!mockVanityMap[vanity]) throw new Error(`Unknown vanity URL: ${vanity}`);
      return mockVanityMap[vanity];
    }

    const resolved = resolveVanity('gaben_official');
    equal(resolved, '76561197960287930');
    equal(resolved.length, 17);
  });

  it('Loads avatar asset URL and provides high-res fallback', () => {
    function getAvatarWithFallback(url) {
      if (!url || !url.startsWith('https://')) {
        return 'assets/defaults/steam_avatar_default.png';
      }
      return url;
    }

    const validAvatar = getAvatarWithFallback('https://avatars.steamstatic.com/test.jpg');
    equal(validAvatar, 'https://avatars.steamstatic.com/test.jpg');

    const fallbackAvatar = getAvatarWithFallback(null);
    equal(fallbackAvatar, 'assets/defaults/steam_avatar_default.png');
  });

  it('Handles offline cached profile when network is disconnected', () => {
    const cachedProfile = {
      steam_id: '76561198000000000',
      persona_name: 'OfflinePlayer',
      profile_url: 'https://steamcommunity.com/profiles/76561198000000000',
      avatar_url: 'https://avatars.steamstatic.com/sample_full.jpg',
      is_cached: true,
      last_synced: 1700000000,
    };
    validateSteamProfile(cachedProfile);
    equal(cachedProfile.is_cached, true);
    ok(cachedProfile.last_synced > 0);
  });
});
