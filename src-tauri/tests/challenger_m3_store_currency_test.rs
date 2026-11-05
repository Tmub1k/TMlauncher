//! Milestone 3 Store & Currency Adversarial Empirical Challenge Suite.
//!
//! Evaluates:
//! 1. Currency calculation precision (micro-cents, massive numbers, negative values, non-finites).
//! 2. KZT integer rounding and currency symbol formatting (`₸`, `$`).
//! 3. Store search query injection attacks (SQL, NoSQL, shell metacharacters, HTML tags, long strings).
//! 4. Rapid pagination race conditions, out-of-range page requests (e.g. page 9999), empty result handling, catalog exhaustion banner.
//! 5. Tag filter boundary tests (empty tags, unknown tags, case sensitivity, injection strings).

use omnilauncher_lib::commands::store::{
    convert_currency, convert_price_to_kzt, format_price_string, get_store_recommendations,
    sanitize_search_query, FIXED_EXCHANGE_RATE,
};

#[test]
fn test_currency_precision_and_micro_pricing() {
    assert_eq!(FIXED_EXCHANGE_RATE, 495.0);

    // Standard game pricing
    assert_eq!(convert_price_to_kzt(59.99), 29695);
    assert_eq!(convert_price_to_kzt(69.99), 34645);
    assert_eq!(convert_price_to_kzt(19.99), 9895);
    assert_eq!(convert_price_to_kzt(9.99), 4945);

    // Micro-cents ($0.01 and $0.001)
    // 0.01 * 495.0 = 4.95 -> rounds to 5
    assert_eq!(convert_price_to_kzt(0.01), 5);
    // 0.001 * 495.0 = 0.495 -> rounds to 0.0, but clamped to minimum 1 KZT floor
    assert_eq!(convert_price_to_kzt(0.001), 1);
    assert_eq!(convert_price_to_kzt(0.0001), 1);
    assert_eq!(convert_price_to_kzt(0.000001), 1);

    // Free to play (0.0 USD) -> 0 KZT
    assert_eq!(convert_price_to_kzt(0.0), 0);
    assert_eq!(convert_price_to_kzt(-0.0), 0);

    // Negative values clamped to 0
    assert_eq!(convert_price_to_kzt(-0.01), 0);
    assert_eq!(convert_price_to_kzt(-59.99), 0);
    assert_eq!(convert_price_to_kzt(-99999.0), 0);

    // Massive values ($99,999,999.00) without 32-bit or 64-bit overflow
    let massive = convert_price_to_kzt(99_999_999.0);
    assert_eq!(massive, 49_499_999_505);

    // Even larger numbers (1 trillion USD)
    let trillion = convert_price_to_kzt(1_000_000_000_000.0);
    assert_eq!(trillion, 495_000_000_000_000);

    // Non-finite values: must not panic or cause SIGFPE
    let inf_val = convert_price_to_kzt(f64::INFINITY);
    assert!(inf_val > 0, "Infinity converts without panic");

    let neg_inf_val = convert_price_to_kzt(f64::NEG_INFINITY);
    assert_eq!(neg_inf_val, 0, "Negative infinity returns 0");

    let nan_val = convert_price_to_kzt(f64::NAN);
    assert_eq!(nan_val, 1, "NaN evaluates safely without panic");
}

#[test]
fn test_currency_symbol_formatting_and_grouping() {
    // 0 USD / KZT
    assert_eq!(format_price_string(0.0, "USD"), "Free to Play");
    assert_eq!(format_price_string(0.0, "KZT"), "Free to Play");

    // USD format: dollar symbol, 2 decimal places
    assert_eq!(format_price_string(59.99, "USD"), "$59.99");
    assert_eq!(format_price_string(0.01, "USD"), "$0.01");
    assert_eq!(format_price_string(99999999.0, "USD"), "$99999999.00");

    // KZT format: tenge symbol ₸, integer rounding, space-separated thousands
    let kzt_59 = format_price_string(59.99, "KZT");
    assert_eq!(kzt_59, "29 695 ₸");

    let kzt_micro = format_price_string(0.001, "KZT");
    assert_eq!(kzt_micro, "1 ₸");

    let kzt_huge = format_price_string(99_999_999.0, "KZT");
    assert_eq!(kzt_huge, "49 499 999 505 ₸");

    // Verify negative values in format_price_string
    let kzt_neg = format_price_string(-50.0, "KZT");
    assert_eq!(kzt_neg, "0 ₸");

    let usd_neg = format_price_string(-50.0, "USD");
    assert_eq!(usd_neg, "$-50.00");
}

