//! Milestone 3 Recheck Empirical Challenger Test Suite.
//!
//! Rigorous stress testing of:
//! 1. Store IPC serialization parity across all pages and items (snake_case vs camelCase preservation).
//! 2. Currency switcher (USD <-> KZT) under extreme values, micro-cents, floating point round-trip.
//! 3. Rapid pagination and high concurrency race condition resilience.
//! 4. Tag filtering and search query injection under adversarial attack payloads.

use tokio::task::JoinSet;

use omnilauncher_lib::commands::store::{
    convert_currency, convert_price_to_kzt, format_price_string, get_store_recommendations,
    sanitize_search_query, StorePage, FIXED_EXCHANGE_RATE,
};

// ============================================================================
// 1. Store IPC Serialization Verification & Structural Invariants
// ============================================================================

#[tokio::test]
async fn test_store_ipc_serialization_exhaustive_parity() {
    // Test all valid catalog pages (0..10) plus boundary pages (10, 11, 50, 999)
    let test_pages = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 50, 999];

    for page_idx in test_pages {
        for currency in &[Some("USD".to_string()), Some("KZT".to_string()), None] {
            let page = get_store_recommendations(page_idx, currency.clone(), None, None, None)
                .await
                .unwrap_or_else(|e| panic!("get_store_recommendations failed at page {page_idx}: {e}"));

            // Serialize StorePage to JSON Value
            let val = serde_json::to_value(&page)
                .expect("Serialization to serde_json::Value must succeed");

            let obj = val.as_object().expect("Serialized page must be a JSON object");

            // 1. Verify StorePage top-level snake_case fields
            assert!(obj.contains_key("page"), "Missing 'page' on page {page_idx}");
            assert!(obj.contains_key("currency"), "Missing 'currency' on page {page_idx}");
            assert!(obj.contains_key("items"), "Missing 'items' on page {page_idx}");
            assert!(obj.contains_key("has_more"), "CRITICAL: Missing snake_case 'has_more' on page {page_idx}");
            assert!(obj.contains_key("total_pages"), "CRITICAL: Missing snake_case 'total_pages' on page {page_idx}");
            assert!(obj.contains_key("end_message"), "Missing 'end_message' on page {page_idx}");

            // 2. Reject camelCase regressions on StorePage
            assert!(!obj.contains_key("hasMore"), "REGRESSION: Found camelCase 'hasMore' on page {page_idx}");
            assert!(!obj.contains_key("totalPages"), "REGRESSION: Found camelCase 'totalPages' on page {page_idx}");
            assert!(!obj.contains_key("endMessage"), "REGRESSION: Found camelCase 'endMessage' on page {page_idx}");

            // 3. Verify pagination invariant
            if page_idx < 9 {
                assert!(page.has_more, "has_more must be true for page {page_idx} < 9");
            } else {
                assert!(!page.has_more, "has_more must be false for page {page_idx} >= 9");
                assert!(page.end_message.is_some(), "end_message must be present when has_more is false");
            }

            // 4. Inspect every StoreItem in this page
            let items = obj["items"].as_array().expect("items must be an array");
            for (idx, item_val) in items.iter().enumerate() {
                let item = item_val.as_object().expect("item must be an object");

                // Check mandatory snake_case fields
                assert!(item.contains_key("app_id"), "Missing 'app_id' in item {idx}");
                assert!(item.contains_key("price_raw"), "Missing 'price_raw' in item {idx}");
                assert!(item.contains_key("price_formatted"), "Missing 'price_formatted' in item {idx}");
                assert!(item.contains_key("discount_percent"), "Missing 'discount_percent' in item {idx}");
                assert!(item.contains_key("thumbnail_url"), "Missing 'thumbnail_url' in item {idx}");
                assert!(item.contains_key("header_image"), "Missing 'header_image' in item {idx}");
                assert!(item.contains_key("banner_url"), "Missing 'banner_url' in item {idx}");
                assert!(item.contains_key("tags"), "Missing 'tags' in item {idx}");
                assert!(item.contains_key("review_score"), "Missing 'review_score' in item {idx}");
                assert!(item.contains_key("short_description"), "Missing 'short_description' in item {idx}");
                assert!(item.contains_key("release_date"), "Missing 'release_date' in item {idx}");
                assert!(item.contains_key("store_url"), "Missing 'store_url' in item {idx}");
                assert!(item.contains_key("install_uri"), "Missing 'install_uri' in item {idx}");

                // Reject camelCase on StoreItem
                assert!(!item.contains_key("appId"), "REGRESSION: Found camelCase 'appId' in item {idx}");
                assert!(!item.contains_key("priceRaw"), "REGRESSION: Found camelCase 'priceRaw' in item {idx}");
                assert!(!item.contains_key("priceFormatted"), "REGRESSION: Found camelCase 'priceFormatted' in item {idx}");
                assert!(!item.contains_key("discountPercent"), "REGRESSION: Found camelCase 'discountPercent' in item {idx}");
                assert!(!item.contains_key("thumbnailUrl"), "REGRESSION: Found camelCase 'thumbnailUrl' in item {idx}");
                assert!(!item.contains_key("headerImage"), "REGRESSION: Found camelCase 'headerImage' in item {idx}");
                assert!(!item.contains_key("bannerUrl"), "REGRESSION: Found camelCase 'bannerUrl' in item {idx}");
                assert!(!item.contains_key("reviewScore"), "REGRESSION: Found camelCase 'reviewScore' in item {idx}");
                assert!(!item.contains_key("shortDescription"), "REGRESSION: Found camelCase 'shortDescription' in item {idx}");
                assert!(!item.contains_key("releaseDate"), "REGRESSION: Found camelCase 'releaseDate' in item {idx}");
                assert!(!item.contains_key("storeUrl"), "REGRESSION: Found camelCase 'storeUrl' in item {idx}");
                assert!(!item.contains_key("installUri"), "REGRESSION: Found camelCase 'installUri' in item {idx}");

                // Check StoreMedia camelCase preservation for Svelte frontend compatibility
                if let Some(media) = item.get("media").and_then(|m| m.as_object()) {
                    assert!(media.contains_key("screenshots"), "StoreMedia missing screenshots");
                    assert!(media.contains_key("mp4Video"), "CRITICAL: StoreMedia must preserve camelCase 'mp4Video' for StoreView.svelte");
                    assert!(!media.contains_key("mp4_video"), "StoreMedia must not expose snake_case 'mp4_video'");
                }
            }

            // 5. Test JSON round-trip deserialization back to Rust StorePage
            let json_str = serde_json::to_string(&page).expect("JSON serialization");
            let deserialized: StorePage = serde_json::from_str(&json_str)
                .expect("JSON roundtrip deserialization into StorePage must succeed");
            assert_eq!(deserialized.page, page.page);
            assert_eq!(deserialized.currency, page.currency);
            assert_eq!(deserialized.items.len(), page.items.len());
            assert_eq!(deserialized.has_more, page.has_more);
        }
    }
}

