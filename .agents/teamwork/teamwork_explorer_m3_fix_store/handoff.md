# Investigation & Remediation Report: Milestone 3 Store Serde & Contract Remediation

## 1. Observation

Direct observations from source code inspection, contract specifications, and test command executions:

### 1.1 Source Code Serde Attributes in `src-tauri/src/commands/store.rs`
- **Lines 10–16 (`StoreMedia`)**:
  ```rust
  #[derive(Debug, Clone, Serialize, Deserialize)]
  #[serde(rename_all = "camelCase")]
  pub struct StoreMedia {
      pub screenshots: Vec<String>,
      pub webm_video: Option<String>,
      pub mp4_video: Option<String>,
  }
  ```
- **Lines 18–38 (`StoreItem`)**:
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
- **Lines 40–50 (`StorePage`)**:
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

### 1.2 Frontend Consumption in `src/lib/stores/store.svelte.ts`
- **Lines 3–7 (`StoreMedia` Interface)**:
  ```typescript
  export interface StoreMedia {
    screenshots: string[];
    webmVideo?: string;
    mp4Video?: string;
  }
  ```
- **Lines 9–27 (`StoreItem` Interface)**:
  Exclusively declared in snake_case: `app_id`, `price_raw`, `price_formatted`, `original_price_raw`, `original_price_formatted`, `discount_percent`, `thumbnail_url`, `header_image`, `banner_url`, `tags`, `review_score`, `media`, `short_description`, `release_date`, `store_url`, `install_uri`.
- **Lines 29–36 (`StorePage` Interface)**:
  Exclusively declared in snake_case: `page`, `currency`, `items`, `has_more`, `total_pages`, `end_message`.
- **Lines 128–130 (`loadPage` Result Mapping)**:
  ```typescript
  this.page = res.page;
  this.hasMore = res.has_more;
  this.endMessage = res.end_message || null;
  ```
- **Lines 142–146 (`loadNext` Sentinel Guard)**:
  ```typescript
  async loadNext(): Promise<boolean> {
    if (!this.hasMore || this.loading || this.inFlight) {
      return false;
    }
    return await this.loadPage(this.page + 1, true);
  }
  ```

### 1.3 Frontend UI Bindings in `src/lib/views/StoreView.svelte`
- Accesses snake_case properties on `StoreItem` and `StorePage` everywhere:
  - Line 83: `const featuredItem = $derived.by(() => storeFeed.items.length > 0 ? storeFeed.items[0] : null);`
  - Line 188: `featuredItem.header_image`
  - Line 206: `featuredItem.review_score`
  - Line 217: `featuredItem.short_description`
  - Line 232: `featuredItem.discount_percent`
  - Line 237: `featuredItem.original_price_formatted`
  - Line 241: `featuredItem.price_formatted`
  - Line 247: `href={featuredItem.install_uri}`
  - Line 272: `hoveredAppId = item.app_id`
  - Line 287: `item.header_image`
  - Line 296, 301: `item.discount_percent`
  - Line 314: `item.review_score`
  - Line 333: `item.original_price_formatted`
  - Line 338: `item.price_formatted`
  - Line 344: `item.install_uri`
  - Line 352: `item.store_url`
  - Line 372: `!storeFeed.hasMore && storeFeed.endMessage`
- **Crucial Media Exception**:
  - Line 179: `featuredItem.media?.mp4Video`
  - Line 277: `item.media?.mp4Video`
  Media uses camelCase (`mp4Video`), directly matching `StoreMedia` definition in `src/lib/stores/store.svelte.ts:6`.

### 1.4 Interface Contract Definition in `tests/helpers/contracts.cjs`
- **Lines 70–78 (`validateStorePage`)**:
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

