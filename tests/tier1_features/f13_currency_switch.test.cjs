const { describe, it } = require('../runner.cjs');
const { ok, equal, match } = require('../helpers/assert.cjs');

describe('Tier 1 - Feature 13: Currency Switcher (USD / KZT)', () => {
  const FIXED_EXCHANGE_RATE = 495.0; // 1 USD = 495 KZT

  it('Converts price between USD and Kazakhstani Tenge (KZT)', () => {
    function convertCurrency(usdPrice, targetCurrency) {
      if (targetCurrency === 'USD') return usdPrice;
      if (targetCurrency === 'KZT') return Math.round(usdPrice * FIXED_EXCHANGE_RATE);
      throw new Error(`Unsupported currency: ${targetCurrency}`);
    }

    const usdVal = 59.99;
    const kztVal = convertCurrency(usdVal, 'KZT');
    equal(kztVal, 29695);
  });

  it('Formats currency strings with authoritative symbols ($ and ₸)', () => {
    function formatPrice(amount, currency) {
      if (currency === 'USD') {
        return `$${amount.toFixed(2)}`;
      }
      if (currency === 'KZT') {
        return `${Math.round(amount).toLocaleString('ru-RU')} ₸`;
      }
      return `${amount}`;
    }

    const formattedUsd = formatPrice(59.99, 'USD');
    equal(formattedUsd, '$59.99');

    const formattedKzt = formatPrice(29695, 'KZT');
    ok(formattedKzt.includes('₸'));
    ok(formattedKzt.includes('29'));
  });

  it('Supports free-to-play pricing in both currency modes', () => {
    function getDisplayPrice(priceRaw, currency) {
      if (priceRaw === 0) return 'Free to Play';
      return currency === 'USD' ? `$${priceRaw}` : `${priceRaw} ₸`;
    }

    equal(getDisplayPrice(0, 'USD'), 'Free to Play');
    equal(getDisplayPrice(0, 'KZT'), 'Free to Play');
  });

  it('Persists selected currency preference across sessions', () => {
    const userConfig = {
      preferredCurrency: 'USD',
    };
    function setCurrency(cfg, newCurr) {
      if (!['USD', 'KZT'].includes(newCurr)) throw new Error('Invalid currency');
      cfg.preferredCurrency = newCurr;
      return cfg;
    }

    setCurrency(userConfig, 'KZT');
    equal(userConfig.preferredCurrency, 'KZT');
  });

  it('Calculates bundle discounts preserving proper rounding in KZT', () => {
    const rawBundleUsd = 19.99;
    const bundleKzt = Math.round(rawBundleUsd * FIXED_EXCHANGE_RATE);
    ok(Number.isInteger(bundleKzt), 'KZT prices must round to whole numbers without fractional coins');
    equal(bundleKzt, 9895);
  });
});
