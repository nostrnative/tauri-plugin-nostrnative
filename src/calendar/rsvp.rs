use nostr_sdk::prelude::*;
use std::borrow::Cow;
use std::time::Duration;

use crate::models::EventResponse;
use crate::utils::{get_client, parse_public_key};

pub async fn fetch_rsvps_core(
    event_coordinate: &str,
    relays: &[String],
) -> Result<Vec<EventResponse>, String> {
    let keys = Keys::generate();
    let client = get_client(&keys, relays).await?;

    let filter = if event_coordinate.contains(':') {
        Filter::new()
            .kind(Kind::from(31925))
            .custom_tag(SingleLetterTag::lowercase(Alphabet::A), event_coordinate)
    } else if let Ok(event_id) = EventId::from_hex(event_coordinate) {
        Filter::new().kind(Kind::from(31925)).event(event_id)
    } else {
        return Err("Invalid event ID".to_string());
    };

    let events = client
        .fetch_events(filter, Duration::from_secs(10))
        .await
        .map_err(|e| e.to_string())?;

    let response = events
        .into_iter()
        .map(|e| EventResponse {
            id: e.id.to_hex(),
            pubkey: e.pubkey.to_hex(),
            created_at: e.created_at.as_secs() as i64,
            kind: e.kind.as_u16(),
            tags: e.tags.iter().map(|t| t.clone().to_vec()).collect(),
            content: e.content.to_string(),
            is_private: false,
            bech32: None,
            private_tags: None,
            source: None,
        })
        .collect();

    Ok(response)
}

pub async fn fetch_user_rsvps_core(
    pubkey: &str,
    relays: &[String],
) -> Result<Vec<EventResponse>, String> {
    let pk = parse_public_key(pubkey)?;
    let keys = Keys::generate();
    let client = get_client(&keys, relays).await?;

    let filter = Filter::new().kind(Kind::from(31925)).author(pk).limit(1000);

    let events = client
        .fetch_events(filter, Duration::from_secs(10))
        .await
        .map_err(|e| e.to_string())?;

    let response = events
        .into_iter()
        .map(|e| EventResponse {
            id: e.id.to_hex(),
            pubkey: e.pubkey.to_hex(),
            created_at: e.created_at.as_secs() as i64,
            kind: e.kind.as_u16(),
            tags: e.tags.iter().map(|t| t.clone().to_vec()).collect(),
            content: e.content.to_string(),
            is_private: false,
            bech32: None,
            private_tags: None,
            source: None,
        })
        .collect();

    Ok(response)
}

pub async fn fetch_received_rsvps_core(
    pubkey: &str,
    relays: &[String],
) -> Result<Vec<EventResponse>, String> {
    let pk = parse_public_key(pubkey)?;
    let keys = Keys::generate();
    let client = get_client(&keys, relays).await?;

    let filter = Filter::new().kind(Kind::from(31925)).pubkey(pk).limit(1000);

    let events = client
        .fetch_events(filter, Duration::from_secs(10))
        .await
        .map_err(|e| e.to_string())?;

    let response = events
        .into_iter()
        .map(|e| EventResponse {
            id: e.id.to_hex(),
            pubkey: e.pubkey.to_hex(),
            created_at: e.created_at.as_secs() as i64,
            kind: e.kind.as_u16(),
            tags: e.tags.iter().map(|t| t.clone().to_vec()).collect(),
            content: e.content.to_string(),
            is_private: false,
            bech32: None,
            private_tags: None,
            source: None,
        })
        .collect();

    Ok(response)
}

pub async fn publish_rsvp_core(
    nsec: &str,
    relays: &[String],
    event_coordinate: String,
    status: String,
    event_author: Option<String>,
    client_nsec: Option<&str>,
) -> Result<String, String> {
    let client = crate::utils::get_client_with_signer(nsec, relays, client_nsec).await?;

    let mut tags = Vec::new();

    if event_coordinate.contains(':') {
        tags.push(Tag::custom(
            TagKind::Custom(Cow::Borrowed("a")),
            vec![event_coordinate],
        ));
    } else {
        tags.push(Tag::custom(
            TagKind::Custom(Cow::Borrowed("e")),
            vec![event_coordinate],
        ));
    }

    tags.push(Tag::custom(
        TagKind::Custom(Cow::Borrowed("status")),
        vec![status],
    ));

    if let Some(author) = event_author {
        tags.push(Tag::custom(
            TagKind::Custom(Cow::Borrowed("p")),
            vec![author],
        ));
    }

    let identifier = Keys::generate().public_key().to_hex();
    tags.push(Tag::identifier(identifier));

    let event_builder = EventBuilder::new(Kind::from(31925), "").tags(tags);

    let event_id = client
        .send_event_builder(event_builder)
        .await
        .map_err(|e| e.to_string())?;

    Ok(event_id.to_hex())
}
