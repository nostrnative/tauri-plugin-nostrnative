pub mod calendar_sync;
pub mod rsvp;

use nostr_sdk::prelude::*;
use serde_json::json;
use std::borrow::Cow;
use std::time::Duration;

use crate::models::{CalendarEventRequest, CalendarRequest, EventResponse};
use crate::utils::{get_client, get_event_bech32, get_public_key_from_signer, parse_public_key};

pub async fn fetch_calendar_events_core(
    pubkey: &str,
    nsec: Option<&str>,
    relays: &[String],
    range_start: Option<i64>,
    range_end: Option<i64>,
    authors: Option<Vec<String>>,
    client_nsec: Option<&str>,
) -> Result<Vec<EventResponse>, String> {
    let public_key = parse_public_key(pubkey)?;

    let client = if let Some(ns) = nsec {
        crate::utils::get_client_with_signer(ns, relays, client_nsec).await?
    } else {
        get_client(&Keys::generate(), relays).await?
    };

    let mut authors_pks = vec![public_key];
    if let Some(auths) = &authors {
        for a in auths {
            if let Ok(pk) = PublicKey::from_hex(a)
                && !authors_pks.contains(&pk)
            {
                authors_pks.push(pk);
            }
        }
    }

    let mut filter = Filter::new()
        .authors(authors_pks)
        .kinds([Kind::from(31922), Kind::from(31923)]);

    if let Some(start) = range_start {
        filter = filter.since(Timestamp::from(start as u64));
    }
    if let Some(end) = range_end {
        filter = filter.until(Timestamp::from(end as u64));
    }

    let owned_events = client
        .fetch_events(filter, Duration::from_secs(10))
        .await
        .map_err(|e| e.to_string())?;

    let mut invited_filter = Filter::new()
        .kinds([Kind::from(31922), Kind::from(31923)])
        .pubkey(public_key);

    if let Some(start) = range_start {
        invited_filter = invited_filter.since(Timestamp::from(start as u64));
    }
    if let Some(end) = range_end {
        invited_filter = invited_filter.until(Timestamp::from(end as u64));
    }

    if let Some(auths) = authors {
        let pks: Result<Vec<PublicKey>, String> = auths
            .into_iter()
            .map(|a| PublicKey::from_hex(&a).map_err(|e| e.to_string()))
            .collect();
        invited_filter = invited_filter.authors(pks?);
    }

    let invited_events = client
        .fetch_events(invited_filter, Duration::from_secs(10))
        .await
        .map_err(|e| e.to_string())?;

    let events: Vec<_> = owned_events.into_iter().chain(invited_events).collect();

    let mut response = Vec::new();
    for e in events {
        let mut start_val: Option<i64> = None;
        let mut end_val: Option<i64> = None;
        let mut is_private = false;
        let mut private_title: Option<String> = None;
        let mut private_description: Option<String> = None;
        let mut start_tzid: Option<String> = None;
        let mut end_tzid: Option<String> = None;

        for tag in e.tags.iter() {
            let t = tag.clone().to_vec();
            if t.len() >= 2 {
                if t[0] == "start" {
                    start_val = t[1].parse::<i64>().ok();
                } else if t[0] == "end" {
                    end_val = t[1].parse::<i64>().ok();
                } else if t[0] == "private" {
                    is_private = true;
                } else if t[0] == "start_tzid" {
                    start_tzid = Some(t[1].clone());
                } else if t[0] == "end_tzid" {
                    end_tzid = Some(t[1].clone());
                }
            }
        }

        let start = start_val.unwrap_or(e.created_at.as_secs() as i64);

        let mut include = true;
        if let Some(rs) = range_start {
            if let Some(ee) = end_val {
                if ee < rs {
                    include = false;
                }
            } else if start < rs {
                include = false;
            }
        }
        if let Some(re) = range_end
            && start > re
        {
            include = false;
        }

        if include {
            if is_private {
                let signer = client.signer().await.map_err(|e| e.to_string())?;
                if let Ok(decrypted) = signer.nip44_decrypt(&e.pubkey, &e.content).await
                    && let Ok(json_content) = serde_json::from_str::<serde_json::Value>(&decrypted)
                {
                    private_title = json_content["title"].as_str().map(|s| s.to_string());
                    private_description =
                        json_content["description"].as_str().map(|s| s.to_string());
                }
            }

            let (final_title, final_content) = if is_private {
                if let (Some(t), Some(d)) = (private_title, private_description) {
                    (Some(t), d)
                } else {
                    (None, "Encrypted Content".to_string())
                }
            } else {
                (None, e.content.to_string())
            };

            let mut response_tags = e
                .tags
                .iter()
                .map(|t| t.clone().to_vec())
                .collect::<Vec<_>>();

            if let Some(t) = final_title {
                response_tags.retain(|tag| tag.is_empty() || tag[0] != "title");
                response_tags.push(vec!["title".to_string(), t]);
            } else if is_private
                && !response_tags
                    .iter()
                    .any(|tag| !tag.is_empty() && tag[0] == "title")
            {
                response_tags.push(vec!["title".to_string(), "Private Event".to_string()]);
            }

            // Ensure timezone tags are included in the response if they exist
            if let Some(start_tz) = &start_tzid {
                response_tags.retain(|tag| tag.is_empty() || tag[0] != "start_tzid");
                response_tags.push(vec!["start_tzid".to_string(), start_tz.clone()]);
            }
            if let Some(end_tz) = &end_tzid {
                response_tags.retain(|tag| tag.is_empty() || tag[0] != "end_tzid");
                response_tags.push(vec!["end_tzid".to_string(), end_tz.clone()]);
            }

            response.push(EventResponse {
                id: e.id.to_hex(),
                pubkey: e.pubkey.to_hex(),
                created_at: e.created_at.as_secs() as i64,
                kind: e.kind.as_u16(),
                tags: response_tags,
                content: final_content,
                is_private,
                bech32: Some(get_event_bech32(&e, Some(relays.to_vec()))),
                private_tags: None,
                source: None,
            });
        }
    }

    Ok(response)
}

