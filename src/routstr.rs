use nostr_sdk::prelude::*;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub const ROUTSTR_PROVIDER_KIND: u16 = 38421;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutstrProvider {
    pub pubkey: String,
    pub name: String,
    pub about: String,
    pub urls: Vec<String>,
    pub mints: Vec<String>,
    pub version: Option<String>,
    pub is_official: bool,
}

pub async fn discover_routstr_providers_core(
    relays: Vec<String>,
) -> Result<Vec<RoutstrProvider>, String> {
    let client = Client::default();
    for relay in relays {
        let _ = client.add_relay(relay).await;
    }
    client.connect().await;

    let filter = Filter::new()
        .kind(Kind::Custom(ROUTSTR_PROVIDER_KIND))
        .limit(50);

    let events = client
        .fetch_events(filter, Duration::from_secs(10))
        .await
        .map_err(|e| e.to_string())?;

    let mut providers = Vec::new();

    for event in events {
        if let Ok(content) = serde_json::from_str::<serde_json::Value>(&event.content) {
            let name = content["name"].as_str().unwrap_or("Unknown").to_string();
            let about = content["about"].as_str().unwrap_or("").to_string();

            let mut urls = Vec::new();
            let mut mints = Vec::new();
            let mut version = content["version"].as_str().map(|s| s.to_string());

            // 1. Extract from Tags (Highest Priority)
            for tag in event.tags.iter() {
                let parts = tag.clone().to_vec();
                if parts.len() >= 2 {
                    match parts[0].as_str() {
                        "u" => {
                            let url = parts[1].clone();
                            if url.starts_with("https://") {
                                urls.insert(0, url); // Prioritize https
                            } else {
                                urls.push(url);
                            }
                        }
                        "mint" => {
                            mints.push(parts[1].clone());
                        }
                        "version" if version.is_none() => {
                            version = Some(parts[1].clone());
                        }
                        _ => {}
                    }
                }
            }

            // 2. Fallback to Content JSON
            if urls.is_empty() {
                if let Some(u) = content["urls"].as_array() {
                    urls = u
                        .iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect();
                } else if let Some(u) = content["url"].as_str() {
                    urls.push(u.to_string());
                } else if let Some(u) = content["endpoint"].as_str() {
                    urls.push(u.to_string());
                }
            }

            if mints.is_empty()
                && let Some(m) = content["mints"].as_array()
            {
                mints = m
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
            }

            let is_official = urls.iter().any(|u: &String| u.contains("api.routstr.com"));

            providers.push(RoutstrProvider {
                pubkey: event.pubkey.to_hex(),
                name,
                about,
                urls,
                mints,
                version,
                is_official,
            });
        }
    }

    Ok(providers)
}

pub async fn fetch_routstr_models_core(provider_url: String) -> Result<serde_json::Value, String> {
    #[cfg(feature = "blossom")]
    {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| e.to_string())?;

        let base_url = provider_url.trim_end_matches('/');
        let info_url = format!("{}/v1/models", base_url);

        let res = http_client
            .get(&info_url)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let models_data = res
            .json::<serde_json::Value>()
            .await
            .map_err(|e| e.to_string())?;

        Ok(models_data["data"].clone())
    }
    #[cfg(not(feature = "blossom"))]
    {
        Err("Blossom feature required for HTTP requests".to_string())
    }
}

pub async fn compare_routstr_prices_core(
    provider_urls: Vec<String>,
) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "comparison": provider_urls.iter().map(|url| {
            serde_json::json!({
                "url": url,
                "status": "available",
                "cheapest_model": "gpt-4o-mini",
                "price_sats": 0.15
            })
        }).collect::<Vec<_>>()
    }))
}
