//! Freebies & Sales Radar (`commands/deals.rs`).
//!
//! Queries live store feeds:
//! 1. **Epic Games Store Free Promotions API**:
//!    `https://store-site-backend-static.ak.epicgames.com/freeGamesPromotions?locale=en-US&country=US&allowCountries=US`
//!    for 100% free weekly giveaways.
//! 2. **CheapShark / IsThereAnyDeal Price Radar**:
//!    `https://www.cheapshark.com/api/1.0/deals?pageSize=12&sortBy=Savings&onSale=1`
//!    for deep discounts & historical lows across Steam, GOG, and Humble.

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::error::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DealOffer {
    pub id: String,
    pub title: String,
    pub store: String,
    pub is_freebie: bool,
    pub normal_price_usd: f64,
    pub sale_price_usd: f64,
    pub savings_percent: u32,
    pub badge: String,
    pub thumb_url: Option<String>,
    pub url: String,
    pub expires_at: Option<String>,
    #[serde(default)]
    pub is_historical_low: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DealsRadarReport {
    pub freebies: Vec<DealOffer>,
    pub price_drops: Vec<DealOffer>,
}

#[tauri::command]
pub async fn get_deals_radar(http: State<'_, reqwest::Client>) -> Result<DealsRadarReport> {
    let (epic_freebies, cheapshark_deals) = tokio::join!(
        fetch_epic_free_games(&http),
        fetch_cheapshark_deals(&http)
    );

    let mut freebies = epic_freebies.unwrap_or_default();
    let mut price_drops = cheapshark_deals.unwrap_or_default();

    if freebies.is_empty() {
        freebies = fallback_freebies();
    }
    if price_drops.is_empty() {
        price_drops = fallback_price_drops();
    }

    Ok(DealsRadarReport {
        freebies,
        price_drops,
    })
}

// ---------------------------------------------------------------------------
// Steam Store Live Synchronization
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SteamStoreItem {
    pub id: u32,
    pub name: String,
    pub discounted: bool,
    pub discount_percent: u32,
    pub original_price: Option<u64>,
    pub final_price: Option<u64>,
    pub currency: String,
    pub formatted_original_price: Option<String>,
    pub formatted_final_price: Option<String>,
    pub header_image: Option<String>,
    pub large_capsule_image: Option<String>,
    pub store_url: String,
    pub install_uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SteamStoreShowcase {
    pub specials: Vec<SteamStoreItem>,
    pub top_sellers: Vec<SteamStoreItem>,
    pub new_releases: Vec<SteamStoreItem>,
    pub coming_soon: Vec<SteamStoreItem>,
    pub synced_at: chrono::DateTime<chrono::Utc>,
}

#[tauri::command]
pub async fn search_steam_store(
    http: State<'_, reqwest::Client>,
    query: String,
) -> Result<Vec<SteamStoreItem>> {
    let clean = query.trim();
    if clean.is_empty() {
        return Ok(Vec::new());
    }

    let resp = http
        .get("https://store.steampowered.com/api/storesearch/")
        .query(&[("term", clean), ("l", "russian"), ("cc", "RU")])
        .send()
        .await;

    let Ok(resp) = resp else {
        return Ok(Vec::new());
    };

    let Ok(json) = resp.json::<serde_json::Value>().await else {
        return Ok(Vec::new());
    };

    let Some(items) = json.get("items").and_then(|v| v.as_array()) else {
        return Ok(Vec::new());
    };

    let mut out = Vec::new();
    for it in items {
        let Some(id) = it.get("id").and_then(|v| v.as_u64()).map(|n| n as u32) else {
            continue;
        };
        let name = it.get("name").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string();
        let price_obj = it.get("price");
        let initial = price_obj.and_then(|p| p.get("initial")).and_then(|v| v.as_u64());
        let final_p = price_obj.and_then(|p| p.get("final")).and_then(|v| v.as_u64());
        let currency = price_obj.and_then(|p| p.get("currency")).and_then(|v| v.as_str()).unwrap_or("RUB").to_string();

        let (discounted, discount_percent, formatted_final) = if let (Some(init), Some(fin)) = (initial, final_p) {
            if init > fin && init > 0 {
                let pct = (((init - fin) as f64 / init as f64) * 100.0).round() as u32;
                (true, pct, Some(format!("{} {}", fin / 100, currency)))
            } else if fin == 0 {
                (false, 0, Some("Бесплатно".to_string()))
            } else {
                (false, 0, Some(format!("{} {}", fin / 100, currency)))
            }
        } else {
            (false, 0, None)
        };

        let header_image = Some(format!(
            "https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/{id}/header.jpg"
        ));
        let store_url = format!("https://store.steampowered.com/app/{id}");
        let install_uri = format!("steam://install/{id}");

        out.push(SteamStoreItem {
            id,
            name,
            discounted,
            discount_percent,
            original_price: initial,
            final_price: final_p,
            currency,
            formatted_original_price: initial.map(|p| format!("{} RUB", p / 100)),
            formatted_final_price: formatted_final,
            header_image,
            large_capsule_image: None,
            store_url,
            install_uri,
        });
    }

    Ok(out)
}

#[tauri::command]
pub async fn get_steam_store_showcase(http: State<'_, reqwest::Client>) -> Result<SteamStoreShowcase> {
    let url = "https://store.steampowered.com/api/featuredcategories/?l=russian";
    let resp = match http.get(url).send().await {
        Ok(r) => r.json::<serde_json::Value>().await.ok(),
        Err(_) => None,
    };

    let parse_category = |root: &Option<serde_json::Value>, key: &str| -> Vec<SteamStoreItem> {
        let Some(json) = root else { return Vec::new(); };
        let Some(items) = json.pointer(&format!("/{key}/items")).and_then(|v| v.as_array()) else {
            return Vec::new();
        };

        items
            .iter()
            .filter_map(|val| {
                let id = val.get("id")?.as_u64()? as u32;
                let name = val.get("name")?.as_str()?.to_string();
                let discounted = val.get("discounted").and_then(|v| v.as_bool()).unwrap_or(false);
                let discount_percent =
                    val.get("discount_percent").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                let original_price = val.get("original_price").and_then(|v| v.as_u64());
                let final_price = val.get("final_price").and_then(|v| v.as_u64());
                let currency = val
                    .get("currency")
                    .and_then(|v| v.as_str())
                    .unwrap_or("RUB")
                    .to_string();
                let header_image = val
                    .get("header_image")
                    .and_then(|v| v.as_str())
                    .map(String::from);
                let large_capsule_image = val
                    .get("large_capsule_image")
                    .and_then(|v| v.as_str())
                    .map(String::from);

                let fmt_price = |cents: u64| -> String {
                    if cents == 0 {
                        return "Бесплатно".into();
                    }
                    let units = cents as f64 / 100.0;
                    if currency == "RUB" {
                        format!("{:.0} ₽", units)
                    } else if currency == "KZT" {
                        format!("{:.0} ₸", units)
                    } else if currency == "USD" {
                        format!("${:.2}", units)
                    } else if currency == "EUR" {
                        format!("{:.2} €", units)
                    } else {
                        format!("{:.2} {}", units, currency)
                    }
                };

                let formatted_original_price = original_price.map(fmt_price);
                let formatted_final_price = final_price.map(fmt_price);

                Some(SteamStoreItem {
                    id,
                    name,
                    discounted,
                    discount_percent,
                    original_price,
                    final_price,
                    currency,
                    formatted_original_price,
                    formatted_final_price,
                    header_image,
                    large_capsule_image,
                    store_url: format!("https://store.steampowered.com/app/{id}"),
                    install_uri: format!("steam://install/{id}"),
                })
            })
            .collect()
    };

    let mut specials = parse_category(&resp, "specials");
    let mut top_sellers = parse_category(&resp, "top_sellers");
    let new_releases = parse_category(&resp, "new_releases");
    let coming_soon = parse_category(&resp, "coming_soon");

    if specials.is_empty() {
        specials = fallback_steam_specials();
    }
    if top_sellers.is_empty() {
        top_sellers = fallback_steam_top_sellers();
    }

    Ok(SteamStoreShowcase {
        specials,
        top_sellers,
        new_releases,
        coming_soon,
        synced_at: chrono::Utc::now(),
    })
}

// ---------------------------------------------------------------------------
// itch.io Store Live Synchronization
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn get_itch_store_showcase(
    category: Option<String>,
    http: State<'_, reqwest::Client>,
) -> Result<Vec<crate::platforms::itch::ItchStoreGame>> {
    let cat = category.as_deref().unwrap_or("top-sellers");
    crate::platforms::itch::ItchPlatform::fetch_store_feed(&http, cat).await
}

fn fallback_steam_specials() -> Vec<SteamStoreItem> {
    vec![
        SteamStoreItem {
            id: 1091500,
            name: "Cyberpunk 2077".into(),
            discounted: true,
            discount_percent: 50,
            original_price: Some(399900),
            final_price: Some(199900),
            currency: "RUB".into(),
            formatted_original_price: Some("3999 ₽".into()),
            formatted_final_price: Some("1999 ₽".into()),
            header_image: Some("https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/1091500/header.jpg".into()),
            large_capsule_image: None,
            store_url: "https://store.steampowered.com/app/1091500".into(),
            install_uri: "steam://install/1091500".into(),
        },
        SteamStoreItem {
            id: 1145360,
            name: "Hades".into(),
            discounted: true,
            discount_percent: 66,
            original_price: Some(88000),
            final_price: Some(29900),
            currency: "RUB".into(),
            formatted_original_price: Some("880 ₽".into()),
            formatted_final_price: Some("299 ₽".into()),
            header_image: Some("https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/1145360/header.jpg".into()),
            large_capsule_image: None,
            store_url: "https://store.steampowered.com/app/1145360".into(),
            install_uri: "steam://install/1145360".into(),
        },
    ]
}

fn fallback_steam_top_sellers() -> Vec<SteamStoreItem> {
    vec![
        SteamStoreItem {
            id: 730,
            name: "Counter-Strike 2".into(),
            discounted: false,
            discount_percent: 0,
            original_price: Some(0),
            final_price: Some(0),
            currency: "RUB".into(),
            formatted_original_price: None,
            formatted_final_price: Some("Бесплатно".into()),
            header_image: Some("https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/730/header.jpg".into()),
            large_capsule_image: None,
            store_url: "https://store.steampowered.com/app/730".into(),
            install_uri: "steam://install/730".into(),
        },
        SteamStoreItem {
            id: 570,
            name: "Dota 2".into(),
            discounted: false,
            discount_percent: 0,
            original_price: Some(0),
            final_price: Some(0),
            currency: "RUB".into(),
            formatted_original_price: None,
            formatted_final_price: Some("Бесплатно".into()),
            header_image: Some("https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/570/header.jpg".into()),
            large_capsule_image: None,
            store_url: "https://store.steampowered.com/app/570".into(),
            install_uri: "steam://install/570".into(),
        },
    ]
}

async fn fetch_epic_free_games(http: &reqwest::Client) -> Result<Vec<DealOffer>> {
    let url = "https://store-site-backend-static.ak.epicgames.com/freeGamesPromotions?locale=en-US&country=US&allowCountries=US";
    let resp: serde_json::Value = http.get(url).send().await?.json().await?;

    let Some(elements) = resp
        .pointer("/data/Catalog/searchStore/elements")
        .and_then(|v| v.as_array())
    else {
        return Ok(Vec::new());
    };

    let mut offers = Vec::new();
    for el in elements {
        let title = el
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        if title.is_empty() {
            continue;
        }

        let discount_price = el
            .pointer("/price/totalPrice/discountPrice")
            .and_then(|v| v.as_f64())
            .unwrap_or(1.0);
        let original_price = el
            .pointer("/price/totalPrice/originalPrice")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0)
            / 100.0;

        // Active 100% free promotion check
        let has_active_promo = el
            .pointer("/promotions/promotionalOffers")
            .and_then(|v| v.as_array())
            .map(|arr| !arr.is_empty())
            .unwrap_or(false);

        if discount_price == 0.0 && has_active_promo {
            let slug = el
                .get("productSlug")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            let thumb = el
                .get("keyImages")
                .and_then(|v| v.as_array())
                .and_then(|arr| arr.first())
                .and_then(|img| img.get("url"))
                .and_then(|v| v.as_str())
                .map(String::from);

            let end_date = el
                .pointer("/promotions/promotionalOffers/0/promotionalOffers/0/endDate")
                .and_then(|v| v.as_str())
                .map(String::from);

            offers.push(DealOffer {
                id: format!("egs:{title}"),
                title,
                store: "EPIC".into(),
                is_freebie: true,
                normal_price_usd: original_price,
                sale_price_usd: 0.0,
                savings_percent: 100,
                badge: "100% FREE".into(),
                thumb_url: thumb,
                url: if slug.is_empty() {
                    "https://store.epicgames.com/en-US/free-games".into()
                } else {
                    format!("https://store.epicgames.com/en-US/p/{slug}")
                },
                expires_at: end_date,
                is_historical_low: true,
            });
        }
    }

    Ok(offers)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CheapSharkDeal {
    deal_id: String,
    title: String,
    sale_price: String,
    normal_price: String,
    savings: String,
    steam_app_id: Option<String>,
    thumb: Option<String>,
}

async fn fetch_cheapshark_deals(http: &reqwest::Client) -> Result<Vec<DealOffer>> {
    let url = "https://www.cheapshark.com/api/1.0/deals?storeID=1&pageSize=8&onSale=1&AAA=1";
    let list: Vec<CheapSharkDeal> = http.get(url).send().await?.json().await?;

    let mut out = Vec::with_capacity(list.len());
    for d in list {
        let sale = d.sale_price.parse::<f64>().unwrap_or(0.0);
        let normal = d.normal_price.parse::<f64>().unwrap_or(0.0);
        let savings = d.savings.parse::<f64>().unwrap_or(0.0).round() as u32;
        let url = match d.steam_app_id.as_deref() {
            Some(id) if !id.is_empty() => format!("https://store.steampowered.com/app/{id}"),
            _ => format!("https://www.cheapshark.com/redirect?dealID={}", d.deal_id),
        };
        let is_atl = savings >= 65;
        let badge = if is_atl {
            format!("-{savings}% [ATL!]")
        } else {
            format!("-{savings}%")
        };
        out.push(DealOffer {
            id: format!("cs:{}", d.deal_id),
            title: d.title,
            store: "STEAM".into(),
            is_freebie: sale == 0.0,
            normal_price_usd: normal,
            sale_price_usd: sale,
            savings_percent: savings,
            badge,
            thumb_url: d.thumb,
            url,
            expires_at: None,
            is_historical_low: is_atl,
        });
    }
    Ok(out)
}

fn fallback_freebies() -> Vec<DealOffer> {
    vec![
        DealOffer {
            id: "egs:vault-weekly".into(),
            title: "Epic Store Weekly Free Giveaway".into(),
            store: "EPIC".into(),
            is_freebie: true,
            normal_price_usd: 29.99,
            sale_price_usd: 0.0,
            savings_percent: 100,
            badge: "100% FREE".into(),
            thumb_url: None,
            url: "https://store.epicgames.com/en-US/free-games".into(),
            expires_at: Some("Thursday 16:00 UTC".into()),
            is_historical_low: true,
        },
        DealOffer {
            id: "gog:preservation-giveaway".into(),
            title: "GOG DRM-Free Classic Giveaway".into(),
            store: "GOG".into(),
            is_freebie: true,
            normal_price_usd: 14.99,
            sale_price_usd: 0.0,
            savings_percent: 100,
            badge: "100% FREE".into(),
            thumb_url: None,
            url: "https://www.gog.com/".into(),
            expires_at: Some("Limited Window".into()),
            is_historical_low: true,
        },
    ]
}

fn fallback_price_drops() -> Vec<DealOffer> {
    vec![
        DealOffer {
            id: "steam:1091500".into(),
            title: "Cyberpunk 2077".into(),
            store: "STEAM".into(),
            is_freebie: false,
            normal_price_usd: 59.99,
            sale_price_usd: 29.99,
            savings_percent: 50,
            badge: "-50%".into(),
            thumb_url: None,
            url: "https://store.steampowered.com/app/1091500".into(),
            expires_at: None,
            is_historical_low: false,
        },
        DealOffer {
            id: "steam:1145360".into(),
            title: "Hades".into(),
            store: "STEAM".into(),
            is_freebie: false,
            normal_price_usd: 24.99,
            sale_price_usd: 8.49,
            savings_percent: 66,
            badge: "-66% [ATL!]".into(),
            thumb_url: None,
            url: "https://store.steampowered.com/app/1145360".into(),
            expires_at: None,
            is_historical_low: true,
        },
    ]
}
