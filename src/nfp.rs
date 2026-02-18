use nostr_sdk::prelude::*;
use serde_json::json;
use std::borrow::Cow;
use std::time::Duration;

use crate::models::{EventResponse, NFPCycleRequest, NFPObservationRequest, NFPSettingsRequest};
use crate::utils::{get_client, get_event_bech32, parse_public_key};

// Custom NFP event kinds
pub const NFP_OBSERVATION_KIND: u16 = 31103; // Daily observations
pub const NFP_CYCLE_KIND: u16 = 11103; // Cycle data
pub const NFP_SETTINGS_KIND: u16 = 31104; // User settings

pub async fn fetch_nfp_observations_core(
    pubkey: &str,
    nsec: Option<&str>,
    relays: &[String],
    range_start: Option<i64>,
    range_end: Option<i64>,
    client_nsec: Option<&str>,
) -> Result<Vec<EventResponse>, String> {
    let public_key = parse_public_key(pubkey)?;

    let client = if let Some(ns) = nsec {
        crate::utils::get_client_with_signer(ns, relays, client_nsec).await?
    } else {
        get_client(&Keys::generate(), relays).await?
    };

    let mut filter = Filter::new()
        .author(public_key)
        .kind(Kind::from(NFP_OBSERVATION_KIND));

    if let Some(start) = range_start {
        filter = filter.since(Timestamp::from(start as u64));
    }
    if let Some(end) = range_end {
        filter = filter.until(Timestamp::from(end as u64));
    }

    let events = client
        .fetch_events(filter, Duration::from_secs(10))
        .await
        .map_err(|e| e.to_string())?;

    let mut response = Vec::new();
    for e in events {
        let final_content = if let Some(_ns) = nsec {
            let signer = client.signer().await.map_err(|e| e.to_string())?;
            match signer.nip44_decrypt(&e.pubkey, &e.content).await {
                Ok(decrypted) => decrypted,
                Err(err) => format!("Error decrypting NFP Data: {}", err),
            }
        } else {
            "Encrypted NFP Data (No Secret Key)".to_string()
        };

        response.push(EventResponse {
            id: e.id.to_hex(),
            pubkey: e.pubkey.to_hex(),
            created_at: e.created_at.as_secs() as i64,
            kind: e.kind.as_u16(),
            tags: e.tags.iter().map(|t| t.clone().to_vec()).collect(),
            content: final_content,
            is_private: true,
            bech32: Some(get_event_bech32(&e, Some(relays.to_vec()))),
            private_tags: None,
            source: None,
        });
    }

    Ok(response)
}

pub async fn publish_nfp_observation_core(
    nsec: &str,
    relays: &[String],
    observation: NFPObservationRequest,
    client_nsec: Option<&str>,
) -> Result<String, String> {
    let client = crate::utils::get_client_with_signer(nsec, relays, client_nsec).await?;
    let my_public_key = client.public_key().await.map_err(|e| e.to_string())?;

    // Create tags
    let mut tags = vec![
        Tag::identifier(&observation.identifier),
        Tag::custom(
            TagKind::Custom(Cow::Borrowed("nfp_type")),
            vec!["observation".to_string()],
        ),
        Tag::custom(
            TagKind::Custom(Cow::Borrowed("date")),
            vec![observation.date.clone()],
        ),
    ];

    // Prepare content as JSON for encryption
    let content_json = json!({
        "date": observation.date,
        "temperature": observation.temperature,
        "cervical_fluid": observation.cervical_fluid,
        "cervix_position": observation.cervix_position,
        "bleeding": observation.bleeding,
        "notes": observation.notes,
        "disturbances": observation.disturbances
    });

    // Always encrypt NFP observations for privacy
    let signer = client.signer().await.map_err(|e| e.to_string())?;
    let encrypted_content = signer
        .nip44_encrypt(&my_public_key, &content_json.to_string())
        .await
        .map_err(|e| format!("Failed to encrypt observation: {e}"))?;

    tags.push(Tag::custom(
        TagKind::Custom(Cow::Borrowed("encrypted")),
        vec!["nip44".to_string()],
    ));

    let event_builder =
        EventBuilder::new(Kind::from(NFP_OBSERVATION_KIND), encrypted_content).tags(tags);

    let event_id = client
        .send_event_builder(event_builder)
        .await
        .map_err(|e| e.to_string())?;

    Ok(event_id.to_hex())
}

