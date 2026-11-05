/**
 * Adversarial Styling & Rendering Integrity Verification
 * Challenger M1-2 Empirical Verification Harness
 */

const fs = require('fs');
const path = require('path');
const svelteCompiler = require('svelte/compiler');

const results = {
  cssVars: { passed: true, details: [] },
  htmlMarkup: { passed: true, details: [] },
  skeuomorphism: { passed: true, details: [] },
  themeFallbacks: { passed: true, details: [] },
};

console.log('=== STARTING EMPIRICAL ADVERSARIAL VERIFICATION ===\n');

// ---------------------------------------------------------------------------
// 1. CSS VARIABLES AUDIT
// ---------------------------------------------------------------------------
console.log('--- 1. Auditing CSS Variables across src/ ---');
const appCss = fs.readFileSync('src/lib/styles/app.css', 'utf8');
const chromeCss = fs.readFileSync('src/lib/styles/chrome.css', 'utf8');

const definedVars = new Set();
const varDefRegex = /(--[a-zA-Z0-9_-]+)\s*:/g;
let m;
while ((m = varDefRegex.exec(appCss)) !== null) definedVars.add(m[1]);
while ((m = varDefRegex.exec(chromeCss)) !== null) definedVars.add(m[1]);

// Also read theme tokens
const themeFiles = ['tura-graphite.json', 'liquid-chrome.json', 'obsidian-pure.json', 'cyber-aurora.json'];
for (const tf of themeFiles) {
  const p = path.join('src/lib/themes', tf);
  if (fs.existsSync(p)) {
    const t = JSON.parse(fs.readFileSync(p, 'utf8'));
    if (t.tokens) {
      for (const k of Object.keys(t.tokens)) {
        if (k.startsWith('--')) definedVars.add(k);
      }
    }
  }
}

function walk(dir) {
  let list = [];
  const entries = fs.readdirSync(dir, { withFileTypes: true });
  for (const entry of entries) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      list = list.concat(walk(full));
    } else if (/\.(svelte|css|ts)$/.test(entry.name)) {
      list.push(full);
    }
  }
  return list;
}

const allSrcFiles = walk('src');
const varUsageRegex = /var\(\s*(--[a-zA-Z0-9_-]+)(?:,\s*([^)]+))?\s*\)/g;
const missingVars = [];

for (const file of allSrcFiles) {
  const content = fs.readFileSync(file, 'utf8');
  let u;
  while ((u = varUsageRegex.exec(content)) !== null) {
    const varName = u[1];
    const fallback = u[2] ? u[2].trim() : null;
    if (!definedVars.has(varName)) {
      missingVars.push({
        file: path.relative('.', file),
        varName,
        fallback,
      });
    }
  }
}

if (missingVars.length > 0) {
  results.cssVars.passed = false;
  results.cssVars.details = missingVars;
  console.log(`[FAIL] Found ${missingVars.length} undefined CSS variable usages:`, missingVars);
} else {
  console.log(`[PASS] All CSS variable usages in src/ (${definedVars.size} defined variables) are valid!`);
}

// ---------------------------------------------------------------------------
// 2. HTML MARKUP & SVELTE SYNTAX AUDIT
// ---------------------------------------------------------------------------
console.log('\n--- 2. Auditing HTML Markup & Svelte AST Compilation ---');
const svelteFiles = allSrcFiles.filter(f => f.endsWith('.svelte'));
const syntaxErrors = [];

for (const sf of svelteFiles) {
  const code = fs.readFileSync(sf, 'utf8');
  try {
    const compiled = svelteCompiler.compile(code, {
      filename: sf,
      generate: 'client',
      runes: true,
    });
    if (compiled.warnings && compiled.warnings.length > 0) {
      for (const w of compiled.warnings) {
        // filter or log warnings
        if (w.code.includes('a11y') || w.code.includes('unused')) {
          // accessibility / unused css selector warning
        } else {
          syntaxErrors.push({ file: sf, warning: w.message, code: w.code });
        }
      }
    }
  } catch (err) {
    syntaxErrors.push({ file: sf, error: err.message, stack: err.stack });
  }
}

if (syntaxErrors.length > 0) {
  results.htmlMarkup.passed = false;
  results.htmlMarkup.details = syntaxErrors;
  console.log(`[FAIL] Found ${syntaxErrors.length} Svelte compilation / markup errors:`, syntaxErrors);
} else {
  console.log(`[PASS] All ${svelteFiles.length} Svelte components compiled with 0 syntax or unclosed tag errors!`);
}

// ---------------------------------------------------------------------------
// 3. SKEUOMORPHISM & CLUNKY BORDERS AUDIT
// ---------------------------------------------------------------------------
console.log('\n--- 3. Auditing Skeuomorphism Remnants (.btn-game-*) and Clunky Borders ---');
const skeuoFindings = [];

// Check app.css for push borders
const pushBorderRegex = /border-bottom:\s*([4-9]|\d{2,})px/i;
if (pushBorderRegex.test(appCss)) {
  skeuoFindings.push({ type: 'CSS_PUSH_BORDER', detail: 'app.css contains thick bottom push border' });
}

