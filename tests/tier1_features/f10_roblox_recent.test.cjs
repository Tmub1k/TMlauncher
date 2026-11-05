const { describe, it } = require('../runner.cjs');
const { ok, equal, match, hasProperties } = require('../helpers/assert.cjs');
const { SAMPLE_ROBLOX_LOG } = require('../helpers/fixtures.cjs');
const { validateRobloxPlace } = require('../helpers/contracts.cjs');

describe('Tier 1 - Feature 10: Roblox Recently Played Games', () => {
  it('Parses Windows Roblox Client logs to extract PlaceId and UniverseId', () => {
    function parseRobloxLog(logContent) {
      const matchJoin = logContent.match(/Joining place (\d+) \[([^\]]+)\] with universe (\d+)/);
      if (!matchJoin) return null;
      return {
        place_id: Number(matchJoin[1]),
        name: matchJoin[2],
        universe_id: Number(matchJoin[3]),
      };
    }

    const place = parseRobloxLog(SAMPLE_ROBLOX_LOG);
    ok(place !== null);
    equal(place.place_id, 189707);
    equal(place.name, 'Natural Disaster Survival');
    equal(place.universe_id, 61578);
  });

  it('Validates RobloxPlace contract schema', () => {
    const place = {
      place_id: 189707,
      name: 'Natural Disaster Survival',
      last_played_timestamp: 1700000000,
      thumbnail_url: 'https://thumbnails.roblox.com/v1/places/gameicons?placeIds=189707&size=512x512',
    };
    validateRobloxPlace(place);
    ok(place.place_id > 0);
  });

  it('Generates native Roblox deep link launch URL', () => {
    function generateRobloxLaunchUrl(placeId) {
      return `roblox://experiences/start?placeId=${placeId}`;
    }

    const url = generateRobloxLaunchUrl(189707);
    equal(url, 'roblox://experiences/start?placeId=189707');
  });

  it('Builds high-resolution Roblox CDN thumbnail endpoint query', () => {
    function getRobloxThumbnailEndpoint(placeIds) {
      const ids = placeIds.join(',');
      return `https://thumbnails.roblox.com/v1/places/gameicons?placeIds=${ids}&size=512x512&format=Png&isCircular=false`;
    }

    const endpoint = getRobloxThumbnailEndpoint([189707, 606849626]);
    ok(endpoint.includes('placeIds=189707,606849626'));
    ok(endpoint.includes('size=512x512'));
  });

  it('Deduplicates and sorts recently played Roblox experiences chronologically', () => {
    const rawSessions = [
      { place_id: 100, name: 'Game A', timestamp: 1000 },
      { place_id: 200, name: 'Game B', timestamp: 3000 },
      { place_id: 100, name: 'Game A', timestamp: 2000 },
    ];

    function deduplicateRecentPlaces(sessions) {
      const map = new Map();
      for (const s of sessions) {
        const existing = map.get(s.place_id);
        if (!existing || s.timestamp > existing.timestamp) {
          map.set(s.place_id, s);
        }
      }
      return Array.from(map.values()).sort((a, b) => b.timestamp - a.timestamp);
    }

    const sorted = deduplicateRecentPlaces(rawSessions);
    equal(sorted.length, 2);
    equal(sorted[0].place_id, 200);
    equal(sorted[1].place_id, 100);
    equal(sorted[1].timestamp, 2000);
  });
});