pub async fn update_nfp_observation_core(
    nsec: &str,
    relays: &[String],
    event_id: &str,
    observation: NFPObservationRequest,
    client_nsec: Option<&str>,
) -> Result<String, String> {
    let client = crate::utils::get_client_with_signer(nsec, relays, client_nsec).await?;
    let my_public_key = client.public_key().await.map_err(|e| e.to_string())?;
    let target_id = EventId::from_hex(event_id).map_err(|e| e.to_string())?;

    // Create tags
    let mut tags = vec![
        Tag::identifier(&observation.identifier),
        Tag::custom(
            TagKind::Custom(Cow::Borrowed("nfp_type")),
            vec!["observation".to_string()],
        ),
        Tag::custom(
            TagKind::Custom(Cow::Borrowed("date")),
            vec![observation.date.clone()],
        ),
    ];

    // Prepare content as JSON for encryption
    let content_json = json!({
        "date": observation.date,
        "temperature": observation.temperature,
        "cervical_fluid": observation.cervical_fluid,
        "cervix_position": observation.cervix_position,
        "bleeding": observation.bleeding,
        "notes": observation.notes,
        "disturbances": observation.disturbances
    });

    let signer = client.signer().await.map_err(|e| e.to_string())?;
    let encrypted_content = signer
        .nip44_encrypt(&my_public_key, &content_json.to_string())
        .await
        .map_err(|e| format!("Failed to encrypt observation: {e}"))?;

    tags.push(Tag::custom(
        TagKind::Custom(Cow::Borrowed("encrypted")),
        vec!["nip44".to_string()],
    ));

    // Add 'e' tag to point to old version
    tags.push(Tag::event(target_id));

    let event_builder =
        EventBuilder::new(Kind::from(NFP_OBSERVATION_KIND), encrypted_content).tags(tags);

    let new_event_id = client
        .send_event_builder(event_builder)
        .await
        .map_err(|e| e.to_string())?;

    Ok(new_event_id.to_hex())
}

pub async fn fetch_nfp_cycles_core(
    pubkey: &str,
    nsec: Option<&str>,
    relays: &[String],
    client_nsec: Option<&str>,
) -> Result<Vec<EventResponse>, String> {
    let public_key = parse_public_key(pubkey)?;

    let client = if let Some(ns) = nsec {
        crate::utils::get_client_with_signer(ns, relays, client_nsec).await?
    } else {
        get_client(&Keys::generate(), relays).await?
    };

    let filter = Filter::new()
        .author(public_key)
        .kind(Kind::from(NFP_CYCLE_KIND));

    let events = client
        .fetch_events(filter, Duration::from_secs(10))
        .await
        .map_err(|e| e.to_string())?;

    let mut response = Vec::new();
    for e in events {
        let final_content = if let Some(_ns) = nsec {
            let signer = client.signer().await.map_err(|e| e.to_string())?;
            match signer.nip44_decrypt(&e.pubkey, &e.content).await {
                Ok(decrypted) => decrypted,
                Err(err) => format!("Error decrypting Cycle Data: {}", err),
            }
        } else {
            "Encrypted Cycle Data (No Secret Key)".to_string()
        };

        response.push(EventResponse {
            id: e.id.to_hex(),
            pubkey: e.pubkey.to_hex(),
            created_at: e.created_at.as_secs() as i64,
            kind: e.kind.as_u16(),
            tags: e.tags.iter().map(|t| t.clone().to_vec()).collect(),
            content: final_content,
            is_private: true,
            bech32: Some(get_event_bech32(&e, Some(relays.to_vec()))),
            private_tags: None,
            source: None,
        });
    }

    Ok(response)
}

pub async fn publish_nfp_cycle_core(
    nsec: &str,
    relays: &[String],
    cycle: NFPCycleRequest,
    client_nsec: Option<&str>,
) -> Result<String, String> {
    let client = crate::utils::get_client_with_signer(nsec, relays, client_nsec).await?;
    let my_public_key = client.public_key().await.map_err(|e| e.to_string())?;

    // Create tags
    let mut tags = vec![
        Tag::identifier(&cycle.identifier),
        Tag::custom(
            TagKind::Custom(Cow::Borrowed("nfp_type")),
            vec!["cycle".to_string()],
        ),
        Tag::custom(
            TagKind::Custom(Cow::Borrowed("start_date")),
            vec![cycle.start_date.clone()],
        ),
    ];

    if let Some(end_date) = &cycle.end_date {
        tags.push(Tag::custom(
            TagKind::Custom(Cow::Borrowed("end_date")),
            vec![end_date.clone()],
        ));
    }

    // Prepare content as JSON for encryption
    let content_json = json!({
        "identifier": cycle.identifier,
        "start_date": cycle.start_date,
        "end_date": cycle.end_date,
        "length": cycle.length,
        "fertility_window_start": cycle.fertility_window_start,
        "fertility_window_end": cycle.fertility_window_end,
        "peak_day": cycle.peak_day,
        "ovulation_day": cycle.ovulation_day,
        "luteal_phase_length": cycle.luteal_phase_length
    });

    // Always encrypt NFP cycle data for privacy
    let signer = client.signer().await.map_err(|e| e.to_string())?;
    let encrypted_content = signer
        .nip44_encrypt(&my_public_key, &content_json.to_string())
        .await
        .map_err(|e| format!("Failed to encrypt cycle data: {e}"))?;

    tags.push(Tag::custom(
        TagKind::Custom(Cow::Borrowed("encrypted")),
        vec!["nip44".to_string()],
    ));

    let event_builder = EventBuilder::new(Kind::from(NFP_CYCLE_KIND), encrypted_content).tags(tags);

    let event_id = client
        .send_event_builder(event_builder)
        .await
        .map_err(|e| e.to_string())?;

    Ok(event_id.to_hex())
}

