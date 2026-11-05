//! Steam Store Infinite Recommendations & USD/KZT Currency Switcher.

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::core::config::ConfigStore;

pub const FIXED_EXCHANGE_RATE: f64 = 495.0; // 1 USD = 495 KZT

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoreMedia {
    pub screenshots: Vec<String>,
    pub webm_video: Option<String>,
    pub mp4_video: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorePage {
    pub page: usize,
    pub currency: String,
    pub items: Vec<StoreItem>,
    pub has_more: bool,
    pub total_pages: usize,
    pub end_message: Option<String>,
}

#[derive(Debug, Clone)]
struct StaticStoreItem {
    app_id: u32,
    title: &'static str,
    base_usd: f64,
    discount_pct: u32,
    tags: &'static [&'static str],
    review_score: &'static str,
    short_desc: &'static str,
    release_date: &'static str,
    mp4_video: &'static str,
}

const STORE_CATALOG: &[StaticStoreItem] = &[
    StaticStoreItem {
        app_id: 1091500,
        title: "Cyberpunk 2077",
        base_usd: 59.99,
        discount_pct: 50,
        tags: &["Action", "RPG", "FPS", "Open World", "Sci-Fi"],
        review_score: "Very Positive (91%)",
        short_desc: "An open-world, action-adventure RPG set in the megalopolis of Night City.",
        release_date: "10 Dec, 2020",
        mp4_video: "https://video.akamai.steamstatic.com/store_trailers/257082775/movie480_vp9.webm",
    },
    StaticStoreItem {
        app_id: 1145360,
        title: "Hades",
        base_usd: 24.99,
        discount_pct: 35,
        tags: &["Roguelike", "Action", "Indie"],
        review_score: "Overwhelmingly Positive (98%)",
        short_desc: "Defy the god of the dead as you hack and slash out of the Underworld.",
        release_date: "17 Sep, 2020",
        mp4_video: "https://video.akamai.steamstatic.com/store_trailers/256801252/movie480_vp9.webm",
    },
    StaticStoreItem {
        app_id: 782330,
        title: "DOOM Eternal",
        base_usd: 39.99,
        discount_pct: 67,
        tags: &["FPS", "Action", "Sci-Fi"],
        review_score: "Very Positive (91%)",
        short_desc: "Hell’s armies have invaded Earth. Become the Slayer in an epic single-player campaign.",
        release_date: "20 Mar, 2020",
        mp4_video: "https://video.akamai.steamstatic.com/store_trailers/257182101/movie480_vp9.webm",
    },
    StaticStoreItem {
        app_id: 289070,
        title: "Sid Meier's Civilization VI",
        base_usd: 59.99,
        discount_pct: 90,
        tags: &["Strategy", "Turn-Based", "Simulation"],
        review_score: "Very Positive (86%)",
        short_desc: "Civilization VI offers new ways to interact with your world, expand your empire, and advance your culture.",
        release_date: "21 Oct, 2016",
        mp4_video: "https://video.akamai.steamstatic.com/store_trailers/256672694/movie480.mp4",
    },
    StaticStoreItem {
        app_id: 1245620,
        title: "ELDEN RING",
        base_usd: 59.99,
        discount_pct: 0,
        tags: &["RPG", "Action", "Open World"],
        review_score: "Very Positive (92%)",
        short_desc: "Rise, Tarnished, and be guided by grace to brandish the power of the Elden Ring.",
        release_date: "25 Feb, 2022",
        mp4_video: "https://video.akamai.steamstatic.com/store_trailers/256889452/movie480_vp9.webm",
    },
    StaticStoreItem {
        app_id: 730,
        title: "Counter-Strike 2",
        base_usd: 0.0,
        discount_pct: 0,
        tags: &["FPS", "Action"],
        review_score: "Very Positive (87%)",
        short_desc: "For over two decades, Counter-Strike has offered an elite competitive experience.",
        release_date: "21 Aug, 2012",
        mp4_video: "https://video.akamai.steamstatic.com/store_trailers/256972298/movie480_vp9.webm",
    },
    StaticStoreItem {
        app_id: 1086940,
        title: "Baldur's Gate 3",
        base_usd: 59.99,
        discount_pct: 20,
        tags: &["RPG", "Strategy", "Turn-Based"],
        review_score: "Overwhelmingly Positive (96%)",
        short_desc: "Gather your party and return to the Forgotten Realms in a tale of fellowship and betrayal.",
        release_date: "3 Aug, 2023",
        mp4_video: "https://video.akamai.steamstatic.com/store_trailers/256987424/movie480_vp9.webm",
    },
    StaticStoreItem {
        app_id: 1174180,
        title: "Red Dead Redemption 2",
        base_usd: 59.99,
        discount_pct: 60,
        tags: &["Action", "Open World"],
        review_score: "Very Positive (91%)",
        short_desc: "Winner of over 175 Game of the Year Awards and recipient of over 250 perfect scores.",
        release_date: "5 Dec, 2019",
        mp4_video: "https://video.akamai.steamstatic.com/store_trailers/256767979/movie480.mp4",
    },
    StaticStoreItem {
        app_id: 646570,
        title: "Slay the Spire",
        base_usd: 24.99,
        discount_pct: 66,
        tags: &["Roguelike", "Strategy", "Indie"],
        review_score: "Overwhelmingly Positive (97%)",
        short_desc: "We fused card games and roguelikes together to make the best single player deckbuilder we could.",
        release_date: "23 Jan, 2019",
        mp4_video: "https://video.akamai.steamstatic.com/store_trailers/256740869/movie480.mp4",
    },
    StaticStoreItem {
        app_id: 413150,
        title: "Stardew Valley",
        base_usd: 14.99,
        discount_pct: 20,
        tags: &["Simulation", "Indie", "RPG"],
        review_score: "Overwhelmingly Positive (98%)",
        short_desc: "You've inherited your grandfather's old farm plot in Stardew Valley.",
        release_date: "26 Feb, 2016",
        mp4_video: "https://video.akamai.steamstatic.com/store_trailers/256815967/movie480_vp9.webm",
    },
    StaticStoreItem {
        app_id: 550,
        title: "Left 4 Dead 2",
        base_usd: 9.99,
        discount_pct: 80,
        tags: &["FPS", "Action", "Horror"],
        review_score: "Overwhelmingly Positive (97%)",
        short_desc: "Set in the zombie apocalypse, Left 4 Dead 2 is the highly anticipated sequel to the award-winning Left 4 Dead.",
        release_date: "17 Nov, 2009",
        mp4_video: "https://video.akamai.steamstatic.com/store_trailers/5952/movie480.mp4",
    },
    StaticStoreItem {
        app_id: 250900,
        title: "The Binding of Isaac: Rebirth",
        base_usd: 14.99,
        discount_pct: 33,
        tags: &["Roguelike", "Indie", "Action"],
        review_score: "Overwhelmingly Positive (97%)",
        short_desc: "The Binding of Isaac: Rebirth is a randomly generated action RPG shooter with heavy Rogue-like elements.",
        release_date: "4 Nov, 2014",
        mp4_video: "https://video.akamai.steamstatic.com/store_trailers/2034156/movie480.mp4",
    },
    StaticStoreItem {
        app_id: 570,
        title: "Dota 2",
        base_usd: 0.0,
        discount_pct: 0,
        tags: &["Strategy", "Action"],
        review_score: "Very Positive (82%)",
        short_desc: "Every day, millions of players worldwide enter battle as one of over a hundred Dota heroes.",
        release_date: "9 Jul, 2013",
        mp4_video: "https://video.akamai.steamstatic.com/store_trailers/256692021/movie480.mp4",
    },
    StaticStoreItem {
        app_id: 892970,
        title: "Valheim",
        base_usd: 19.99,
        discount_pct: 40,
        tags: &["Action", "Open World", "Indie"],
        review_score: "Overwhelmingly Positive (95%)",
        short_desc: "A brutal exploration and survival game for 1-10 players, set in a procedurally-generated purgatory inspired by Norse mythology.",
        release_date: "2 Feb, 2021",
        mp4_video: "https://video.akamai.steamstatic.com/store_trailers/256820008/movie480_vp9.webm",
    },
    StaticStoreItem {
        app_id: 578080,
        title: "PUBG: BATTLEGROUNDS",
        base_usd: 0.0,
        discount_pct: 0,
        tags: &["FPS", "Action", "Sci-Fi"],
        review_score: "Very Positive (80%)",
        short_desc: "Land on strategic locations, loot weapons and supplies, and survive to become the last team standing across various Battlegrounds.",
        release_date: "21 Dec, 2017",
        mp4_video: "https://video.akamai.steamstatic.com/store_trailers/257200883/movie480_vp9.webm",
    },
    StaticStoreItem {
        app_id: 2050650,
        title: "Resident Evil 4",
        base_usd: 39.99,
        discount_pct: 25,
        tags: &["Horror", "Action"],
        review_score: "Overwhelmingly Positive (97%)",
        short_desc: "Survival is just the beginning. Six years have passed since the biological disaster in Raccoon City.",
        release_date: "24 Mar, 2023",
        mp4_video: "https://video.akamai.steamstatic.com/store_trailers/256998128/movie480_vp9.webm",
    },
    StaticStoreItem {
        app_id: 105600,
        title: "Terraria",
        base_usd: 9.99,
        discount_pct: 50,
        tags: &["Action", "Indie", "Open World"],
        review_score: "Overwhelmingly Positive (97%)",
        short_desc: "Dig, Fight, Explore, Build: The very world is at your fingertips as you fight for survival, fortune, and glory.",
        release_date: "16 May, 2011",
        mp4_video: "https://video.akamai.steamstatic.com/store_trailers/256785003/movie480_vp9.webm",
    },
    StaticStoreItem {
        app_id: 1817070,
        title: "Marvel’s Spider-Man Remastered",
        base_usd: 59.99,
        discount_pct: 40,
        tags: &["Action", "Open World"],
        review_score: "Overwhelmingly Positive (96%)",
        short_desc: "In Marvel’s Spider-Man Remastered, the worlds of Peter Parker and Spider-Man collide in an original action-packed story.",
        release_date: "12 Aug, 2022",
        mp4_video: "https://video.akamai.steamstatic.com/store_trailers/256900369/movie480_vp9.webm",
    },
];

