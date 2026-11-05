#!/usr/bin/env node
/**
 * OmniLauncher Automated E2E Test Runner
 * Executable via: node tests/runner.cjs [options]
 */

const fs = require('node:fs');
const path = require('node:path');
const process = require('node:process');

// ANSI Color constants
const RESET = '\x1b[0m';
const BOLD = '\x1b[1m';
const RED = '\x1b[31m';
const GREEN = '\x1b[32m';
const YELLOW = '\x1b[33m';
const BLUE = '\x1b[34m';
const CYAN = '\x1b[36m';
const GRAY = '\x1b[90m';

// Global Test Registry
const suites = [];
let currentSuite = null;

function describe(title, fn) {
  const suite = {
    title,
    tests: [],
    beforeEachHooks: [],
    afterEachHooks: [],
    beforeAllHooks: [],
    afterAllHooks: [],
  };
  suites.push(suite);
  const prevSuite = currentSuite;
  currentSuite = suite;
  try {
    fn();
  } finally {
    currentSuite = prevSuite;
  }
}

function test(title, fn) {
  if (!currentSuite) {
    describe('Default Suite', () => {
      currentSuite.tests.push({ title, fn });
    });
  } else {
    currentSuite.tests.push({ title, fn });
  }
}

const it = test;

function beforeEach(fn) {
  if (currentSuite) currentSuite.beforeEachHooks.push(fn);
}

function afterEach(fn) {
  if (currentSuite) currentSuite.afterEachHooks.push(fn);
}

function beforeAll(fn) {
  if (currentSuite) currentSuite.beforeAllHooks.push(fn);
}

function afterAll(fn) {
  if (currentSuite) currentSuite.afterAllHooks.push(fn);
}

// Export test DSL to global scope and module exports for test files
global.describe = describe;
global.test = test;
global.it = it;
global.beforeEach = beforeEach;
global.afterEach = afterEach;
global.beforeAll = beforeAll;
global.afterAll = afterAll;

module.exports = {
  describe,
  test,
  it,
  beforeEach,
  afterEach,
  beforeAll,
  afterAll,
  suites,
};


// Parse CLI Arguments
const args = process.argv.slice(2);
const options = {
  tier: null,
  feature: null,
  filter: null,
  verbose: false,
  bail: false,
  json: false,
};

for (const arg of args) {
  if (arg.startsWith('--tier=')) {
    options.tier = arg.split('=')[1].trim();
  } else if (arg.startsWith('--feature=')) {
    options.feature = arg.split('=')[1].trim().toLowerCase();
  } else if (arg.startsWith('--filter=')) {
    options.filter = arg.split('=')[1].trim();
  } else if (arg === '--verbose' || arg === '-v') {
    options.verbose = true;
  } else if (arg === '--bail' || arg === '-b') {
    options.bail = true;
  } else if (arg === '--json') {
    options.json = true;
  }
}