pub async fn fetch_nfp_settings_core(
    pubkey: &str,
    nsec: Option<&str>,
    relays: &[String],
    client_nsec: Option<&str>,
) -> Result<Option<EventResponse>, String> {
    let public_key = parse_public_key(pubkey)?;

    let client = if let Some(ns) = nsec {
        crate::utils::get_client_with_signer(ns, relays, client_nsec).await?
    } else {
        get_client(&Keys::generate(), relays).await?
    };

    let filter = Filter::new()
        .author(public_key)
        .kind(Kind::from(NFP_SETTINGS_KIND))
        .identifier("user-settings");

    let events = client
        .fetch_events(filter, Duration::from_secs(10))
        .await
        .map_err(|e| e.to_string())?;

    if let Some(e) = events.into_iter().next() {
        let final_content = if let Some(_ns) = nsec {
            let signer = client.signer().await.map_err(|e| e.to_string())?;
            match signer.nip44_decrypt(&e.pubkey, &e.content).await {
                Ok(decrypted) => decrypted,
                Err(err) => format!("Error decrypting Settings Data: {}", err),
            }
        } else {
            "Encrypted Settings Data (No Secret Key)".to_string()
        };

        Ok(Some(EventResponse {
            id: e.id.to_hex(),
            pubkey: e.pubkey.to_hex(),
            created_at: e.created_at.as_secs() as i64,
            kind: e.kind.as_u16(),
            tags: e.tags.iter().map(|t| t.clone().to_vec()).collect(),
            content: final_content,
            is_private: true,
            bech32: Some(get_event_bech32(&e, Some(relays.to_vec()))),
            private_tags: None,
            source: None,
        }))
    } else {
        Ok(None)
    }
}

pub async fn publish_nfp_settings_core(
    nsec: &str,
    relays: &[String],
    settings: NFPSettingsRequest,
    client_nsec: Option<&str>,
) -> Result<String, String> {
    let client = crate::utils::get_client_with_signer(nsec, relays, client_nsec).await?;
    let my_public_key = client.public_key().await.map_err(|e| e.to_string())?;

    // Create tags
    let mut tags = vec![
        Tag::identifier(&settings.identifier), // Always "user-settings"
        Tag::custom(
            TagKind::Custom(Cow::Borrowed("nfp_type")),
            vec!["settings".to_string()],
        ),
    ];

    // Prepare content as JSON for encryption
    let content_json = json!({
        "temperature_unit": settings.temperature_unit,
        "temperature_method": settings.temperature_method,
        "reminder_time": settings.reminder_time,
        "average_cycle_length": settings.average_cycle_length,
        "luteal_phase_length": settings.luteal_phase_length,
        "require_pin_on_open": settings.require_pin_on_open,
        "auto_lock_after_minutes": settings.auto_lock_after_minutes,
        "sync_enabled": settings.sync_enabled,
        "encrypted_backup": settings.encrypted_backup,
        "theme": settings.theme,
        "chart_style": settings.chart_style,
        "show_predictions": settings.show_predictions,
        "temperature_reminder": settings.temperature_reminder,
        "ovulation_prediction": settings.ovulation_prediction,
        "cycle_end_prediction": settings.cycle_end_prediction
    });

    // Always encrypt NFP settings for privacy
    let signer = client.signer().await.map_err(|e| e.to_string())?;
    let encrypted_content = signer
        .nip44_encrypt(&my_public_key, &content_json.to_string())
        .await
        .map_err(|e| format!("Failed to encrypt settings: {e}"))?;

    tags.push(Tag::custom(
        TagKind::Custom(Cow::Borrowed("encrypted")),
        vec!["nip44".to_string()],
    ));

    let event_builder =
        EventBuilder::new(Kind::from(NFP_SETTINGS_KIND), encrypted_content).tags(tags);

    let event_id = client
        .send_event_builder(event_builder)
        .await
        .map_err(|e| e.to_string())?;

    Ok(event_id.to_hex())
}