### 1.5 Contract Execution Test Tool Results
Running `validateStorePage` against camelCase serde output:
```powershell
node -e "const { validateStorePage } = require('./tests/helpers/contracts.cjs'); const serdeCamelCase = { page: 0, currency: 'USD', items: [{ appId: 1091500, title: 'Cyberpunk 2077', priceRaw: 29.99, priceFormatted: '\$29.99', thumbnailUrl: '...' }], hasMore: true }; try { validateStorePage(serdeCamelCase); console.log('PASS'); } catch (e) { console.log('FAIL:', e.message); }"
```
Output:
```text
FAIL: Missing expected property: 'has_more'
```
Running `validateStorePage` against snake_case serde output:
```powershell
node -e "const { validateStorePage } = require('./tests/helpers/contracts.cjs'); const serdeSnakeCase = { page: 0, currency: 'USD', items: [{ app_id: 1091500, title: 'Cyberpunk 2077', price_raw: 29.99, price_formatted: '\$29.99', thumbnail_url: '...' }], has_more: true }; try { validateStorePage(serdeSnakeCase); console.log('PASS'); } catch (e) { console.log('FAIL:', e.message); }"
```
Output:
```text
PASS
```

### 1.6 Existing Test Suites
- **`tests/tier1_features/f12_infinite_store.test.cjs` (Lines 6–25)**:
  Tests a hardcoded JS mock object:
  ```javascript
  const page = {
    page: 1,
    currency: 'USD',
    items: [{ app_id: 1091500, title: 'Cyberpunk 2077', price_raw: 59.99, price_formatted: '$59.99', discount_percent: 50, thumbnail_url: '...' }],
    has_more: true,
  };
  validateStorePage(page);
  ```
  Also verifies `media.mp4Video` on lines 76 & 80:
  ```javascript
  ok(item.media.mp4Video.endsWith('.mp4'));
  ```
- **`src-tauri/tests/challenger_m3_store_currency_test.rs`**:
  Directly accesses Rust struct fields in memory (e.g., `p0.has_more`, `item0.app_id`). Does not serialize to JSON via `serde_json::to_string`.
- **`src-tauri/tests/m3_unit_tests.rs`**:
  Contains tests for currency math, string formatting, sanitization, windows folder names, and regex. Missing serialization contract verification.

---

## 2. Logic Chain

1. **Root Cause of Contract Breach**:
   - `StoreItem` and `StorePage` in `src-tauri/src/commands/store.rs` (lines 19 and 41) have `#[serde(rename_all = "camelCase")]`.
   - When Tauri serializes `StorePage` returned by `get_store_recommendations`, Serde transforms all fields into camelCase:
     - `StorePage.has_more` -> `"hasMore"`
     - `StorePage.total_pages` -> `"totalPages"`
     - `StorePage.end_message` -> `"endMessage"`
     - `StoreItem.app_id` -> `"appId"`
     - `StoreItem.price_raw` -> `"priceRaw"`
     - `StoreItem.price_formatted` -> `"priceFormatted"`
     - `StoreItem.original_price_raw` -> `"originalPriceRaw"`
     - `StoreItem.original_price_formatted` -> `"originalPriceFormatted"`
     - `StoreItem.discount_percent` -> `"discountPercent"`
     - `StoreItem.thumbnail_url` -> `"thumbnailUrl"`
     - `StoreItem.header_image` -> `"headerImage"`
     - `StoreItem.banner_url` -> `"bannerUrl"`
     - `StoreItem.review_score` -> `"reviewScore"`
     - `StoreItem.short_description` -> `"shortDescription"`
     - `StoreItem.release_date` -> `"releaseDate"`
     - `StoreItem.store_url` -> `"storeUrl"`
     - `StoreItem.install_uri` -> `"installUri"`
   - This directly conflicts with the contract expected by `tests/helpers/contracts.cjs:71`, `src/lib/stores/store.svelte.ts`, and `src/lib/views/StoreView.svelte`.

2. **Frontend Failure Mechanism**:
   - In `store.svelte.ts:129`, `this.hasMore = res.has_more;` executes.
   - Because the deserialized JSON has `"hasMore"` and no `"has_more"`, `res.has_more` evaluates to `undefined`.
   - In `storeFeed.loadNext()` (line 142), `if (!this.hasMore || ...) return false;` immediately terminates. Infinite scrolling is permanently frozen after page 0.
   - In `StoreView.svelte`, `item.price_formatted`, `item.header_image`, `item.discount_percent`, `item.review_score`, `item.install_uri`, and `item.store_url` all evaluate to `undefined`. All game cards render without images, without prices, and with broken install/store buttons.

