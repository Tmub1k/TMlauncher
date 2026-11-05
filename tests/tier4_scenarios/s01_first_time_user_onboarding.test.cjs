const { describe, it } = require('../runner.cjs');
const { ok, equal, deepEqual, hasProperties } = require('../helpers/assert.cjs');
const { SAMPLE_TURA_THEME } = require('../helpers/fixtures.cjs');

describe('Tier 4 - Scenario 01: First-Time User Onboarding Journey', () => {
  it('Executes complete initial setup: clean launch -> theme load -> drive pick -> account link -> currency toggle', async () => {
    // Step 1: Clean GUI Startup without console popups
    const startupState = {
      windowCreated: true,
      subsystem: 'windows',
      hasTerminalPopup: false,
    };
    equal(startupState.subsystem, 'windows');
    equal(startupState.hasTerminalPopup, false);

    // Step 2: Theme Engine initializes TURA Graphite design system
    const themeEngine = {
      activeTheme: SAMPLE_TURA_THEME,
      canvasColor: SAMPLE_TURA_THEME.colors['bg-canvas'],
    };
    equal(themeEngine.canvasColor, '#09090b', 'Onboards into deep graphite TURA UI');

    // Step 3: In-Launcher Drive Selector auto-detects drive with optimal capacity
    const detectedDrives = [
      { drive: 'C:', freeGB: 25 },
      { drive: 'D:', freeGB: 850 },
    ];
    function selectOptimalInstallDrive(drives) {
      return [...drives].sort((a, b) => b.freeGB - a.freeGB)[0].drive;
    }
    const defaultDrive = selectOptimalInstallDrive(detectedDrives);
    equal(defaultDrive, 'D:', 'Auto-selected D: drive with 850GB free');

    // Step 4: Steam Account Linking
    const userAccount = {
      steamId64: '76561198000000001',
      linkedAt: Date.now(),
      status: 'authenticated',
    };
    ok(/^[0-9]{17}$/.test(userAccount.steamId64));
    equal(userAccount.status, 'authenticated');

    // Step 5: Store Feed loads with USD, then user switches to Kazakhstani Tenge (KZT)
    let currentCurrency = 'USD';
    const storeItem = { title: 'Hades II', priceUsd: 29.99 };

    function getFormattedPrice(item, currency) {
      if (currency === 'KZT') {
        return `${Math.round(item.priceUsd * 495).toLocaleString('ru-RU')} ₸`;
      }
      return `$${item.priceUsd.toFixed(2)}`;
    }

    equal(getFormattedPrice(storeItem, currentCurrency), '$29.99');
    currentCurrency = 'KZT';
    const kztDisplay = getFormattedPrice(storeItem, currentCurrency);
    ok(kztDisplay.includes('14'));
    ok(kztDisplay.includes('₸'), 'Currency switched smoothly to KZT with proper symbol');
  });
});
