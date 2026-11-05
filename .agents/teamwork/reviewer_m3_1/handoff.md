# Handoff Report: Milestone 3 Review (Store & Currency Implementation)

## 1. Observation

Direct observations from independent code inspection, build tool executions, and adversarial runtime contract stress-testing:

### 1.1 Direct Build & Verification Tool Command Outputs
- **Rust Backend Compilation (`cargo check --manifest-path src-tauri/Cargo.toml`)**:
  ```text
  Checking omnilauncher v0.1.0 (C:\Users\Tmub\Documents\TMlauncher\src-tauri)
  Finished `dev` profile [unoptimized + debuginfo] target(s) in 8.56s
  ```
  Result: Exit code 0, 0 errors, 0 warnings.

- **Rust Test Suite (`cargo test --manifest-path src-tauri/Cargo.toml`)**:
  ```text
  Running unittests src\lib.rs: 7 passed; 0 failed
  Running unittests src\main.rs: 0 passed; 0 failed
  Running tests\challenger_utf8_truncation_test.rs: 7 passed; 0 failed
  Running tests\m2_stress_tests.rs: 10 passed; 0 failed
  Running tests\m3_unit_tests.rs: 5 passed; 0 failed
  Total: 29 passed; 0 failed; 0 ignored; finished in 1.00s
  ```
  Result: Exit code 0, 29 passed.

- **TypeScript Typecheck (`npx tsc --noEmit`)**:
  Result: Exit code 0, 0 errors.

- **Frontend Vite Production Build (`npm run build`)**:
  ```text
  vite v6.4.5 building for production...
  ✓ 153 modules transformed.
  dist/index.html                   0.45 kB │ gzip:  0.30 kB
  dist/assets/index-BUhbveJ0.css   83.18 kB │ gzip: 13.54 kB
  dist/assets/window-Bg6tPslh.js   14.24 kB │ gzip:  3.49 kB
  dist/assets/index-BAuqpZHk.js   241.99 kB │ gzip: 73.35 kB
  ✓ built in 43.51s
  ```
  Result: Exit code 0.

- **Automated E2E Test Runner**:
  - `node tests/runner.cjs --feature=F12`: 5 / 5 passed (Tier 1).
  - `node tests/runner.cjs --feature=F13`: 5 / 5 passed (Tier 1).
  - `node tests/runner.cjs` (full suite): 335 / 335 passed across Tiers 1-4.

### 1.2 Inspection of Implementation Code
- **`src-tauri/src/commands/store.rs`**:
  - Line 8: `pub const FIXED_EXCHANGE_RATE: f64 = 495.0;`
  - Lines 18–38:
    ```rust
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct StoreItem {
        pub app_id: u32,
        pub title: String,
        pub price_raw: f64,
        pub price_formatted: String,
        pub original_price_raw: Option<f64>,
        pub original_price_formatted: Option<String>,
        pub discount_percent: u32,
        pub thumbnail_url: String,
        pub header_image: Option<String>,
        pub banner_url: Option<String>,
        pub tags: Vec<String>,
        pub review_score: Option<String>,
        pub media: Option<StoreMedia>,
        pub short_description: Option<String>,
        pub release_date: Option<String>,
        pub store_url: String,
        pub install_uri: String,
    }
    ```
  - Lines 40–50:
    ```rust
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct StorePage {
        pub page: usize,
        pub currency: String,
        pub items: Vec<StoreItem>,
        pub has_more: bool,
        pub total_pages: usize,
        pub end_message: Option<String>,
    }
    ```
  - Lines 265–275:
    `sanitize_search_query` cleanly strips `['\'', '"', ';', '{', '}', '(', ')', '$']`.
    `convert_price_to_kzt(usd: f64) -> u64` returns `0` if `usd <= 0.0`, else `(usd * FIXED_EXCHANGE_RATE).round().max(1.0) as u64`.
  - Lines 277–297: `format_price_string` formats `0.0` as `"Free to Play"`, KZT as space-grouped integers (`29 695 ₸`), and USD as `"$59.99"`.
  - Lines 455–479: `get_preferred_currency` and `set_preferred_currency` atomically write to `config.toml` via `ConfigStore`.
  - Lines 481–491: `convert_currency` rejects `< 0.0`, `is_nan()`, and `is_infinite()`.