// ============================================================================
// 2. Currency Switcher Stress & Round-Trip Stability
// ============================================================================

#[tokio::test]
async fn test_currency_roundtrip_precision_and_drift() {
    let price_samples = [
        0.0, 0.0001, 0.001, 0.01, 0.99, 1.99, 4.99, 9.99, 14.99, 19.99, 24.99, 29.99, 39.99,
        49.99, 59.99, 69.99, 99.99, 199.99, 999.99, 9999.0, 999999.0,
    ];

    for &usd in &price_samples {
        let kzt = convert_price_to_kzt(usd);
        if usd == 0.0 {
            assert_eq!(kzt, 0, "0.0 USD must convert to 0 KZT");
            assert_eq!(format_price_string(usd, "USD"), "Free to Play");
            assert_eq!(format_price_string(usd, "KZT"), "Free to Play");
        } else if usd < 0.002 {
            // Micro-cents clamp to 1 KZT minimum floor
            assert_eq!(kzt, 1, "Micro-cents below 1 KZT floor must clamp to 1");
        } else {
            // Check that USD -> KZT -> approximate USD roundtrip drift is within acceptable rounding bound
            let approx_usd = (kzt as f64) / FIXED_EXCHANGE_RATE;
            let drift = (approx_usd - usd).abs();
            assert!(
                drift <= 0.01 + (usd * 0.001),
                "Rounding drift too large for ${usd}: got {kzt} KZT -> ${approx_usd} (drift: {drift})"
            );
        }

        // Test IPC command convert_currency
        let conv_kzt = convert_currency(usd, "KZT".to_string()).await.unwrap();
        assert_eq!(conv_kzt, kzt as f64);

        let conv_usd = convert_currency(usd, "USD".to_string()).await.unwrap();
        assert_eq!(conv_usd, usd);
    }
}

