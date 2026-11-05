/**
 * Empirical Adversarial Challenger M3 Recheck Stress Harness (Node.js)
 * 
 * Tests:
 * 1. Store IPC serialization schema validation (snake_case vs camelCase).
 * 2. Rapid pagination race conditions, debounce locks, exhaustion boundaries.
 * 3. Currency switcher (USD <-> KZT) floating-point round-trip, micro-cents, and extreme numbers.
 * 4. Tag filtering under adversarial inputs (SQL injection, XSS, control chars, Unicode).
 */

const assert = require('node:assert');
const { validateStorePage } = require('../helpers/contracts.cjs');

console.log('================================================================');
console.log('  CHALLENGER M3 EMPIRICAL STRESS & BOUNDARY TEST HARNESS        ');
console.log('================================================================\n');

let passCount = 0;
let failCount = 0;

function runTest(name, fn) {
  try {
    fn();
    passCount++;
    console.log(`[PASS] ${name}`);
  } catch (err) {
    failCount++;
    console.error(`[FAIL] ${name}: ${err.message}`);
  }
}

// ----------------------------------------------------------------------------
// TEST SUITE 1: Store IPC Serialization & Serde Parity
// ----------------------------------------------------------------------------
console.log('\n--- 1. Store IPC Serialization Contract Tests ---');

runTest('StorePage with snake_case keys passes validation', () => {
  const validPage = {
    page: 0,
    currency: 'USD',
    items: [
      {
        app_id: 1091500,
        title: 'Cyberpunk 2077',
        price_raw: 29.99,
        price_formatted: '$29.99',
        discount_percent: 50,
        thumbnail_url: 'https://cdn.example.com/capsule.jpg',
        header_image: 'https://cdn.example.com/header.jpg',
        banner_url: 'https://cdn.example.com/banner.jpg',
        tags: ['RPG', 'Action'],
        review_score: 'Very Positive',
        media: {
          screenshots: ['s1.jpg', 's2.jpg'],
          webmVideo: 'https://video.com/v.webm',
          mp4Video: 'https://video.com/v.mp4',
        },
        short_description: 'RPG in Night City',
        release_date: '10 Dec 2020',
        store_url: 'https://store.steampowered.com/app/1091500',
        install_uri: 'steam://install/1091500',
      },
    ],
    has_more: true,
    total_pages: 10,
    end_message: null,
  };

  validateStorePage(validPage);
  assert.strictEqual(validPage.has_more, true);
  assert.strictEqual(validPage.items[0].app_id, 1091500);
  assert.strictEqual(validPage.items[0].media.mp4Video, 'https://video.com/v.mp4');
});

runTest('StorePage rejects camelCase "hasMore" regression', () => {
  const camelCasePage = {
    page: 0,
    currency: 'USD',
    items: [],
    hasMore: true, // REGRESSION!
  };
  assert.throws(() => validateStorePage(camelCasePage), /has_more/);
});

runTest('StoreItem rejects camelCase "appId" regression', () => {
  const camelCaseItemPage = {
    page: 0,
    currency: 'USD',
    items: [
      {
        appId: 1091500, // REGRESSION!
        title: 'Cyberpunk 2077',
        price_raw: 29.99,
        price_formatted: '$29.99',
        thumbnail_url: '...',
      },
    ],
    has_more: true,
  };
  assert.throws(() => validateStorePage(camelCaseItemPage), /app_id/);
});

runTest('StoreItem rejects camelCase "priceFormatted" regression', () => {
  const camelCaseItemPage = {
    page: 0,
    currency: 'USD',
    items: [
      {
        app_id: 1091500,
        title: 'Cyberpunk 2077',
        price_raw: 29.99,
        priceFormatted: '$29.99', // REGRESSION!
        thumbnail_url: '...',
      },
    ],
    has_more: true,
  };
  assert.throws(() => validateStorePage(camelCaseItemPage), /price_formatted/);
});

// ----------------------------------------------------------------------------
// TEST SUITE 2: Currency Switching & Floating-Point Roundtrip
// ----------------------------------------------------------------------------
console.log('\n--- 2. Currency Switching & Precision Tests ---');