#[test]
fn test_search_query_injection_sanitization() {
    // 1. SQL injection attacks
    let sql1 = "Cyberpunk'; DROP TABLE Games; --";
    assert_eq!(sanitize_search_query(sql1), "Cyberpunk DROP TABLE Games --");

    let sql2 = "' OR '1'='1";
    assert_eq!(sanitize_search_query(sql2), "OR 1=1");

    let sql3 = "UNION SELECT null, username, password FROM users --";
    assert_eq!(sanitize_search_query(sql3), "UNION SELECT null, username, password FROM users --");

    // 2. NoSQL injection attacks
    let nosql1 = r#"{"$gt": ""}"#;
    assert_eq!(sanitize_search_query(nosql1), "gt:");

    let nosql2 = r#"{"$where": "sleep(5000)"}"#;
    assert_eq!(sanitize_search_query(nosql2), "where: sleep5000");

    // 3. Shell metacharacters
    let shell1 = "rm -rf /; $(whoami); ${PATH}; `calc.exe`";
    let shell_cleaned = sanitize_search_query(shell1);
    assert!(!shell_cleaned.contains(';'));
    assert!(!shell_cleaned.contains('$'));
    assert!(!shell_cleaned.contains('{'));
    assert!(!shell_cleaned.contains('}'));
    assert!(!shell_cleaned.contains('('));
    assert!(!shell_cleaned.contains(')'));

    // 4. HTML / XSS Injection tags
    let xss1 = "<script>alert('XSS')</script>";
    let xss_cleaned = sanitize_search_query(xss1);
    assert!(!xss_cleaned.contains('\''));
    assert!(!xss_cleaned.contains('('));
    assert!(!xss_cleaned.contains(')'));
    assert_eq!(xss_cleaned, "<script>alertXSS</script>");

    // 5. Empty & whitespace strings
    assert_eq!(sanitize_search_query(""), "");
    assert_eq!(sanitize_search_query("   \t\n  "), "");

    // 6. Non-ASCII characters (Cyrillic, CJK, Emojis)
    assert_eq!(sanitize_search_query("Ведьмак 3"), "Ведьмак 3");
    assert_eq!(sanitize_search_query("ゼルダの伝説"), "ゼルダの伝説");
    assert_eq!(sanitize_search_query("🎮 Game 🚀"), "🎮 Game 🚀");

    // 7. Extremely long string (50,000 chars)
    let long_str = "a".repeat(50_000);
    let cleaned_long = sanitize_search_query(&long_str);
    assert_eq!(cleaned_long.len(), 50_000);
}

#[tokio::test]
async fn test_get_store_recommendations_pagination_and_exhaustion() {
    // Page 0: initial load
    let p0 = get_store_recommendations(0, Some("USD".to_string()), None, None, None).await.unwrap();
    assert_eq!(p0.page, 0);
    assert_eq!(p0.currency, "USD");
    assert_eq!(p0.items.len(), 6);
    assert_eq!(p0.has_more, true);
    assert_eq!(p0.total_pages, 10);
    assert!(p0.end_message.is_none());

    // Verify first item in page 0
    let item0 = &p0.items[0];
    assert!(item0.app_id > 0);
    assert!(!item0.title.is_empty());
    assert!(item0.price_formatted.starts_with('$') || item0.price_formatted == "Free to Play");
    assert!(item0.thumbnail_url.contains("capsule"));
    assert!(item0.store_url.starts_with("https://store.steampowered.com/app/"));
    assert!(item0.install_uri.starts_with("steam://install/"));

    // Page 8: approaching end
    let p8 = get_store_recommendations(8, Some("USD".to_string()), None, None, None).await.unwrap();
    assert_eq!(p8.page, 8);
    assert_eq!(p8.items.len(), 6);
    assert_eq!(p8.has_more, true);

    // Page 9: last page before exhaustion
    let p9 = get_store_recommendations(9, Some("USD".to_string()), None, None, None).await.unwrap();
    assert_eq!(p9.page, 9);
    assert_eq!(p9.items.len(), 6);
    assert_eq!(p9.has_more, false, "Page 9 has_more must be false (total 10 pages)");
    assert_eq!(
        p9.end_message,
        Some("You have caught up with all current recommendations".to_string())
    );

    // Page 10: catalog exhaustion boundary
    let p10 = get_store_recommendations(10, Some("USD".to_string()), None, None, None).await.unwrap();
    assert_eq!(p10.page, 10);
    assert_eq!(p10.items.len(), 0, "Exhausted page 10 must return empty items");
    assert_eq!(p10.has_more, false);
    assert_eq!(
        p10.end_message,
        Some("You have caught up with all current recommendations".to_string())
    );

    // Out-of-range page request: page 9999
    let p9999 = get_store_recommendations(9999, Some("USD".to_string()), None, None, None).await.unwrap();
    assert_eq!(p9999.page, 9999);
    assert_eq!(p9999.items.len(), 0);
    assert_eq!(p9999.has_more, false);
    assert_eq!(
        p9999.end_message,
        Some("You have caught up with all current recommendations".to_string())
    );

    // Extreme page request: usize::MAX - 1 (overflow protection check)
    let p_extreme = get_store_recommendations(usize::MAX - 1, None, None, None, None).await.unwrap();
    assert_eq!(p_extreme.items.len(), 0);
    assert_eq!(p_extreme.has_more, false);
}

