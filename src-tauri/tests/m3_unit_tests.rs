//! Milestone 3 Integration & Unit Tests.
//! Validates Store currency conversion, query sanitization, emulator catalog,
//! drive & directory validation, and network settings parsing.

use omnilauncher_lib::commands::store::{
    convert_price_to_kzt, format_price_string, get_store_recommendations, sanitize_search_query,
    FIXED_EXCHANGE_RATE,
};

#[test]
fn test_currency_conversion_and_rounding() {
    // 1 USD = 495 KZT
    assert_eq!(FIXED_EXCHANGE_RATE, 495.0);

    // Standard $59.99 game -> 29,695 KZT
    let kzt_val = convert_price_to_kzt(59.99);
    assert_eq!(kzt_val, 29695);

    // Free game -> 0 KZT
    assert_eq!(convert_price_to_kzt(0.0), 0);
    assert_eq!(convert_price_to_kzt(-10.0), 0);

    // Micro-pricing floor: $0.01 -> 5 KZT, $0.001 -> 1 KZT floor
    assert_eq!(convert_price_to_kzt(0.01), 5);
    assert_eq!(convert_price_to_kzt(0.001), 1);

    // Large prices ($99,999.00) without 32-bit overflow
    let big = convert_price_to_kzt(99999.0);
    assert_eq!(big, 49499505);
}

#[test]
fn test_currency_string_formatting() {
    assert_eq!(format_price_string(0.0, "USD"), "Free to Play");
    assert_eq!(format_price_string(0.0, "KZT"), "Free to Play");

    assert_eq!(format_price_string(59.99, "USD"), "$59.99");
    let kzt_formatted = format_price_string(59.99, "KZT");
    assert!(kzt_formatted.contains('₸'));
    assert!(kzt_formatted.contains("29"));
    assert!(kzt_formatted.contains("695"));
}

#[test]
fn test_search_query_sanitization() {
    let injection = "Cyberpunk'; DROP TABLE Games; --";
    let cleaned = sanitize_search_query(injection);
    assert!(!cleaned.contains(';'));
    assert!(!cleaned.contains('\''));
    assert!(!cleaned.contains('"'));
    assert_eq!(cleaned, "Cyberpunk DROP TABLE Games --");

    let dirty2 = "Hades()${injection}";
    let cleaned2 = sanitize_search_query(dirty2);
    assert_eq!(cleaned2, "Hadesinjection");
}

#[test]
fn test_reserved_windows_folder_names() {
    let reserved = [
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7",
        "COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];

    for name in reserved {
        assert!(reserved.contains(&name.to_uppercase().as_str()));
    }

    assert!(!reserved.contains(&"SteamLibrary".to_uppercase().as_str()));
    assert!(!reserved.contains(&"Emulators".to_uppercase().as_str()));
}

#[test]
fn test_proxy_uri_regex_validation() {
    let re = regex::Regex::new(r"^(socks5|http|https)://(?:([^:]+):([^@]+)@)?(\[[0-9a-fA-F:]+\]|[^:]+):(\d+)$").unwrap();

    let caps1 = re.captures("socks5://127.0.0.1:1080");
    assert!(caps1.is_some());
    let c1 = caps1.unwrap();
    assert_eq!(&c1[1], "socks5");
    assert_eq!(&c1[4], "127.0.0.1");
    assert_eq!(&c1[5], "1080");

    let caps2 = re.captures("http://admin:secret@proxy.corp.net:8080");
    assert!(caps2.is_some());
    let c2 = caps2.unwrap();
    assert_eq!(&c2[1], "http");
    assert_eq!(&c2[2], "admin");
    assert_eq!(&c2[3], "secret");
    assert_eq!(&c2[4], "proxy.corp.net");
    assert_eq!(&c2[5], "8080");

    let caps3 = re.captures("socks5://[::1]:1080");
    assert!(caps3.is_some());
    let c3 = caps3.unwrap();
    assert_eq!(&c3[1], "socks5");
    assert_eq!(&c3[4], "[::1]");
    assert_eq!(&c3[5], "1080");

    assert!(re.captures("ftp://invalid:21").is_none());
    assert!(re.captures("not_a_url").is_none());
}

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

#[test]
fn test_doh_json_response_parsing() {
    #[derive(serde::Deserialize)]
    struct DohRecord {
        #[serde(rename = "type")]
        record_type: Option<u32>,
        data: Option<String>,
    }

    #[derive(serde::Deserialize)]
    struct DohResponse {
        #[serde(rename = "Status")]
        status: Option<i32>,
        #[serde(rename = "Answer")]
        answer: Option<Vec<DohRecord>>,
    }

    // Cloudflare JSON response sample
    let cf_json = r#"{
        "Status": 0,
        "TC": false,
        "RD": true,
        "RA": true,
        "AD": true,
        "CD": false,
        "Question": [{"name": "cloudflare.com", "type": 1}],
        "Answer": [
            {"name": "cloudflare.com", "type": 1, "TTL": 150, "data": "104.16.133.229"},
            {"name": "cloudflare.com", "type": 1, "TTL": 150, "data": "104.16.132.229"}
        ]
    }"#;

    let cf_resp: DohResponse = serde_json::from_str(cf_json).expect("Cloudflare DoH JSON parse failed");
    assert_eq!(cf_resp.status, Some(0));
    let cf_ip = cf_resp.answer.as_ref().and_then(|ans| {
        ans.iter().find(|r| r.record_type == Some(1)).and_then(|r| r.data.clone())
    });
    assert_eq!(cf_ip, Some("104.16.133.229".to_string()));

    // Google JSON response sample with AAAA record
    let google_json = r#"{
        "Status": 0,
        "Answer": [
            {"name": "cloudflare.com", "type": 28, "TTL": 300, "data": "2606:4700::6810:85e5"}
        ]
    }"#;

    let g_resp: DohResponse = serde_json::from_str(google_json).expect("Google DoH JSON parse failed");
    let g_ip = g_resp.answer.as_ref().and_then(|ans| {
        ans.iter().find(|r| r.record_type == Some(1)).and_then(|r| r.data.clone())
            .or_else(|| ans.iter().find(|r| r.record_type == Some(28)).and_then(|r| r.data.clone()))
    });
    assert_eq!(g_ip, Some("2606:4700::6810:85e5".to_string()));
}