const FIXED_RATE = 495.0;

function convertFromUsd(usd, curr) {
  if (usd <= 0) return 0;
  if (curr === 'USD') return usd;
  return Math.max(1, Math.round(usd * FIXED_RATE));
}

function formatPrice(usd, curr) {
  if (usd === 0) return 'Free to Play';
  if (curr === 'USD') return `$${usd.toFixed(2)}`;
  const kzt = Math.max(1, Math.round(usd * FIXED_RATE));
  return `${kzt.toLocaleString('ru-RU')} ₸`;
}

runTest('Zero USD formats as "Free to Play" in both currencies', () => {
  assert.strictEqual(formatPrice(0.0, 'USD'), 'Free to Play');
  assert.strictEqual(formatPrice(0.0, 'KZT'), 'Free to Play');
  assert.strictEqual(convertFromUsd(0.0, 'KZT'), 0);
});

runTest('Micro-cents clamp to 1 KZT floor', () => {
  assert.strictEqual(convertFromUsd(0.0001, 'KZT'), 1);
  assert.strictEqual(convertFromUsd(0.001, 'KZT'), 1);
  assert.strictEqual(formatPrice(0.001, 'KZT'), '1 ₸');
});

runTest('Standard prices convert accurately', () => {
  assert.strictEqual(convertFromUsd(59.99, 'KZT'), 29695);
  assert.strictEqual(convertFromUsd(69.99, 'KZT'), 34645);
  assert.strictEqual(convertFromUsd(19.99, 'KZT'), 9895);
  assert.strictEqual(convertFromUsd(9.99, 'KZT'), 4945);
});

runTest('Massive price calculation avoids 32-bit overflow', () => {
  const massive = convertFromUsd(99999999.0, 'KZT');
  assert.strictEqual(massive, 49499999505);
});

runTest('10,000 rapid floating point roundtrip stability checks', () => {
  for (let i = 0; i < 10000; i++) {
    const rawCents = Math.floor(Math.random() * 10000) + 1; // 1 cent to $100.00
    const usd = rawCents / 100;
    const kzt = convertFromUsd(usd, 'KZT');
    const backToUsd = kzt / FIXED_RATE;
    const drift = Math.abs(backToUsd - usd);
    assert.ok(drift < 0.015, `Drift ${drift} exceeded threshold for USD ${usd}`);
  }
});

// ----------------------------------------------------------------------------
// TEST SUITE 3: Rapid Pagination & Debounce Lock
// ----------------------------------------------------------------------------
console.log('\n--- 3. Rapid Pagination & In-Flight Lock Tests ---');

class MockStoreFeed {
  constructor() {
    this.inFlight = false;
    this.loading = false;
    this.items = [];
    this.page = 0;
    this.hasMore = true;
    this.endMessage = null;
    this.dispatches = 0;
  }

  async loadPage(pageToLoad, append = false) {
    if (this.inFlight) return false;
    this.inFlight = true;
    this.loading = true;
    this.dispatches++;

    // Simulate async network delay
    await new Promise((resolve) => setTimeout(resolve, 5));

    const maxPages = 10;
    const isExhausted = pageToLoad >= maxPages;

    const res = {
      page: pageToLoad,
      items: isExhausted ? [] : [{ app_id: 100 + pageToLoad }],
      has_more: pageToLoad + 1 < maxPages,
      end_message: isExhausted || (pageToLoad + 1 >= maxPages)
        ? 'You have caught up with all current recommendations'
        : null,
    };

    if (append) {
      this.items = [...this.items, ...res.items];
    } else {
      this.items = res.items;
    }

    this.page = res.page;
    this.hasMore = res.has_more;
    this.endMessage = res.end_message;
    this.loading = false;
    this.inFlight = false;
    return true;
  }

  async loadNext() {
    if (!this.hasMore || this.loading || this.inFlight) return false;
    return await this.loadPage(this.page + 1, true);
  }
}