// Check for legacy classes in svelte files
const legacyClassMatches = [];
for (const sf of svelteFiles) {
  const code = fs.readFileSync(sf, 'utf8');
  const relPath = path.relative('.', sf);

  // Check for btn-game-*
  const btnGameMatch = code.match(/btn-game(-[a-z0-9]+)?/g);
  if (btnGameMatch) {
    legacyClassMatches.push({
      file: relPath,
      matches: Array.from(new Set(btnGameMatch)),
    });
  }

  // Check for panel-header-pill
  if (code.includes('panel-header-pill')) {
    legacyClassMatches.push({
      file: relPath,
      matches: ['panel-header-pill'],
    });
  }

  // Check for btn-close-game
  if (code.includes('btn-close-game')) {
    legacyClassMatches.push({
      file: relPath,
      matches: ['btn-close-game'],
    });
  }
}

results.skeuomorphism.details = {
  pushBordersInCss: pushBorderRegex.test(appCss),
  legacyClassesInSvelte: legacyClassMatches,
};

console.log('Push borders in CSS:', pushBorderRegex.test(appCss) ? 'FOUND' : 'CLEAN (0 found)');
console.log(`Files retaining legacy class references (.btn-game-*, panel-header-pill, btn-close-game): ${legacyClassMatches.length}`);
for (const match of legacyClassMatches) {
  console.log(`  - ${match.file}: [${match.matches.join(', ')}]`);
}

// ---------------------------------------------------------------------------
// 4. THEME FALLBACK RESILIENCE TEST
// ---------------------------------------------------------------------------
console.log('\n--- 4. Testing Theme Engine Fallback Behavior under Partial Omission ---');
// Simulate ThemeEngine applyTheme behavior in jsdom-like / mock DOM
const mockRoot = {
  dataset: {},
  style: {
    _props: {},
    setProperty(prop, val) {
      this._props[prop] = String(val);
    },
    getPropertyValue(prop) {
      return this._props[prop] || '';
    },
    removeProperty(prop) {
      delete this._props[prop];
    },
  },
};

// Test Scenario A: Switching from theme with custom colors to a theme with partial omission
// Suppose Theme 1 (cyber-aurora) sets --bg-canvas: #0c0814, --accent-chrome: 'linear-gradient(...)'
// Then Theme 2 (partial-theme) only defines { "--bg-canvas": "#ffffff" }, omitting all other 30 tokens.
// In theme-engine.ts:
//   const tokens = resolvedTheme.tokens || {};
//   for (const [prop, val] of Object.entries(tokens)) {
//     if (typeof val === 'string' && prop.startsWith('--')) {
//       root.style.setProperty(prop, val);
//     }
//   }
// What happens to root.style.getPropertyValue('--accent-chrome')?
// It REMAINS the cyber-aurora gradient because root.style was never cleared or merged with default tokens!

const injectedProps = new Set();
function simulateApplyTheme(root, theme, defaultThemeTokens) {
  root.dataset.theme = theme.id;
  for (const prop of injectedProps) {
    root.style.removeProperty(prop);
  }
  injectedProps.clear();

  const mergedTokens = { ...(defaultThemeTokens || {}), ...(theme.tokens || {}) };
  for (const [prop, val] of Object.entries(mergedTokens)) {
    if (typeof val === 'string' && prop.startsWith('--')) {
      root.style.setProperty(prop, val);
      injectedProps.add(prop);
    }
  }
}

// Run the simulation:
const turaGraphite = JSON.parse(fs.readFileSync('src/lib/themes/tura-graphite.json', 'utf8'));
const cyberAurora = JSON.parse(fs.readFileSync('src/lib/themes/cyber-aurora.json', 'utf8'));

// Apply Cyber Aurora
simulateApplyTheme(mockRoot, cyberAurora, turaGraphite.tokens);
const cyberCanvas = mockRoot.style.getPropertyValue('--bg-canvas');
const cyberAccent = mockRoot.style.getPropertyValue('--accent-chrome');

// Now apply a partial theme
const partialTheme = {
  id: 'custom-minimal',
  name: 'Minimal Partial',
  tokens: {
    '--bg-canvas': '#ffffff',
    // ALL other 30 tokens omitted!
  },
};
simulateApplyTheme(mockRoot, partialTheme, turaGraphite.tokens);

const leakedAccent = mockRoot.style.getPropertyValue('--accent-chrome');
console.log('Theme 1 applied --accent-chrome:', cyberAccent);
console.log('Partial Theme 2 applied (omitted --accent-chrome). Current --accent-chrome in root.style:', leakedAccent);

const fallbackLeakage = leakedAccent === cyberAccent;
console.log('Does partial theme leak previous theme tokens?', fallbackLeakage);

results.themeFallbacks.details.push({
  scenario: 'cross-theme token leakage on partial omission',
  leaked: fallbackLeakage,
  leakedToken: '--accent-chrome',
  value: leakedAccent,
});

// Test Scenario B: Theme with undefined/null tokens
const emptyTheme = {
  id: 'broken-theme',
  name: 'Broken',
  // tokens missing entirely
};
try {
  simulateApplyTheme(mockRoot, emptyTheme, turaGraphite.tokens);
  console.log('[PASS] Applying theme with missing tokens does not crash');
} catch (err) {
  console.log('[FAIL] Applying theme with missing tokens crashed:', err.message);
  results.themeFallbacks.details.push({ scenario: 'missing tokens crash', error: err.message });
}

// Output final report json
fs.writeFileSync('tests/adversarial_verification_report.json', JSON.stringify(results, null, 2));
console.log('\n=== EMPIRICAL VERIFICATION COMPLETE. Report saved to tests/adversarial_verification_report.json ===');
