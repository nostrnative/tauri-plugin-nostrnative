use nostr_sdk::prelude::*;
use std::time::Duration;

use crate::models::{Contact, UserProfile};
use crate::utils::{get_client, parse_public_key};

pub async fn fetch_profiles_core(
    pubkeys: Vec<String>,
    relays: &[String],
) -> Result<Vec<UserProfile>, String> {
    let mut pks = Vec::new();
    for pk in pubkeys {
        if let Ok(p) = parse_public_key(&pk) {
            pks.push(p);
        }
    }

    if pks.is_empty() {
        return Ok(Vec::new());
    }

    let ephemeral_keys = Keys::generate();
    let client = get_client(&ephemeral_keys, relays).await?;

    let filter = Filter::new().authors(pks).kind(Kind::Metadata);

    let events = client
        .fetch_events(filter, Duration::from_secs(10))
        .await
        .map_err(|e| e.to_string())?;

    let mut profiles = Vec::new();
    for event in events {
        if let Ok(content) = serde_json::from_str::<serde_json::Value>(&event.content) {
            profiles.push(UserProfile {
                pubkey: event.pubkey.to_hex(),
                name: content["name"].as_str().map(|s| s.to_string()),
                display_name: content["display_name"].as_str().map(|s| s.to_string()),
                about: content["about"].as_str().map(|s| s.to_string()),
                picture: content["picture"].as_str().map(|s| s.to_string()),
                banner: content["banner"].as_str().map(|s| s.to_string()),
                website: content["website"].as_str().map(|s| s.to_string()),
                nip05: content["nip05"].as_str().map(|s| s.to_string()),
            });
        }
    }

    Ok(profiles)
}

pub async fn fetch_contact_list_core(
    pubkey: &str,
    relays: &[String],
) -> Result<Vec<Contact>, String> {
    let public_key = parse_public_key(pubkey)?;
    let ephemeral_keys = Keys::generate();
    let client = get_client(&ephemeral_keys, relays).await?;

    let filter = Filter::new()
        .authors(vec![public_key])
        .kind(Kind::ContactList);

    let events = client
        .fetch_events(filter, Duration::from_secs(10))
        .await
        .map_err(|e| e.to_string())?;

    let mut events: Vec<_> = events.into_iter().collect();
    events.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    if let Some(event) = events.first() {
        let contacts: Vec<Contact> = event
            .tags
            .iter()
            .filter_map(|tag| {
                let t = tag.clone().to_vec();
                if t.len() >= 2 && t[0] == "p" {
                    let pubkey = t[1].clone();
                    let alias = if t.len() >= 4 {
                        Some(t[3].clone())
                    } else {
                        None
                    };
                    Some(Contact { pubkey, alias })
                } else {
                    None
                }
            })
            .collect();
        Ok(contacts)
    } else {
        Ok(Vec::new())
    }
}

pub async fn update_contact_list_core(
    nsec: &str,
    relays: &[String],
    contacts: Vec<Contact>,
) -> Result<String, String> {
    let client = crate::utils::get_client_with_signer(nsec, relays).await?;

    let mut tags = Vec::new();
    let mut seen_pubkeys = std::collections::HashSet::new();

    for contact in contacts {
        if let Ok(pubkey) = parse_public_key(&contact.pubkey) {
            let hex = pubkey.to_hex();

            if seen_pubkeys.contains(&hex) {
                continue;
            }
            seen_pubkeys.insert(hex.clone());

            tags.push(Tag::custom(
                TagKind::from("p"),
                vec![hex, "".to_string(), contact.alias.unwrap_or_default()],
            ));
        }
    }

    let event_builder = EventBuilder::new(Kind::ContactList, "").tags(tags);

    let event_id = client
        .send_event_builder(event_builder)
        .await
        .map_err(|e| e.to_string())?;

    Ok(event_id.to_hex())
}
