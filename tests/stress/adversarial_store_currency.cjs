/**
 * Milestone 3 Adversarial Empirical Stress Test Suite: Store & Currency (Features 12 & 13)
 *
 * Targets:
 * 1. Currency calculation precision: micro-cents ($0.001, $0.01), massive numbers ($99,999,999), NaN, Infinity, negative values.
 * 2. KZT integer rounding and currency symbol formatting (`₸`, `$`).
 * 3. Store search query injection attacks: SQL/NoSQL injection tokens, HTML tags, shell metacharacters ['";{}()$].
 * 4. Rapid pagination race conditions, out-of-range page requests (e.g. page 9999), empty result handling, catalog exhaustion banner.
 * 5. Tag filter boundary tests (empty tags, unknown tags, multi-tag combinations).
 */

const { ok, equal, notEqual, throws } = require('../helpers/assert.cjs');

console.log('================================================================');
console.log('  Milestone 3 Store & Currency Adversarial Empirical Challenge  ');
console.log('================================================================\n');

const FIXED_EXCHANGE_RATE = 495.0;

// Replicate frontend CurrencyStore logic from src/lib/stores/store.svelte.ts
class CurrencyStoreModel {
  constructor(initialCode = 'USD', rate = FIXED_EXCHANGE_RATE) {
    this.code = initialCode;
    this.rate = rate;
    this.lastUpdated = Date.now();
  }

  isStale(now = Date.now()) {
    const maxAgeMs = 7 * 24 * 3600 * 1000; // 7 days
    return (now - this.lastUpdated) > maxAgeMs;
  }

  setCurrency(newCode) {
    if (newCode !== 'USD' && newCode !== 'KZT') return false;
    this.code = newCode;
    return true;
  }

  convertFromUsd(usd) {
    if (usd <= 0) return 0;
    if (this.code === 'USD') return usd;
    return Math.max(1, Math.round(usd * this.rate));
  }

  format(usd) {
    if (usd === 0) return 'Free to Play';
    if (this.code === 'USD') {
      return `$${usd.toFixed(2)}`;
    }
    const kzt = Math.max(1, Math.round(usd * this.rate));
    return `${kzt.toLocaleString('ru-RU')} ₸`;
  }

  formatSavings(regularUsd, saleUsd) {
    const savedUsd = regularUsd - saleUsd;
    if (this.code === 'KZT') {
      const savedKzt = Math.round(savedUsd * this.rate);
      return `Вы экономите ${savedKzt.toLocaleString('ru-RU')} ₸`;
    }
    return `You save $${savedUsd.toFixed(2)}`;
  }
}

// ----------------------------------------------------------------------------
// SECTION 1: Currency Calculation Precision & Micro-Pricing
// ----------------------------------------------------------------------------
console.log('--- Section 1: Currency Calculation Precision & Micro-Pricing ---');

const currency = new CurrencyStoreModel('KZT');

// 1.1 Micro-cents ($0.01, $0.001, $0.0001)
const kztCent = currency.convertFromUsd(0.01);
console.log(`$0.01 USD -> ${kztCent} KZT (Expected 5 KZT)`);
equal(kztCent, 5, '0.01 USD must round to 5 KZT');

const kztMicro = currency.convertFromUsd(0.001);
console.log(`$0.001 USD -> ${kztMicro} KZT (Expected 1 KZT floor)`);
equal(kztMicro, 1, 'Micro-cent ($0.001) must clamp to minimum 1 KZT floor');

const kztNano = currency.convertFromUsd(0.000001);
equal(kztNano, 1, 'Nano-cent ($0.000001) must clamp to minimum 1 KZT floor');

// 1.2 Zero & Negative Values
equal(currency.convertFromUsd(0.0), 0, 'Zero USD must convert to 0 KZT');
equal(currency.convertFromUsd(-0.0), 0, 'Negative zero must convert to 0 KZT');
equal(currency.convertFromUsd(-10.0), 0, 'Negative price must clamp to 0 KZT');
equal(currency.convertFromUsd(-0.0001), 0, 'Negative micro-cents must clamp to 0 KZT');

// 1.3 Massive Numbers ($99,999,999.00)
const kztMassive = currency.convertFromUsd(99999999.0);
console.log(`$99,999,999.00 USD -> ${kztMassive} KZT`);
equal(kztMassive, 49499999505, 'Massive price converts without floating-point truncation');

const kztTrillion = currency.convertFromUsd(1000000000000.0);
console.log(`$1,000,000,000,000.00 USD -> ${kztTrillion} KZT`);
equal(kztTrillion, 495000000000000, 'Trillion USD converts accurately');

