/**
 * Adversarial Stress Harness for ThemeEngine
 */

const fs = require('fs');

// We can read theme-engine.ts or simulate the exact implementation logic from theme-engine.ts
const turaGraphite = JSON.parse(fs.readFileSync('src/lib/themes/tura-graphite.json', 'utf8'));
const liquidChrome = JSON.parse(fs.readFileSync('src/lib/themes/liquid-chrome.json', 'utf8'));
const obsidianPure = JSON.parse(fs.readFileSync('src/lib/themes/obsidian-pure.json', 'utf8'));
const cyberAurora = JSON.parse(fs.readFileSync('src/lib/themes/cyber-aurora.json', 'utf8'));

const BUILTIN_THEMES = {
  'tura-graphite': turaGraphite,
  'liquid-chrome': liquidChrome,
  'obsidian-pure': obsidianPure,
  'cyber-aurora': cyberAurora,
};

function createMockEnvironment() {
  const domStyles = {};
  const root = {
    dataset: {},
    style: {
      setProperty(k, v) { domStyles[k] = String(v); },
      getPropertyValue(k) { return domStyles[k] || ''; },
      removeProperty(k) { delete domStyles[k]; },
      get _all() { return { ...domStyles }; }
    }
  };
  const storage = {};
  const mockLocalStorage = {
    getItem(k) { return storage[k] ?? null; },
    setItem(k, v) { storage[k] = String(v); },
    removeItem(k) { delete storage[k]; }
  };
  return { root, mockLocalStorage, domStyles };
}

function deepMerge(base, override) {
  const result = { ...base };
  if (!override || typeof override !== 'object') return result;
  for (const [key, val] of Object.entries(override)) {
    if (val !== undefined && val !== null) {
      if (
        typeof val === 'object' &&
        !Array.isArray(val) &&
        typeof result[key] === 'object' &&
        !Array.isArray(result[key])
      ) {
        result[key] = deepMerge(result[key], val);
      } else {
        result[key] = val;
      }
    }
  }
  return result;
}

// Emulate ThemeEngine class logic exactly as written in src/lib/themes/theme-engine.ts
class TestThemeEngine {
  constructor(env) {
    this.env = env;
    this.activeTheme = BUILTIN_THEMES['tura-graphite'];
    this.availableThemes = new Map();
    this.injectedCustomProperties = new Set();
    for (const [id, theme] of Object.entries(BUILTIN_THEMES)) {
      this.availableThemes.set(id, theme);
    }
  }

  applyTheme(themeOrId) {
    let resolvedTheme;
    if (!themeOrId) {
      resolvedTheme = BUILTIN_THEMES['tura-graphite'];
    } else if (typeof themeOrId === 'string') {
      resolvedTheme = this.availableThemes.get(themeOrId) || BUILTIN_THEMES[themeOrId];
      if (!resolvedTheme) {
        resolvedTheme = BUILTIN_THEMES['tura-graphite'];
      }
    } else if (typeof themeOrId === 'object') {
      if (!themeOrId.id) {
        resolvedTheme = BUILTIN_THEMES['tura-graphite'];
      } else {
        resolvedTheme = themeOrId;
        this.availableThemes.set(resolvedTheme.id, resolvedTheme);
      }
    } else {
      resolvedTheme = BUILTIN_THEMES['tura-graphite'];
    }

    const defaultTokens = BUILTIN_THEMES['tura-graphite'].tokens || {};
    const targetTokens = resolvedTheme.tokens || {};
    const mergedTokens = deepMerge(defaultTokens, targetTokens);

    resolvedTheme = {
      ...resolvedTheme,
      tokens: mergedTokens,
    };

    this.activeTheme = resolvedTheme;

    const root = this.env.root;
    root.dataset.theme = resolvedTheme.id;

    for (const prop of this.injectedCustomProperties) {
      root.style.removeProperty(prop);
    }
    this.injectedCustomProperties.clear();

    for (const [prop, val] of Object.entries(mergedTokens)) {
      if (typeof val === 'string' && prop.startsWith('--')) {
        root.style.setProperty(prop, val);
        this.injectedCustomProperties.add(prop);
      }
    }

    const tokenMap = mergedTokens;
    if (!tokenMap['--bg-surface-1'] && tokenMap['--bg-surface-base']) {
      root.style.setProperty('--bg-surface-1', tokenMap['--bg-surface-base']);
      this.injectedCustomProperties.add('--bg-surface-1');
    }
    if (!tokenMap['--bg-surface-2'] && tokenMap['--bg-surface-elevated']) {
      root.style.setProperty('--bg-surface-2', tokenMap['--bg-surface-elevated']);
      this.injectedCustomProperties.add('--bg-surface-2');
    }
    if (!tokenMap['--bg-surface-3'] && tokenMap['--bg-surface-floating']) {
      root.style.setProperty('--bg-surface-3', tokenMap['--bg-surface-floating']);
      this.injectedCustomProperties.add('--bg-surface-3');
    }
  }
}