#[tokio::test]
async fn test_get_store_recommendations_currency_toggle() {
    // USD mode
    let usd_page = get_store_recommendations(0, Some("USD".to_string()), None, None, None).await.unwrap();
    assert_eq!(usd_page.currency, "USD");
    for item in &usd_page.items {
        if item.price_raw == 0.0 {
            assert_eq!(item.price_formatted, "Free to Play");
        } else {
            assert!(item.price_formatted.starts_with('$'));
        }
    }

    // KZT mode
    let kzt_page = get_store_recommendations(0, Some("KZT".to_string()), None, None, None).await.unwrap();
    assert_eq!(kzt_page.currency, "KZT");
    for item in &kzt_page.items {
        if item.price_raw == 0.0 {
            assert_eq!(item.price_formatted, "Free to Play");
        } else {
            assert!(item.price_formatted.ends_with("₸"), "KZT price must end with ₸ symbol");
            // Must have integer raw price in KZT
            assert_eq!(item.price_raw.fract(), 0.0, "KZT price_raw must be an integer");
        }
    }

    // Fallback on invalid currency string
    let fallback_page = get_store_recommendations(0, Some("INVALID_CURRENCY".to_string()), None, None, None).await.unwrap();
    assert_eq!(fallback_page.currency, "USD", "Invalid currency falls back to USD");

    // Empty string currency
    let empty_curr_page = get_store_recommendations(0, Some("".to_string()), None, None, None).await.unwrap();
    assert_eq!(empty_curr_page.currency, "USD");
}

