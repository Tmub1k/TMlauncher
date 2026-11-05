# Milestone 1 Adversarial Challenge Report: CSS, Styling & Visual Rendering Integrity

**Agent**: `teamwork_challenger` (Milestone 1 Challenger 2)  
**Date**: 2026-11-04T23:21:00Z  
**Parent**: `teamwork_orchestrator` (`75c17a26-5c14-4320-bf9e-bf30909202d5`)  
**Scope**: Adversarial verification of CSS variables, HTML tags, skeuomorphism remnants (.btn-game-*), and theme fallback behavior.  
**Empirical Verdict**: **REJECT** (pending targeted fixes detailed below)

---

## 1. Observation

### Observation 1.1: Build & Type Check Verification
Commands executed independently from a clean state:
- `npx tsc --noEmit` exited with code 0 (zero type errors).
- `npm run build` exited with code 0:
  ```
  vite v6.4.5 building for production...
  ✓ 140 modules transformed.
  dist/index.html                   0.45 kB │ gzip:  0.30 kB
  dist/assets/index-SjD6Zzlc.css   69.19 kB │ gzip: 11.84 kB
  dist/assets/window-BO0WFcgw.js   14.24 kB │ gzip:  3.49 kB
  dist/assets/index-oTwrgfCY.js   162.00 kB │ gzip: 50.87 kB
  ✓ built in 38.26s
  ```

### Observation 1.2: CSS Variable Integrity & Definition Coverage
Ran empirical AST/regex cross-reference script `tests/adversarial_styling_verification.cjs` analyzing all CSS variables in `src/lib/styles/app.css`, `src/lib/styles/chrome.css`, and the 4 theme manifests (`tura-graphite.json`, `liquid-chrome.json`, `obsidian-pure.json`, `cyber-aurora.json`):
- Total defined CSS variables: 52 unique custom properties.
- Total `var(--*)` usages across all 14 `.svelte` components and CSS stylesheets: 89 occurrences.
- Undefined CSS variables referenced: **0**. Every variable referenced in the frontend is explicitly defined.

### Observation 1.3: HTML & Svelte Markup Validation
Ran `svelteCompiler.compile()` across all 14 `.svelte` components in `src/` (`App.svelte` and all 13 files in `src/lib/components/`):
- Syntax errors: **0**.
- Unclosed HTML tags or malformed Svelte block structures: **0**.

### Observation 1.4: Skeuomorphism Remnants and Clunky Borders
Inspected `src/lib/styles/app.css` and searched all components:
- In `src/lib/styles/app.css`:
  - Thick 4px and 5px bottom push borders have been removed (0 occurrences of `border-bottom: [4-9]px` or similar). All `.btn-game` and `.panel-game` rules now use 1px hairlines (`border: 1px solid var(--border-subtle)`).
- In component markup:
  - 14 references to `.btn-game`, `.btn-game-green`, `.btn-game-blue`, `.btn-game-gold`, `.btn-game-red`, `.btn-game-dark`, `.btn-game-white`, `.panel-header-pill`, and `.btn-close-game` linger across 8 Svelte components:
    - `src/lib/components/GameDetailsModal.svelte` (Lines 137, 145, 210, 222, 234, 280-282, 291-293, 301-303)
    - `src/lib/components/StoreBrowserModal.svelte` (Lines 60, 68, 91, 103)
    - `src/lib/components/MediaGallery.svelte` (Lines 120, 132, 146, 153, 160, 230, 270, 318)
    - `src/lib/components/DealsRadar.svelte` (Lines 86, 127, 174)
    - `src/lib/components/FriendsList.svelte` (Lines 99, 108, 117, 154, 241)
    - `src/lib/components/RobloxStudioPanel.svelte` (Lines 333, 410, 436, 484)
    - `src/lib/components/SteamStoreShowcase.svelte` (Lines 271, 280)
    - `src/lib/components/ItchStoreShowcase.svelte` (Lines 226, 235)
  - In `GameDetailsModal.svelte` (lines 137–149) and `StoreBrowserModal.svelte` (lines 60–72), the layout directly renders:
    ```svelte
    <div class="panel-header-pill btn-game-gold text-black shadow-2xl">ИНФОРМАЦИЯ ОБ ИГРЕ</div>
    <button type="button" onclick={onClose} class="btn-close-game" aria-label="Закрыть">✕</button>
    ```
    This preserves the visual structure of casual mobile gaming pills rather than the flat minimal TURA architecture.

### Observation 1.5: Theme Fallback Failure under Partial Omission
Executed empirical stress harness `tests/adversarial_theme_engine.cjs`:
1. **Cross-Theme Token Leakage**:
   - `cyber-aurora` sets `--accent-glow: 0 0 28px rgba(147, 197, 253, 0.25)` and `--bg-canvas: #08090d`.
   - A subsequent custom theme `custom-paper` defines only `tokens: { "--bg-canvas": "#ffffff" }`, omitting `--accent-glow`.
   - In `src/lib/themes/theme-engine.ts` (lines 127–133):
     ```ts
     const tokens = resolvedTheme.tokens || {};
     for (const [prop, val] of Object.entries(tokens)) {
       if (typeof val === 'string' && prop.startsWith('--')) {
         root.style.setProperty(prop, val);
       }
     }
     ```
   - Result: `root.style.getPropertyValue('--accent-glow')` remains `0 0 28px rgba(147, 197, 253, 0.25)`. The previous theme's neon glow leaks into the new theme instead of falling back to default theme tokens or `:root` stylesheet defaults.