pub async fn publish_batch_calendar_events_core(
    nsec: &str,
    relays: &[String],
    events: Vec<CalendarEventRequest>,
    client_nsec: Option<&str>,
) -> Result<Vec<String>, String> {
    let client = crate::utils::get_client_with_signer(nsec, relays, client_nsec).await?;
    let my_public_key = get_public_key_from_signer(nsec)?;

    let mut event_ids = Vec::new();

    for event_data in events {
        if let Some(old_id) = &event_data.old_event_id
            && let Ok(oid) = EventId::from_hex(old_id)
        {
            let event_builder = EventBuilder::delete(EventDeletionRequest::new().ids(vec![oid]));
            let _ = client.send_event_builder(event_builder).await;
        }

        let kind = if event_data.is_all_day {
            Kind::from(31922)
        } else {
            Kind::from(31923)
        };

        let mut tags = vec![
            Tag::identifier(&event_data.identifier),
            Tag::custom(
                TagKind::Custom(Cow::Borrowed("start")),
                vec![event_data.start.to_string()],
            ),
        ];

        let content = if event_data.is_private.unwrap_or(false) {
            tags.push(Tag::custom(
                TagKind::Custom(Cow::Borrowed("private")),
                vec!["true".to_string()],
            ));

            let payload = json!({
                "title": event_data.title,
                "description": event_data.description.clone().unwrap_or_default()
            });

            let signer = client.signer().await.map_err(|e| e.to_string())?;
            match signer
                .nip44_encrypt(&my_public_key, &payload.to_string())
                .await
            {
                Ok(c) => c,
                Err(e) => return Err(format!("Failed to encrypt event: {e}")),
            }
        } else {
            tags.push(Tag::custom(
                TagKind::Custom(Cow::Borrowed("title")),
                vec![event_data.title.clone()],
            ));
            event_data.description.clone().unwrap_or_default()
        };

        if let Some(end) = event_data.end {
            tags.push(Tag::custom(
                TagKind::Custom(Cow::Borrowed("end")),
                vec![end.to_string()],
            ));
        }

        if let Some(loc) = &event_data.location {
            tags.push(Tag::custom(
                TagKind::Custom(Cow::Borrowed("location")),
                vec![loc.clone()],
            ));
        }

        if let Some(reminder) = event_data.reminder_minutes
            && reminder > 0
        {
            tags.push(Tag::custom(
                TagKind::Custom(Cow::Borrowed("reminder")),
                vec![reminder.to_string()],
            ));
        }

        if let Some(c) = &event_data.color {
            tags.push(Tag::custom(
                TagKind::Custom(Cow::Borrowed("color")),
                vec![c.clone()],
            ));
        }

        if let Some(p_tags) = &event_data.p_tags {
            for pk in p_tags {
                tags.push(Tag::custom(
                    TagKind::Custom(Cow::Borrowed("p")),
                    vec![pk.clone()],
                ));
            }
        }

        if let Some(parent) = &event_data.parent {
            tags.push(Tag::custom(
                TagKind::Custom(Cow::Borrowed("parent")),
                vec![parent.clone()],
            ));
        }

        if let Some(freq) = &event_data.freq {
            tags.push(Tag::custom(
                TagKind::Custom(Cow::Borrowed("freq")),
                vec![freq.clone()],
            ));
        }

        if let Some(until) = &event_data.until {
            tags.push(Tag::custom(
                TagKind::Custom(Cow::Borrowed("until")),
                vec![until.to_string()],
            ));
        }

        // Add timezone tags according to NIP-52
        if let Some(start_tzid) = &event_data.start_tzid {
            if !start_tzid.is_empty() && !event_data.is_all_day {
                tags.push(Tag::custom(
                    TagKind::Custom(Cow::Borrowed("start_tzid")),
                    vec![start_tzid.clone()],
                ));

                // Per NIP-52: If end_tzid is omitted and start_tzid is provided,
                // end_tzid defaults to start_tzid
                if let Some(end_tzid) = &event_data.end_tzid {
                    if !end_tzid.is_empty() {
                        tags.push(Tag::custom(
                            TagKind::Custom(Cow::Borrowed("end_tzid")),
                            vec![end_tzid.clone()],
                        ));
                    } else {
                        tags.push(Tag::custom(
                            TagKind::Custom(Cow::Borrowed("end_tzid")),
                            vec![start_tzid.clone()],
                        ));
                    }
                } else {
                    tags.push(Tag::custom(
                        TagKind::Custom(Cow::Borrowed("end_tzid")),
                        vec![start_tzid.clone()],
                    ));
                }
            }
        } else if let Some(end_tzid) = &event_data.end_tzid {
            if !end_tzid.is_empty() && !event_data.is_all_day {
                tags.push(Tag::custom(
                    TagKind::Custom(Cow::Borrowed("end_tzid")),
                    vec![end_tzid.clone()],
                ));
            }
        }

        let mut event_builder = EventBuilder::new(kind, content).tags(tags);

        if event_data.use_different_timestamp.unwrap_or(false) {
            let real_created_at = Timestamp::now();
            event_builder =
                event_builder.custom_created_at(Timestamp::from(event_data.start as u64));
            event_builder = event_builder.tag(Tag::custom(
                TagKind::Custom(Cow::Borrowed("created_at")),
                vec![real_created_at.as_secs().to_string()],
            ));
        }

        if let Some(cal_id) = &event_data.calendar_id
            && !cal_id.is_empty()
        {
            let author = my_public_key.to_hex();
            let coordinate = format!("31924:{author}:{cal_id}");
            event_builder = event_builder.tag(Tag::custom(
                TagKind::Custom(Cow::Borrowed("a")),
                vec![coordinate],
            ));
        }

        match client.send_event_builder(event_builder).await {
            Ok(id) => event_ids.push(id.to_hex()),
            Err(e) => return Err(format!("Failed to publish event: {e}")),
        }
    }

    Ok(event_ids)
}

