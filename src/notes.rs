use nostr_sdk::prelude::*;
use std::time::Duration;

use crate::models::{BookmarkSource, EventResponse};
use crate::utils::{get_client, get_event_bech32, parse_public_key};

#[derive(serde::Deserialize, Debug, Clone)]
pub struct NoteRequest {
    pub content: String,
    pub note_type: String,
    pub identifier: Option<String>,
    pub group_id: Option<String>,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct NotebookRequest {
    pub title: String,
    pub description: Option<String>,
}

pub async fn create_note_core(
    nsec: &str,
    relays: &[String],
    note: NoteRequest,
    client_nsec: Option<&str>,
) -> Result<String, String> {
    let client = crate::utils::get_client_with_signer(nsec, relays, client_nsec).await?;
    let public_key = client
        .signer()
        .await
        .unwrap()
        .get_public_key()
        .await
        .unwrap();

    let identifier = note.identifier.unwrap_or_else(|| {
        format!(
            "note_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        )
    });

    // Encrypt content (NIP-44) via signer (supports both local keys and bunker)
    let content_json = serde_json::json!({
        "content": note.content,
        "group_id": note.group_id
    })
    .to_string();

    let signer = client.signer().await.map_err(|e| e.to_string())?;
    let encrypted_content = signer
        .nip44_encrypt(&public_key, &content_json)
        .await
        .map_err(|e| e.to_string())?;

    let tags = vec![
        Tag::parse(["d", &identifier]).map_err(|e| e.to_string())?,
        Tag::parse(["t", &note.note_type]).map_err(|e| e.to_string())?,
    ];

    let builder = EventBuilder::new(Kind::from(31323), encrypted_content).tags(tags);
    let id = client
        .send_event_builder(builder)
        .await
        .map_err(|e| e.to_string())?;

    // If group_id is present, try to add this note to the notebook
    if let Some(group_id) = note.group_id {
        let _ = add_note_to_notebook_core(nsec, relays, group_id, identifier, client_nsec).await;
    }

    Ok(id.to_hex())
}

pub async fn update_note_core(
    nsec: &str,
    relays: &[String],
    identifier: String,
    note: NoteRequest,
    client_nsec: Option<&str>,
) -> Result<String, String> {
    let client = crate::utils::get_client_with_signer(nsec, relays, client_nsec).await?;
    let public_key = client
        .signer()
        .await
        .unwrap()
        .get_public_key()
        .await
        .unwrap();

    // Encrypt content (NIP-44) via signer (supports both local keys and bunker)
    let content_json = serde_json::json!({
        "content": note.content,
        "group_id": note.group_id
    })
    .to_string();

    let signer = client.signer().await.map_err(|e| e.to_string())?;
    let encrypted_content = signer
        .nip44_encrypt(&public_key, &content_json)
        .await
        .map_err(|e| e.to_string())?;

    let tags = vec![
        Tag::parse(["d", &identifier]).map_err(|e| e.to_string())?,
        Tag::parse(["t", &note.note_type]).map_err(|e| e.to_string())?,
    ];

    let builder = EventBuilder::new(Kind::from(31323), encrypted_content).tags(tags);
    let id = client
        .send_event_builder(builder)
        .await
        .map_err(|e| e.to_string())?;

    if let Some(group_id) = note.group_id {
        let _ = add_note_to_notebook_core(nsec, relays, group_id, identifier, client_nsec).await;
    }

    Ok(id.to_hex())
}

pub async fn delete_note_core(
    nsec: &str,
    relays: &[String],
    note_id: String,
    client_nsec: Option<&str>,
) -> Result<String, String> {
    let client = crate::utils::get_client_with_signer(nsec, relays, client_nsec).await?;

    let _ = EventId::from_hex(&note_id).map_err(|e| e.to_string())?;
    let tags = vec![Tag::parse(["e", &note_id]).map_err(|e| e.to_string())?];

    let builder = EventBuilder::new(Kind::EventDeletion, "").tags(tags);
    let id = client
        .send_event_builder(builder)
        .await
        .map_err(|e| e.to_string())?;

    Ok(id.to_hex())
}

pub async fn fetch_notes_core(
    pubkey: &str,
    nsec: Option<&str>,
    relays: &[String],
    note_type: Option<String>,
    client_nsec: Option<&str>,
) -> Result<Vec<EventResponse>, String> {
    let public_key = parse_public_key(pubkey)?;

    let client = if let Some(ns) = nsec {
        crate::utils::get_client_with_signer(ns, relays, client_nsec).await?
    } else {
        get_client(&Keys::generate(), relays).await?
    };

    let filter = Filter::new()
        .authors(vec![public_key])
        .kind(Kind::from(31323));

    let events = client
        .fetch_events(filter, Duration::from_secs(10))
        .await
        .map_err(|e| e.to_string())?;

    let mut responses: Vec<EventResponse> = Vec::new();

    for e in events {
        let mut content = e.content.to_string();
        let mut group_id = None;

        if nsec.is_some() {
            let signer = client.signer().await.map_err(|e| e.to_string())?;
            if let Ok(decrypted) = signer.nip44_decrypt(&e.pubkey, &e.content).await {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&decrypted) {
                    if let Some(c) = json.get("content").and_then(|v| v.as_str()) {
                        content = c.to_string();
                    }
                    if let Some(g) = json.get("group_id").and_then(|v| v.as_str()) {
                        group_id = Some(g.to_string());
                    }
                } else {
                    content = decrypted;
                }
            }
        }

        let mut tags = e
            .tags
            .iter()
            .map(|t| t.as_slice().to_vec())
            .collect::<Vec<_>>();
        if let Some(gid) = group_id {
            tags.push(vec!["group_id".to_string(), gid]);
        }

        responses.push(EventResponse {
            id: e.id.to_hex(),
            pubkey: e.pubkey.to_hex(),
            created_at: e.created_at.as_secs() as i64,
            kind: e.kind.as_u16(),
            tags,
            content,
            is_private: true,
            bech32: Some(get_event_bech32(&e, Some(relays.to_vec()))),
            private_tags: None,
            source: None,
        });
    }

    if let Some(nt) = note_type {
        responses.retain(|r| {
            r.tags.iter().any(|t| {
                let s = t.as_slice();
                s.len() >= 2 && s[0] == "t" && s[1] == nt
            })
        });
    }

    Ok(responses)
}

pub async fn create_notebook_core(
    nsec: &str,
    relays: &[String],
    notebook: NotebookRequest,
    client_nsec: Option<&str>,
) -> Result<String, String> {
    let client = crate::utils::get_client_with_signer(nsec, relays, client_nsec).await?;
    let public_key = client
        .signer()
        .await
        .unwrap()
        .get_public_key()
        .await
        .unwrap();

    let identifier = format!(
        "notebook_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    );

    let mut tags = vec![
        Tag::parse(["d", &identifier]).map_err(|e| e.to_string())?,
        Tag::parse(["title", &notebook.title]).map_err(|e| e.to_string())?,
        Tag::parse(["t", "notebook"]).map_err(|e| e.to_string())?,
    ];

    if let Some(desc) = notebook.description {
        tags.push(Tag::parse(["description", &desc]).map_err(|e| e.to_string())?);
    }

    let signer = client.signer().await.map_err(|e| e.to_string())?;
    let encrypted_content = signer
        .nip44_encrypt(&public_key, "[]")
        .await
        .map_err(|e| e.to_string())?;

    let builder = EventBuilder::new(Kind::from(31301), encrypted_content).tags(tags);
    let id = client
        .send_event_builder(builder)
        .await
        .map_err(|e| e.to_string())?;

    Ok(id.to_hex())
}

pub async fn fetch_notebooks_core(
    pubkey: &str,
    relays: &[String],
    nsec: Option<&str>,
    client_nsec: Option<&str>,
) -> Result<Vec<EventResponse>, String> {
    let public_key = parse_public_key(pubkey)?;

    let client = if let Some(ns) = nsec {
        crate::utils::get_client_with_signer(ns, relays, client_nsec).await?
    } else {
        get_client(&Keys::generate(), relays).await?
    };

    let filter = Filter::new()
        .authors(vec![public_key])
        .kind(Kind::from(31301));

    let events = client
        .fetch_events(filter, Duration::from_secs(10))
        .await
        .map_err(|e| e.to_string())?;

    let mut responses = Vec::new();

    for e in events {
        let mut private_tags = None;
        if !e.content.is_empty() && nsec.is_some() {
            let signer = client.signer().await.map_err(|e| e.to_string())?;
            if let Ok(decrypted) = signer.nip44_decrypt(&e.pubkey, &e.content).await
                && let Ok(ptags) = serde_json::from_str::<Vec<Vec<String>>>(&decrypted)
            {
                private_tags = Some(ptags);
            }
        }

        responses.push(EventResponse {
            id: e.id.to_hex(),
            pubkey: e.pubkey.to_hex(),
            created_at: e.created_at.as_secs() as i64,
            kind: e.kind.as_u16(),
            tags: e.tags.iter().map(|t| t.as_slice().to_vec()).collect(),
            content: e.content.to_string(),
            is_private: true,
            bech32: Some(get_event_bech32(&e, Some(relays.to_vec()))),
            private_tags,
            source: Some(BookmarkSource::Private),
        });
    }

    Ok(responses)
}

pub async fn add_note_to_notebook_core(
    nsec: &str,
    relays: &[String],
    notebook_id: String,
    note_id: String,
    client_nsec: Option<&str>,
) -> Result<String, String> {
    let client = crate::utils::get_client_with_signer(nsec, relays, client_nsec).await?;
    let my_public_key = client
        .signer()
        .await
        .unwrap()
        .get_public_key()
        .await
        .unwrap();

    let filter = Filter::new()
        .authors(vec![my_public_key])
        .kind(Kind::from(31301))
        .identifier(notebook_id.clone())
        .limit(1);

    let events = client
        .fetch_events(filter, Duration::from_secs(5))
        .await
        .map_err(|e| e.to_string())?;

    let event = events.first().ok_or("Notebook not found")?;

    let signer = client.signer().await.map_err(|e| e.to_string())?;

    let mut current_tags: Vec<Vec<String>> = Vec::new();

    if !event.content.is_empty()
        && let Ok(decrypted) = signer.nip44_decrypt(&event.pubkey, &event.content).await
        && let Ok(tags) = serde_json::from_str::<Vec<Vec<String>>>(&decrypted)
    {
        current_tags = tags;
    }

    let coordinate = format!("{}:{}:{}", 31323, my_public_key.to_hex(), note_id);

    if current_tags
        .iter()
        .any(|t| t.len() >= 2 && t[1] == coordinate)
    {
        return Ok("already_exists".to_string());
    }

    current_tags.push(vec!["a".to_string(), coordinate]);

    let json = serde_json::to_string(&current_tags).map_err(|e| e.to_string())?;
    let encrypted_content = signer
        .nip44_encrypt(&event.pubkey, &json)
        .await
        .map_err(|e| e.to_string())?;

    let builder = EventBuilder::new(Kind::from(31301), encrypted_content).tags(event.tags.clone());
    let id = client
        .send_event_builder(builder)
        .await
        .map_err(|e| e.to_string())?;

    Ok(id.to_hex())
}

pub async fn remove_note_from_notebook_core(
    nsec: &str,
    relays: &[String],
    notebook_id: String,
    note_id: String,
    client_nsec: Option<&str>,
) -> Result<String, String> {
    let client = crate::utils::get_client_with_signer(nsec, relays, client_nsec).await?;
    let my_public_key = client
        .signer()
        .await
        .unwrap()
        .get_public_key()
        .await
        .unwrap();

    let filter = Filter::new()
        .authors(vec![my_public_key])
        .kind(Kind::from(31301))
        .identifier(notebook_id)
        .limit(1);

    let events = client
        .fetch_events(filter, Duration::from_secs(5))
        .await
        .map_err(|e| e.to_string())?;

    let event = events.first().ok_or("Notebook not found")?;
    let signer = client.signer().await.map_err(|e| e.to_string())?;

    let mut current_tags: Vec<Vec<String>> = Vec::new();

    if !event.content.is_empty()
        && let Ok(decrypted) = signer.nip44_decrypt(&event.pubkey, &event.content).await
        && let Ok(tags) = serde_json::from_str::<Vec<Vec<String>>>(&decrypted)
    {
        current_tags = tags;
    }

    let coordinate = format!("{}:{}:{}", 31323, my_public_key.to_hex(), note_id);
    let initial_len = current_tags.len();
    current_tags.retain(|t| !(t.len() >= 2 && t[1] == coordinate));

    if current_tags.len() == initial_len {
        return Ok("not_found".to_string());
    }

    let json = serde_json::to_string(&current_tags).map_err(|e| e.to_string())?;
    let encrypted_content = signer
        .nip44_encrypt(&event.pubkey, &json)
        .await
        .map_err(|e| e.to_string())?;

    let builder = EventBuilder::new(Kind::from(31301), encrypted_content).tags(event.tags.clone());
    let id = client
        .send_event_builder(builder)
        .await
        .map_err(|e| e.to_string())?;

    Ok(id.to_hex())
}

pub async fn delete_notebook_core(
    nsec: &str,
    relays: &[String],
    notebook_id: String,
    client_nsec: Option<&str>,
) -> Result<String, String> {
    let client = crate::utils::get_client_with_signer(nsec, relays, client_nsec).await?;
    let public_key = client
        .signer()
        .await
        .unwrap()
        .get_public_key()
        .await
        .unwrap();
    let coordinate = format!("{}:{}:{}", 31301, public_key.to_hex(), notebook_id);

    let tags = vec![Tag::parse(["a", &coordinate]).map_err(|e| e.to_string())?];
    let builder = EventBuilder::new(Kind::EventDeletion, "Deleting notebook").tags(tags);

    let id = client
        .send_event_builder(builder)
        .await
        .map_err(|e| e.to_string())?;

    Ok(id.to_hex())
}
