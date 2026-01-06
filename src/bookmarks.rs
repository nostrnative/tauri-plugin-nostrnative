use nostr_sdk::prelude::*;
use std::collections::HashSet;
use std::time::Duration;

use crate::models::{BookmarkSource, EventResponse};
use crate::utils::{get_client, get_event_bech32, parse_public_key};

pub async fn fetch_bookmarks_core(
    pubkey: &str,
    nsec: Option<&str>,
    relays: &[String],
) -> Result<Vec<EventResponse>, String> {
    let public_key = parse_public_key(pubkey)?;
    let ephemeral_keys = Keys::generate();
    let client = get_client(&ephemeral_keys, relays).await?;

    let filter = Filter::new().authors(vec![public_key]).kinds([
        Kind::Bookmarks,   // Bookmark List
        Kind::BookmarkSet, // Bookmark List (Standard)
        Kind::from(30001), // Bookmark Sets
        Kind::from(39701), // Website Bookmarks
    ]);

    let list_events = client
        .fetch_events(filter, Duration::from_secs(10))
        .await
        .map_err(|e| e.to_string())?;

    let mut public_ids = HashSet::new();
    let mut private_ids = HashSet::new();
    let mut website_events = Vec::new();

    let keys = nsec.and_then(|s| Keys::parse(s).ok());

    for e in list_events {
        match e.kind.as_u16() {
            39701 => {
                website_events.push(e);
            }
            30001 | 10003 | 30003 => {
                for tag in e.tags.iter() {
                    let t = tag.clone().to_vec();
                    if t.len() >= 2 && (t[0] == "e" || t[0] == "a") {
                        public_ids.insert(t[1].to_string());
                    }
                }

                if !e.content.is_empty() {
                    if let Some(k) = &keys {
                        let sk = k.secret_key();
                        let decrypted =
                            if let Ok(content) = nip44::decrypt(sk, &e.pubkey, &e.content) {
                                Some(content)
                            } else {
                                #[allow(deprecated)]
                                nip04::decrypt(sk, &e.pubkey, &e.content).ok()
                            };

                        if let Some(content) = decrypted {
                            if let Ok(private_tags) =
                                serde_json::from_str::<Vec<Vec<String>>>(&content)
                            {
                                for t in private_tags {
                                    if t.len() >= 2 && (t[0] == "e" || t[0] == "a") {
                                        private_ids.insert(t[1].clone());
                                    }
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    let mut responses = Vec::new();
    let mut seen_website_ids = HashSet::new();

    for e in website_events {
        if seen_website_ids.insert(e.id) {
            responses.push(EventResponse {
                id: e.id.to_hex(),
                pubkey: e.pubkey.to_hex(),
                created_at: e.created_at.as_secs() as i64,
                kind: e.kind.as_u16(),
                tags: e.tags.iter().map(|t| t.clone().to_vec()).collect(),
                content: e.content.to_string(),
                is_private: false,
                bech32: Some(get_event_bech32(&e, Some(relays.to_vec()))),
                private_tags: None,
                source: Some(BookmarkSource::Website),
            });
        }
    }

    for id in public_ids {
        responses.push(EventResponse {
            id,
            pubkey: String::new(),
            created_at: 0,
            kind: 0,
            tags: Vec::new(),
            content: String::new(),
            is_private: false,
            bech32: None,
            private_tags: None,
            source: Some(BookmarkSource::Public),
        });
    }

    for id in private_ids {
        responses.push(EventResponse {
            id,
            pubkey: String::new(),
            created_at: 0,
            kind: 0,
            tags: Vec::new(),
            content: String::new(),
            is_private: true,
            bech32: None,
            private_tags: None,
            source: Some(BookmarkSource::Private),
        });
    }

    Ok(responses)
}

pub async fn fetch_event_details_core(
    identifiers: Vec<String>,
    relays: &[String],
) -> Result<Vec<EventResponse>, String> {
    let ephemeral_keys = Keys::generate();
    let client = get_client(&ephemeral_keys, relays).await?;

    let mut event_ids = Vec::new();
    let mut coordinate_filters = Vec::new();

    for id in identifiers {
        let mut clean_id = id.trim();
        if clean_id.starts_with("nostr:") {
            clean_id = &clean_id[6..];
        }
        if clean_id.starts_with('@') {
            clean_id = &clean_id[1..];
        }

        if clean_id.contains(':') {
            let parts: Vec<&str> = clean_id.split(':').collect();
            if parts.len() >= 3 {
                if let (Ok(kind), Ok(pubkey)) =
                    (parts[0].parse::<u16>(), parse_public_key(parts[1]))
                {
                    let filter = Filter::new()
                        .kind(Kind::from(kind))
                        .author(pubkey)
                        .identifier(parts[2]);
                    coordinate_filters.push(filter);
                }
            }
        } else if let Ok(event_id) = EventId::from_hex(clean_id) {
            event_ids.push(event_id);
        } else if let Ok(nip19) = Nip19::from_bech32(clean_id) {
            match nip19 {
                Nip19::Event(e) => event_ids.push(e.event_id),
                Nip19::EventId(id) => event_ids.push(id),
                Nip19::Coordinate(c) => {
                    let filter = Filter::new()
                        .kind(c.kind)
                        .author(c.public_key)
                        .identifier(c.identifier.clone());
                    coordinate_filters.push(filter);
                }
                _ => {}
            }
        }
    }

    if event_ids.is_empty() && coordinate_filters.is_empty() {
        return Ok(Vec::new());
    }

    let mut all_events = Vec::new();

    if !event_ids.is_empty() {
        let filter = Filter::new().ids(event_ids);
        if let Ok(events) = client.fetch_events(filter, Duration::from_secs(10)).await {
            all_events.extend(events);
        }
    }

    for filter in coordinate_filters {
        if let Ok(events) = client.fetch_events(filter, Duration::from_secs(5)).await {
            all_events.extend(events);
        }
    }

    let response = all_events
        .into_iter()
        .map(|e| EventResponse {
            id: e.id.to_hex(),
            pubkey: e.pubkey.to_hex(),
            created_at: e.created_at.as_secs() as i64,
            kind: e.kind.as_u16(),
            tags: e.tags.iter().map(|t| t.clone().to_vec()).collect(),
            content: e.content.to_string(),
            is_private: false,
            bech32: Some(get_event_bech32(&e, Some(relays.to_vec()))),
            private_tags: None,
            source: None,
        })
        .collect();

    Ok(response)
}

pub async fn save_bookmark_core(
    nsec: &str,
    relays: &[String],
    event_id: String,
    source: BookmarkSource,
) -> Result<String, String> {
    let keys = Keys::parse(nsec).map_err(|e| e.to_string())?;
    let client = get_client(&keys, relays).await?;

    let filter = Filter::new()
        .authors(vec![keys.public_key()])
        .kind(Kind::from(30001))
        .identifier("bookmark")
        .limit(1);

    let events = client
        .fetch_events(filter, Duration::from_secs(5))
        .await
        .map_err(|e| e.to_string())?;

    let mut public_tags = Vec::new();
    let mut content = String::new();

    if let Some(existing_event) = events.first() {
        public_tags = existing_event
            .tags
            .iter()
            .map(|t| t.clone().to_vec())
            .collect();
        content = existing_event.content.clone();
    } else {
        public_tags.push(vec!["d".to_string(), "bookmark".to_string()]);
    }

    match source {
        BookmarkSource::Public => {
            let exists_public = public_tags.iter().any(|t| {
                t.get(1).map(|v| v.as_str()) == Some(event_id.as_str())
                    && (t.first().map(|v| v.as_str()) == Some("e")
                        || t.first().map(|v| v.as_str()) == Some("a"))
            });

            if exists_public {
                return Ok("already_exists".to_string());
            }

            if event_id.contains(':') {
                public_tags.push(vec!["a".to_string(), event_id.clone()]);
            } else {
                public_tags.push(vec!["e".to_string(), event_id.clone()]);
            }
        }
        BookmarkSource::Private => {
            let sk = keys.secret_key();
            let mut private_tags = Vec::new();
            let mut used_nip44 = true;

            if !content.is_empty() {
                let decrypted = if let Ok(c) = nip44::decrypt(sk, &keys.public_key(), &content) {
                    Some(c)
                } else {
                    used_nip44 = false;
                    #[allow(deprecated)]
                    nip04::decrypt(sk, &keys.public_key(), &content).ok()
                };

                if let Some(c) = decrypted {
                    if let Ok(tags) = serde_json::from_str::<Vec<Vec<String>>>(&c) {
                        private_tags = tags;
                    }
                }
            }

            let exists_private = private_tags
                .iter()
                .any(|t| t.len() >= 2 && t[1] == event_id && (t[0] == "e" || t[0] == "a"));

            if exists_private {
                return Ok("already_exists".to_string());
            }

            if event_id.contains(':') {
                private_tags.push(vec!["a".to_string(), event_id.clone()]);
            } else {
                private_tags.push(vec!["e".to_string(), event_id.clone()]);
            }

            let json = serde_json::to_string(&private_tags).map_err(|e| e.to_string())?;
            content = if used_nip44 {
                nip44::encrypt(sk, &keys.public_key(), json, nip44::Version::V2)
                    .map_err(|e| e.to_string())?
            } else {
                #[allow(deprecated)]
                nip04::encrypt(sk, &keys.public_key(), json).map_err(|e| e.to_string())?
            };
        }
        BookmarkSource::Website => {
            return Err("Cannot save website bookmark via this command".to_string());
        }
    }

    let tags: Vec<Tag> = public_tags
        .into_iter()
        .filter_map(|t| Tag::parse(&t).ok())
        .collect();
    let builder = EventBuilder::new(Kind::from(30001), content).tags(tags);
    let id = client
        .send_event_builder(builder)
        .await
        .map_err(|e| e.to_string())?;

    Ok(id.to_hex())
}

pub async fn remove_bookmark_core(
    nsec: &str,
    relays: &[String],
    event_id: String,
    source: BookmarkSource,
) -> Result<String, String> {
    let keys = Keys::parse(nsec).map_err(|e| e.to_string())?;
    let client = get_client(&keys, relays).await?;

    let filter = Filter::new()
        .authors(vec![keys.public_key()])
        .kind(Kind::from(30001))
        .identifier("bookmark")
        .limit(1);

    let events = client
        .fetch_events(filter, Duration::from_secs(5))
        .await
        .map_err(|e| e.to_string())?;

    if let Some(existing_event) = events.first() {
        let mut public_tags: Vec<Vec<String>> = existing_event
            .tags
            .iter()
            .map(|t| t.clone().to_vec())
            .collect();
        let mut content = existing_event.content.clone();
        let mut modified = false;

        match source {
            BookmarkSource::Public => {
                let old_pub_len = public_tags.len();
                public_tags.retain(|t| {
                    let matches_id = t.get(1).map(|v| v.as_str()) == Some(event_id.as_str());
                    let is_bookmark_tag = t.first().map(|v| v.as_str()) == Some("e")
                        || t.first().map(|v| v.as_str()) == Some("a");

                    !(is_bookmark_tag && matches_id)
                });

                if public_tags.len() < old_pub_len {
                    modified = true;
                }
            }
            BookmarkSource::Private => {
                if !content.is_empty() {
                    let sk = keys.secret_key();
                    let mut used_nip44 = true;
                    let decrypted = if let Ok(c) = nip44::decrypt(sk, &keys.public_key(), &content)
                    {
                        Some(c)
                    } else {
                        used_nip44 = false;
                        #[allow(deprecated)]
                        nip04::decrypt(sk, &keys.public_key(), &content).ok()
                    };

                    if let Some(c) = decrypted {
                        if let Ok(mut private_tags) = serde_json::from_str::<Vec<Vec<String>>>(&c) {
                            let old_priv_len = private_tags.len();
                            private_tags.retain(|t| {
                                !(t.len() >= 2 && t[1] == event_id && (t[0] == "e" || t[0] == "a"))
                            });

                            if private_tags.len() < old_priv_len {
                                let json = serde_json::to_string(&private_tags)
                                    .map_err(|e| e.to_string())?;
                                content = if used_nip44 {
                                    nip44::encrypt(sk, &keys.public_key(), json, nip44::Version::V2)
                                        .map_err(|e| e.to_string())?
                                } else {
                                    #[allow(deprecated)]
                                    nip04::encrypt(sk, &keys.public_key(), json)
                                        .map_err(|e| e.to_string())?
                                };
                                modified = true;
                            }
                        }
                    }
                }
            }
            BookmarkSource::Website => {
                return Err("Removing website bookmarks is not yet supported".to_string());
            }
        }

        if modified {
            let tags: Vec<Tag> = public_tags
                .into_iter()
                .filter_map(|t| Tag::parse(&t).ok())
                .collect();
            let builder = EventBuilder::new(Kind::from(30001), content).tags(tags);
            let id = client
                .send_event_builder(builder)
                .await
                .map_err(|e| e.to_string())?;
            return Ok(id.to_hex());
        }
    }

    Ok("not_found".to_string())
}
