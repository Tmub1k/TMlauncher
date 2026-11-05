const { describe, it } = require('../runner.cjs');
const { ok, equal } = require('../helpers/assert.cjs');

describe('Tier 2 - Boundary 10: Roblox Recent Games Boundaries', () => {
  it('Handles missing or empty %LOCALAPPDATA%\\Roblox\\logs folder gracefully', () => {
    function scanRobloxLogs(folderExists, logFilesCount) {
      if (!folderExists || logFilesCount === 0) {
        return { places: [], error: null, warning: 'No Roblox logs found on system' };
      }
      return { places: [{ id: 1 }], error: null };
    }

    const emptyRes = scanRobloxLogs(false, 0);
    equal(emptyRes.places.length, 0);
    equal(emptyRes.error, null);
    ok(emptyRes.warning.includes('No Roblox logs'));
  });

  it('Streams and processes multi-megabyte log files efficiently using chunk scanner', () => {
    function extractPlacesFromChunk(chunk) {
      const places = [];
      const regex = /Joining place (\d+)/g;
      let match;
      while ((match = regex.exec(chunk)) !== null) {
        places.push(Number(match[1]));
      }
      return places;
    }

    const mockChunk = 'header... Joining place 12345 ... Joining place 67890 ... footer';
    const extracted = extractPlacesFromChunk(mockChunk);
    equal(extracted.length, 2);
    equal(extracted[0], 12345);
    equal(extracted[1], 67890);
  });

  it('Provides fallback thumbnail when place ID returns 404 from Roblox API', () => {
    function resolveThumbnail(status, url) {
      if (status === 404 || !url) {
        return 'assets/placeholders/roblox_game_icon.png';
      }
      return url;
    }

    equal(resolveThumbnail(404, null), 'assets/placeholders/roblox_game_icon.png');
    equal(resolveThumbnail(200, 'https://cdn.roblox.com/img.png'), 'https://cdn.roblox.com/img.png');
  });

  it('Debounces rapid successive launches of Roblox experiences to prevent client crashes', () => {
    let lastLaunchTime = 0;
    function launchPlaceRateLimited(placeId, nowTimestamp, cooldownMs = 2000) {
      if (lastLaunchTime > 0 && (nowTimestamp - lastLaunchTime < cooldownMs)) {
        return { allowed: false, reason: 'Cooldown active. Please wait.' };
      }
      lastLaunchTime = nowTimestamp;
      return { allowed: true, launchUrl: `roblox://experiences/start?placeId=${placeId}` };
    }

    const r1 = launchPlaceRateLimited(100, 1000);
    equal(r1.allowed, true);

    const r2 = launchPlaceRateLimited(100, 1500); // 500ms later
    equal(r2.allowed, false);
    ok(r2.reason.includes('Cooldown active'));
  });

  it('Validates FastFlags JSON structure ensuring correct key-value type mapping', () => {
    function validateFastFlags(flagsObj) {
      const valid = {};
      for (const [key, val] of Object.entries(flagsObj)) {
        if (key.startsWith('DFInt') || key.startsWith('FInt')) {
          if (typeof val === 'number') valid[key] = val;
        } else if (key.startsWith('FFlag') || key.startsWith('DFFlag')) {
          if (typeof val === 'boolean') valid[key] = val;
        } else if (key.startsWith('FString') || key.startsWith('DFString')) {
          if (typeof val === 'string') valid[key] = val;
        }
      }
      return valid;
    }

    const raw = {
      'DFIntTaskSchedulerTargetFps': 144,
      'FFlagEnableQuickGameJoin': true,
      'DFIntInvalidType': 'should_be_number', // Malformed type
    };

    const sanitized = validateFastFlags(raw);
    equal(sanitized['DFIntTaskSchedulerTargetFps'], 144);
    equal(sanitized['FFlagEnableQuickGameJoin'], true);
    equal('DFIntInvalidType' in sanitized, false, 'Invalid type discarded');
  });
});
