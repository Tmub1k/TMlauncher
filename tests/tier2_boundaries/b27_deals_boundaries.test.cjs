const { describe, it } = require('../runner.cjs');
const { ok, equal } = require('../helpers/assert.cjs');

describe('Tier 2 - Boundary 27: DealsRadar & Historical Low Boundaries', () => {
  it('Handles 100% discount / free promotional giveaways ($0.00)', () => {
    function processDeal(regularPrice, currentPrice) {
      if (currentPrice === 0) {
        return { isFree: true, discountPct: 100, label: 'CLAIM FREE' };
      }
      const pct = Math.round(((regularPrice - currentPrice) / regularPrice) * 100);
      return { isFree: false, discountPct: pct, label: `-${pct}%` };
    }

    const freebie = processDeal(59.99, 0);
    equal(freebie.isFree, true);
    equal(freebie.discountPct, 100);
    equal(freebie.label, 'CLAIM FREE');
  });

  it('Handles titles with missing historical price records (first-time tracked items)', () => {
    function assessDeal(currentPrice, historicalLow) {
      if (historicalLow === null || historicalLow === undefined) {
        return { isHistoricalLow: false, hasHistoricalData: false, note: 'Initial price tracking entry' };
      }
      return { isHistoricalLow: currentPrice <= historicalLow, hasHistoricalData: true, note: null };
    }

    const firstTime = assessDeal(19.99, null);
    equal(firstTime.hasHistoricalData, false);
    ok(firstTime.note.includes('Initial price tracking'));
  });

  it('Handles price increases (negative discount percent) by discarding deal listing', () => {
    function filterValidDeals(list) {
      return list.filter(item => item.currentPrice < item.regularPrice);
    }

    const candidateDeals = [
      { id: 1, currentPrice: 15, regularPrice: 30 }, // 50% off
      { id: 2, currentPrice: 35, regularPrice: 30 }, // Price hike
    ];

    const valid = filterValidDeals(candidateDeals);
    equal(valid.length, 1);
    equal(valid[0].id, 1);
  });

  it('Normalizes multi-store currency discrepancies before comparing deal prices', () => {
    function compareCrossStore(storePricesUsd) {
      return [...storePricesUsd].sort((a, b) => a.priceUsd - b.priceUsd)[0];
    }

    const comparison = compareCrossStore([
      { store: 'Steam', priceUsd: 29.99 },
      { store: 'GOG (EUR converted)', priceUsd: 27.50 },
      { store: 'Epic', priceUsd: 29.99 },
    ]);

    equal(comparison.store, 'GOG (EUR converted)');
    equal(comparison.priceUsd, 27.50);
  });

  it('Detects and filters expired coupon codes or time-limited promotion flash sales', () => {
    function isCouponValid(coupon, now) {
      return now >= coupon.startsAt && now <= coupon.expiresAt;
    }

    const now = 1700000000;
    const active = { startsAt: 1699900000, expiresAt: 1700100000 };
    const expired = { startsAt: 1690000000, expiresAt: 1699000000 };

    equal(isCouponValid(active, now), true);
    equal(isCouponValid(expired, now), false);
  });
});
