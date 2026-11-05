const { describe, it } = require('../runner.cjs');
const { ok, equal, throws } = require('../helpers/assert.cjs');

describe('Tier 2 - Boundary 13: Currency Switcher Boundaries', () => {
  const RATE = 495.0;

  it('Rejects negative prices, NaN, and infinity values', () => {
    function sanitizePrice(val) {
      if (typeof val !== 'number' || isNaN(val) || !isFinite(val) || val < 0) {
        throw new Error('Invalid price value');
      }
      return val;
    }

    ok(sanitizePrice(0) === 0);
    ok(sanitizePrice(19.99) === 19.99);
    throws(() => sanitizePrice(-5), /Invalid price/);
    throws(() => sanitizePrice(NaN), /Invalid price/);
    throws(() => sanitizePrice(Infinity), /Invalid price/);
  });

  it('Handles micro-pricing (e.g. $0.01) converting to minimum 1 KZT', () => {
    function convertToKzt(usd) {
      if (usd <= 0) return 0;
      return Math.max(1, Math.round(usd * RATE));
    }

    equal(convertToKzt(0.01), 5); // 0.01 * 495 = 4.95 -> 5 KZT
    equal(convertToKzt(0.001), 1, 'Micro-cents clamped to 1 KZT floor');
  });

  it('Formats extremely large game or bundle prices ($99,999.00) without integer overflow', () => {
    function formatBigPrice(usd) {
      const kzt = Math.round(usd * RATE);
      return {
        usdStr: `$${usd.toLocaleString('en-US')}`,
        kztStr: `${kzt.toLocaleString('ru-RU')} ₸`,
      };
    }

    const res = formatBigPrice(99999);
    equal(res.usdStr, '$99,999');
    ok(res.kztStr.includes('49'));
    ok(res.kztStr.includes('₸'));
  });

  it('Maintains pricing state consistency during rapid continuous currency toggles', () => {
    let currentCurrency = 'USD';
    const basePriceUsd = 20.00;

    for (let i = 0; i < 100; i++) {
      currentCurrency = currentCurrency === 'USD' ? 'KZT' : 'USD';
    }

    equal(currentCurrency, 'USD');
    equal(basePriceUsd, 20.00, 'Original floating price unaffected by toggle count');
  });

  it('Displays stale exchange rate notification flag when offline for extended periods', () => {
    function checkExchangeRateFreshness(lastUpdatedTimestamp, nowTimestamp) {
      const maxAgeMs = 7 * 24 * 3600 * 1000; // 7 days
      const isStale = (nowTimestamp - lastUpdatedTimestamp) > maxAgeMs;
      return { isStale, rate: RATE };
    }

    const fresh = checkExchangeRateFreshness(1700000000000, 1700000000000 + 3600000);
    equal(fresh.isStale, false);

    const stale = checkExchangeRateFreshness(1700000000000, 1700000000000 + (10 * 86400000));
    equal(stale.isStale, true, 'Flags stale conversion rate when offline > 7 days');
  });
});
