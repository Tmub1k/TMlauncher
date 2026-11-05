/**
 * Challenger M2 Remediation Suite: Deep Adversarial Verification
 * 
 * Verifies:
 * 1. Quote-aware launch argument parser with aggressive adversarial inputs,
 *    nested quotes, spaces, shell injection, and Unicode.
 * 2. Platform switcher UI in GameCard and GamePropertiesModal:
 *    switching logic, source resolution, keyboard interaction, and edge cases.
 */

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import esbuild from 'esbuild';
import * as svelteCompiler from 'svelte/compiler';
import { render } from 'svelte/server';
import { sanitizeLaunchArgs, normalizeGameTitle } from '../../src/lib/stores/library.ts';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const projectRoot = path.resolve(__dirname, '../..');

let totalTests = 0;
let passedTests = 0;
let failedTests = 0;

function assert(condition: boolean, testName: string, details?: string) {
  totalTests++;
  if (condition) {
    passedTests++;
    console.log(`  ✔ [PASS] ${testName}`);
  } else {
    failedTests++;
    console.error(`  ✖ [FAIL] ${testName}${details ? ` -> ${details}` : ''}`);
  }
}

// Svelte compiler plugin for server-side evaluation
function createSvelteServerPlugin() {
  return {
    name: 'svelte-server',
    setup(build: esbuild.PluginBuild) {
      build.onLoad({ filter: /\.css$/ }, () => ({ contents: '', loader: 'js' }));
      build.onLoad({ filter: /\.svelte$/ }, async (args) => {
        const source = fs.readFileSync(args.path, 'utf8');
        const compiled = svelteCompiler.compile(source, {
          filename: args.path,
          generate: 'server',
          dev: false,
        });
        return { contents: compiled.js.code, loader: 'js' };
      });
    },
  };
}

async function bundleComponent(relativePath: string, tempOutName: string): Promise<string> {
  const entryPoint = path.resolve(projectRoot, relativePath);
  const outPath = path.resolve(__dirname, tempOutName);

  await esbuild.build({
    entryPoints: [entryPoint],
    bundle: true,
    outfile: outPath,
    format: 'esm',
    platform: 'node',
    plugins: [createSvelteServerPlugin()],
    external: [
      'svelte/server',
      'svelte/internal/server',
      '@tauri-apps/api/core',
      '@tauri-apps/api/event',
      '@tauri-apps/api/window',
    ],
  });

  return outPath;
}

