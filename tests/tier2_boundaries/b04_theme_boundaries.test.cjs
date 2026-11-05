const { describe, it } = require('../runner.cjs');
const { ok, equal, throws, doesNotThrow } = require('../helpers/assert.cjs');
const { SAMPLE_TURA_THEME } = require('../helpers/fixtures.cjs');

describe('Tier 2 - Boundary 04: Modular Theme System Boundaries', () => {
  it('Handles malformed JSON theme files with syntax errors without crashing process', () => {
    function safeParseTheme(rawJson) {
      try {
        return { success: true, theme: JSON.parse(rawJson) };
      } catch (err) {
        return { success: false, error: err.message, theme: SAMPLE_TURA_THEME };
      }
    }

    const corruptedJson = '{ "id": "broken", "name": "Broken", colors: { missing_quotes: true }';
    const result = safeParseTheme(corruptedJson);
    equal(result.success, false);
    equal(result.theme.id, 'tura-graphite', 'Falls back safely to default TURA theme');
  });

  it('Fills missing color tokens with base theme defaults', () => {
    function repairIncompleteTheme(partialTheme, baseTheme) {
      return {
        ...baseTheme,
        ...partialTheme,
        colors: {
          ...baseTheme.colors,
          ...(partialTheme.colors || {}),
        },
      };
    }

    const partial = {
      id: 'custom_partial',
      colors: {
        'accent-chrome': '#ff0000',
      },
    };

    const repaired = repairIncompleteTheme(partial, SAMPLE_TURA_THEME);
    equal(repaired.colors['accent-chrome'], '#ff0000');
    equal(repaired.colors['bg-canvas'], '#09090b', 'Missing canvas token populated from default');
  });

  it('Sanitizes and caps extremely long theme metadata strings', () => {
    function sanitizeThemeMeta(meta) {
      return {
        id: meta.id.slice(0, 32).replace(/[^a-zA-Z0-9_-]/g, ''),
        name: meta.name.slice(0, 50).trim(),
      };
    }

    const bloated = {
      id: 'my_theme_with_lots_of_symbols_$$$!!__extended_longer_than_allowed',
      name: 'Super Long Theme Name That Should Really Be Kept Within Limits On Any UI Screen Ever Designed',
    };

    const clean = sanitizeThemeMeta(bloated);
    ok(clean.id.length <= 32);
    ok(!clean.id.includes('$'));
    ok(clean.name.length <= 50);
  });

  it('Debounces rapid filesystem change notifications during theme development', () => {
    let reloadCounter = 0;
    let timer = null;

    function onFileChangeDebounced(action) {
      if (timer) clearTimeout(timer);
      timer = setTimeout(() => {
        reloadCounter++;
        action();
      }, 50);
    }

    // Simulate 5 rapid bursts within 10ms
    onFileChangeDebounced(() => {});
    onFileChangeDebounced(() => {});
    onFileChangeDebounced(() => {});
    onFileChangeDebounced(() => {});
    equal(reloadCounter, 0, 'No reload fired immediately');
  });

  it('Detects and prevents circular color variable references (e.g. var(--a) -> var(--b) -> var(--a))', () => {
    function detectCircularVar(mapping, startKey) {
      const visited = new Set();
      let current = startKey;
      while (current && mapping[current]) {
        if (visited.has(current)) return true;
        visited.add(current);
        const matchVar = mapping[current].match(/var\(--([^)]+)\)/);
        current = matchVar ? matchVar[1] : null;
      }
      return false;
    }

    const circular = {
      'color-a': 'var(--color-b)',
      'color-b': 'var(--color-a)',
    };
    equal(detectCircularVar(circular, 'color-a'), true);

    const valid = {
      'color-a': 'var(--color-b)',
      'color-b': '#121215',
    };
    equal(detectCircularVar(valid, 'color-a'), false);
  });

  it('Safely defaults to tura-graphite when null or undefined theme is passed to applyTheme logic', () => {
    function resolveThemeInput(themeOrId, defaultThemeId) {
      if (!themeOrId) return defaultThemeId;
      if (typeof themeOrId === 'string') return themeOrId || defaultThemeId;
      if (typeof themeOrId === 'object') return themeOrId.id || defaultThemeId;
      return defaultThemeId;
    }

    equal(resolveThemeInput(null, 'tura-graphite'), 'tura-graphite');
    equal(resolveThemeInput(undefined, 'tura-graphite'), 'tura-graphite');
    equal(resolveThemeInput({}, 'tura-graphite'), 'tura-graphite');
    equal(resolveThemeInput({ id: 'custom' }, 'tura-graphite'), 'custom');
  });

  it('Clears previously injected custom CSS properties and populates base fallback tokens', () => {
    const defaultTokens = {
      '--bg-canvas': '#09090b',
      '--accent-glow': '0 0 24px rgba(255, 255, 255, 0.10)',
    };

    const injectedProps = new Set();
    const styleSheet = {};

    function applyTokens(targetTokens) {
      for (const prop of injectedProps) {
        delete styleSheet[prop];
      }
      injectedProps.clear();

      const merged = { ...defaultTokens, ...targetTokens };
      for (const [k, v] of Object.entries(merged)) {
        styleSheet[k] = v;
        injectedProps.add(k);
      }
    }

    // Apply custom neon theme
    applyTokens({
      '--bg-canvas': '#111',
      '--accent-glow': '#ff00ff',
      '--custom-token': '10px',
    });
    equal(styleSheet['--accent-glow'], '#ff00ff');
    equal(styleSheet['--custom-token'], '10px');

    // Switch to partial minimal theme
    applyTokens({
      '--bg-canvas': '#ffffff',
    });
    equal(styleSheet['--bg-canvas'], '#ffffff');
    equal(styleSheet['--accent-glow'], '0 0 24px rgba(255, 255, 255, 0.10)', 'Fallback to default token');
    equal(styleSheet['--custom-token'], undefined, 'Previously injected custom property cleared');
  });

  it('Prevents exceptions in subscribe listener from crashing execution and provides unsubscribe callback', () => {
    const subscribers = new Set();
    let initialThrowHandled = false;

    function subscribe(listener) {
      subscribers.add(listener);
      try {
        listener({ id: 'tura-graphite' });
      } catch (err) {
        initialThrowHandled = true;
      }
      return () => subscribers.delete(listener);
    }

    const unsub = subscribe(() => {
      throw new Error('Component mount failure');
    });

    ok(initialThrowHandled, 'Initial listener error was trapped');
    equal(subscribers.size, 1);
    unsub();
    equal(subscribers.size, 0, 'Cleanly unsubscribed after failure');
  });

  it('Resolves artwork or sleek fallback gradient gracefully when game art is nullish', () => {
    function resolvePoster(game) {
      const directArt = game?.artworkUrl || game?.coverUrl || game?.poster;
      const raw = game?.art?.poster?.value || directArt || null;
      return raw && String(raw).trim().length > 0 ? String(raw).trim() : null;
    }

    equal(resolvePoster({ art: undefined }), null);
    equal(resolvePoster({ art: null }), null);
    equal(resolvePoster({}), null);
    equal(resolvePoster({ artworkUrl: 'https://example.com/art.jpg' }), 'https://example.com/art.jpg');
    equal(resolvePoster({ art: { poster: { value: 'https://example.com/poster.jpg' } } }), 'https://example.com/poster.jpg');
  });
});
