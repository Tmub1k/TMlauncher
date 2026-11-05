# Milestone 2 Adversarial Challenge Report: Frontend UI & Game Properties Modal

**Assigned Agent**: Challenger M2 (Instance 2)  
**Target Component**: Frontend UI, GamePropertiesModal.svelte, GameCard.svelte, BentoHub.svelte, Launch Options & Platform Switching  
**Verdict**: **REJECT**  
**Overall Risk Assessment**: **CRITICAL**

---

## 1. Observation

Direct observations from source code inspection, AST analysis, and empirical stress test execution across all 4 assigned scopes:

### 1.1 Scope 1: GamePropertiesModal.svelte 5-Tab Rapid Switching
- **File**: `src/lib/components/GamePropertiesModal.svelte`
  - Lines 37-39:
    ```svelte
    type TabId = 'general' | 'installed_files' | 'cloud' | 'achievements' | 'compatibility';
    let activeTab = $state<TabId>('general');
    ```
  - Lines 295-315: Render loop defining all 5 tab navigation triggers (`General`, `Installed Files`, `Cloud Saves`, `Achievements`, `Compatibility`).
  - Lines 166-197: `handleVerifyFiles()`:
    ```svelte
    const interval = setInterval(() => {
      if (verifyProgress < 90) {
        verifyProgress += 15;
      }
    }, 200);

    try {
      const res = await verifyGameFiles(gameId);
      clearInterval(interval);
      verifyProgress = 100;
      verifyResult = res;
    } catch (err: any) {
      clearInterval(interval);
      ...
    ```
  - **Empirical Execution**:
    - 1,000 rapid sequential tab switches across all 5 tabs executed in `0.04ms` with 0 reactive state exceptions.
    - SSR compilation rendered all 5 tabs without compiler errors.
    - **Defect Observed**: When `handleVerifyFiles()` is triggered and the user rapidly switches tabs or calls `onClose()`, the `interval` is NOT registered with Svelte component lifecycle (`onDestroy` or `$effect cleanup`). If `verifyGameFiles(gameId)` hangs or takes extended time, `setInterval` leaks in the background.