2. **Unhandled Null / Undefined Input Crash**:
   - In `theme-engine.ts` (line 115):
     ```ts
     } else {
       resolvedTheme = themeOrId;
       if (resolvedTheme?.id) { ... }
     }
     this.activeTheme = resolvedTheme;
     ...
     root.dataset.theme = resolvedTheme.id; // Uncaught TypeError if resolvedTheme is null/undefined!
     ```
   - `engine.applyTheme(null)` throws:
     `TypeError: Cannot read properties of null (reading 'id')`.

---

## 2. Logic Chain

1. **CSS Variable & HTML Integrity (PASS)**:
   - *Observation 1.1, 1.2, 1.3*: The build passes cleanly with code 0. AST checking confirmed 52/52 variables defined, and all 14 Svelte components compile with 0 syntax or tag errors.
   - *Inference*: The core frontend infrastructure has sound syntactic and structural integrity.

2. **Skeuomorphism & Clunky Borders (PARTIAL FAILURE)**:
   - *Observation 1.4*: In `src/lib/styles/app.css`, thick 4px/5px push borders were removed. However, the legacy class names (`.btn-game-*`, `.panel-header-pill`, `.btn-close-game`) were not replaced across existing modal components (`GameDetailsModal.svelte`, `StoreBrowserModal.svelte`).
   - *Inference*: While the worker prevented visual breakage by restyling `.btn-game` in CSS, the component markup in multiple modals still relies on `.panel-header-pill` and `.btn-game-gold` / `.btn-game-blue`, which directly violates Requirement R1 ("убрав лишние надписи и поля с пояснениями", eliminate casual 3D skeuomorphism) and leaves obsolete UI paradigms intact.

3. **Theme Fallback Robustness (CRITICAL FAILURE)**:
   - *Observation 1.5*: When a custom drop-in theme omits tokens, `ThemeEngine.applyTheme` iterates only over the keys present in `resolvedTheme.tokens`. It neither resets `root.style` nor merges the partial theme against the base default theme (`BUILTIN_THEMES['tura-graphite'].tokens`).
   - *Inference*: If a user switches from Theme A to Theme B, any tokens omitted by Theme B permanently retain Theme A's inline values. Conversely, if the app boots directly into Theme B, omitted tokens fall back to `:root`. This creates non-deterministic styling that corrupts the UI based on navigation history.
   - *Observation 1.5*: Invoking `applyTheme(null)` or `applyTheme(undefined)` crashes with an uncaught `TypeError`.

---

## 3. Caveats

- **Caveat 1**: The legacy components (`GameDetailsModal.svelte`, `StoreBrowserModal.svelte`) are slated for feature expansions in Milestone 2 (`GamePropertiesModal`) and Milestone 3 (`StoreView`). However, because they are actively rendered in `BentoHub.svelte` and user-accessible right now, their lingering skeuomorphic classes and pills impact visual integrity today.
- **Caveat 2**: Windows Mica/Acrylic effects (`setVibrancy`) cannot be visually rendered in a headless command-line environment; however, IPC error handling in `theme-engine.ts` was verified to catch errors gracefully without throwing.

---

## 4. Conclusion

**Verdict: REJECT**

The Milestone 1 work product successfully establishes the core TURA palette, liquid chrome styling, frameless titlebar, and zero build/type errors. However, it fails two critical adversarial criteria:
1. **Defective Theme Fallback**: State leakage exists when custom themes partially omit tokens, causing styles from previous themes to cross-contaminate the active theme. Furthermore, `applyTheme(null)` crashes.
2. **Lingering Skeuomorphic Elements**: Remnants of casual game styling (`.panel-header-pill`, `.btn-close-game`, `.btn-game-*`) persist in modal components (`GameDetailsModal.svelte`, `StoreBrowserModal.svelte`, `MediaGallery.svelte`).

### Required Action Items to Achieve Approval:
1. **Fix `src/lib/themes/theme-engine.ts`**:
   - Merge incoming theme tokens with default fallback tokens before applying:
     ```ts
     const baseTokens = BUILTIN_THEMES['tura-graphite'].tokens;
     const resolvedTokens = { ...baseTokens, ...(resolvedTheme.tokens || {}) };
     ```
   - Clear obsolete inline style properties on `root.style` or set all known design tokens from `resolvedTokens`.
   - Add null/undefined safety guard:
     ```ts
     if (!themeOrId) {
       resolvedTheme = BUILTIN_THEMES['tura-graphite'];
     }
     ```
2. **Modernize Modal Headers**:
   - Remove `.panel-header-pill` and `.btn-close-game` from `GameDetailsModal.svelte` and `StoreBrowserModal.svelte`, replacing them with frameless minimalist card headers matching TURA / Liquid Chrome specifications.

---

## 5. Verification Method

To independently reproduce and verify this verdict:

1. **Execute Empirical Adversarial Verification Suite**:
   ```powershell
   node tests/adversarial_styling_verification.cjs
   ```
   *Verified Output*:
   - CSS Variables: 52 defined, 0 missing.
   - Svelte AST: 14 components compile cleanly.
   - Skeuomorphism: 14 legacy class occurrences found across 8 files.
   - Token Leakage: `Does partial theme leak previous theme tokens? true`.

2. **Execute Theme Engine Adversarial Stress Test**:
   ```powershell
   node tests/adversarial_theme_engine.cjs
   ```
   *Verified Output*:
   - `applyTheme(null)` throws `TypeError: Cannot read properties of null (reading 'id')`.
   - State leakage confirmed: Omitted tokens retain previous theme inline styles.

3. **Invalidation Condition**:
   This rejection is invalidated once `node tests/adversarial_theme_engine.cjs` reports `[PASS]` on null safety and clean fallback isolation, and `tests/adversarial_styling_verification.cjs` confirms zero lingering `.panel-header-pill` elements.