pub async fn delete_calendar_event_core(
    nsec: &str,
    relays: &[String],
    event_ids: Vec<String>,
    client_nsec: Option<&str>,
) -> Result<String, String> {
    let client = crate::utils::get_client_with_signer(nsec, relays, client_nsec).await?;

    let mut event_id_objs = Vec::new();
    for id in event_ids {
        if let Ok(oid) = EventId::from_hex(&id) {
            event_id_objs.push(oid);
        }
    }

    if event_id_objs.is_empty() {
        return Ok("".to_string());
    }

    let event_builder = EventBuilder::delete(EventDeletionRequest::new().ids(event_id_objs));

    let deletion_id = client
        .send_event_builder(event_builder)
        .await
        .map_err(|e| e.to_string())?;

    Ok(deletion_id.to_hex())
}

pub async fn fetch_calendars_core(
    pubkey: &str,
    relays: &[String],
) -> Result<Vec<EventResponse>, String> {
    let public_key = parse_public_key(pubkey)?;
    let ephemeral_keys = Keys::generate();
    let client = get_client(&ephemeral_keys, relays).await?;

    let filter = Filter::new()
        .authors(vec![public_key])
        .kind(Kind::from(31924));

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
            bech32: Some(get_event_bech32(&e, Some(relays.to_vec()))),
            private_tags: None,
            source: None,
        })
        .collect();

    Ok(response)
}

pub async fn publish_calendar_core(
    nsec: &str,
    relays: &[String],
    calendar: CalendarRequest,
    client_nsec: Option<&str>,
) -> Result<String, String> {
    let client = crate::utils::get_client_with_signer(nsec, relays, client_nsec).await?;

    let tags = vec![
        Tag::identifier(&calendar.identifier),
        Tag::custom(TagKind::Custom(Cow::Borrowed("name")), vec![calendar.name]),
        Tag::custom(
            TagKind::Custom(Cow::Borrowed("description")),
            vec![calendar.description.unwrap_or_default()],
        ),
    ];

    let event_builder = EventBuilder::new(Kind::from(31924), "").tags(tags);

    let event_id = client
        .send_event_builder(event_builder)
        .await
        .map_err(|e| e.to_string())?;

    Ok(event_id.to_hex())
}

pub async fn delete_calendar_core(
    nsec: &str,
    relays: &[String],
    identifier: String,
    client_nsec: Option<&str>,
) -> Result<String, String> {
    let client = crate::utils::get_client_with_signer(nsec, relays, client_nsec).await?;
    let my_public_key = get_public_key_from_signer(nsec)?;

    let coordinate = Coordinate::new(Kind::from(31924), my_public_key).identifier(identifier);
    let event_builder =
        EventBuilder::delete(EventDeletionRequest::new().coordinates(vec![coordinate.clone()]))
            .tag(Tag::custom(
                TagKind::Custom(Cow::Borrowed("k")),
                vec![Kind::from(31924).as_u16().to_string()],
            ));

    let deletion_id = client
        .send_event_builder(event_builder)
        .await
        .map_err(|e| e.to_string())?;

    Ok(deletion_id.to_hex())
}