### 1.2 Scope 2: Launch Options Input Stress (Shell Injection, Spaces, Quotes)
- **File**: `src/lib/stores/library.ts`
  - Lines 386-393:
    ```ts
    export function sanitizeLaunchArgs(rawInput: string): string[] {
      if (!rawInput) return [];
      return rawInput
        .replace(/[;&|`$]/g, '')
        .trim()
        .split(/\s+/)
        .filter(arg => arg.length > 0);
    }
    ```
- **File**: `src/lib/components/GamePropertiesModal.svelte`
  - Lines 149-153:
    ```svelte
    properties.launch_args = properties.launch_args
      .replace(/[;&|`$]/g, '')
      .slice(0, 4096);
    ```
- **File**: `src-tauri/src/commands/platforms.rs`
  - Lines 338-348:
    ```rust
    let dangerous_chars = [';', '&', '|', '`', '$'];
    properties.launch_args = properties
        .launch_args
        .chars()
        .filter(|c| !dangerous_chars.contains(c))
        .collect();

    if properties.launch_args.len() > 4096 {
        properties.launch_args.truncate(4096);
    }
    ```
- **Empirical Execution Results**:
  1. **Quotes & Spaces Shattering**:
     Input: `+connect "192.168.1.100:27015" -game "Half Life 2" -user "John Doe"`
     Output tokens: `['+connect', '"192.168.1.100:27015"', '-game', '"Half', 'Life', '2"', '-user', '"John', 'Doe"']`
     **Result**: Quoted paths and multi-word parameters with spaces are shattered into invalid separate tokens. Game executables receiving this argv fail or crash.
  2. **Shell Injection Bypass**:
     Input: `-novid > C:\Windows\System32\payload.dll < in.txt ^& whoami %COMSPEC% (calc.exe) -opt\0bad`
     Result: Metacharacters `>`, `<`, `^`, `%`, `(`, `)`, and `\0` are completely preserved and passed through to launch arguments.
  3. **Rust String::truncate(4096) Char Boundary Panic**:
     Input: 4095 ASCII bytes + 2-byte Cyrillic character `'Ж'` (0xD0, 0x96). Total byte length = 4097.
     Execution of `test_s.truncate(4096)`:
     ```
     thread 'main' panicked at assertion failed: self.is_char_boundary(new_len)
     ```
     **Result**: Fatal Rust panic in backend IPC handler when a user inputs launch options containing multi-byte UTF-8 characters near the 4096-byte boundary.

### 1.3 Scope 3: Platform Badges ([STEAM], [EPIC]) Rendering & Layout
- **File**: `src/lib/components/GameCard.svelte`
  - Lines 33-39:
    ```svelte
    const platformBadges = $derived.by(() => {
      if (game?.platforms && Array.isArray(game.platforms) && game.platforms.length > 0) {
        return game.platforms.map((p: string) => p.toUpperCase());
      }
      const plat = game?.platform ? String(game.platform).toUpperCase() : 'GAME';
      return [plat];
    });
    ```
  - Lines 58-64:
    ```svelte
    const PLATFORM_LABELS: Record<PlatformKind, string> = {
      steam: 'STEAM',
      epic: 'EPIC',
      roblox: 'ROBLOX',
      itch: 'ITCH',
      emulator: 'RETRO',
    };
    ```
  - Lines 161-169:
    ```svelte
    <div class="flex flex-wrap gap-1 max-w-[140px]">
      {#each platformBadges as plat}
        <span
          class="rounded border border-white/15 bg-black/75 px-1.5 py-0.5 font-mono text-[8px] font-bold tracking-wider text-zinc-200 backdrop-blur-md"
        >
          {plat}
        </span>
      {/each}
    </div>
    ```
- **Empirical Execution Results**:
  1. `[STEAM]` and `[EPIC]` badges render with clean typography (`font-mono text-[8px] font-bold tracking-wider`).
  2. For 1, 2, and 3 badges, width is constrained and fits within the top region (`< 140px`).
  3. For 5+ badges, `flex-wrap` wraps badges to a second row without clipping card borders.
  4. **Dead Code Defect**: `PLATFORM_LABELS` is defined on line 58 but never referenced. Platform `'emulator'` displays as `'EMULATOR'` instead of `'RETRO'`.

### 1.4 Scope 4: Deduplicated Games Active Platform Switching
- **File**: `src/lib/components/BentoHub.svelte`
  - Lines 66-90:
    ```svelte
    const displayGames = $derived.by(() => {
      if (dedupEnabled && selectedPlatform === 'all' && unifiedGames.length > 0) {
        return unifiedGames.map((ug) => ({
          id: ug.id,
          platform: ug.preferred_platform as any,
          platforms: ug.platforms,
          nativeId: ug.sources[0]?.native_id || '',
          title: ug.primary_title,
          installed: ug.installed,
          installDir: ug.sources[0]?.install_dir || null,
          executable: ug.sources[0]?.executable || null,
          sizeBytes: ug.sources[0]?.size_bytes || null,
          playtimeMinutes: ug.total_playtime_minutes,
          lastPlayed: ug.sources[0]?.last_played || null,
          art: { ... },
          tags: [],
          sources: ug.sources,
        }));
      }
      return games;
    });
    ```
- **File**: `src/lib/components/GameCard.svelte`
  - Lines 163-169: Badges are inert `<span>` elements without `onclick` handlers.
  - Lines 74: `await launchGameById(game.id);` where `game.id` is `ug.id` (`"unified_..."`).
- **File**: `src-tauri/src/core/platform.rs`
  - Lines 93-105:
    ```rust
    impl FromStr for GameId {
        type Err = LauncherError;

        fn from_str(s: &str) -> Result<Self> {
            let (platform, native) = s
                .split_once(':')
                .ok_or_else(|| LauncherError::InvalidInput(format!("malformed game id `{s}`")))?;
            if native.is_empty() {
                return Err(LauncherError::InvalidInput(format!("malformed game id `{s}`")));
            }
            Ok(Self::new(platform.parse()?, native))
        }
    }
    ```
- **Empirical Execution Results**:
  1. **Zero UI Interaction for Platform Switching**: No interactive control, dropdown, or click handler exists on the card badges or in modals to allow switching the active/preferred platform.
  2. **BentoHub Source Desynchronization**: `displayGames` hardcodes `ug.sources[0]` instead of locating the source matching `ug.preferred_platform`. If `preferred_platform` is Steam but `sources[0]` is Epic, native ID and directory point to Epic.
  3. **Fatal Wire Protocol Error on Launch**: Deduplicated games use ID format `"unified_{normKey}"`. When launched via `launchGameById(game.id)`, the backend `launch_game(game_id: GameId)` command attempts to parse the string with `split_once(':')`. Because `"unified_{normKey}"` lacks a colon, the command fails with `LauncherError::InvalidInput("malformed game id unified_...")`. Deduplicated games cannot be launched.

---

## 2. Logic Chain

1. **Premise**: In Feature 11 and Acceptance Criteria, deduplicated games must support active platform priority switching and seamless launching.
2. **Observation 1.4**: In `GameCard.svelte`, platform badges are inert `<span>` elements with no click handlers, no active selection indicators, and no platform switching dispatch.
3. **Observation 1.4**: In `BentoHub.svelte`, `displayGames` hardcodes `sources[0]` for metadata and assigns `id: ug.id` (`unified_{title}`).
4. **Observation 1.4**: In `src-tauri/src/core/platform.rs`, `GameId::from_str` strictly enforces `platform:native_id` syntax.
5. **Deduction**: Launching any deduplicated game fails immediately at the IPC layer with `malformed game id`. Furthermore, users cannot switch active platform.
6. **Premise**: Launch options sanitization must be secure against command injection while preserving valid game parameters.
7. **Observation 1.2**: In `library.ts`, `sanitizeLaunchArgs` splits on whitespace `split(/\s+/)`, breaking quoted arguments (`-exec "my config.cfg"` -> `['-exec', '"my', 'config.cfg"']`).
8. **Observation 1.2**: Metacharacters `>`, `<`, `^`, `%`, `()`, `\0` are unstripped, exposing command injection attack surface.
9. **Observation 1.2**: In `src-tauri/src/commands/platforms.rs` line 347, `truncate(4096)` panics on UTF-8 boundaries, which aborts the Tauri process upon saving multi-byte launch options.
10. **Conclusion**: The frontend UI and Game Properties Modal contain critical security vulnerabilities, breaking runtime bugs, and missing platform switching interactions. The milestone deliverables must be REJECTED.

---

## 3. Caveats

- **Visual Rendering**: CSS layout and monospace styling of badges were verified through automated SSR and layout boundary analysis. Visual aesthetics under liquid-chrome and tura-graphite themes are pleasing and conform to design specs, but functional interactive wiring is deficient.
- **Worker E2E Tests**: The worker's 335 passed tests in `tests/runner.cjs` passed because they evaluated isolated, mock in-file JavaScript data structures rather than real component state, IPC serialization, or live quote parsing.

---

## 4. Conclusion

**Verdict: REJECT**

The frontend UI and Game Properties Modal cannot be approved due to 8 verified empirical defects across the 4 review scopes:

| Issue ID | Severity | Category | Description |
|---|---|---|---|
| **VULN-01** | CRITICAL | Launch Wire Protocol | Unified game cards pass `id: "unified_{title}"` to `launch_game`, which crashes IPC with `malformed game id` because `GameId` requires `<platform>:<native_id>`. |
| **VULN-02** | HIGH | Platform Switching | No UI interaction exists to switch active platform on deduplicated cards (`<span>` badges lack click handlers). |
| **VULN-03** | HIGH | Launch Args Parsing | Naive `split(/\s+/)` shatters quoted arguments with spaces (`"Half Life"` -> `['"Half', 'Life"']`), breaking legitimate launch options. |
| **VULN-04** | HIGH | Rust Panic | `truncate(4096)` in `save_game_properties` panics on multi-byte UTF-8 character boundaries, crashing the backend. |
| **VULN-05** | MEDIUM | Shell Injection | Sanitizer omits redirection operators (`>`, `<`), carets (`^`), env vars (`%`), subshells (`()`), and null bytes (`\0`). |
| **VULN-06** | MEDIUM | State Desync | `BentoHub.svelte` hardcodes `sources[0]` instead of matching `preferred_platform`. |
| **VULN-07** | LOW | Resource Leak | `handleVerifyFiles()` `setInterval` lacks lifecycle cleanup when switching tabs or closing modal. |
| **VULN-08** | LOW | Dead Code | `PLATFORM_LABELS` in `GameCard.svelte` is unused; `'emulator'` displays as `'EMULATOR'` instead of `'RETRO'`. |

---

## 5. Verification Method

To independently verify all findings and reproduce every failure mode, execute the following commands in the workspace root:

### 5.1 Run the Challenger M2 Adversarial Test Suite
```powershell
npx tsx tests/stress/adversarial_challenger_m2_2.ts
```
*Expected Result*: Exits with code 1, reporting 6 PASSED, 8 VULNERABILITIES FOUND, and verdict `REJECT`.

### 5.2 Reproduce Rust Backend Panics & Wire Protocol Invalidation
```powershell
rustc tests/stress/rust_empirical_repro.rs -o tests/stress/rust_empirical_repro.exe
.\tests\stress\rust_empirical_repro.exe
Remove-Item tests/stress/rust_empirical_repro.exe, tests/stress/rust_empirical_repro.pdb -ErrorAction SilentlyContinue
```
*Expected Verbatim Output*:
```
=== Running Rust Empirical Reproductions ===
thread 'main' panicked at assertion failed: self.is_char_boundary(new_len)
[CONFIRMED_BUG] String::truncate(4096) panicked with: "assertion failed: self.is_char_boundary(new_len)"
[CONFIRMED_BUG] GameId::from_str("unified_cyberpunk2077") rejected: malformed game id (no colon)
```

### 5.3 Invalidation Conditions
This rejection is invalidated only when:
1. `GameCard.svelte` provides interactive platform switching controls (e.g. clickable badge pills with active selection ring or dropdown) that switch active launch source.
2. `BentoHub.svelte` matches `preferred_platform` against `sources` and passes a valid `GameId` (e.g. `steam:1091500`) to `launchGameById`.
3. `sanitizeLaunchArgs` correctly tokenizes quoted strings with whitespace.
4. `save_game_properties` in `platforms.rs` safely truncates UTF-8 strings using `char_indices` without panicking.
5. Shell sanitizer strips redirections, environment variables, carets, and null bytes.
