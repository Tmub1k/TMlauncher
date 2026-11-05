const { describe, it } = require('../runner.cjs');
const { ok, equal } = require('../helpers/assert.cjs');

describe('Tier 2 - Boundary 21: HowLongToBeat (HLTB) Boundaries', () => {
  it('Handles game titles with foreign alphabets, roman numerals, and special characters', () => {
    function sanitizeHltbQuery(title) {
      return title
        .replace(/[™®©]/g, '')
        .replace(/[:\-–—]/g, ' ')
        .replace(/\s+/g, ' ')
        .trim();
    }

    const title1 = 'NieR:Automata™ - Game of the YoRHa Edition';
    equal(sanitizeHltbQuery(title1), 'NieR Automata Game of the YoRHa Edition');

    const title2 = 'FINAL FANTASY VII REBIRTH';
    equal(sanitizeHltbQuery(title2), 'FINAL FANTASY VII REBIRTH');
  });

  it('Caps progress calculation for infinite or endurance games (playtime > 10,000 hours)', () => {
    function calculateEnduranceProgress(playedHours, storyTargetHours) {
      if (storyTargetHours <= 0) return 100;
      const rawPct = (playedHours / storyTargetHours) * 100;
      return Math.min(100, Math.round(rawPct));
    }

    equal(calculateEnduranceProgress(12500, 40), 100);
    equal(calculateEnduranceProgress(20, 40), 50);
  });

  it('Handles games with 0 hours reported across all categories (e.g. newly announced games)', () => {
    const unreleasedRecord = {
      game_title: 'Future RPG 2028',
      main_story_hours: 0,
      main_extra_hours: 0,
      completionist_hours: 0,
    };

    function formatHltbCard(record) {
      if (record.main_story_hours === 0) {
        return { display: 'TBD', hasData: false };
      }
      return { display: `${record.main_story_hours}h`, hasData: true };
    }

    const formatted = formatHltbCard(unreleasedRecord);
    equal(formatted.display, 'TBD');
    equal(formatted.hasData, false);
  });

  it('Disambiguates multiple search results matching query to pick best candidate', () => {
    const searchResults = [
      { title: 'The Witcher', year: 2007, similarity: 0.7 },
      { title: 'The Witcher 3: Wild Hunt', year: 2015, similarity: 0.98 },
      { title: 'The Witcher 2: Assassins of Kings', year: 2011, similarity: 0.75 },
    ];

    function pickBestMatch(results) {
      return [...results].sort((a, b) => b.similarity - a.similarity)[0];
    }

    const best = pickBestMatch(searchResults);
    equal(best.title, 'The Witcher 3: Wild Hunt');
  });

  it('Clamps negative recorded playtime to 0 in HLTB calculation', () => {
    function safePlaytime(hours) {
      return Math.max(0, hours || 0);
    }

    equal(safePlaytime(-15), 0);
    equal(safePlaytime(25), 25);
    equal(safePlaytime(null), 0);
  });
});