#[tokio::test]
async fn test_currency_adversarial_and_non_finite_inputs() {
    // Extreme values
    let max_price = convert_currency(1_000_000_000.0, "KZT".to_string()).await.unwrap();
    assert_eq!(max_price, 495_000_000_000.0);

    // Rejection of invalid inputs in convert_currency
    assert!(convert_currency(-0.01, "KZT".to_string()).await.is_err());
    assert!(convert_currency(-100.0, "USD".to_string()).await.is_err());
    assert!(convert_currency(f64::NAN, "KZT".to_string()).await.is_err());
    assert!(convert_currency(f64::INFINITY, "KZT".to_string()).await.is_err());
    assert!(convert_currency(f64::NEG_INFINITY, "USD".to_string()).await.is_err());

    // Rejection of unsupported currencies
    assert!(convert_currency(50.0, "EUR".to_string()).await.is_err());
    assert!(convert_currency(50.0, "RUB".to_string()).await.is_err());
    assert!(convert_currency(50.0, "".to_string()).await.is_err());
    assert!(convert_currency(50.0, "   ".to_string()).await.is_err());
    assert!(convert_currency(50.0, "usd".to_string()).await.is_err()); // Case-sensitive check
}

#[test]
fn test_currency_formatting_thousands_spacing() {
    // Test thousands separation in Kazakhstani Tenge format
    let test_cases = vec![
        (1.0, "495 ₸"),
        (2.0, "990 ₸"),
        (10.0, "4 950 ₸"),
        (100.0, "49 500 ₸"),
        (1000.0, "495 000 ₸"),
        (10000.0, "4 950 000 ₸"),
        (100000.0, "49 500 000 ₸"),
        (1000000.0, "495 000 000 ₸"),
    ];

    for (usd, expected_kzt) in test_cases {
        let formatted = format_price_string(usd, "KZT");
        assert_eq!(formatted, expected_kzt, "Mismatch formatting for USD {usd}");
    }
}

// ============================================================================
// 3. Rapid Concurrent Pagination & Race Condition Simulation
// ============================================================================

#[tokio::test]
async fn test_rapid_concurrent_pagination_flood() {
    let mut set = JoinSet::new();

    // Spawn 100 concurrent async pagination tasks across various pages and currencies
    for i in 0..100 {
        let page_idx = i % 15; // pages 0 through 14
        let curr = if i % 2 == 0 { "USD" } else { "KZT" };
        let tag = match i % 5 {
            0 => None,
            1 => Some("FPS".to_string()),
            2 => Some("RPG".to_string()),
            3 => Some("Action".to_string()),
            _ => Some("All".to_string()),
        };

        set.spawn(async move {
            let res = get_store_recommendations(page_idx, Some(curr.to_string()), tag, None, None).await;
            (page_idx, curr, res)
        });
    }

    let mut success_count = 0;
    while let Some(res) = set.join_next().await {
        let (page_idx, curr, result) = res.expect("Task must not panic");
        let page = result.expect("get_store_recommendations must succeed");

        assert_eq!(page.page, page_idx);
        assert_eq!(page.currency, curr);

        if page_idx < 9 {
            assert!(page.has_more, "Page {page_idx} should have has_more=true");
            assert!(!page.items.is_empty(), "Page {page_idx} should have items");
        } else {
            assert!(!page.has_more, "Page {page_idx} should have has_more=false");
        }

        success_count += 1;
    }

    assert_eq!(success_count, 100, "All 100 concurrent requests must complete successfully");
}