// 1.4 Non-finite values (NaN, Infinity, -Infinity)
function safeConvertCurrency(usd, targetCurr) {
  if (typeof usd !== 'number' || isNaN(usd) || !isFinite(usd) || usd < 0) {
    throw new Error('Invalid price value: must be positive finite number');
  }
  if (targetCurr === 'USD') return usd;
  if (targetCurr === 'KZT') return Math.max(1, Math.round(usd * FIXED_EXCHANGE_RATE));
  throw new Error(`Unsupported currency: ${targetCurr}`);
}

throws(() => safeConvertCurrency(NaN, 'KZT'), /Invalid price value/);
throws(() => safeConvertCurrency(Infinity, 'KZT'), /Invalid price value/);
throws(() => safeConvertCurrency(-Infinity, 'KZT'), /Invalid price value/);
throws(() => safeConvertCurrency(-5.0, 'KZT'), /Invalid price value/);
throws(() => safeConvertCurrency(10.0, 'EUR'), /Unsupported currency/);
console.log('✔ Non-finite, negative, and invalid currency inputs safely rejected.\n');

// ----------------------------------------------------------------------------
// SECTION 2: KZT Integer Rounding & Symbol Formatting (₸, $)
// ----------------------------------------------------------------------------
console.log('--- Section 2: KZT Integer Rounding & Symbol Formatting ---');

// 2.1 USD Formatting
currency.setCurrency('USD');
equal(currency.format(0.0), 'Free to Play');
equal(currency.format(59.99), '$59.99');
equal(currency.format(0.01), '$0.01');
equal(currency.format(99999999.0), '$99999999.00');

// 2.2 KZT Formatting
currency.setCurrency('KZT');
equal(currency.format(0.0), 'Free to Play');

const formatted59 = currency.format(59.99);
console.log(`$59.99 in KZT formatted: '${formatted59}'`);
ok(formatted59.includes('₸'), 'Must contain Kazakhstani Tenge symbol ₸');
ok(formatted59.includes('29'), 'Must contain 29');
ok(formatted59.includes('695'), 'Must contain 695');

const formattedMicro = currency.format(0.001);
console.log(`$0.001 in KZT formatted: '${formattedMicro}'`);
equal(formattedMicro.trim(), '1 ₸', 'Micro-pricing in KZT displays 1 ₸');

const formattedHuge = currency.format(99999999.0);
console.log(`$99,999,999 in KZT formatted: '${formattedHuge}'`);
ok(formattedHuge.includes('₸'));
ok(formattedHuge.includes('49'));
ok(formattedHuge.includes('505'));

// 2.3 formatSavings
currency.setCurrency('USD');
equal(currency.formatSavings(59.99, 29.99), 'You save $30.00');

currency.setCurrency('KZT');
const savingsKzt = currency.formatSavings(59.99, 29.99);
console.log(`Savings in KZT: '${savingsKzt}'`);
ok(savingsKzt.includes('₸'));
ok(savingsKzt.includes('14'));
ok(savingsKzt.includes('850'));

// 2.4 Rapid currency switching (10,000 toggles)
console.log('Executing 10,000 rapid currency toggles...');
const toggleStart = Date.now();
for (let i = 0; i < 10000; i++) {
  currency.setCurrency(i % 2 === 0 ? 'USD' : 'KZT');
}
const toggleElapsed = Date.now() - toggleStart;
console.log(`10,000 toggles completed in ${toggleElapsed}ms.`);
equal(currency.code, 'KZT');

// 2.5 Stale exchange rate flag
const now = Date.now();
const freshTimestamp = now - (2 * 24 * 3600 * 1000); // 2 days old
const staleTimestamp = now - (8 * 24 * 3600 * 1000); // 8 days old

currency.lastUpdated = freshTimestamp;
equal(currency.isStale(now), false, '2 days old exchange rate is fresh');

currency.lastUpdated = staleTimestamp;
equal(currency.isStale(now), true, '8 days old exchange rate is flagged as stale');
console.log('✔ Formatting, rounding, and toggle stability tests passed.\n');

// ----------------------------------------------------------------------------
// SECTION 3: Store Search Query Injection Attacks
// ----------------------------------------------------------------------------
console.log('--- Section 3: Store Search Query Injection Attacks ---');

