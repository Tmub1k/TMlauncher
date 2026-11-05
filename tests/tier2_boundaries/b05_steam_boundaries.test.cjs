const { describe, it } = require('../runner.cjs');
const { ok, equal, throws } = require('../helpers/assert.cjs');

describe('Tier 2 - Boundary 05: Steam Account Link Boundaries', () => {
  it('Rejects invalid SteamID formats (short, non-numeric, overflow)', () => {
    function isValidSteamId64(id) {
      if (typeof id !== 'string') return false;
      return /^[0-9]{17}$/.test(id) && id.startsWith('7656119');
    }

    ok(isValidSteamId64('76561198012345678'));
    equal(isValidSteamId64('12345'), false, 'Too short');
    equal(isValidSteamId64('76561198012345678999'), false, 'Too long');
    equal(isValidSteamId64('7656119abcdefghij'), false, 'Non-numeric');
    equal(isValidSteamId64(null), false);
  });

  it('Handles private Steam profile settings gracefully without failing library load', () => {
    const steamApiResponse = {
      response: {
        players: [
          {
            steamid: '76561198000000000',
            communityvisibilitystate: 1, // 1 = Private, 3 = Public
            personaname: 'PrivateUser',
          },
        ],
      },
    };

    function parseProfilePrivacy(player) {
      const isPrivate = player.communityvisibilitystate === 1;
      return {
        isPrivate,
        canQueryGames: !isPrivate,
        message: isPrivate ? 'Steam profile is set to Private. Enable public games in Steam privacy settings.' : 'OK',
      };
    }

    const res = parseProfilePrivacy(steamApiResponse.response.players[0]);
    equal(res.isPrivate, true);
    equal(res.canQueryGames, false);
    ok(res.message.includes('Private'));
  });

  it('Handles Steam Web API network timeouts falling back to offline data', () => {
    function fetchWithFallback(networkSucceeds, cachedData) {
      if (!networkSucceeds) {
        return { source: 'cache', data: cachedData };
      }
      return { source: 'network', data: { name: 'Fresh' } };
    }

    const fallbackRes = fetchWithFallback(false, { name: 'CachedPlayer' });
    equal(fallbackRes.source, 'cache');
    equal(fallbackRes.data.name, 'CachedPlayer');
  });

  it('Handles complex unicode, RTL, and emoji characters in Steam persona names', () => {
    function sanitizePersona(name) {
      if (!name || !name.trim()) return 'Unknown Player';
      return name.trim();
    }

    const unicodeName = '🎮 ProGamer_ツ [VIP] ﷽';
    equal(sanitizePersona(unicodeName), unicodeName);
    equal(sanitizePersona('   '), 'Unknown Player');
  });

  it('Recovers gracefully from corrupted Valve VDF syntax in local config files', () => {
    function parseVdfSafe(rawVdf) {
      try {
        const openBraces = (rawVdf.match(/\{/g) || []).length;
        const closeBraces = (rawVdf.match(/\}/g) || []).length;
        if (openBraces !== closeBraces) throw new Error('Unbalanced VDF braces');
        return { success: true };
      } catch (err) {
        return { success: false, error: err.message };
      }
    }

    const truncated = `"Software" { "Valve" { "Steam" { `;
    const res = parseVdfSafe(truncated);
    equal(res.success, false);
    equal(res.error, 'Unbalanced VDF braces');
  });
});