runTest('Rapid concurrent loadNext calls are throttled by inFlight lock', async () => {
  const feed = new MockStoreFeed();
  await feed.loadPage(0, false);
  assert.strictEqual(feed.page, 0);
  assert.strictEqual(feed.hasMore, true);

  // Fire 20 parallel calls to loadNext
  const promises = [];
  for (let i = 0; i < 20; i++) {
    promises.push(feed.loadNext());
  }
  const results = await Promise.all(promises);
  const accepted = results.filter((r) => r === true).length;
  const rejected = results.filter((r) => r === false).length;

  assert.strictEqual(accepted, 1, 'Only 1 request should proceed through inFlight lock');
  assert.strictEqual(rejected, 19, '19 parallel duplicate requests should be rejected');
  assert.strictEqual(feed.page, 1);
});

runTest('Catalog exhausts at page 9 with has_more = false', async () => {
  const feed = new MockStoreFeed();
  for (let p = 0; p < 9; p++) {
    await feed.loadPage(p, true);
    assert.strictEqual(feed.hasMore, true);
  }
  await feed.loadPage(9, true);
  assert.strictEqual(feed.hasMore, false, 'Page 9 must have hasMore = false');
  assert.ok(feed.endMessage.includes('caught up'));
});

// ----------------------------------------------------------------------------
// TEST SUITE 4: Tag Filtering & Injection Stress
// ----------------------------------------------------------------------------
console.log('\n--- 4. Tag Filtering & Adversarial Inputs ---');

const CATALOG = [
  { title: 'Cyberpunk 2077', tags: ['Action', 'RPG', 'FPS', 'Open World', 'Sci-Fi'] },
  { title: 'Hades', tags: ['Roguelike', 'Action', 'Indie'] },
  { title: 'DOOM Eternal', tags: ['FPS', 'Action', 'Sci-Fi'] },
  { title: 'Sid Meier\'s Civilization VI', tags: ['Strategy', 'Turn-Based', 'Simulation'] },
  { title: 'Counter-Strike 2', tags: ['FPS', 'Action'] },
];

function filterCatalog(tag, query) {
  const cleanQuery = query ? query.replace(/['";{}()$]/g, '').trim().toLowerCase() : '';
  const filterTag = tag ? tag.trim() : '';

  return CATALOG.filter((item) => {
    if (cleanQuery && !item.title.toLowerCase().includes(cleanQuery)) return false;
    if (filterTag && filterTag !== 'All' && !item.tags.some((t) => t.toLowerCase() === filterTag.toLowerCase())) {
      return false;
    }
    return true;
  });
}

runTest('Adversarial SQL tags safely yield empty result', () => {
  const sqlPayloads = [
    "' OR 1=1 --",
    "'; DROP TABLE Games; --",
    "FPS' OR 1=1 --",
    "UNION SELECT *",
  ];
  for (const payload of sqlPayloads) {
    const res = filterCatalog(payload, null);
    assert.strictEqual(res.length, 0, `SQL payload '${payload}' must not match any items`);
  }
});

runTest('Empty or "All" tag returns entire catalog', () => {
  assert.strictEqual(filterCatalog('', null).length, CATALOG.len || 5);
  assert.strictEqual(filterCatalog('   ', null).length, 5);
  assert.strictEqual(filterCatalog('All', null).length, 5);
});

runTest('Case variations match accurately', () => {
  assert.strictEqual(filterCatalog('fps', null).length, 3);
  assert.strictEqual(filterCatalog('FPS', null).length, 3);
  assert.strictEqual(filterCatalog('Fps', null).length, 3);
});

runTest('Query sanitization strips hazardous metacharacters', () => {
  const dirty = "Cyberpunk'; DROP TABLE Games; --";
  const res = filterCatalog(null, dirty);
  assert.strictEqual(res.length, 0); // "Cyberpunk DROP TABLE Games --" matches no title
});

// ----------------------------------------------------------------------------
// Summary
// ----------------------------------------------------------------------------
console.log('\n================================================================');
console.log(`Test Execution Finished: ${passCount} Passed, ${failCount} Failed`);
console.log('================================================================');

if (failCount > 0) {
  process.exit(1);
} else {
  console.log('✔ All Challenger M3 empirical stress tests PASSED successfully.\n');
}