pub async fn delete_nfp_data_core(
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

pub async fn update_nfp_cycle_core(
    nsec: &str,
    relays: &[String],
    event_id: &str,
    cycle: NFPCycleRequest,
    client_nsec: Option<&str>,
) -> Result<String, String> {
    let client = crate::utils::get_client_with_signer(nsec, relays, client_nsec).await?;
    let my_public_key = client.public_key().await.map_err(|e| e.to_string())?;
    let target_id = EventId::from_hex(event_id).map_err(|e| e.to_string())?;

    let mut tags = vec![
        Tag::identifier(&cycle.identifier),
        Tag::custom(
            TagKind::Custom(Cow::Borrowed("nfp_type")),
            vec!["cycle".to_string()],
        ),
        Tag::custom(
            TagKind::Custom(Cow::Borrowed("start_date")),
            vec![cycle.start_date.clone()],
        ),
    ];

    if let Some(end_date) = &cycle.end_date {
        tags.push(Tag::custom(
            TagKind::Custom(Cow::Borrowed("end_date")),
            vec![end_date.clone()],
        ));
    }

    let content_json = json!({
        "identifier": cycle.identifier,
        "start_date": cycle.start_date,
        "end_date": cycle.end_date,
        "length": cycle.length,
        "fertility_window_start": cycle.fertility_window_start,
        "fertility_window_end": cycle.fertility_window_end,
        "peak_day": cycle.peak_day,
        "ovulation_day": cycle.ovulation_day,
        "luteal_phase_length": cycle.luteal_phase_length
    });

    let signer = client.signer().await.map_err(|e| e.to_string())?;
    let encrypted_content = signer
        .nip44_encrypt(&my_public_key, &content_json.to_string())
        .await
        .map_err(|e| format!("Failed to encrypt cycle data: {e}"))?;

    tags.push(Tag::custom(
        TagKind::Custom(Cow::Borrowed("encrypted")),
        vec!["nip44".to_string()],
    ));

    tags.push(Tag::event(target_id));

    let event_builder = EventBuilder::new(Kind::from(NFP_CYCLE_KIND), encrypted_content).tags(tags);

    let new_event_id = client
        .send_event_builder(event_builder)
        .await
        .map_err(|e| e.to_string())?;

    Ok(new_event_id.to_hex())
}

pub async fn update_nfp_settings_core(
    nsec: &str,
    relays: &[String],
    event_id: &str,
    settings: NFPSettingsRequest,
    client_nsec: Option<&str>,
) -> Result<String, String> {
    let client = crate::utils::get_client_with_signer(nsec, relays, client_nsec).await?;
    let my_public_key = client.public_key().await.map_err(|e| e.to_string())?;
    let target_id = EventId::from_hex(event_id).map_err(|e| e.to_string())?;

    let mut tags = vec![
        Tag::identifier(&settings.identifier),
        Tag::custom(
            TagKind::Custom(Cow::Borrowed("nfp_type")),
            vec!["settings".to_string()],
        ),
    ];

    let content_json = json!({
        "temperature_unit": settings.temperature_unit,
        "temperature_method": settings.temperature_method,
        "reminder_time": settings.reminder_time,
        "average_cycle_length": settings.average_cycle_length,
        "luteal_phase_length": settings.luteal_phase_length,
        "require_pin_on_open": settings.require_pin_on_open,
        "auto_lock_after_minutes": settings.auto_lock_after_minutes,
        "sync_enabled": settings.sync_enabled,
        "encrypted_backup": settings.encrypted_backup,
        "theme": settings.theme,
        "chart_style": settings.chart_style,
        "show_predictions": settings.show_predictions,
        "temperature_reminder": settings.temperature_reminder,
        "ovulation_prediction": settings.ovulation_prediction,
        "cycle_end_prediction": settings.cycle_end_prediction
    });

    let signer = client.signer().await.map_err(|e| e.to_string())?;
    let encrypted_content = signer
        .nip44_encrypt(&my_public_key, &content_json.to_string())
        .await
        .map_err(|e| format!("Failed to encrypt settings: {e}"))?;

    tags.push(Tag::custom(
        TagKind::Custom(Cow::Borrowed("encrypted")),
        vec!["nip44".to_string()],
    ));

    tags.push(Tag::event(target_id));

    let event_builder =
        EventBuilder::new(Kind::from(NFP_SETTINGS_KIND), encrypted_content).tags(tags);

    let new_event_id = client
        .send_event_builder(event_builder)
        .await
        .map_err(|e| e.to_string())?;

    Ok(new_event_id.to_hex())
}