#[tokio::test]
async fn test_get_store_recommendations_tag_and_query_boundary_filtering() {
    // 1. Tag filtering
    let fps_page = get_store_recommendations(0, None, Some("FPS".to_string()), None, None).await.unwrap();
    assert!(!fps_page.items.is_empty());
    for item in &fps_page.items {
        assert!(item.tags.iter().any(|t| t.eq_ignore_ascii_case("FPS")));
    }

    // Tag case insensitivity
    let fps_lower = get_store_recommendations(0, None, Some("fps".to_string()), None, None).await.unwrap();
    assert_eq!(fps_page.items.len(), fps_lower.items.len());

    // Tag 'All' returns full catalog
    let all_tag_page = get_store_recommendations(0, None, Some("All".to_string()), None, None).await.unwrap();
    assert_eq!(all_tag_page.items.len(), 6);

    // Empty tag returns full catalog
    let empty_tag_page = get_store_recommendations(0, None, Some("".to_string()), None, None).await.unwrap();
    assert_eq!(empty_tag_page.items.len(), 6);

    // Whitespace tag returns full catalog
    let ws_tag_page = get_store_recommendations(0, None, Some("   ".to_string()), None, None).await.unwrap();
    assert_eq!(ws_tag_page.items.len(), 6);

    // Unknown / non-existent tag
    let unknown_tag = get_store_recommendations(0, None, Some("UnknownGenre999".to_string()), None, None).await.unwrap();
    assert_eq!(unknown_tag.items.len(), 0);
    assert_eq!(unknown_tag.has_more, false);
    assert_eq!(unknown_tag.total_pages, 0);
    assert_eq!(
        unknown_tag.end_message,
        Some("No recommendations found for this filter".to_string())
    );

    // 2. Query filtering with injection strings
    // 'Cyberpunk'; sanitizes to 'Cyberpunk' and matches Cyberpunk 2077
    let single_injected = get_store_recommendations(
        0,
        None,
        None,
        Some("Cyberpunk';".to_string()),
        None,
    ).await.unwrap();
    assert!(!single_injected.items.is_empty(), "Sanitized query Cyberpunk'; matches Cyberpunk 2077");
    assert!(single_injected.items[0].title.contains("Cyberpunk"));

    // Multi-word SQL injection safely executes without error and matches 0 items
    let injected_query = get_store_recommendations(
        0,
        None,
        None,
        Some("Cyberpunk'; DROP TABLE Games; --".to_string()),
        None,
    ).await.unwrap();
    assert_eq!(injected_query.items.len(), 0, "SQL payload safely produces empty results");
    assert_eq!(injected_query.end_message, Some("No recommendations found for this filter".to_string()));

    // Query for non-existent game
    let no_match = get_store_recommendations(
        0,
        None,
        None,
        Some("NonExistentGameXyz123456789".to_string()),
        None,
    ).await.unwrap();
    assert_eq!(no_match.items.len(), 0);
    assert_eq!(no_match.has_more, false);
    assert_eq!(no_match.total_pages, 0);
    assert_eq!(
        no_match.end_message,
        Some("No recommendations found for this filter".to_string())
    );

    // Query with HTML / XSS tags
    let xss_query = get_store_recommendations(
        0,
        None,
        None,
        Some("<script>alert('xss')</script>".to_string()),
        None,
    ).await.unwrap();
    assert_eq!(xss_query.items.len(), 0);
    assert_eq!(xss_query.has_more, false);
    assert_eq!(
        xss_query.end_message,
        Some("No recommendations found for this filter".to_string())
    );

    // Query with 10,000 repeated characters
    let long_query = get_store_recommendations(
        0,
        None,
        None,
        Some("a".repeat(10_000)),
        None,
    ).await.unwrap();
    assert_eq!(long_query.items.len(), 0);
}

#[tokio::test]
async fn test_convert_currency_ipc_command_boundaries() {
    // Standard conversion
    let kzt = convert_currency(59.99, "KZT".to_string()).await.unwrap();
    assert_eq!(kzt, 29695.0);

    let usd = convert_currency(59.99, "USD".to_string()).await.unwrap();
    assert_eq!(usd, 59.99);

    // Zero
    let zero_kzt = convert_currency(0.0, "KZT".to_string()).await.unwrap();
    assert_eq!(zero_kzt, 0.0);

    // Micro-cents ($0.01 and $0.001)
    let micro_kzt = convert_currency(0.001, "KZT".to_string()).await.unwrap();
    assert_eq!(micro_kzt, 1.0, "Micro-cents convert to 1 KZT floor");

    // Massive numbers ($99,999,999)
    let massive_kzt = convert_currency(99_999_999.0, "KZT".to_string()).await.unwrap();
    assert_eq!(massive_kzt, 49_499_999_505.0);

    // Rejection of negative numbers
    let neg_res = convert_currency(-5.0, "KZT".to_string()).await;
    assert!(neg_res.is_err(), "Negative numbers must be rejected");

    // Rejection of NaN
    let nan_res = convert_currency(f64::NAN, "KZT".to_string()).await;
    assert!(nan_res.is_err(), "NaN must be rejected");

    // Rejection of Infinity
    let inf_res = convert_currency(f64::INFINITY, "KZT".to_string()).await;
    assert!(inf_res.is_err(), "Infinity must be rejected");

    let neg_inf_res = convert_currency(f64::NEG_INFINITY, "KZT".to_string()).await;
    assert!(neg_inf_res.is_err(), "Negative infinity must be rejected");

    // Rejection of unsupported currency
    let unsupp = convert_currency(10.0, "EUR".to_string()).await;
    assert!(unsupp.is_err(), "Unsupported currency EUR must be rejected");
}