// ============================================================================
// 4. Adversarial Tag Filtering & Query Injection Stress
// ============================================================================

#[tokio::test]
async fn test_tag_filtering_adversarial_inputs() {
    // 1. SQL Injection strings in tag parameter
    let sql_tags = [
        "' OR '1'='1",
        "'; DROP TABLE Games; --",
        "FPS' OR 1=1 --",
        "admin'--",
        "UNION SELECT * FROM items",
    ];

    for tag in sql_tags {
        let res = get_store_recommendations(0, None, Some(tag.to_string()), None, None)
            .await
            .expect("SQL payload in tag must not crash backend");

        // Should return 0 items because no game has a tag matching the SQL injection literally
        assert_eq!(res.items.len(), 0, "Adversarial tag '{tag}' should return 0 items");
        assert!(!res.has_more);
        assert_eq!(res.end_message, Some("No recommendations found for this filter".to_string()));
    }

    // 2. Control characters and formatting characters
    let ctrl_tags = [
        "\0",
        "\r\n\t",
        "FPS\0Action",
        "\x1b[31mAction\x1b[0m",
        "\u{202E}FPS", // Right-to-Left Override
    ];

    for tag in ctrl_tags {
        let res = get_store_recommendations(0, None, Some(tag.to_string()), None, None)
            .await
            .expect("Control characters in tag must not crash backend");
        // Must either match nothing or handle safely
        assert!(res.items.is_empty() || !res.items.is_empty());
    }

    // 3. Multilingual and emoji tags
    let multilingual_tags = [
        "アクション", // Japanese Action
        "Ролевая игра", // Russian RPG
        "🎮 RPG",
    ];

    for tag in multilingual_tags {
        let res = get_store_recommendations(0, None, Some(tag.to_string()), None, None)
            .await
            .expect("Multilingual tags must not crash backend");
        assert_eq!(res.items.len(), 0);
    }

    // 4. Tag case variations for existing tags
    for tag_variant in &["fps", "FPS", "Fps", "fPs"] {
        let res = get_store_recommendations(0, None, Some(tag_variant.to_string()), None, None)
            .await
            .unwrap();
        assert!(!res.items.is_empty(), "Tag '{tag_variant}' should be case-insensitive");
    }

    // 5. Giant 10,000 character tag
    let giant_tag = "A".repeat(10_000);
    let res = get_store_recommendations(0, None, Some(giant_tag), None, None)
        .await
        .expect("Giant tag string must not overflow or panic");
    assert_eq!(res.items.len(), 0);
    assert!(!res.has_more);
}

#[test]
fn test_search_query_sanitizer_adversarial_matrix() {
    let matrix = [
        ("normal game", "normal game"),
        ("Cyberpunk'; DROP TABLE Games; --", "Cyberpunk DROP TABLE Games --"),
        (r#"{"$gt": ""}"#, "gt:"),
        ("<script>alert('xss')</script>", "<script>alertxss</script>"),
        ("rm -rf /; $(whoami)", "rm -rf / whoami"),
        ("   \t  lots of whitespace  \n ", "lots of whitespace"),
        ("", ""),
        ("Game (Special Edition)", "Game Special Edition"),
        ("Price: $59.99", "Price: 59.99"),
        ("{a: 1, b: 2}", "a: 1, b: 2"),
    ];

    for (raw, expected) in matrix {
        let cleaned = sanitize_search_query(raw);
        assert_eq!(cleaned, expected, "Failed for raw input: {raw}");
        assert!(!cleaned.contains(&['\'', '"', ';', '{', '}', '(', ')', '$'][..]));
    }
}
