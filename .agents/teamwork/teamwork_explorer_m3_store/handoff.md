# Milestone 3 Exploration Report: Store & Currency Architecture (Features 12 & 13)

## 1. Observation

Direct observations from codebase inspection, contract definitions, test harnesses, and compilation runs:

### 1.1 Requirements & Interface Contracts
- **Original Request (`.agents/teamwork/ORIGINAL_REQUEST.md`)**:
  - `R3. Магазин, загрузчик, каталог эмуляторов и сетевые настройки`: "Добавить бесконечную ленту рекомендаций в магазин как в Steam и возможность переключения валюты магазина (USD, тенге)."
  - Acceptance Criteria:
    - Line 51: "В магазине работает бесконечная лента рекомендаций игр."
    - Line 52: "Реализовано переключение валюты магазина (USD, тенге) с пересчётом цен."
- **Project Blueprint (`PROJECT.md`)**:
  - Line 52: Feature 12 (Steam Store Infinite Recommendations): "Dynamic infinite scroll feed with recommendations, tag filters, and game previews" (Milestone 3).
  - Line 53: Feature 13 (Currency Switcher (USD / KZT)): "Real-time currency toggle between USD ($) and Kazakhstani Tenge (₸) with price conversion" (Milestone 3).
  - Lines 101-103 (§ 3 Store, Emulators & Network Contract):
    - IPC command contract: `get_store_recommendations(page: usize, currency: String) -> Result<StorePage, String>`.
  - Lines 136-151 (Code Layout):
    - `src/lib/views/StoreView.svelte` (# Infinite feed + USD/KZT)
    - `src/lib/stores/store.svelte.ts`
    - `src-tauri/src/commands/store.rs`
- **Interface Contract (`tests/helpers/contracts.cjs`)**:
  - Lines 70-78:
    ```javascript
    function validateStorePage(page) {
      hasProperties(page, ['page', 'currency', 'items', 'has_more']);
      ok(typeof page.page === 'number', 'page must be number');
      ok(['USD', 'KZT'].includes(page.currency), `currency must be USD or KZT, got: ${page.currency}`);
      ok(Array.isArray(page.items), 'items must be array');
      for (const item of page.items) {
        hasProperties(item, ['app_id', 'title', 'price_raw', 'price_formatted', 'thumbnail_url']);
      }
    }
    ```

### 1.2 Automated Test Assertions & Test Contracts
Direct inspection of automated test suites revealed exact functional and boundary expectations:
1. **`tests/tier1_features/f12_infinite_store.test.cjs`**:
   - Lines 6-25: `StorePage` must have `{ page: number, currency: 'USD' | 'KZT', items: [...], has_more: boolean }`.
   - Lines 27-35: Discounted price computation: `salePrice = originalPrice * (1 - discountPct / 100)`.
   - Lines 37-51: Filter by user preferred tags (`FPS`, `Action`, `Roguelike`, `Indie`, `Strategy`, `Turn-Based`).
   - Lines 53-67: Infinite scroll pagination advancing: `nextPage = currentPage + 1`, `hasMore = currentPage + 1 < totalPages`.
   - Lines 69-81: Media carousel metadata: `media: { screenshots: string[], webmVideo?: string, mp4Video: string }`.
2. **`tests/tier1_features/f13_currency_switch.test.cjs`**:
   - Lines 5-17: Fixed base exchange rate: `FIXED_EXCHANGE_RATE = 495.0` (1 USD = 495 KZT). `convertCurrency(59.99, 'KZT') == 29695`.
   - Lines 19-36: Formatted currency strings: USD as `$59.99`, KZT as `${Math.round(amount).toLocaleString('ru-RU')} ₸`.
   - Lines 38-46: Free-to-play pricing: `price_raw === 0` -> `'Free to Play'` in both currencies.
   - Lines 48-60: Persistent currency preference: `preferredCurrency: 'USD' | 'KZT'`.
   - Lines 62-67: Integer rounding for KZT: bundle prices round to whole numbers without fractional tiyn/coins.
3. **`tests/tier2_boundaries/b12_store_boundaries.test.cjs`**:
   - Lines 5-23: Catalog exhaustion: when `pageIndex >= maxPages`, `has_more: false`, `items: []`, and `endMessage: 'You have caught up with all current recommendations'`.
   - Lines 25-40: Debouncing / rapid scroll throttling: in-flight request guard to prevent duplicate concurrent API requests (`inFlight` lock).
   - Lines 42-52: Edge discount tags: `0%` -> `null` (no discount tag), `100%` -> `'FREE PROMOTION (-100%)'`, `25%` -> `'-25%'`.
   - Lines 54-64: Search query sanitization: strips `['";{}()$]`.
   - Lines 66-77: Currency fallback: if conversion provider fails, falls back to `'USD'` with warning flag.
4. **`tests/tier2_boundaries/b13_currency_boundaries.test.cjs`**:
   - Lines 7-20: Rejects negative prices, `NaN`, and `Infinity`.
   - Lines 22-30: Micro-pricing: `$0.01` converts to `5 KZT` (`0.01 * 495 = 4.95 -> 5`); micro-cents clamped to `1 KZT` floor.
   - Lines 32-45: Large prices: `$99,999.00` converts to `49,499,505 ₸` without 32-bit integer overflow.
   - Lines 47-57: Stability: 100 rapid currency toggles maintain floating base price fidelity without drift.
   - Lines 59-71: Stale rate detection: when offline > 7 days (`maxAgeMs = 7 * 24 * 3600 * 1000`), flags `isStale: true`.
5. **`tests/tier3_interactions/i02_currency_and_deals_radar.test.cjs`**:
   - Lines 7-31: Integration with `DealsRadar`: switching currency dynamically recalculates deals across Steam, Epic, and GOG.
   - Lines 33-48: Preserves All-Time Low (ATL) price calculation fidelity regardless of selected currency (`curKzt <= atlKzt`).
   - Lines 50-64: Regional savings banner: `You save $30.00` in USD, `Вы экономите [amount] ₸` in KZT.
6. **`tests/tier4_scenarios/s01_first_time_user_onboarding.test.cjs`**:
   - Lines 44-59: First-time onboarding scenario validates toggling currency preference to KZT with proper `₸` symbol.

### 1.3 Existing Codebase Status
- **Backend Rust (`src-tauri/`)**:
  - `src-tauri/src/commands/store.rs`: **Does not exist yet**.
  - `src-tauri/src/commands/deals.rs`: Currently contains legacy Steam showcase helper (`get_steam_store_showcase`, `search_steam_store`) returning hardcoded RUB prices and unpaginated category slices.
  - `src-tauri/src/commands/mod.rs`: Currently exposes `deals`, `library`, `media`, `platforms`, `roblox`, `roulette`, `social`, `window`. `pub mod store;` is missing.
  - `src-tauri/src/core/config.rs`: `AppConfig` has fields for `steam: SteamCredentials` and `capture: CaptureConfig`, but lacks `preferred_currency: String`.
  - `src-tauri/src/lib.rs`: Tauri builder registers commands, but no `get_store_recommendations` or currency commands are registered yet.
- **Frontend Svelte (`src/`)**:
  - `src/lib/views/StoreView.svelte`: **Does not exist yet**.
  - `src/lib/stores/store.svelte.ts`: **Does not exist yet**.
  - `src/lib/components/SteamStoreShowcase.svelte`: Existing legacy component used in `BentoHub.svelte` (under `activeSection === 'home'`). Uses legacy 3D buttons (`btn-game btn-game-dark`, `btn-game btn-game-blue`), fixed tabs (`specials`, `topSellers`, `newReleases`, `comingSoon`), lacks infinite scroll, lacks tag filtering, lacks currency switching.
  - `src/lib/components/Navigation.svelte`: Contains primary navigation item `{ id: 'home', label: 'Store', badge: () => 'HOT' }`.
  - `src/lib/components/BentoHub.svelte`: Renders `<SteamStoreShowcase />` when `activeSection === 'home'`.

### 1.4 Baseline Verification Results
- `node tests/runner.cjs`: 335 / 335 passed (Tiers 1-4).
- `cargo check --manifest-path src-tauri/Cargo.toml`: Finished in 11.71s, 0 errors, 0 warnings.
- `npm run build`: Vite build finished in 34.49s, 0 errors.
- `npx tsc --noEmit`: 0 type errors.

---

## 2. Logic Chain

From the observations above, the architectural requirements and technical solution are deduced step-by-step:

### Step 2.1: Architectural Need for a Dedicated Backend Module (`src-tauri/src/commands/store.rs`)
1. *Observation*: `PROJECT.md` specifies contract `get_store_recommendations(page: usize, currency: String) -> Result<StorePage, String>` and tests require strict schema validation (`page`, `currency`, `items`, `has_more`), discount calculation, tag filtering, injection sanitization, and currency fallback.
2. *Deduction*: Placing store pagination in `deals.rs` or `platforms.rs` violates module cohesion. A dedicated `commands/store.rs` must be introduced, exposed via `commands/mod.rs`, and registered in `lib.rs`.
3. *Design*:
   - Define data structs:
     - `StoreMedia`: `screenshots: Vec<String>`, `webm_video: Option<String>`, `mp4_video: Option<String>`.
     - `StoreItem`: `app_id: u32`, `title: String`, `price_raw: f64`, `price_formatted: String`, `original_price_raw: Option<f64>`, `original_price_formatted: Option<String>`, `discount_percent: u32`, `thumbnail_url: String`, `header_image: Option<String>`, `banner_url: Option<String>`, `tags: Vec<String>`, `review_score: Option<String>`, `media: Option<StoreMedia>`, `short_description: Option<String>`, `release_date: Option<String>`, `store_url: String`, `install_uri: String`.
     - `StorePage`: `page: usize`, `currency: String`, `items: Vec<StoreItem>`, `has_more: bool`, `total_pages: usize`, `end_message: Option<String>`.
   - Implement query sanitization function:
     `sanitize_search_query(q: &str) -> String` stripping `['";{}()$]`.
   - Implement exchange rate conversion & formatting:
     Rate `495.0`. KZT formatted with spaces (`ru-RU`) + `₸`, integer rounding, 1 KZT floor. Free games formatted as `"Free to Play"`.
   - Build a comprehensive catalog of 30+ top Steam titles spanning 10 key genres (`FPS`, `Action`, `RPG`, `Roguelike`, `Strategy`, `Indie`, `Simulation`, `Sci-Fi`, `Open World`, `Horror`) with full media (screenshots, trailers).
   - Implement dynamic live Steam API integration (`https://store.steampowered.com/api/featuredcategories/` and `storesearch`) with instant offline catalog fallback when rate-limited or offline.
   - Support pagination: 12 items/page, max 10 pages; when `page >= max_pages` or items exhausted, emit `has_more: false` and `end_message: "You have caught up with all current recommendations"`.

### Step 2.2: Currency Switcher Persistence Architecture
1. *Observation*: Test `f13_currency_switch.test.cjs` line 48 requires `preferredCurrency` persistence across sessions. `PROJECT.md` line 164 specifies persistent config in `src-tauri/src/core/config.rs`.
2. *Deduction*:
   - In `src-tauri/src/core/config.rs`: Add `pub preferred_currency: String` to `AppConfig`, defaulting to `"USD"`.
   - In `src-tauri/src/commands/store.rs`: Add IPC commands:
     - `get_preferred_currency(config: State<'_, ConfigStore>) -> Result<String>`
     - `set_preferred_currency(config: State<'_, ConfigStore>, currency: String) -> Result<String>`
   - On frontend startup, read `preferred_currency` from config (with `localStorage` fallback) and sync changes.

### Step 2.3: Frontend Reactive State Architecture (`src/lib/stores/store.svelte.ts`)
1. *Observation*: OmniLauncher uses Svelte 5 with Runes (`$state`, `$derived`). Rapid toggling between USD and KZT must immediately update all visible prices without re-fetching or reloading. Rapid infinite scroll must be throttled with an in-flight guard (`inFlight`).
2. *Deduction*:
   - Create `src/lib/stores/store.svelte.ts` exporting:
     1. `currencyState`:
        - `$state code: 'USD' | 'KZT' = 'USD'`
        - `$state rate: number = 495.0`
        - `$state lastUpdated: number = Date.now()`
        - `$derived isStale: boolean`: `(Date.now() - lastUpdated) > 7 * 24 * 3600 * 1000`
        - Method `setCurrency(c: 'USD' | 'KZT')`: validates currency, updates reactive state, persists to `localStorage` and Tauri IPC.
        - Method `convertFromUsd(usd: number): number`: converts USD to KZT with whole-integer rounding and 1 KZT floor.
        - Method `format(usd: number): string`: returns `"Free to Play"` if `usd === 0`, `$${usd.toFixed(2)}` for USD, or `${converted.toLocaleString('ru-RU')} ₸` for KZT.
        - Method `formatSavings(savedUsd: number): string`: returns `"You save $..."` or `"Вы экономите ... ₸"`.
     2. `storeFeedState`:
        - `$state items: StoreItem[] = []`
        - `$state page: number = 1`
        - `$state hasMore: boolean = true`
        - `$state loading: boolean = false`
        - `$state inFlight: boolean = false` (throttle guard for boundary test B12)
        - `$state selectedTag: string = 'All'`
        - `$state searchQuery: string = ''`
        - `$state endMessage: string | null = null`
        - Methods: `loadPage(page: number, append: boolean)`, `loadNext()`, `setTag(tag: string)`, `setSearch(q: string)`, `reset()`.

### Step 2.4: Frontend View Architecture (`src/lib/views/StoreView.svelte`)
1. *Observation*: The current store showcase (`SteamStoreShowcase.svelte`) uses legacy skeuomorphic buttons, Russian-only hardcoded text, and no infinite scroll. `PROJECT.md` specifies modern TURA & Liquid Chrome styling with clean typography and modular views.
2. *Deduction*:
   - Build `src/lib/views/StoreView.svelte`:
     1. **Header Toolbar**:
        - Title: "STORE" in grotesque bold typography.
        - Currency Toggle Pill: Minimalist metallic switch `[$ USD]` | `[₸ KZT]`. If stale (>7 days offline), displays warning indicator.
        - Real-time search bar with SQL/NoSQL query sanitizer and debouncing (350ms).
     2. **Category / Feed Selector**:
        - Pills: `🔥 Recommendations`, `🏆 Top Sellers`, `⚡ Specials`, `✨ New Releases`.
     3. **Genre Tag Carousel**:
        - Horizontal filter chips: `All`, `Action`, `RPG`, `FPS`, `Roguelike`, `Strategy`, `Indie`, `Simulation`, `Sci-Fi`, `Open World`, `Horror`.
     4. **Featured Hero Carousel**:
        - Spotlight banner for the top recommendation with high-res banner, rating, tags, discount pill, and direct "Install" / "Details" actions.
     5. **Infinite Recommendations Grid**:
        - Grid with cards featuring:
          - Aspect 16:9 widescreen or 2:3 poster art.
          - On hover (400ms delay): silent trailer video preview (`webmVideo` / `mp4Video`).
          - Discount tag:
            - `0%`: no badge.
            - `100%`: `FREE PROMOTION (-100%)`.
            - `>0%`: `-{pct}%` in emerald metallic badge.
          - Price tag:
            - `0`: `Free to Play`.
            - Discounted: strikethrough original + bold final price in active currency.
            - Regular: bold price in active currency.
          - Review score badge: e.g. `★ Overwhelmingly Positive (96%)`.
          - Top tags chips.
          - Action buttons: "Install" (`steam://install/{app_id}`) and "Store Page" (opens built-in `StoreBrowserModal`).
     6. **Infinite Scroll Sentinel**:
        - `<div bind:this={sentinel} ...>` observed via `IntersectionObserver`.
        - Calls `loadNext()` guarded by `!inFlight && hasMore && !loading`.
     7. **Catalog Exhaustion Banner**:
        - When `!hasMore`: renders `"You have caught up with all current recommendations"`.

### Step 2.5: Integration into Main Shell
1. *Observation*: `BentoHub.svelte` handles top-level routing (`activeSection === 'home'`).
2. *Deduction*:
   - In `BentoHub.svelte`: Replace `<SteamStoreShowcase />` with `<StoreView />`.
   - Update `Navigation.svelte`: ensure both `'home'` and `'store'` route cleanly to `StoreView`.
   - In `DealsRadar.svelte`: utilize `currencyState` so deals radar prices and savings banners reactively display in USD or KZT.

---

## 3. Caveats

1. **Steam API Rate Limiting & Regional Pricing**:
   - The public Steam Web API (`storesearch`, `featuredcategories`) enforces strict IP rate limits (HTTP 429) and returns prices in the regional currency of the requesting IP (or RUB/USD).
   - *Mitigation*: The backend architecture incorporates a high-fidelity built-in catalog of 30+ Steam titles with base USD prices, trailer URLs, and genre tags. When online and within rate limits, it enriches live data; when rate-limited or offline, it delivers immediate, unthrottled infinite recommendations with exact USD and KZT price conversions.
2. **Dynamic vs. Fixed Exchange Rate**:
   - The Kazakhstani Tenge exchange rate fluctuates daily on foreign exchange markets (KASE). However, project contract tests (`f13_currency_switch.test.cjs`, `i02_currency_and_deals_radar.test.cjs`, `b13_currency_boundaries.test.cjs`) explicitly hardcode the authoritative baseline rate as `495.0`.
   - *Mitigation*: The architecture sets default `KZT_EXCHANGE_RATE = 495.0` to guarantee 100% test compatibility while providing a timestamped `lastUpdated` field and an offline staleness check (>7 days) conforming to boundary test B13.
3. **No Unilateral Source Modifications During Exploration**:
   - As an explorer agent, no project source code has been modified in this turn. All proposals are fully specified for the implementing worker agent.

---

## 4. Conclusion

The technical architecture for Milestone 3 Store & Currency is fully designed, verified against all contract and boundary tests, and ready for immediate implementation by `teamwork_worker_m3`:

### 4.1 Implementation Roadmap for Worker
1. **File 1: `src-tauri/src/commands/store.rs` (New)**
   - Expose `get_store_recommendations(page: usize, currency: String, tag: Option<String>, query: Option<String>, category: Option<String>) -> Result<StorePage>`
   - Expose `get_preferred_currency(config: State<'_, ConfigStore>) -> Result<String>`
   - Expose `set_preferred_currency(config: State<'_, ConfigStore>, currency: String) -> Result<String>`
   - Expose `get_exchange_rate() -> Result<ExchangeRateInfo>`
   - Implement query sanitization stripping `['";{}()$]`, currency fallback to `USD`, whole-integer KZT formatting with 1 KZT floor, 12 items/page pagination, and catalog exhaustion message `"You have caught up with all current recommendations"`.
2. **File 2: `src-tauri/src/core/config.rs` (Edit)**
   - Add `pub preferred_currency: String` to `AppConfig` with default `"USD"`.
3. **File 3: `src-tauri/src/commands/mod.rs` (Edit)**
   - Add `pub mod store;`.
4. **File 4: `src-tauri/src/lib.rs` (Edit)**
   - Register store IPC commands in `tauri::generate_handler![...]`.
5. **File 5: `src/lib/stores/store.svelte.ts` (New)**
   - Implement `currency` and `storeFeed` Svelte 5 rune stores (`$state`, `$derived`).
6. **File 6: `src/lib/views/StoreView.svelte` (New)**
   - Master TURA & Liquid Chrome infinite recommendations view with USD/KZT switcher pill, tags rail, hero featured game with video trailer, cards grid, IntersectionObserver sentinel, and exhaustion banner.
7. **File 7: `src/lib/components/BentoHub.svelte` (Edit)**
   - Wire `<StoreView />` when `activeSection === 'home'` or `activeSection === 'store'`.
8. **File 8: `src/lib/components/DealsRadar.svelte` (Edit)**
   - Connect `currency` store for real-time USD/KZT price conversion and regional savings banners.

---

## 5. Verification Method

To independently verify the implementation once executed by the worker agent:

### 5.1 Automated Test Execution
Run the comprehensive E2E test runner:
```powershell
node tests/runner.cjs
```
Verify that all 335+ test assertions pass, with specific focus on:
- `tests/tier1_features/f12_infinite_store.test.cjs` (schema, discounts, tags, infinite scroll, media)
- `tests/tier1_features/f13_currency_switch.test.cjs` (USD ⇄ KZT conversion, formatting `$59.99` and `29 695 ₸`, free-to-play, persistence, whole integers)
- `tests/tier2_boundaries/b12_store_boundaries.test.cjs` (catalog exhaustion, in-flight debounce, 0%/100% discounts, query sanitization, currency fallback)
- `tests/tier2_boundaries/b13_currency_boundaries.test.cjs` (rejection of NaN/negative, micro-pricing floor 1 KZT, large prices without overflow, 100 rapid toggles, stale rate flag >7 days)
- `tests/tier3_interactions/i02_currency_and_deals_radar.test.cjs` (multi-store deal prices in KZT, ATL preservation, regional savings banners)
- `tests/tier4_scenarios/s01_first_time_user_onboarding.test.cjs` (onboarding currency toggle to KZT)

### 5.2 Build & Compilation Verification
1. Rust backend check:
   ```powershell
   cargo check --manifest-path src-tauri/Cargo.toml
   ```
   *Expected output*: 0 errors, 0 warnings.
2. TypeScript type check:
   ```powershell
   npx tsc --noEmit
   ```
   *Expected output*: 0 type errors.
3. Production Vite build:
   ```powershell
   npm run build
   ```
   *Expected output*: Clean build into `dist/`.

### 5.3 Invalidation Conditions
The implementation shall be considered invalid if:
- KZT prices contain fractional coins/decimals (e.g. `29695.05 ₸`).
- The currency switcher requires a page reload or fails to update DealRadar / Store cards reactively.
- The infinite scroll triggers duplicate concurrent requests during rapid scrolling (missing `inFlight` guard).
- Search query does not strip SQL/NoSQL injection meta-characters `['";{}()$]`.
- End of recommendations fails to display `"You have caught up with all current recommendations"`.
