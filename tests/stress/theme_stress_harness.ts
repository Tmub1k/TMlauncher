/**
 * OmniLauncher Milestone 1 — Adversarial Theme Engine Stress Harness
 * Validates Modular Theme System under extreme load, corrupted inputs, and edge conditions.
 */

import { ThemeEngine, BUILTIN_THEMES } from '../../src/lib/themes/theme-engine.ts';
import type { ThemeDefinition } from '../../src/lib/themes/types.ts';

// Test execution results
interface TestResult {
  name: string;
  category: string;
  status: 'PASS' | 'FAIL' | 'BUG_DETECTED';
  durationMs: number;
  details: string;
}

const results: TestResult[] = [];

function record(name: string, category: string, status: 'PASS' | 'FAIL' | 'BUG_DETECTED', durationMs: number, details: string) {
  results.push({ name, category, status, durationMs, details });
  const icon = status === 'PASS' ? '✔' : status === 'BUG_DETECTED' ? '⚠ BUG' : '✖ FAIL';
  console.log(`  ${icon} [${category}] ${name} (${durationMs.toFixed(1)}ms): ${details}`);
}

// Mock DOM Environment
class MockStyleDeclaration {
  private props = new Map<string, string>();

  setProperty(prop: string, val: string) {
    this.props.set(prop, String(val));
  }

  getPropertyValue(prop: string): string {
    return this.props.get(prop) || '';
  }

  removeProperty(prop: string): string {
    const val = this.props.get(prop) || '';
    this.props.delete(prop);
    return val;
  }

  get length(): number {
    return this.props.size;
  }

  entries(): [string, string][] {
    return Array.from(this.props.entries());
  }

  clear() {
    this.props.clear();
  }
}

class MockElement {
  style = new MockStyleDeclaration();
  dataset: Record<string, string> = {};
}

// Global mocks setup
const mockDocElement = new MockElement();
(globalThis as any).document = {
  documentElement: mockDocElement,
};

const mockLocalStorage = new Map<string, string>();
(globalThis as any).localStorage = {
  getItem: (key: string) => mockLocalStorage.get(key) ?? null,
  setItem: (key: string, val: string) => mockLocalStorage.set(key, String(val)),
  removeItem: (key: string) => mockLocalStorage.delete(key),
  clear: () => mockLocalStorage.clear(),
};

(globalThis as any).window = {
  localStorage: (globalThis as any).localStorage,
};

// Mock Audio
class MockAudio {
  src: string;
  loop = false;
  volume = 1;
  isPlaying = false;

  constructor(src: string) {
    this.src = src;
  }

  play(): Promise<void> {
    this.isPlaying = true;
    return Promise.resolve();
  }

  pause(): void {
    this.isPlaying = false;
  }
}
(globalThis as any).Audio = MockAudio;