- **`src/lib/stores/store.svelte.ts`**:
  - Lines 9–27 (`StoreItem` interface) and lines 29–36 (`StorePage` interface) declare fields in snake_case:
    `app_id`, `price_raw`, `price_formatted`, `original_price_raw`, `original_price_formatted`, `discount_percent`, `thumbnail_url`, `header_image`, `has_more`, `total_pages`, `end_message`.
  - Lines 128–130:
    ```typescript
    this.page = res.page;
    this.hasMore = res.has_more;
    this.endMessage = res.end_message || null;
    ```
  - Lines 142–146:
    ```typescript
    async loadNext(): Promise<boolean> {
      if (!this.hasMore || this.loading || this.inFlight) {
        return false;
      }
      return await this.loadPage(this.page + 1, true);
    }
    ```

- **`src/lib/views/StoreView.svelte`**:
  - Reads snake_case properties exclusively:
    - Line 206: `featuredItem.review_score`
    - Line 216: `featuredItem.short_description`
    - Line 232: `featuredItem.discount_percent`
    - Line 237: `featuredItem.original_price_formatted`
    - Line 241: `featuredItem.price_formatted`
    - Line 247: `featuredItem.install_uri`
    - Line 272: `hoveredAppId = item.app_id`
    - Line 287: `item.header_image`
    - Line 301: `item.discount_percent`
    - Line 314: `item.review_score`
    - Line 333: `item.original_price_formatted`
    - Line 338: `item.price_formatted`
    - Line 344: `item.install_uri`
    - Line 352: `item.store_url`
    - Line 372: `!storeFeed.hasMore && storeFeed.endMessage`

- **`tests/helpers/contracts.cjs`**:
  - Lines 70–78:
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

### 1.3 Adversarial Contract Validation Tool Result
Running `validateStorePage` against the real serialized JSON emitted by Rust `#[serde(rename_all = "camelCase")]`:
```powershell
node -e "
const { validateStorePage } = require('./tests/helpers/contracts.cjs');
const serdeOutput = {
  page: 0,
  currency: 'USD',
  hasMore: true,
  totalPages: 10,
  items: [{ appId: 1091500, title: 'Cyberpunk 2077', priceRaw: 29.99, priceFormatted: '\$29.99', thumbnailUrl: '...' }]
};
validateStorePage(serdeOutput);
"
```
**Direct Output**:
```text
AssertionError: Missing expected property: 'has_more'
```
When `has_more` is supplied, it immediately fails on:
```text
AssertionError: Missing expected property: 'app_id'
```

---

## 2. Logic Chain

1. **Serialization Contract Breach**:
   - `StoreItem` and `StorePage` in `src-tauri/src/commands/store.rs` (lines 19 and 41) are decorated with `#[serde(rename_all = "camelCase")]`.
   - When Tauri serializes `StorePage` over IPC, keys are renamed: `has_more` -> `hasMore`, `total_pages` -> `totalPages`, `end_message` -> `endMessage`, `app_id` -> `appId`, `price_raw` -> `priceRaw`, `price_formatted` -> `priceFormatted`, `header_image` -> `headerImage`, `discount_percent` -> `discountPercent`, `install_uri` -> `installUri`, `store_url` -> `storeUrl`.
   - This directly violates the interface contract defined in `tests/helpers/contracts.cjs:71`, which requires snake_case properties (`has_more`, `app_id`, `price_raw`, `price_formatted`, `thumbnail_url`).

2. **Cascade Breakdown in Frontend Reactivity & Infinite Scrolling**:
   - In `src/lib/stores/store.svelte.ts:128`, `this.hasMore = res.has_more;` is executed.
   - Because Tauri's deserialized object has `hasMore` instead of `has_more`, `res.has_more` evaluates to `undefined`.
   - `this.hasMore` becomes `undefined` (falsy).
   - In `storeFeed.loadNext()` (line 142), `if (!this.hasMore) return false;` evaluates to `true`.
   - **Conclusion**: The infinite scroll sentinel fails after page 0. Page 1 to 9 will NEVER load at runtime.

3. **Cascade Breakdown in UI Card Rendering**:
   - In `StoreView.svelte`, every binding accesses snake_case properties:
     - `featuredItem.price_formatted` and `item.price_formatted` -> `undefined` (all prices rendered as empty strings).
     - `item.header_image` -> `undefined` (all game cards render without artwork).
     - `item.discount_percent` -> `undefined` (no discount badges are shown).
     - `item.install_uri` and `item.store_url` -> `undefined` (all action buttons and links are broken).
     - `storeFeed.endMessage` -> `null` (exhaustion message is never displayed).