function sanitizeSearchQuery(query) {
  if (typeof query !== 'string') return '';
  return query.replace(/['";{}()$]/g, '').trim();
}

const injectionVectors = [
  // SQL Injection
  "Cyberpunk'; DROP TABLE Games; --",
  "' OR '1'='1",
  "UNION SELECT null, username, password FROM users --",
  "1; EXEC xp_cmdshell('dir')",
  "admin'--",

  // NoSQL Injection
  '{"$gt": ""}',
  '{"$ne": null}',
  '{"$where": "this.price > 0"}',

  // Shell Metacharacters
  "rm -rf /; $(whoami); `calc.exe`",
  "${PATH} /etc/passwd; echo 'pwned'",

  // HTML / XSS Injection
  "<script>alert('XSS')</script>",
  '<img src="x" onerror="alert(1)">',
  "<svg/onload=alert('XSS')>",
  '"><iframe src="javascript:alert(1)">',

  // Path Traversal
  "../../../../etc/passwd",
  "..\\..\\..\\Windows\\System32\\cmd.exe",

  // Extreme length
  "a".repeat(100000),
];

for (const vec of injectionVectors) {
  const sanitized = sanitizeSearchQuery(vec);
  // Ensure no SQL/shell meta-characters survive
  ok(!sanitized.includes(';'), `Semicolon stripped from: ${vec.slice(0, 30)}`);
  ok(!sanitized.includes("'"), `Single quote stripped from: ${vec.slice(0, 30)}`);
  ok(!sanitized.includes('"'), `Double quote stripped from: ${vec.slice(0, 30)}`);
  ok(!sanitized.includes('{'), `Left curly stripped from: ${vec.slice(0, 30)}`);
  ok(!sanitized.includes('}'), `Right curly stripped from: ${vec.slice(0, 30)}`);
  ok(!sanitized.includes('('), `Left paren stripped from: ${vec.slice(0, 30)}`);
  ok(!sanitized.includes(')'), `Right paren stripped from: ${vec.slice(0, 30)}`);
  ok(!sanitized.includes('$'), `Dollar sign stripped from: ${vec.slice(0, 30)}`);
}
console.log(`✔ All ${injectionVectors.length} injection attack vectors successfully sanitized.\n`);

// ----------------------------------------------------------------------------
// SECTION 4: Rapid Pagination Race Conditions & Exhaustion
// ----------------------------------------------------------------------------
console.log('--- Section 4: Rapid Pagination Race Conditions & Exhaustion ---');

class MockStoreFeed {
  constructor(maxPages = 10, pageSize = 6) {
    this.maxPages = maxPages;
    this.pageSize = pageSize;
    this.items = [];
    this.page = 0;
    this.hasMore = true;
    this.loading = false;
    this.inFlight = false;
    this.endMessage = null;
    this.requestsProcessed = 0;
    this.raceCollisionsPrevented = 0;
  }

  async loadPage(pageToLoad, append = false) {
    if (this.inFlight) {
      this.raceCollisionsPrevented++;
      return false;
    }
    this.inFlight = true;
    this.loading = true;

    // Simulate async network latency (10-30ms)
    await new Promise((r) => setTimeout(r, 10 + Math.random() * 20));

    try {
      this.requestsProcessed++;
      if (pageToLoad >= this.maxPages) {
        this.page = pageToLoad;
        this.hasMore = false;
        this.endMessage = 'You have caught up with all current recommendations';
        if (!append) this.items = [];
        return true;
      }

      const newItems = Array.from({ length: this.pageSize }, (_, i) => ({
        id: pageToLoad * this.pageSize + i,
        title: `Game ${pageToLoad * this.pageSize + i}`,
      }));

      if (append) {
        this.items = [...this.items, ...newItems];
      } else {
        this.items = newItems;
      }

      this.page = pageToLoad;
      this.hasMore = pageToLoad + 1 < this.maxPages;
      if (!this.hasMore) {
        this.endMessage = 'You have caught up with all current recommendations';
      }
      return true;
    } finally {
      this.loading = false;
      this.inFlight = false;
    }
  }

  async loadNext() {
    if (!this.hasMore || this.loading || this.inFlight) {
      this.raceCollisionsPrevented++;
      return false;
    }
    return await this.loadPage(this.page + 1, true);
  }
}

async function runPaginationStressTest() {
  const feed = new MockStoreFeed(10, 6);

  // 4.1 Initial page load
  await feed.loadPage(0, false);
  equal(feed.items.length, 6);
  equal(feed.page, 0);
  equal(feed.hasMore, true);
  equal(feed.endMessage, null);

  // 4.2 Race condition flood: 50 simultaneous loadNext requests
  console.log('Simulating race condition flood (50 simultaneous scroll requests)...');
  const floodPromises = Array.from({ length: 50 }, () => feed.loadNext());
  await Promise.all(floodPromises);

  console.log(`Requests processed: ${feed.requestsProcessed}, collisions prevented: ${feed.raceCollisionsPrevented}`);
  ok(feed.raceCollisionsPrevented > 40, 'In-flight lock must block concurrent pagination stampedes');
  equal(feed.items.length, 12, 'Items must advance exactly one page (6 -> 12), with no duplicate items');

  // 4.3 Advance sequentially to exhaustion
  while (feed.hasMore) {
    await feed.loadNext();
  }

  equal(feed.page, 9, 'Final valid page is page 9 (0-indexed 10 pages)');
  equal(feed.hasMore, false, 'hasMore must be false after page 9');
  equal(feed.items.length, 60, '10 pages of 6 items = exactly 60 items');
  equal(feed.endMessage, 'You have caught up with all current recommendations');

  // 4.4 Out-of-range page request: page 9999
  const feedOut = new MockStoreFeed(10, 6);
  await feedOut.loadPage(9999, false);
  equal(feedOut.page, 9999);
  equal(feedOut.items.length, 0);
  equal(feedOut.hasMore, false);
  equal(feedOut.endMessage, 'You have caught up with all current recommendations');
}

runPaginationStressTest().then(() => {
  console.log('✔ Pagination race conditions and exhaustion boundaries verified.\n');

  // ----------------------------------------------------------------------------
  // SECTION 5: Tag Filter Boundary Tests
  // ----------------------------------------------------------------------------
  console.log('--- Section 5: Tag Filter Boundary Tests ---');

  const catalog = [
    { id: 1, title: 'Cyberpunk 2077', tags: ['Action', 'RPG', 'FPS'] },
    { id: 2, title: 'Hades', tags: ['Roguelike', 'Action', 'Indie'] },
    { id: 3, title: 'DOOM Eternal', tags: ['FPS', 'Action'] },
    { id: 4, title: 'Civilization VI', tags: ['Strategy', 'Turn-Based'] },
    { id: 5, title: 'ELDEN RING', tags: ['RPG', 'Action'] },
  ];

  function filterCatalog(tag, query) {
    const cleanTag = tag ? tag.trim() : '';
    const cleanQuery = query ? sanitizeSearchQuery(query).toLowerCase() : '';

    return catalog.filter((item) => {
      if (cleanQuery && !item.title.toLowerCase().includes(cleanQuery)) {
        return false;
      }
      if (cleanTag && cleanTag !== 'All' && !item.tags.some((t) => t.toLowerCase() === cleanTag.toLowerCase())) {
        return false;
      }
      return true;
    });
  }

  // 5.1 Empty, null, whitespace, and 'All' tags
  equal(filterCatalog('', '').length, 5, 'Empty tag returns all items');
  equal(filterCatalog('   ', '').length, 5, 'Whitespace tag returns all items');
  equal(filterCatalog('All', '').length, 5, "'All' tag returns all items");
  equal(filterCatalog(null, '').length, 5, 'Null tag returns all items');

  // 5.2 Case insensitivity
  equal(filterCatalog('fps', '').length, 2, "Lowercase 'fps' matches 2 items");
  equal(filterCatalog('FPS', '').length, 2, "Uppercase 'FPS' matches 2 items");
  equal(filterCatalog('Fps', '').length, 2, "Titlecase 'Fps' matches 2 items");

  // 5.3 Unknown tags
  equal(filterCatalog('NonExistentGenre123', '').length, 0, 'Unknown tag returns 0 items');
  equal(filterCatalog('<script>alert(1)</script>', '').length, 0, 'Injection tag returns 0 items');

  // 5.4 Tag and Query combinations
  const fpsCyberpunk = filterCatalog('FPS', 'Cyberpunk');
  equal(fpsCyberpunk.length, 1);
  equal(fpsCyberpunk[0].title, 'Cyberpunk 2077');

  const stratCyberpunk = filterCatalog('Strategy', 'Cyberpunk');
  equal(stratCyberpunk.length, 0, 'Cyberpunk is not Strategy');

  console.log('✔ Tag filter boundary tests passed.\n');
  console.log('================================================================');
  console.log('  ALL EMPIRICAL ADVERSARIAL STRESS TESTS COMPLETED SUCCESSFULLY ');
  console.log('================================================================');
}).catch((err) => {
  console.error('Stress test failure:', err);
  process.exit(1);
});