console.log('=== THEME ENGINE ADVERSARIAL STRESS SUITE ===\n');

// Test 1: Null/undefined input
console.log('Test 1: applyTheme(null) and applyTheme(undefined)');
const env1 = createMockEnvironment();
const engine1 = new TestThemeEngine(env1);
try {
  engine1.applyTheme(null);
  console.log('[PASS] applyTheme(null) survived');
} catch (err) {
  console.log('[FAIL] applyTheme(null) threw error:', err.message);
}
try {
  engine1.applyTheme(undefined);
  console.log('[PASS] applyTheme(undefined) survived');
} catch (err) {
  console.log('[FAIL] applyTheme(undefined) threw error:', err.message);
}

// Test 2: Theme object without tokens
console.log('\nTest 2: Theme without tokens property');
const env2 = createMockEnvironment();
const engine2 = new TestThemeEngine(env2);
try {
  engine2.applyTheme({ id: 'empty-theme', name: 'No Tokens' });
  console.log('[PASS] Theme without tokens did not crash, dataset.theme =', env2.root.dataset.theme);
} catch (err) {
  console.log('[FAIL] Theme without tokens crashed:', err.message);
}

// Test 3: Token contamination / state leakage between themes
console.log('\nTest 3: Token Contamination / State Leakage');
const env3 = createMockEnvironment();
const engine3 = new TestThemeEngine(env3);

// 1. Switch to cyber-aurora (neon purple theme)
engine3.applyTheme('cyber-aurora');
const auroraGlow = env3.root.style.getPropertyValue('--accent-glow');
const auroraCanvas = env3.root.style.getPropertyValue('--bg-canvas');
console.log('cyber-aurora --accent-glow:', auroraGlow);
console.log('cyber-aurora --bg-canvas:', auroraCanvas);

// 2. Switch to a partial custom theme that ONLY overrides --bg-canvas to #ffffff
engine3.applyTheme({
  id: 'custom-paper',
  name: 'Paper',
  tokens: {
    '--bg-canvas': '#ffffff'
  }
});

const paperCanvas = env3.root.style.getPropertyValue('--bg-canvas');
const paperGlow = env3.root.style.getPropertyValue('--accent-glow');
console.log('custom-paper --bg-canvas (overridden):', paperCanvas);
console.log('custom-paper --accent-glow (omitted):', paperGlow);

if (paperGlow === auroraGlow) {
  console.log('[VULNERABILITY CONFIRMED] State leakage: Omitted tokens retain previous theme inline styles instead of falling back to default theme or clearing!');
} else {
  console.log('[PASS] Omitted tokens fell back cleanly');
}

// Test 4: Token type safety (number, boolean, null in tokens)
console.log('\nTest 4: Non-string values in tokens');
const env4 = createMockEnvironment();
const engine4 = new TestThemeEngine(env4);
try {
  engine4.applyTheme({
    id: 'corrupt-tokens',
    name: 'Corrupt',
    tokens: {
      '--bg-canvas': 12345,
      '--text-primary': null,
      '--accent': true
    }
  });
  console.log('[PASS] Non-string values in tokens did not crash');
} catch (err) {
  console.log('[FAIL] Non-string values crashed:', err.message);
}