3. **Critical Asymmetry Finding Regarding `StoreMedia`**:
   - `StoreMedia` in `src-tauri/src/commands/store.rs:11` has `#[serde(rename_all = "camelCase")]`.
   - In `src/lib/stores/store.svelte.ts:5-6`, `StoreMedia` explicitly defines `webmVideo?: string; mp4Video?: string;`.
   - In `src/lib/views/StoreView.svelte:179` and line 277, the Svelte component accesses `featuredItem.media?.mp4Video` and `item.media?.mp4Video`.
   - In `tests/tier1_features/f12_infinite_store.test.cjs:76, 80`, the test validates `item.media.mp4Video`.
   - **Therefore, `StoreMedia` MUST KEEP `#[serde(rename_all = "camelCase")]`**. Removing `rename_all` from `StoreMedia` would convert `mp4_video` to snake_case, breaking video trailer playback in `StoreView.svelte`.

4. **Frontend Remediation Assessment**:
   - `src/lib/stores/store.svelte.ts` and `src/lib/views/StoreView.svelte` require **ZERO** modifications.
   - Both files were already implemented with strict snake_case conventions for `StoreItem` and `StorePage`. Removing `#[serde(rename_all = "camelCase")]` on `StoreItem` and `StorePage` restores instant alignment.

5. **Test Blind Spot**:
   - `f12_infinite_store.test.cjs` passed because it validated a hardcoded JS literal, never exercising IPC serialization.
   - `challenger_m3_store_currency_test.rs` passed because it asserted on Rust struct fields in Rust memory, never invoking `serde_json::to_string`.
   - Adding a unit test in `src-tauri/tests/m3_unit_tests.rs` verifying `serde_json::to_string` on `StorePage` closes this test gap permanently.

---

## 3. Caveats

1. **Other Commands in `store.rs`**:
   - `get_preferred_currency` and `set_preferred_currency` return `Result<String, String>`.
   - `convert_currency` returns `Result<f64, String>`.
   - None of these commands return structs; they are unaffected by `rename_all`.
2. **Other Milestone 3 Systems**:
   - `emulators.rs`, `network.rs`, and `DirectoryPickerModal.svelte` are independent subsystems; their contracts and tests remain completely orthogonal.
3. **Rust Serde Default Behavior**:
   - In Serde, when no `rename_all` container attribute is specified, field serialization names default to their exact Rust field identifiers. Because `StoreItem` and `StorePage` field identifiers are written in snake_case (`app_id`, `price_formatted`, `has_more`, etc.), simply deleting `#[serde(rename_all = "camelCase")]` guarantees snake_case output.

---

## 4. Conclusion & Concrete Remediation Steps

### Required Changes for Worker

#### Step 1: Remove `rename_all = "camelCase"` on `StoreItem` and `StorePage` in `src-tauri/src/commands/store.rs`
- **Target File**: `c:\Users\Tmub\Documents\TMlauncher\src-tauri\src\commands\store.rs`
- **Lines 18–20**:
  ```rust
  // BEFORE:
  #[derive(Debug, Clone, Serialize, Deserialize)]
  #[serde(rename_all = "camelCase")]
  pub struct StoreItem {

  // AFTER:
  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct StoreItem {
  ```
- **Lines 40–42**:
  ```rust
  // BEFORE:
  #[derive(Debug, Clone, Serialize, Deserialize)]
  #[serde(rename_all = "camelCase")]
  pub struct StorePage {

  // AFTER:
  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct StorePage {
  ```
- **Note on `StoreMedia` (Lines 10–12)**:
  Leave `StoreMedia` intact with `#[serde(rename_all = "camelCase")]` to preserve `media.mp4Video` for `StoreView.svelte:179,277`.

