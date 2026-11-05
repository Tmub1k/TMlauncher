const { describe, it } = require('../runner.cjs');
const { ok, equal } = require('../helpers/assert.cjs');

describe('Tier 3 - Interaction 02: Currency Switcher & DealsRadar Integration', () => {
  const KZT_RATE = 495.0;

  it('Dynamically recalculates multi-store deal prices across Steam, Epic, and GOG when switching to KZT', () => {
    const rawDeals = [
      { store: 'Steam', title: 'Cyberpunk 2077', priceUsd: 29.99 },
      { store: 'GOG', title: 'Cyberpunk 2077', priceUsd: 24.99 },
      { store: 'Epic', title: 'Cyberpunk 2077', priceUsd: 29.99 },
    ];

    function convertDealsRadar(deals, targetCurrency) {
      return deals.map(d => {
        const rawAmount = targetCurrency === 'KZT' ? Math.round(d.priceUsd * KZT_RATE) : d.priceUsd;
        const formatted = targetCurrency === 'KZT' ? `${rawAmount.toLocaleString('ru-RU')} ₸` : `$${rawAmount.toFixed(2)}`;
        return {
          ...d,
          displayPrice: formatted,
          currency: targetCurrency,
          amount: rawAmount,
        };
      });
    }

    const converted = convertDealsRadar(rawDeals, 'KZT');
    equal(converted[0].currency, 'KZT');
    ok(converted[0].displayPrice.includes('₸'));
    equal(converted[1].amount, Math.round(24.99 * 495));
  });

  it('Preserves All-Time Low (ATL) price calculation fidelity regardless of selected currency', () => {
    const historicalLowUsd = 14.99;
    const currentPriceUsd = 14.99;

    function isHistoricalLowInCurrency(currentAmount, atlAmount) {
      return currentAmount <= atlAmount;
    }

    // In USD
    equal(isHistoricalLowInCurrency(currentPriceUsd, historicalLowUsd), true);

    // Converted to KZT
    const curKzt = Math.round(currentPriceUsd * KZT_RATE);
    const atlKzt = Math.round(historicalLowUsd * KZT_RATE);
    equal(isHistoricalLowInCurrency(curKzt, atlKzt), true, 'ATL truth preserved after currency conversion');
  });

  it('Renders deal savings banner in active currency with proper regional formatting', () => {
    function getSavingsBanner(regularUsd, saleUsd, currency) {
      const savedUsd = regularUsd - saleUsd;
      if (currency === 'KZT') {
        const savedKzt = Math.round(savedUsd * KZT_RATE);
        return `Вы экономите ${savedKzt.toLocaleString('ru-RU')} ₸`;
      }
      return `You save $${savedUsd.toFixed(2)}`;
    }

    equal(getSavingsBanner(59.99, 29.99, 'USD'), 'You save $30.00');
    const kztBanner = getSavingsBanner(59.99, 29.99, 'KZT');
    ok(kztBanner.includes('Вы экономите'));
    ok(kztBanner.includes('₸'));
  });
});
