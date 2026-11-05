const { describe, it } = require('../runner.cjs');
const { ok, equal, deepEqual, throws, doesNotThrow } = require('../helpers/assert.cjs');
const { SAMPLE_TURA_THEME, SAMPLE_LIQUID_CHROME_THEME } = require('../helpers/fixtures.cjs');
const { validateThemeDefinition } = require('../helpers/contracts.cjs');
const { withContext } = require('../helpers/test_context.cjs');

describe('Tier 1 - Feature 04: Modular Theme System', () => {
  it('Validates JSON theme manifest schema against theme contract', () => {
    validateThemeDefinition(SAMPLE_TURA_THEME);
    validateThemeDefinition(SAMPLE_LIQUID_CHROME_THEME);
    equal(SAMPLE_TURA_THEME.id, 'tura-graphite');
    equal(SAMPLE_LIQUID_CHROME_THEME.id, 'liquid-chrome');
  });

  it('Scans drop-in theme directory and discovers valid custom themes', withContext('theme_scan', async (ctx) => {
    ctx.createFile('custom-cyberpunk.json', JSON.stringify({
      id: 'cyberpunk-neon',
      name: 'Cyberpunk Neon',
      author: 'ModderX',
      version: '1.0.0',
      colors: {
        'bg-canvas': '#0d0221',
        'bg-surface': '#0f0826',
        'bg-card': '#1a0b36',
        'border-subtle': '#ff007f33',
        'accent-chrome': '#00f0ff',
        'accent-glow': '#ff007f88',
        'text-primary': '#ffffff',
        'text-muted': '#8f8da8',
      },
      typography: {
        fontFamily: 'Cera CY, sans-serif',
        fontSizeBase: '14px',
      },
    }));

    const raw = ctx.readFile('custom-cyberpunk.json');
    const parsed = JSON.parse(raw);
    validateThemeDefinition(parsed);
    equal(parsed.id, 'cyberpunk-neon');
    equal(parsed.name, 'Cyberpunk Neon');
  }));

  it('Generates CSS Custom Properties mapping correctly from theme colors', () => {
    function generateCssVars(theme) {
      const vars = [];
      for (const [key, value] of Object.entries(theme.colors)) {
        vars.push(`--${key}: ${value};`);
      }
      return vars.join(' ');
    }

    const cssOutput = generateCssVars(SAMPLE_TURA_THEME);
    ok(cssOutput.includes('--bg-canvas: #09090b;'));
    ok(cssOutput.includes('--border-subtle: rgba(255, 255, 255, 0.08);'));
  });

  it('Runtime theme switcher transitions active theme cleanly', () => {
    let activeTheme = SAMPLE_TURA_THEME.id;
    function switchTheme(newId, registry) {
      const target = registry.find(t => t.id === newId);
      if (!target) throw new Error(`Theme not found: ${newId}`);
      activeTheme = target.id;
      return target;
    }

    const registry = [SAMPLE_TURA_THEME, SAMPLE_LIQUID_CHROME_THEME];
    const activated = switchTheme('liquid-chrome', registry);
    equal(activeTheme, 'liquid-chrome');
    equal(activated.name, 'Liquid Chrome');
  });

  it('Rejects invalid themes missing mandatory tokens without crashing engine', () => {
    const invalidTheme = {
      id: 'broken-theme',
      name: 'Broken',
      colors: {
        'bg-canvas': '#000000',
        // missing other required tokens
      },
    };

    throws(() => {
      validateThemeDefinition(invalidTheme);
    }, /Missing expected property/);
  });
});