4. **Why Automated Tests Did Not Catch This Bug**:
   - `tests/tier1_features/f12_infinite_store.test.cjs:6-25` tested a manually instantiated JavaScript mock object:
     `const page = { page: 1, currency: 'USD', items: [{ app_id: 1091500, ... }], has_more: true };`
   - The test verified the mock against `validateStorePage`, rather than invoking the Rust command or validating `serde_json` serialization output.
   - Consequently, the test suite showed 100% pass (335/335) while real runtime IPC was fundamentally disconnected.

---

## 3. Caveats

1. **Other Milestone 3 Components**:
   - Emulators engine (`emulators.rs`), Windows drive enumeration (`network.rs`), proxy validation, and `DirectoryPickerModal.svelte` were built cleanly and verified to compile without error.
2. **Design Language & Visual Refinement**:
   - `StoreView.svelte` faithfully implements the TURA deep graphite and Liquid Chrome aesthetic, Grotesque typography, and English headers. Once the serialization issue is fixed, the UI layout and features are high quality.
3. **Currency Conversion Core Math**:
   - The exchange rate (495.0), 1 KZT floor for micro-pricing, whole-integer rounding, space formatting, and negative price rejection in `commands/store.rs` are mathematically solid and pass unit tests.

---

## 4. Conclusion

### **VERDICT: REQUEST_CHANGES**

The Store & Currency implementation has a **Critical finding** that prevents approval:

### Findings

#### [Critical] Finding 1: Serialization Contract Mismatch (`rename_all = "camelCase"`)
- **What**: `StoreItem` and `StorePage` structs in `src-tauri/src/commands/store.rs` are decorated with `#[serde(rename_all = "camelCase")]`, converting all fields to camelCase over IPC.
- **Where**: `src-tauri/src/commands/store.rs`, lines 19 and 41.
- **Why**:
  1. Violates the interface contract in `tests/helpers/contracts.cjs:71`.
  2. Breaks runtime infinite scrolling (`res.has_more` is `undefined`, locking pagination at page 0).
  3. Breaks all store UI rendering (`item.price_formatted`, `item.header_image`, `item.install_uri`, `item.discount_percent` evaluate to `undefined`).
- **Required Action**: Remove `#[serde(rename_all = "camelCase")]` from `StoreItem` and `StorePage` in `src-tauri/src/commands/store.rs`, allowing them to serialize in snake_case to match `validateStorePage`, `store.svelte.ts`, and `StoreView.svelte`.

#### [Major] Finding 2: Self-Certifying Mock Testing in Feature 12 Spec
- **What**: `tests/tier1_features/f12_infinite_store.test.cjs` constructs a hardcoded JavaScript mock object instead of exercising the Rust IPC command serialization.
- **Where**: `tests/tier1_features/f12_infinite_store.test.cjs`, lines 7–21.
- **Why**: Permitted a critical communication breakdown between backend and frontend to go undetected despite 100% test pass scores.
- **Required Action**: Add a unit test in `src-tauri/tests/m3_unit_tests.rs` verifying that `serde_json::to_string(&StorePage)` outputs JSON containing snake_case keys (`has_more`, `app_id`, `price_raw`, `price_formatted`).

---

## 5. Verification Method

To independently verify the resolution of these findings:

1. **Verify Serde JSON Serialization**:
   Run a test or check `serde_json::to_string` on `get_store_recommendations`:
   ```powershell
   cargo test --test m3_unit_tests --manifest-path src-tauri/Cargo.toml
   ```
   Confirm JSON contains `"has_more": true`, `"app_id": 1091500`, `"price_formatted": "$..."`, `"header_image": "..."`.

2. **Verify Interface Contract Compatibility**:
   In Node.js, validate the serialized JSON against `validateStorePage`:
   ```powershell
   node -e "
   const { validateStorePage } = require('./tests/helpers/contracts.cjs');
   const page = JSON.parse(process.argv[1]);
   validateStorePage(page);
   " "<serialized_json_from_get_store_recommendations>"
   ```
   Must exit 0 without `AssertionError: Missing expected property: 'has_more'`.

3. **Verify Full Build and Test Suite**:
   ```powershell
   cargo check --manifest-path src-tauri/Cargo.toml
   cargo test --manifest-path src-tauri/Cargo.toml
   npx tsc --noEmit
   npm run build
   node tests/runner.cjs
   ```
   All must pass with 0 errors.

### Invalidation Conditions
- Any occurrence of `#[serde(rename_all = "camelCase")]` on `StoreItem` or `StorePage` that obscures `has_more` or `app_id`.
- `res.has_more` evaluating to `undefined` in `storeFeed.loadPage`.
- Game card price or cover image rendering as `undefined` in `StoreView.svelte`.