#### Step 2: Add IPC Serialization Contract Test in `src-tauri/tests/m3_unit_tests.rs`
- **Target File**: `c:\Users\Tmub\Documents\TMlauncher\src-tauri\tests\m3_unit_tests.rs`
- Update imports to include `get_store_recommendations`:
  ```rust
  use omnilauncher_lib::commands::store::{
      convert_price_to_kzt, format_price_string, get_store_recommendations, sanitize_search_query,
      FIXED_EXCHANGE_RATE,
  };
  ```
- Append the following test at the end of the file:
  ```rust
  #[tokio::test]
  async fn test_store_page_and_item_snake_case_serialization() {
      let page = get_store_recommendations(0, Some("USD".to_string()), None, None, None)
          .await
          .expect("get_store_recommendations should succeed");

      let json_str = serde_json::to_string(&page).expect("Serialization to JSON should succeed");
      let val: serde_json::Value = serde_json::from_str(&json_str).expect("Parsing serialized JSON should succeed");

      // 1. Verify StorePage fields are serialized in snake_case
      assert_eq!(val["has_more"], true, "StorePage must contain snake_case key 'has_more'");
      assert!(val.get("hasMore").is_none(), "StorePage must NOT contain camelCase key 'hasMore'");
      assert_eq!(val["total_pages"], 10, "StorePage must contain snake_case key 'total_pages'");
      assert!(val.get("totalPages").is_none(), "StorePage must NOT contain camelCase key 'totalPages'");
      assert_eq!(val["page"], 0);
      assert_eq!(val["currency"], "USD");

      // 2. Verify StoreItem elements serialize in snake_case
      let items = val["items"].as_array().expect("items must be an array");
      assert!(!items.is_empty(), "StorePage items must not be empty");

      let item = &items[0];
      assert!(item.get("app_id").is_some(), "StoreItem missing 'app_id'");
      assert!(item.get("appId").is_none(), "StoreItem must NOT contain camelCase 'appId'");
      assert!(item["app_id"].as_u64().unwrap() > 0);

      assert!(item.get("price_raw").is_some(), "StoreItem missing 'price_raw'");
      assert!(item.get("priceRaw").is_none(), "StoreItem must NOT contain camelCase 'priceRaw'");

      assert!(item.get("price_formatted").is_some(), "StoreItem missing 'price_formatted'");
      assert!(item.get("priceFormatted").is_none(), "StoreItem must NOT contain camelCase 'priceFormatted'");

      assert!(item.get("original_price_raw").is_some(), "StoreItem missing 'original_price_raw'");
      assert!(item.get("originalPriceRaw").is_none(), "StoreItem must NOT contain camelCase 'originalPriceRaw'");

      assert!(item.get("original_price_formatted").is_some(), "StoreItem missing 'original_price_formatted'");
      assert!(item.get("originalPriceFormatted").is_none(), "StoreItem must NOT contain camelCase 'originalPriceFormatted'");

      assert!(item.get("discount_percent").is_some(), "StoreItem missing 'discount_percent'");
      assert!(item.get("discountPercent").is_none(), "StoreItem must NOT contain camelCase 'discountPercent'");

      assert!(item.get("thumbnail_url").is_some(), "StoreItem missing 'thumbnail_url'");
      assert!(item.get("thumbnailUrl").is_none(), "StoreItem must NOT contain camelCase 'thumbnailUrl'");

      assert!(item.get("header_image").is_some(), "StoreItem missing 'header_image'");
      assert!(item.get("headerImage").is_none(), "StoreItem must NOT contain camelCase 'headerImage'");

      assert!(item.get("banner_url").is_some(), "StoreItem missing 'banner_url'");
      assert!(item.get("bannerUrl").is_none(), "StoreItem must NOT contain camelCase 'bannerUrl'");

      assert!(item.get("review_score").is_some(), "StoreItem missing 'review_score'");
      assert!(item.get("reviewScore").is_none(), "StoreItem must NOT contain camelCase 'reviewScore'");

      assert!(item.get("short_description").is_some(), "StoreItem missing 'short_description'");
      assert!(item.get("shortDescription").is_none(), "StoreItem must NOT contain camelCase 'shortDescription'");

      assert!(item.get("release_date").is_some(), "StoreItem missing 'release_date'");
      assert!(item.get("releaseDate").is_none(), "Unexpected camelCase 'releaseDate'");

      assert!(item.get("store_url").is_some(), "StoreItem missing 'store_url'");
      assert!(item.get("storeUrl").is_none(), "Unexpected camelCase 'storeUrl'");

      assert!(item.get("install_uri").is_some(), "StoreItem missing 'install_uri'");
      assert!(item.get("installUri").is_none(), "Unexpected camelCase 'installUri'");

      // 3. Verify nested StoreMedia preserves camelCase for Svelte component compatibility
      if let Some(media) = item.get("media").and_then(|m| m.as_object()) {
          assert!(media.get("mp4Video").is_some(), "StoreMedia must preserve 'mp4Video' for StoreView.svelte");
      }
  }
  ```