// Main execution function
async function run() {
  const startTime = Date.now();
  const testsDir = __dirname;

  const tierDirs = {
    '1': path.join(testsDir, 'tier1_features'),
    '2': path.join(testsDir, 'tier2_boundaries'),
    '3': path.join(testsDir, 'tier3_interactions'),
    '4': path.join(testsDir, 'tier4_scenarios'),
  };

  const selectedTiers = options.tier
    ? [options.tier]
    : ['1', '2', '3', '4'];

  const testFilesToRun = [];

  for (const tierKey of selectedTiers) {
    const dir = tierDirs[tierKey];
    if (dir && fs.existsSync(dir)) {
      const files = fs.readdirSync(dir)
        .filter(f => f.endsWith('.test.cjs'))
        .sort();

      for (const file of files) {
        if (options.feature && !file.toLowerCase().includes(options.feature)) {
          continue;
        }
        testFilesToRun.push({
          tier: tierKey,
          filename: file,
          fullPath: path.join(dir, file),
        });
      }
    }
  }

  if (testFilesToRun.length === 0) {
    console.log(`${YELLOW}No test files found matching criteria.${RESET}`);
    process.exit(0);
  }

  if (!options.json) {
    console.log(`\n${BOLD}${CYAN}======================================================${RESET}`);
    console.log(`${BOLD}${CYAN}   OmniLauncher Automated E2E Test Runner            ${RESET}`);
    console.log(`${BOLD}${CYAN}======================================================${RESET}`);
    console.log(`${GRAY}Target Tiers:${RESET} ${selectedTiers.join(', ')}`);
    console.log(`${GRAY}Files Found:${RESET} ${testFilesToRun.length}`);
    if (options.feature) console.log(`${GRAY}Feature Filter:${RESET} ${options.feature}`);
    console.log('');
  }

  const results = {
    total: 0,
    passed: 0,
    failed: 0,
    skipped: 0,
    byTier: { '1': { total: 0, passed: 0, failed: 0 },
              '2': { total: 0, passed: 0, failed: 0 },
              '3': { total: 0, passed: 0, failed: 0 },
              '4': { total: 0, passed: 0, failed: 0 } },
    failures: [],
  };

  for (const item of testFilesToRun) {
    const fileSuitesBefore = suites.length;

    try {
      require(item.fullPath);
    } catch (loadErr) {
      results.failed++;
      results.failures.push({
        file: item.filename,
        title: `Failed to load module: ${item.filename}`,
        error: loadErr,
      });
      if (!options.json) {
        console.log(`  ${RED}✖ [LOAD ERROR] ${item.filename}: ${loadErr.message}${RESET}`);
      }
      if (options.bail) break;
      continue;
    }

    const newSuites = suites.slice(fileSuitesBefore);

    for (const suite of newSuites) {
      if (!options.json && options.verbose) {
        console.log(`\n${BOLD}${BLUE}▸ Suite: ${suite.title}${RESET} ${GRAY}(${item.filename})${RESET}`);
      }

      // Execute beforeAll hooks
      for (const hook of suite.beforeAllHooks) {
        await hook();
      }

      for (const t of suite.tests) {
        if (options.filter && !t.title.includes(options.filter)) {
          results.skipped++;
          continue;
        }

        results.total++;
        if (results.byTier[item.tier]) {
          results.byTier[item.tier].total++;
        }

        // Execute beforeEach hooks
        for (const hook of suite.beforeEachHooks) {
          await hook();
        }

        const tStart = Date.now();
        let pass = false;
        let testError = null;

        try {
          await t.fn();
          pass = true;
          results.passed++;
          if (results.byTier[item.tier]) {
            results.byTier[item.tier].passed++;
          }
        } catch (err) {
          results.failed++;
          if (results.byTier[item.tier]) {
            results.byTier[item.tier].failed++;
          }
          testError = err;
          results.failures.push({
            file: item.filename,
            suite: suite.title,
            title: t.title,
            error: err,
          });
        }

        // Execute afterEach hooks
        for (const hook of suite.afterEachHooks) {
          try {
            await hook();
          } catch (hookErr) {
            console.error('afterEach hook error:', hookErr);
          }
        }

        const duration = Date.now() - tStart;

        if (!options.json) {
          if (pass) {
            if (options.verbose) {
              console.log(`    ${GREEN}✔${RESET} ${t.title} ${GRAY}(${duration}ms)${RESET}`);
            } else {
              process.stdout.write(`${GREEN}.${RESET}`);
            }
          } else {
            if (options.verbose) {
              console.log(`    ${RED}✖${RESET} ${t.title} ${GRAY}(${duration}ms)${RESET}`);
              console.log(`      ${RED}${testError.message}${RESET}`);
            } else {
              process.stdout.write(`${RED}F${RESET}`);
            }
          }
        }

        if (!pass && options.bail) {
          break;
        }
      }

      // Execute afterAll hooks
      for (const hook of suite.afterAllHooks) {
        try {
          await hook();
        } catch (hookErr) {
          console.error('afterAll hook error:', hookErr);
        }
      }

      if (results.failed > 0 && options.bail) {
        break;
      }
    }

    if (results.failed > 0 && options.bail) {
      break;
    }
  }

  const elapsed = ((Date.now() - startTime) / 1000).toFixed(2);

  if (options.json) {
    console.log(JSON.stringify({ ...results, elapsedSec: Number(elapsed) }, null, 2));
  } else {
    console.log('\n');
    console.log(`${BOLD}${CYAN}------------------------------------------------------${RESET}`);
    console.log(`${BOLD}Test Execution Summary:${RESET}`);
    console.log(`  ${BOLD}Total Tests:${RESET}   ${results.total}`);
    console.log(`  ${GREEN}${BOLD}Passed:${RESET}        ${results.passed}`);
    console.log(`  ${RED}${BOLD}Failed:${RESET}        ${results.failed}`);
    if (results.skipped > 0) {
      console.log(`  ${YELLOW}${BOLD}Skipped:${RESET}       ${results.skipped}`);
    }
    console.log(`  ${GRAY}Time Elapsed:${RESET}  ${elapsed}s\n`);

    console.log(`${BOLD}Tier Breakdown:${RESET}`);
    console.log(`  Tier 1 (Feature Coverage):     ${results.byTier['1'].passed} / ${results.byTier['1'].total} passed`);
    console.log(`  Tier 2 (Boundary & Corner):    ${results.byTier['2'].passed} / ${results.byTier['2'].total} passed`);
    console.log(`  Tier 3 (Interactions):         ${results.byTier['3'].passed} / ${results.byTier['3'].total} passed`);
    console.log(`  Tier 4 (Scenarios):            ${results.byTier['4'].passed} / ${results.byTier['4'].total} passed`);
    console.log(`${BOLD}${CYAN}------------------------------------------------------${RESET}`);

    if (results.failures.length > 0) {
      console.log(`\n${RED}${BOLD}FAILURES (${results.failures.length}):${RESET}\n`);
      results.failures.forEach((f, idx) => {
        console.log(`${idx + 1}) [${f.file}] ${f.suite ? f.suite + ' > ' : ''}${f.title}`);
        console.log(`${RED}${f.error.stack || f.error.message}${RESET}\n`);
      });
    } else {
      console.log(`\n${GREEN}${BOLD}✔ All test suites executed successfully!${RESET}\n`);
    }
  }

  process.exit(results.failed === 0 ? 0 : 1);
}

if (require.main === module) {
  run().catch(err => {
    console.error('Test runner fatal error:', err);
    process.exit(1);
  });
}

module.exports = {
  describe,
  test,
  it,
  beforeEach,
  afterEach,
  beforeAll,
  afterAll,
  run,
};
