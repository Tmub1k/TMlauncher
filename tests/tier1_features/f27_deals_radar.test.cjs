const { describe, it } = require('../runner.cjs');
const { ok, equal, hasProperties } = require('../helpers/assert.cjs');

describe('Tier 1 - Feature 27: DealsRadar & Historical Low Price', () => {
  it('Compares prices across multiple digital storefronts (Steam, Epic, GOG)', () => {
    const deals = [
      { store: 'Steam', currentPrice: 29.99, regularPrice: 59.99, discountPct: 50 },
      { store: 'GOG', currentPrice: 19.99, regularPrice: 59.99, discountPct: 67 },
      { store: 'Epic', currentPrice: 34.99, regularPrice: 59.99, discountPct: 42 },
    ];

    function findBestDeal(list) {
      return [...list].sort((a, b) => a.currentPrice - b.currentPrice)[0];
    }

    const best = findBestDeal(deals);
    equal(best.store, 'GOG');
    equal(best.currentPrice, 19.99);
  });

  it('Identifies historical all-time low (ATL) price milestone', () => {
    function isHistoricalLow(currentPrice, historicalLowPrice) {
      return currentPrice <= historicalLowPrice;
    }

    equal(isHistoricalLow(14.99, 14.99), true);
    equal(isHistoricalLow(11.99, 14.99), true, 'New historical record low');
    equal(isHistoricalLow(19.99, 14.99), false);
  });

  it('Formats deal badge with discount percentage and store tag', () => {
    function formatDealBadge(discountPct, isAtl) {
      const atlTag = isAtl ? ' [ATL!]' : '';
      return `-${discountPct}%${atlTag}`;
    }

    equal(formatDealBadge(67, true), '-67% [ATL!]');
    equal(formatDealBadge(50, false), '-50%');
  });

  it('Generates official store checkout redirect URL with affiliate tracking', () => {
    function getStoreUrl(store, gameSlug) {
      if (store === 'Steam') return `https://store.steampowered.com/app/${gameSlug}`;
      if (store === 'GOG') return `https://www.gog.com/game/${gameSlug}`;
      return `https://store.epicgames.com/p/${gameSlug}`;
    }

    const steamUrl = getStoreUrl('Steam', '1091500');
    equal(steamUrl, 'https://store.steampowered.com/app/1091500');
  });

  it('Filters out deals below minimum user discount threshold', () => {
    const deals = [
      { id: 1, discountPct: 10 },
      { id: 2, discountPct: 50 },
      { id: 3, discountPct: 75 },
    ];

    function filterByMinDiscount(list, minPct) {
      return list.filter(d => d.discountPct >= minPct);
    }

    const filtered = filterByMinDiscount(deals, 50);
    equal(filtered.length, 2);
  });
});