async function runThemeStressSuite() {
  console.log('\n======================================================');
  console.log('   OmniLauncher M1 Theme Engine Empirical Stress Test');
  console.log('======================================================\n');

  // --- Suite 1: Built-in Themes Schema Completeness ---
  {
    const start = performance.now();
    const requiredCanonicalTokens = [
      '--bg-canvas',
      '--bg-surface',
      '--bg-surface-base',
      '--bg-surface-elevated',
      '--bg-surface-floating',
      '--border-hairline',
      '--border-subtle',
      '--text-primary',
      '--text-secondary',
      '--text-muted',
      '--accent-chrome',
      '--accent-glow',
    ];

    let allComplete = true;
    const missingReports: string[] = [];

    for (const [id, theme] of Object.entries(BUILTIN_THEMES)) {
      const tokens = theme.tokens || {};
      const missing = requiredCanonicalTokens.filter((tok) => !(tok in tokens));
      if (missing.length > 0) {
        allComplete = false;
        missingReports.push(`${id} missing: [${missing.join(', ')}]`);
      }
    }

    const duration = performance.now() - start;
    if (allComplete) {
      record('All built-in themes have canonical tokens', 'Schema', 'PASS', duration, 'Verified 4 built-ins: tura-graphite, liquid-chrome, obsidian-pure, cyber-aurora');
    } else {
      record('Built-in themes canonical tokens', 'Schema', 'FAIL', duration, missingReports.join('; '));
    }
  }

  // --- Suite 2: CSS Custom Properties Injection into DOM ---
  {
    const start = performance.now();
    const engine = new ThemeEngine();
    mockDocElement.style.clear();
    mockDocElement.dataset = {};

    engine.applyTheme('liquid-chrome');

    const themeIdApplied = mockDocElement.dataset.theme === 'liquid-chrome';
    const bgCanvas = mockDocElement.style.getPropertyValue('--bg-canvas');
    const accentChrome = mockDocElement.style.getPropertyValue('--accent-chrome');
    const surface1 = mockDocElement.style.getPropertyValue('--bg-surface-1');
    const surface2 = mockDocElement.style.getPropertyValue('--bg-surface-2');
    const surface3 = mockDocElement.style.getPropertyValue('--bg-surface-3');

    const duration = performance.now() - start;
    if (themeIdApplied && bgCanvas && accentChrome && surface1 && surface2 && surface3) {
      record('CSS variables and backward-compat aliases applied to DOM', 'DOM Injection', 'PASS', duration, `Variables applied: ${mockDocElement.style.length}, dataset.theme = 'liquid-chrome', surface aliases verified`);
    } else {
      record('CSS variables applied to DOM', 'DOM Injection', 'FAIL', duration, `themeId=${themeIdApplied}, bgCanvas=${bgCanvas}, surface1=${surface1}`);
    }
  }

  // --- Suite 3: High-Frequency Rapid Theme Switching (10,000 iterations) ---
  {
    const start = performance.now();
    const engine = new ThemeEngine();
    const iterations = 10000;
    const themeKeys = ['tura-graphite', 'liquid-chrome', 'obsidian-pure', 'cyber-aurora'];

    for (let i = 0; i < iterations; i++) {
      const targetTheme = themeKeys[i % themeKeys.length];
      engine.applyTheme(targetTheme);
    }

    const duration = performance.now() - start;
    const finalActive = engine.getActiveTheme().id;
    const expectedFinal = themeKeys[(iterations - 1) % themeKeys.length];
    const domMatches = mockDocElement.dataset.theme === expectedFinal;

    if (finalActive === expectedFinal && domMatches) {
      record(`Rapid theme switching (${iterations} sync iterations)`, 'Stress Load', 'PASS', duration, `Completed in ${duration.toFixed(2)}ms (${((duration / iterations) * 1000).toFixed(2)}µs/switch). Final theme: ${finalActive}`);
    } else {
      record('Rapid theme switching', 'Stress Load', 'FAIL', duration, `Mismatch: active=${finalActive}, expected=${expectedFinal}, dom=${mockDocElement.dataset.theme}`);
    }
  }

  // --- Suite 4: Concurrent Asynchronous Theme Switching ---
  {
    const start = performance.now();
    const engine = new ThemeEngine();
    const asyncOps = 500;
    const themeKeys = ['tura-graphite', 'liquid-chrome', 'obsidian-pure', 'cyber-aurora'];

    const promises = Array.from({ length: asyncOps }, (_, i) => {
      return new Promise<void>((resolve) => {
        const delay = Math.floor(Math.random() * 5);
        setTimeout(() => {
          engine.applyTheme(themeKeys[i % themeKeys.length]);
          resolve();
        }, delay);
      });
    });

    await Promise.all(promises);
    const duration = performance.now() - start;
    const active = engine.getActiveTheme();
    const domTheme = mockDocElement.dataset.theme;

    if (active.id === domTheme && themeKeys.includes(active.id)) {
      record(`Concurrent async theme switching (${asyncOps} parallel tasks)`, 'Stress Concurrency', 'PASS', duration, `State consistent. Active: ${active.id}, DOM: ${domTheme}`);
    } else {
      record('Concurrent async theme switching', 'Stress Concurrency', 'FAIL', duration, `Desync: active=${active.id}, DOM=${domTheme}`);
    }
  }

  // --- Suite 5: Fallback Behavior for Non-Existent Theme ID ---
  {
    const start = performance.now();
    const engine = new ThemeEngine();
    engine.applyTheme('non_existent_fake_theme_id_404');
    const duration = performance.now() - start;

    const fallbackActive = engine.getActiveTheme().id;
    if (fallbackActive === 'tura-graphite') {
      record('Fallback on unknown theme ID', 'Robustness', 'PASS', duration, 'Safely defaulted to tura-graphite without throwing');
    } else {
      record('Fallback on unknown theme ID', 'Robustness', 'FAIL', duration, `Unexpected active: ${fallbackActive}`);
    }
  }

  // --- Suite 6: Adversarial Null/Malformed Theme Object Handling ---
  {
    const start = performance.now();
    const engine = new ThemeEngine();

    let nullCrashed = false;
    let nullErrorMsg = '';
    try {
      engine.applyTheme(null as any);
    } catch (err: any) {
      nullCrashed = true;
      nullErrorMsg = err?.message || String(err);
    }

    const duration = performance.now() - start;
    if (nullCrashed) {
      // Calling applyTheme(null) crashes!
      record('Adversarial applyTheme(null) crash vulnerability', 'Vulnerability', 'BUG_DETECTED', duration, `Crashed with: "${nullErrorMsg}". applyTheme lacks null guard on themeOrId parameter.`);
    } else {
      record('Adversarial applyTheme(null) gracefully handled', 'Vulnerability', 'PASS', duration, 'Handled without crashing');
    }
  }

  // --- Suite 7: Missing Color Tokens and Token Leakage Across Themes ---
  {
    const start = performance.now();
    const engine = new ThemeEngine();
    mockDocElement.style.clear();

    // 1. Apply full theme A that has custom extra token
    const themeA: ThemeDefinition = {
      id: 'theme-a-neon',
      name: 'Theme A Neon',
      author: 'Tester',
      version: '1.0.0',
      tokens: {
        '--bg-canvas': '#111111',
        '--accent-glow': '#ff00ff',
        '--custom-token-special': 'calc(100vw - 20px)',
      },
    };
    engine.applyTheme(themeA);

    // 2. Now apply partial theme B that only specifies --bg-canvas
    const themeB: ThemeDefinition = {
      id: 'theme-b-minimal',
      name: 'Theme B Minimal',
      author: 'Tester',
      version: '1.0.0',
      tokens: {
        '--bg-canvas': '#222222',
      },
    };
    engine.applyTheme(themeB);

    const bgCanvas = mockDocElement.style.getPropertyValue('--bg-canvas');
    const leakedAccentGlow = mockDocElement.style.getPropertyValue('--accent-glow');
    const leakedCustomToken = mockDocElement.style.getPropertyValue('--custom-token-special');

    const duration = performance.now() - start;

    if (bgCanvas === '#222222' && (leakedAccentGlow === '#ff00ff' || leakedCustomToken !== '')) {
      // Old tokens are NOT cleaned up or reset on DOM root!
      record('Token pollution / leakage across theme switches', 'Vulnerability', 'BUG_DETECTED', duration, `Tokens from Theme A persisted into Theme B: --accent-glow='${leakedAccentGlow}', --custom-token-special='${leakedCustomToken}'. Missing base tokens not populated.`);
    } else {
      record('Token pollution / leakage across theme switches', 'Edge Cases', 'PASS', duration, 'Tokens properly isolated and reset');
    }
  }

  // --- Suite 8: Subscriber Resilience Under Exception In Callback ---
  {
    const start = performance.now();
    const engine = new ThemeEngine();

    // Test 8a: subscribe() initial invocation exception
    let subscribeThrew = false;
    let subscribeErrorMsg = '';
    try {
      engine.subscribe(() => {
        throw new Error('Initial throw in subscriber');
      });
    } catch (err: any) {
      subscribeThrew = true;
      subscribeErrorMsg = err?.message || String(err);
    }

    // Test 8b: notify() loop resilience
    let sub1Called = false;
    let sub3Called = false;
    let throwingActive = false;

    const unsub1 = engine.subscribe(() => {
      sub1Called = true;
    });

    const unsub2 = engine.subscribe(() => {
      if (throwingActive) {
        throw new Error('Exploding subscriber callback in notify');
      }
    });

    const unsub3 = engine.subscribe(() => {
      sub3Called = true;
    });

    sub1Called = false;
    sub3Called = false;
    throwingActive = true;

    // Trigger theme change
    engine.applyTheme('cyber-aurora');
    const duration = performance.now() - start;

    unsub1();
    unsub2();
    unsub3();

    if (subscribeThrew) {
      record('subscribe() lacks try/catch for initial subscriber callback', 'Vulnerability', 'BUG_DETECTED', duration, `subscribe() threw "${subscribeErrorMsg}" instead of shielding caller, yet kept broken subscriber registered.`);
    } else {
      record('subscribe() initial invocation safe', 'Robustness', 'PASS', duration, 'Safely handled');
    }

    if (sub1Called && sub3Called) {
      record('Subscriber notification resilience to callback exceptions in notify()', 'Fault Tolerance', 'PASS', duration, 'Error in subscriber 2 did not prevent subscriber 1 & 3 from executing');
    } else {
      record('Subscriber notification resilience', 'Fault Tolerance', 'FAIL', duration, `sub1=${sub1Called}, sub3=${sub3Called}`);
    }
  }

  // --- Suite 9: Ambient Media Volume Clamping and Teardown ---
  {
    const start = performance.now();
    const engine = new ThemeEngine();

    const mediaThemeHigh: ThemeDefinition = {
      id: 'media-high',
      name: 'Media High',
      author: 'Tester',
      version: '1.0.0',
      tokens: { '--bg-canvas': '#000' },
      media: {
        ambientAudio: 'http://test/ambient.mp3',
        audioVolume: 99.9, // Out of bounds
      },
    };

    engine.applyTheme(mediaThemeHigh);
    const audioObj = (engine as any).ambientAudio as MockAudio;
    const highVolumeClamped = audioObj && audioObj.volume === 1.0;

    // Switch to theme without audio: should tear down
    engine.applyTheme('tura-graphite');
    const audioTornDown = (engine as any).ambientAudio === null;

    const duration = performance.now() - start;
    if (highVolumeClamped && audioTornDown) {
      record('Ambient media volume clamping and audio teardown', 'Media Engine', 'PASS', duration, 'Volume 99.9 clamped to 1.0, audio instance cleared on theme switch');
    } else {
      record('Ambient media volume clamping', 'Media Engine', 'FAIL', duration, `highClamped=${highVolumeClamped}, tornDown=${audioTornDown}`);
    }
  }

  // --- Summary ---
  console.log('\n------------------------------------------------------');
  const passes = results.filter((r) => r.status === 'PASS').length;
  const bugs = results.filter((r) => r.status === 'BUG_DETECTED').length;
  const fails = results.filter((r) => r.status === 'FAIL').length;
  console.log(`Total Scenarios: ${results.length} | Passed: ${passes} | Bugs/Adversarial Findings: ${bugs} | Failed: ${fails}`);
  console.log('------------------------------------------------------\n');
}

runThemeStressSuite().catch((err) => {
  console.error('Stress test harness unhandled failure:', err);
  process.exit(1);
});