pub fn sanitize_search_query(query: &str) -> String {
    query.replace(&['\'', '"', ';', '{', '}', '(', ')', '$'][..], "").trim().to_string()
}

pub fn convert_price_to_kzt(usd: f64) -> u64 {
    if usd <= 0.0 {
        0
    } else {
        (usd * FIXED_EXCHANGE_RATE).round().max(1.0) as u64
    }
}

pub fn format_price_string(amount: f64, currency: &str) -> String {
    if amount == 0.0 {
        return "Free to Play".to_string();
    }
    if currency == "KZT" {
        let kzt_val = convert_price_to_kzt(amount);
        let s = kzt_val.to_string();
        let mut formatted = String::new();
        let chars: Vec<char> = s.chars().collect();
        let len = chars.len();
        for (i, ch) in chars.iter().enumerate() {
            if i > 0 && (len - i) % 3 == 0 {
                formatted.push(' ');
            }
            formatted.push(*ch);
        }
        format!("{} ₸", formatted)
    } else {
        format!("${:.2}", amount)
    }
}

#[tauri::command]
pub async fn get_store_recommendations(
    page: usize,
    currency: Option<String>,
    tag: Option<String>,
    query: Option<String>,
    _category: Option<String>,
) -> Result<StorePage, String> {
    let curr = match currency.as_deref() {
        Some("KZT") => "KZT",
        _ => "USD",
    };

    let clean_query = query.as_deref().map(sanitize_search_query).unwrap_or_default().to_lowercase();
    let filter_tag = tag.as_deref().map(|t| t.trim()).unwrap_or("");

    // Filter catalog
    let filtered: Vec<&StaticStoreItem> = STORE_CATALOG
        .iter()
        .filter(|item| {
            if !clean_query.is_empty() && !item.title.to_lowercase().contains(&clean_query) {
                return false;
            }
            if !filter_tag.is_empty() && filter_tag != "All" && !item.tags.iter().any(|t| t.eq_ignore_ascii_case(filter_tag)) {
                return false;
            }
            true
        })
        .collect();

    // Loop catalog if needed to provide true infinite scrolling up to 10 pages
    let page_size = 6;
    let max_pages = 10;

    if page >= max_pages {
        return Ok(StorePage {
            page,
            currency: curr.to_string(),
            items: Vec::new(),
            has_more: false,
            total_pages: max_pages,
            end_message: Some("You have caught up with all current recommendations".to_string()),
        });
    }

    if filtered.is_empty() {
        return Ok(StorePage {
            page,
            currency: curr.to_string(),
            items: Vec::new(),
            has_more: false,
            total_pages: 0,
            end_message: Some("No recommendations found for this filter".to_string()),
        });
    }

    // Multiply catalog items to simulate infinite recommendation depth
    let start_idx = page * page_size;
    let total_virtual_items = max_pages * page_size;
    let total_pages = (total_virtual_items + page_size - 1) / page_size;

    let mut page_items = Vec::new();
    let catalog_len = filtered.len();

    for i in 0..page_size {
        let global_idx = start_idx + i;
        if global_idx >= total_virtual_items {
            break;
        }
        let static_item = filtered[global_idx % catalog_len];

        let base_usd = static_item.base_usd;
        let discount = static_item.discount_pct;

        let sale_usd = if discount > 0 {
            let final_price = base_usd * (1.0 - (discount as f64 / 100.0));
            (final_price * 100.0).round() / 100.0
        } else {
            base_usd
        };

        let price_raw = if curr == "KZT" {
            convert_price_to_kzt(sale_usd) as f64
        } else {
            sale_usd
        };

        let price_formatted = format_price_string(sale_usd, curr);

        let (orig_raw, orig_formatted) = if discount > 0 {
            let o_raw = if curr == "KZT" {
                convert_price_to_kzt(base_usd) as f64
            } else {
                base_usd
            };
            let o_fmt = format_price_string(base_usd, curr);
            (Some(o_raw), Some(o_fmt))
        } else {
            (None, None)
        };

        let app_id = static_item.app_id;
        let cdn_base = "https://shared.akamai.steamstatic.com/store_item_assets/steam/apps";
        let header_img = format!("{}/{}/header.jpg", cdn_base, app_id);
        let thumb_img = format!("{}/{}/capsule_231x87.jpg", cdn_base, app_id);
        let banner_img = format!("{}/{}/library_hero.jpg", cdn_base, app_id);

        let screenshots = vec![
            format!("{}/{}/ss_1.jpg", cdn_base, app_id),
            format!("{}/{}/ss_2.jpg", cdn_base, app_id),
        ];

        let media = StoreMedia {
            screenshots,
            webm_video: Some(static_item.mp4_video.to_string()),
            mp4_video: Some(static_item.mp4_video.to_string()),
        };

        page_items.push(StoreItem {
            app_id,
            title: static_item.title.to_string(),
            price_raw,
            price_formatted,
            original_price_raw: orig_raw,
            original_price_formatted: orig_formatted,
            discount_percent: discount,
            thumbnail_url: thumb_img,
            header_image: Some(header_img),
            banner_url: Some(banner_img),
            tags: static_item.tags.iter().map(|&t| t.to_string()).collect(),
            review_score: Some(static_item.review_score.to_string()),
            media: Some(media),
            short_description: Some(static_item.short_desc.to_string()),
            release_date: Some(static_item.release_date.to_string()),
            store_url: format!("https://store.steampowered.com/app/{}", app_id),
            install_uri: format!("steam://install/{}", app_id),
        });
    }

    let has_more = page + 1 < total_pages;
    let end_message = if !has_more {
        Some("You have caught up with all current recommendations".to_string())
    } else {
        None
    };

    Ok(StorePage {
        page,
        currency: curr.to_string(),
        items: page_items,
        has_more,
        total_pages,
        end_message,
    })
}

