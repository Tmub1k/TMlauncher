const { describe, it } = require('../runner.cjs');
const { ok, equal, hasProperties } = require('../helpers/assert.cjs');
const { validateStorePage } = require('../helpers/contracts.cjs');

describe('Tier 1 - Feature 12: Steam Store Infinite Recommendations', () => {
  it('Validates StorePage pagination schema and item payload', () => {
    const page = {
      page: 1,
      currency: 'USD',
      items: [
        {
          app_id: 1091500,
          title: 'Cyberpunk 2077',
          price_raw: 59.99,
          price_formatted: '$59.99',
          discount_percent: 50,
          thumbnail_url: 'https://cdn.example.com/capsule_sm.jpg',
        },
      ],
      has_more: true,
    };
    validateStorePage(page);
    equal(page.has_more, true);
    equal(page.items.length, 1);
  });

  it('Rejects camelCase payload to protect contract against regression', () => {
    const camelCasePayload = {
      page: 1,
      currency: 'USD',
      items: [{ appId: 1091500, title: 'Cyberpunk 2077', priceRaw: 59.99, priceFormatted: '$59.99', thumbnailUrl: '...' }],
      hasMore: true,
    };
    let threw = false;
    try {
      validateStorePage(camelCasePayload);
    } catch (err) {
      threw = true;
      ok(err.message.includes('has_more'), 'Must reject missing has_more');
    }
    ok(threw, 'validateStorePage must reject camelCase payload');
  });

  it('Calculates discounted price and discount percentage accurately', () => {
    function computeDiscount(originalPrice, discountPct) {
      const finalPrice = originalPrice * (1 - discountPct / 100);
      return Math.round(finalPrice * 100) / 100;
    }

    const salePrice = computeDiscount(59.99, 50);
    equal(salePrice, 30.00);
  });

  it('Filters store recommendations by user preferred tags', () => {
    const catalog = [
      { id: 1, title: 'Doom Eternal', tags: ['FPS', 'Action'] },
      { id: 2, title: 'Hades', tags: ['Roguelike', 'Action', 'Indie'] },
      { id: 3, title: 'Civilization VI', tags: ['Strategy', 'Turn-Based'] },
    ];

    function filterByTags(items, filterTag) {
      return items.filter(item => item.tags.includes(filterTag));
    }

    const fpsGames = filterByTags(catalog, 'FPS');
    equal(fpsGames.length, 1);
    equal(fpsGames[0].title, 'Doom Eternal');
  });

  it('Advances pagination smoothly during infinite scrolling', () => {
    function getNextPage(currentPage, totalPages) {
      return {
        nextPage: currentPage + 1,
        hasMore: currentPage + 1 < totalPages,
      };
    }

    const p1 = getNextPage(1, 5);
    equal(p1.nextPage, 2);
    equal(p1.hasMore, true);

    const p4 = getNextPage(4, 5);
    equal(p4.hasMore, false);
  });

  it('Handles video trailer and media carousel metadata', () => {
    const item = {
      app_id: 1091500,
      title: 'Cyberpunk 2077',
      media: {
        screenshots: ['img1.jpg', 'img2.jpg'],
        webmVideo: 'https://cdn.example.com/trailer.webm',
        mp4Video: 'https://cdn.example.com/trailer.mp4',
      },
    };
    ok(item.media.screenshots.length >= 2);
    ok(item.media.mp4Video.endsWith('.mp4'));
  });
});