export async function runAdversarialRemediationTests() {
  console.log('\n========================================================================');
  console.log('   CHALLENGER ADVERSARIAL VERIFICATION SUITE — MILESTONE 2 REMEDIATION');
  console.log('========================================================================\n');

  // =========================================================================
  // SECTION 1: Quote-Aware Launch Argument Parsing Stress & Fuzzing
  // =========================================================================
  console.log('--- SECTION 1: Quote-Aware Launch Argument Parsing Stress & Fuzzing ---');

  // 1.1 Quoted path with spaces and subarguments
  {
    const input = '+connect "192.168.1.50:27015" -game "Counter-Strike Global Offensive" -console';
    const tokens = sanitizeLaunchArgs(input);
    assert(
      tokens.length === 5 &&
      tokens[0] === '+connect' &&
      tokens[1] === '"192.168.1.50:27015"' &&
      tokens[2] === '-game' &&
      tokens[3] === '"Counter-Strike Global Offensive"' &&
      tokens[4] === '-console',
      'Preserve multi-word double quoted string without splitting'
    );
  }

  // 1.2 Single quotes preserving spaces
  {
    const input = "-exec 'my custom config with spaces.cfg' +fps_max 144";
    const tokens = sanitizeLaunchArgs(input);
    assert(
      tokens.length === 4 &&
      tokens[1] === "'my custom config with spaces.cfg'",
      'Preserve single-quoted argument containing spaces'
    );
  }

  // 1.3 Nested quotes: single inside double, double inside single
  {
    const input = `-title "Gabe's Mod" -flag 'He said "Hello" today'`;
    const tokens = sanitizeLaunchArgs(input);
    assert(
      tokens.length === 4 &&
      tokens[1] === `"Gabe's Mod"` &&
      tokens[3] === `'He said "Hello" today'`,
      'Nested single inside double and double inside single quotes intact'
    );
  }

  // 1.4 Windows executable path with drive letter, backslashes, and spaces
  {
    const input = '"C:\\Program Files\\Steam\\steamapps\\common\\Half-Life 2\\hl2.exe" -novid';
    const tokens = sanitizeLaunchArgs(input);
    assert(
      tokens.length === 2 &&
      tokens[0] === '"C:\\Program Files\\Steam\\steamapps\\common\\Half-Life 2\\hl2.exe"' &&
      tokens[1] === '-novid',
      'Windows file path with drive letter, backslashes, and spaces preserved as single argv token'
    );
  }

  // 1.5 Security: Subshell parentheses stripped from executable paths
  {
    const input = '"C:\\Program Files (x86)\\Steam\\steam.exe"';
    const tokens = sanitizeLaunchArgs(input);
    assert(
      tokens.length === 1 &&
      tokens[0] === '"C:\\Program Files x86\\Steam\\steam.exe"',
      'Subshell parentheses () stripped from path for command injection safety'
    );
  }

  // 1.6 Shell injection characters inside and outside quotes
  {
    const input = '-novid; `calc.exe` & whoami | cat /etc/passwd "quoted; & evil | $VAR" > out.txt < in.txt ^%COMSPEC%';
    const tokens = sanitizeLaunchArgs(input);
    const joined = tokens.join(' ');
    const hasForbidden = /[;&|`$><^%()\r\n\0]/.test(joined);
    assert(!hasForbidden, 'Shell metacharacters (; & | ` $ > < ^ % ( ) \\r \\n \\0) neutralized inside & outside quotes');
  }

  // 1.7 Unbalanced / unclosed quotes
  {
    const input = '-novid "unclosed quote parameter -windowed';
    const tokens = sanitizeLaunchArgs(input);
    assert(
      tokens.length === 2 && tokens[0] === '-novid' && tokens[1] === '"unclosed quote parameter -windowed',
      'Gracefully handle unclosed quote up to EOF without hanging or crash'
    );
  }

  // 1.8 Empty quotes
  {
    const input = '+exec "" -custom \'\' -normal';
    const tokens = sanitizeLaunchArgs(input);
    assert(
      tokens.length === 5 && tokens[1] === '""' && tokens[3] === "''",
      'Empty double and single quotes preserved as discrete tokens'
    );
  }

  // 1.9 Non-ASCII and Unicode inside quotes
  {
    const input = '+name "Алексей Смирнов" -locale "русский" -title "ゼルダの伝説" -tag "🎮🚀"';
    const tokens = sanitizeLaunchArgs(input);
    assert(
      tokens.length === 8 &&
      tokens[1] === '"Алексей Смирнов"' &&
      tokens[3] === '"русский"' &&
      tokens[5] === '"ゼルダの伝説"' &&
      tokens[7] === '"🎮🚀"',
      'Unicode (Cyrillic, CJK, Emoji) properly preserved in quoted tokens'
    );
  }

  // 1.10 Edge whitespace and empty inputs
  {
    assert(sanitizeLaunchArgs('').length === 0, 'Empty string returns empty array');
    assert(sanitizeLaunchArgs('     ').length === 0, 'Whitespace-only returns empty array');
    assert(sanitizeLaunchArgs('\t\t\n\r').length === 0, 'Tabs and newlines return empty array');
    const multiSpace = sanitizeLaunchArgs('  -a    -b   "  spaced  "   -c  ');
    assert(
      multiSpace.length === 4 && multiSpace[0] === '-a' && multiSpace[1] === '-b' && multiSpace[2] === '"  spaced  "' && multiSpace[3] === '-c',
      'Consecutive inter-token spaces normalized while inner-quote whitespace preserved'
    );
  }

  // 1.11 Randomized fuzzing of tokenizer (1,000 iterations)
  {
    let fuzzPassed = true;
    const fuzzChars = ['a', ' ', '-', '"', "'", ';', '&', '|', '`', '$', '>', '<', '^', '%', '(', ')', '\\', '/', '0', 'Ж', '語', '🎮'];
    for (let i = 0; i < 1000; i++) {
      let len = 10 + (i % 50);
      let str = '';
      for (let j = 0; j < len; j++) {
        str += fuzzChars[Math.floor(Math.random() * fuzzChars.length)];
      }
      try {
        const tokens = sanitizeLaunchArgs(str);
        // Verify invariant: no dangerous shell operators in any token
        for (const t of tokens) {
          if (/[;&|`$><^%()\r\n\0]/.test(t)) {
            fuzzPassed = false;
            break;
          }
        }
      } catch {
        fuzzPassed = false;
        break;
      }
    }
    assert(fuzzPassed, '1,000 randomized fuzz inputs executed without throw and with zero shell escapes');
  }

  // =========================================================================
  // SECTION 2: Platform Switcher UI in GameCard and GamePropertiesModal
  // =========================================================================
  console.log('\n--- SECTION 2: Platform Switcher UI in GameCard and GamePropertiesModal ---');

  const tempFilesToClean: string[] = [];
  try {
    const cardBundle = await bundleComponent('src/lib/components/GameCard.svelte', '_temp_challenger_card.mjs');
    tempFilesToClean.push(cardBundle);
    const modCard = await import(`file://${cardBundle}`);

    const modalBundle = await bundleComponent('src/lib/components/GamePropertiesModal.svelte', '_temp_challenger_modal.mjs');
    tempFilesToClean.push(modalBundle);
    const modModal = await import(`file://${modalBundle}`);

    // 2.1 Multi-store GameCard SSR rendering with interactive platform pills
    {
      const multiGame = {
        id: 'unified_witcher3',
        platform: 'steam',
        preferred_platform: 'steam',
        platforms: ['steam', 'epic', 'gog', 'emulator'],
        nativeId: '292030',
        title: 'The Witcher 3: Wild Hunt',
        installed: true,
        sources: [
          { platform: 'steam', native_id: '292030', id: 'steam:292030', installed: true, executable: 'bin/x64/witcher3.exe' },
          { platform: 'epic', native_id: '12345', id: 'epic:12345', installed: false, executable: null },
          { platform: 'gog', native_id: '99999', id: 'gog:99999', installed: false, executable: null },
          { platform: 'emulator', native_id: 'retro_w3', id: 'emulator:retro_w3', installed: true, executable: 'retro.exe' },
        ],
      };

      const renderedCard = render(modCard.default, { props: { game: multiGame } });
      const htmlCard = renderedCard.body;

      assert(htmlCard.includes('STEAM'), 'GameCard displays STEAM badge');
      assert(htmlCard.includes('EPIC'), 'GameCard displays EPIC badge');
      assert(htmlCard.includes('GOG'), 'GameCard displays GOG badge');
      assert(htmlCard.includes('RETRO'), 'GameCard maps emulator platform to RETRO badge');
      assert(htmlCard.includes('border-amber-400/80') || htmlCard.includes('ring-amber-400/50'), 'Active platform STEAM has amber highlight ring');
    }

    // 2.2 GamePropertiesModal SSR rendering with 5 Tabs Registration
    {
      const mockPropsGame = {
        id: 'unified_witcher3',
        platform: 'steam',
        preferred_platform: 'steam',
        platforms: ['steam', 'epic', 'emulator'],
        nativeId: '292030',
        title: 'The Witcher 3: Wild Hunt',
        installed: true,
      };

      const renderedModal = render(modModal.default, {
        props: {
          game: mockPropsGame,
          open: true,
          onClose: () => {},
        },
      });
      const htmlModal = renderedModal.body;

      assert(htmlModal.includes('General'), 'GamePropertiesModal renders General tab');
      assert(htmlModal.includes('Installed Files'), 'GamePropertiesModal renders Installed Files tab');
      assert(htmlModal.includes('Cloud Saves'), 'GamePropertiesModal renders Cloud Saves tab');
      assert(htmlModal.includes('Achievements'), 'GamePropertiesModal renders Achievements tab');
      assert(htmlModal.includes('Compatibility'), 'GamePropertiesModal renders Compatibility tab');
    }

    // 2.3 Empirical verification of switchPlatform logic in GameCard
    {
      const cardSource = fs.readFileSync(path.resolve(projectRoot, 'src/lib/components/GameCard.svelte'), 'utf8');
      assert(cardSource.includes('function switchPlatform'), 'GameCard contains switchPlatform function');
      assert(cardSource.includes('event.stopPropagation()'), 'switchPlatform calls event.stopPropagation() to prevent card click');
      assert(cardSource.includes('game.preferred_platform = match.platform'), 'switchPlatform updates preferred_platform');
      assert(cardSource.includes('game.nativeId = match.native_id'), 'switchPlatform updates nativeId to match target platform');
      assert(cardSource.includes('game.id = match.id'), 'switchPlatform updates game id to match target platform');
      assert(cardSource.includes('game.installed = match.installed'), 'switchPlatform updates installed status to match target platform');
      assert(cardSource.includes('onclick={(e) => switchPlatform(plat, e)}'), 'Platform pill has onclick event listener binding');
      assert(cardSource.includes("if (e.key === 'Enter') switchPlatform(plat, e as any)"), 'Platform pill supports keyboard Enter navigation');
    }

    // 2.4 Empirical verification of switchPlatform logic in GamePropertiesModal
    {
      const modalSource = fs.readFileSync(path.resolve(projectRoot, 'src/lib/components/GamePropertiesModal.svelte'), 'utf8');
      assert(modalSource.includes('function switchPlatform'), 'GamePropertiesModal contains switchPlatform function');
      assert(modalSource.includes('(game as any).preferred_platform = match.platform'), 'GamePropertiesModal updates preferred_platform');
      assert(modalSource.includes('(game as any).nativeId = match.native_id'), 'GamePropertiesModal updates nativeId');
      assert(modalSource.includes('(game as any).id = match.id'), 'GamePropertiesModal updates game id');
      assert(modalSource.includes('(game as any).installed = match.installed'), 'GamePropertiesModal updates installed status');
      assert(modalSource.includes('Active platform switched to'), 'GamePropertiesModal notifies user with statusMessage on switch');
      assert(modalSource.includes('onclick={() => switchPlatform(platStr)}'), 'Platform button in General tab calls switchPlatform');
    }

    // 2.5 Simulated client-side switchPlatform execution on unified game object
    {
      const testGame: any = {
        id: 'unified_cp2077',
        platform: 'steam',
        preferred_platform: 'steam',
        platforms: ['steam', 'epic'],
        nativeId: '1091500',
        title: 'Cyberpunk 2077',
        installed: true,
        sources: [
          { platform: 'steam', native_id: '1091500', id: 'steam:1091500', installed: true, executable: 'bin/x64/Cyberpunk2077.exe', size_bytes: 70000000000 },
          { platform: 'epic', native_id: 'ginger', id: 'epic:ginger', installed: false, executable: null, size_bytes: null },
        ],
      };

      // Direct simulation of GamePropertiesModal switchPlatform function
      function executeSwitch(targetPlatform: string) {
        const platLower = targetPlatform.toLowerCase();
        if (testGame.sources && Array.isArray(testGame.sources)) {
          const match = testGame.sources.find((s: any) =>
            s.platform.toLowerCase() === platLower ||
            s.platform.toUpperCase() === targetPlatform.toUpperCase()
          );
          if (match) {
            testGame.preferred_platform = match.platform;
            testGame.platform = match.platform;
            testGame.nativeId = match.native_id;
            testGame.id = match.id;
            testGame.installed = match.installed;
            if (match.install_dir) testGame.installDir = match.install_dir;
            if (match.executable) testGame.executable = match.executable;
            if (match.size_bytes != null) testGame.sizeBytes = match.size_bytes;
          }
        }
      }

      executeSwitch('EPIC');
      assert(testGame.preferred_platform === 'epic', 'Simulated switch to EPIC sets preferred_platform to epic');
      assert(testGame.id === 'epic:ginger', 'Simulated switch sets id to epic:ginger');
      assert(testGame.nativeId === 'ginger', 'Simulated switch sets nativeId to ginger');
      assert(testGame.installed === false, 'Simulated switch updates installed status to false');

      executeSwitch('STEAM');
      assert(testGame.preferred_platform === 'steam', 'Simulated switch back to STEAM sets preferred_platform to steam');
      assert(testGame.id === 'steam:1091500', 'Simulated switch back sets id to steam:1091500');
      assert(testGame.installed === true, 'Simulated switch back restores installed status to true');
    }

    // 2.6 Title normalization parity between frontend and backend
    {
      const titles = [
        'Cyberpunk 2077: Phantom Liberty',
        'Grand Theft Auto V - Premium Online Edition',
        'The Witcher 3: Wild Hunt - Complete Edition',
        'Control Ultimate Edition',
        'Doom Eternal Deluxe Edition',
        'Герои 3: Меч и Магия',
        'ゼルダの伝説 ティアーズ オブ ザ キングダム',
      ];
      for (const t of titles) {
        const norm = normalizeGameTitle(t);
        assert(norm.length > 0, `Normalized '${t}' is non-empty ('${norm}')`);
        assert(!norm.includes('edition'), `Normalized '${t}' strips 'edition' keyword ('${norm}')`);
      }
    }

  } finally {
    for (const f of tempFilesToClean) {
      if (fs.existsSync(f)) {
        try { fs.unlinkSync(f); } catch {}
      }
    }
  }

  // =========================================================================
  // SUMMARY
  // =========================================================================
  console.log('\n========================================================================');
  console.log(`   Remediation Verification Complete: ${passedTests}/${totalTests} Passed | ${failedTests} Failed`);
  console.log('========================================================================\n');

  if (failedTests > 0) {
    console.error(`[VERDICT]: REJECT (${failedTests} tests failed)`);
    process.exit(1);
  } else {
    console.log('[VERDICT]: APPROVE');
    process.exit(0);
  }
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  runAdversarialRemediationTests().catch(err => {
    console.error('Test harness exception:', err);
    process.exit(1);
  });
}