#[tauri::command]
pub async fn get_preferred_currency(store: State<'_, ConfigStore>) -> Result<String, String> {
    let cfg = store.snapshot().await;
    Ok(cfg.preferred_currency)
}

#[tauri::command]
pub async fn set_preferred_currency(
    currency: String,
    store: State<'_, ConfigStore>,
) -> Result<String, String> {
    if currency != "USD" && currency != "KZT" {
        return Err(format!("Unsupported currency: {}. Must be 'USD' or 'KZT'.", currency));
    }

    let updated = store
        .update(|cfg| {
            cfg.preferred_currency = currency.clone();
        })
        .await
        .map_err(|e| e.to_string())?;

    Ok(updated.preferred_currency)
}

#[tauri::command]
pub async fn convert_currency(amount: f64, target_currency: String) -> Result<f64, String> {
    if amount < 0.0 || amount.is_nan() || amount.is_infinite() {
        return Err("Invalid price value: must be positive finite number".to_string());
    }

    match target_currency.as_str() {
        "USD" => Ok(amount),
        "KZT" => Ok(convert_price_to_kzt(amount) as f64),
        other => Err(format!("Unsupported target currency: {}", other)),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GameDescription {
    pub app_id: Option<u32>,
    pub short_description: String,
    pub about: String,
    pub genres: Vec<String>,
    pub developers: Vec<String>,
    pub publishers: Vec<String>,
    pub release_date: Option<String>,
    pub header_image: Option<String>,
    pub screenshots: Vec<String>,
}

fn strip_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_tag = false;
    for c in s.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    out.replace("&quot;", "\"").replace("&amp;", "&").replace("&nbsp;", " ").trim().to_string()
}

#[tauri::command]
pub async fn get_game_description(
    app_id: Option<u32>,
    title: String,
    http: State<'_, reqwest::Client>,
) -> Result<GameDescription, String> {
    let mut id = app_id;
    if id.is_none() && !title.trim().is_empty() {
        let q = sanitize_search_query(&title).replace(' ', "%20");
        let url = format!("https://store.steampowered.com/api/storesearch/?term={q}&l=russian&cc=us");
        if let Ok(resp) = http.get(&url).send().await {
            if let Ok(v) = resp.json::<serde_json::Value>().await {
                id = v["items"][0]["id"].as_u64().map(|x| x as u32);
            }
        }
    }
    let Some(id) = id else { return Ok(GameDescription::default()) };
    let url = format!("https://store.steampowered.com/api/appdetails?appids={id}&l=russian");
    let v: serde_json::Value = http
        .get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;
    let d = &v[id.to_string()]["data"];
    let names = |k: &str| -> Vec<String> {
        d[k].as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default()
    };
    Ok(GameDescription {
        app_id: Some(id),
        short_description: strip_html(d["short_description"].as_str().unwrap_or("")),
        about: strip_html(d["about_the_game"].as_str().unwrap_or("")),
        genres: d["genres"].as_array().map(|a| a.iter().filter_map(|g| g["description"].as_str().map(String::from)).collect()).unwrap_or_default(),
        developers: names("developers"),
        publishers: names("publishers"),
        release_date: d["release_date"]["date"].as_str().map(String::from),
        header_image: d["header_image"].as_str().map(String::from),
        screenshots: d["screenshots"].as_array().map(|a| a.iter().take(6).filter_map(|s| s["path_thumbnail"].as_str().map(String::from)).collect()).unwrap_or_default(),
    })
}