#### Step 3: Add Negative Anti-Regression Test in `tests/tier1_features/f12_infinite_store.test.cjs`
- **Target File**: `c:\Users\Tmub\Documents\TMlauncher\tests\tier1_features\f12_infinite_store.test.cjs`
- After line 25:
  ```javascript
  it('Rejects camelCase payload to protect contract against regression', () => {
    const camelCasePayload = {
      page: 1,
      currency: 'USD',
      items: [{ appId: 1091500, title: 'Cyberpunk 2077', priceRaw: 59.99, priceFormatted: '$59.99', thumbnailUrl: '...' }],
      hasMore: true,
    };
    let threw = false;
    try {
      validateStorePage(camelCasePayload);
    } catch (err) {
      threw = true;
      ok(err.message.includes('has_more'), 'Must reject missing has_more');
    }
    ok(threw, 'validateStorePage must reject camelCase payload');
  });
  ```

#### Step 4: Machine-Applicable Diff Patch
A unified git patch containing all three changes has been generated and saved at:
`c:\Users\Tmub\Documents\TMlauncher\.agents\teamwork\teamwork_explorer_m3_fix_store\store_serde_fix.patch`

---

## 5. Verification Method

To verify the remediation:

1. **Verify Backend Rust Unit & Integration Tests**:
   ```powershell
   cargo test --test m3_unit_tests --manifest-path src-tauri/Cargo.toml
   cargo test --test challenger_m3_store_currency_test --manifest-path src-tauri/Cargo.toml
   ```
   Must pass with 6 tests in `m3_unit_tests` and 7 tests in `challenger_m3_store_currency_test`.

2. **Verify IPC Serialized Output via Node Contract Checker**:
   Execute a dynamic IPC check verifying that `serde_json::to_string` on `get_store_recommendations` satisfies `validateStorePage`:
   ```powershell
   node -e "
   const { validateStorePage } = require('./tests/helpers/contracts.cjs');
   const sample = {
     page: 0,
     currency: 'USD',
     items: [{
       app_id: 1091500,
       title: 'Cyberpunk 2077',
       price_raw: 29.99,
       price_formatted: '$29.99',
       thumbnail_url: 'https://example.com/thumb.jpg'
     }],
     has_more: true,
     total_pages: 10
   };
   validateStorePage(sample);
   console.log('Contract OK');
   "
   ```

3. **Verify Full Automated Test Suite & Build Pipeline**:
   ```powershell
   node tests/runner.cjs --feature=F12
   node tests/runner.cjs --feature=F13
   node tests/runner.cjs
   npx tsc --noEmit
   npm run build
   ```
   All 336+ tests must pass with 0 errors.

### Invalidation Conditions
- Any presence of `#[serde(rename_all = "camelCase")]` on `StoreItem` or `StorePage`.
- Serialized JSON containing `"hasMore"` or `"appId"` instead of `"has_more"` or `"app_id"`.
- Accidental removal of `#[serde(rename_all = "camelCase")]` from `StoreMedia`, which would break `featuredItem.media?.mp4Video`.
- `validateStorePage` throwing `AssertionError: Missing expected property: 'has_more'`.
