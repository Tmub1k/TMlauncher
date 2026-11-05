const { describe, it } = require('../runner.cjs');
const { ok, equal, hasProperties } = require('../helpers/assert.cjs');
const { validateHltbRecord } = require('../helpers/contracts.cjs');

describe('Tier 1 - Feature 21: HowLongToBeat (HLTB) Forecasting', () => {
  it('Validates HltbRecord contract and playtime targets', () => {
    const record = {
      game_title: 'The Witcher 3: Wild Hunt',
      main_story_hours: 51.5,
      main_extra_hours: 102.0,
      completionist_hours: 173.0,
      confidence_score: 0.95,
    };
    validateHltbRecord(record);
    ok(record.main_story_hours > 0);
    ok(record.completionist_hours >= record.main_extra_hours);
  });

  it('Calculates dynamic progress bar percentage against recorded playtime', () => {
    function calculateHltbProgress(playedHours, targetHours) {
      if (targetHours <= 0) return 0;
      const pct = Math.min(100, Math.round((playedHours / targetHours) * 100));
      return pct;
    }

    const progStory = calculateHltbProgress(25, 50);
    equal(progStory, 50);

    const progDone = calculateHltbProgress(60, 50);
    equal(progDone, 100, 'Caps at 100% when exceeding target');
  });

  it('Provides fallback estimate for uncatalogued or indie titles', () => {
    function resolveHltbWithFallback(record) {
      if (!record || !record.main_story_hours) {
        return {
          main_story_hours: 10.0, // Default estimate
          is_estimated_fallback: true,
        };
      }
      return { ...record, is_estimated_fallback: false };
    }

    const fallback = resolveHltbWithFallback(null);
    equal(fallback.main_story_hours, 10.0);
    equal(fallback.is_estimated_fallback, true);
  });

  it('Formats duration hours and minutes into human-readable strings', () => {
    function formatHours(hours) {
      const h = Math.floor(hours);
      const m = Math.round((hours - h) * 60);
      return m > 0 ? `${h}h ${m}m` : `${h}h`;
    }

    equal(formatHours(51.5), '51h 30m');
    equal(formatHours(12.0), '12h');
  });

  it('Computes remaining hours to complete story given current playtime', () => {
    function hoursRemaining(played, target) {
      return Math.max(0, target - played);
    }

    equal(hoursRemaining(30, 50), 20);
    equal(hoursRemaining(55, 50), 0);
  });
});
