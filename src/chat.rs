use crate::models::{ChatMessageRequest, EventResponse, PnsDerivedKeys};
use crate::utils::{get_client, get_event_bech32};
use ::hkdf::Hkdf;
use nostr_sdk::prelude::*;
use sha2::Sha256;
use std::time::Duration;

const KIND_PNS_U16: u16 = 1080;
const KIND_DISCOVERY_U16: u16 = 1081;
const KIND_INNER_U16: u16 = 20001;
const SALT: &[u8] = b"app-sync-v1";

fn derive_pns_key_raw(device_key: &[u8]) -> [u8; 32] {
    let hk = Hkdf::<Sha256>::new(Some(SALT), device_key);
    let mut okm = [0u8; 32];
    hk.expand(&[], &mut okm)
        .expect("32 bytes is a valid length");
    okm
}

pub fn derive_pns_keys_core(device_key_hex: &str) -> Result<PnsDerivedKeys, String> {
    let device_key_bytes: [u8; 32] = if device_key_hex.starts_with("nsec") {
        let keys = Keys::parse(device_key_hex).map_err(|e| e.to_string())?;
        keys.secret_key().to_secret_bytes()
    } else {
        hex::decode(device_key_hex)
            .map_err(|e| e.to_string())?
            .try_into()
            .map_err(|_| "Invalid key length".to_string())?
    };

    let pns_key_bytes = derive_pns_key_raw(&device_key_bytes);
    let keys = Keys::parse(&hex::encode(pns_key_bytes)).map_err(|e| e.to_string())?;

    Ok(PnsDerivedKeys {
        public_key: keys.public_key().to_hex(),
        secret_key: keys.secret_key().to_bech32().map_err(|e| e.to_string())?,
    })
}

pub async fn create_discovery_event_core(
    nsec: &str,
    device_key_to_store: String,
    relays: &[String],
) -> Result<String, String> {
    let client = crate::utils::get_client_with_signer(nsec, relays).await?;
    let my_public_key = client.public_key().await.map_err(|e| e.to_string())?;

    let signer = client.signer().await.map_err(|e| e.to_string())?;
    let encrypted_content = signer
        .nip04_encrypt(&my_public_key, &device_key_to_store)
        .await
        .map_err(|e| e.to_string())?;

    let event_builder =
        EventBuilder::new(Kind::from(KIND_DISCOVERY_U16), encrypted_content).tags([]);
    let event_id = client
        .send_event_builder(event_builder)
        .await
        .map_err(|e| e.to_string())?;

    Ok(event_id.to_hex())
}

pub async fn decrypt_discovery_event_core(nsec: &str, content: &str) -> Result<String, String> {
    let client = crate::utils::get_client_with_signer(nsec, &[]).await?;
    let my_public_key = client.public_key().await.map_err(|e| e.to_string())?;

    let signer = client.signer().await.map_err(|e| e.to_string())?;
    if content.contains("?iv=") {
        if let Ok(decrypted) = signer.nip04_decrypt(&my_public_key, content).await {
            return Ok(decrypted);
        }
    }

    match signer.nip44_decrypt(&my_public_key, content).await {
        Ok(decrypted) => Ok(decrypted),
        Err(e) => Err(format!("Decryption failed (NIP-04/NIP-44: {})", e)),
    }
}

pub async fn publish_chat_message_core(
    pns_nsec: &str,
    main_pubkey: &str,
    msg: ChatMessageRequest,
    relays: &[String],
) -> Result<String, String> {
    let client = crate::utils::get_client_with_signer(pns_nsec, relays).await?;
    let my_public_key = client.public_key().await.map_err(|e| e.to_string())?;

    let mut inner_tags = vec![
        vec!["d".to_string(), msg.conversation_id],
        vec!["role".to_string(), msg.role],
        vec!["client".to_string(), msg.client],
    ];

    if let Some(reply_to) = msg.reply_to {
        inner_tags.push(vec!["e".to_string(), reply_to]);
    }

    if let Some(event_id) = &msg.event_id {
        inner_tags.push(vec!["e_ref".to_string(), event_id.clone()]);
    }

    if let Some(context_ids) = msg.context_ids {
        for cid in context_ids {
            inner_tags.push(vec!["context".to_string(), cid]);
        }
    }

    if let Some(blob_hashes) = msg.blob_hashes {
        for hash in blob_hashes {
            inner_tags.push(vec!["blob".to_string(), hash]);
        }
    }

    let inner_event_json = serde_json::json!({
        "kind": KIND_INNER_U16,
        "content": msg.content,
        "pubkey": main_pubkey,
        "created_at": Timestamp::now().as_secs(),
        "tags": inner_tags,
    })
    .to_string();

    let signer = client.signer().await.map_err(|e| e.to_string())?;
    let encrypted_inner = signer
        .nip44_encrypt(&my_public_key, &inner_event_json)
        .await
        .map_err(|e| e.to_string())?;

    let event_builder = EventBuilder::new(Kind::from(KIND_PNS_U16), encrypted_inner).tags([]);

    let event_id = client
        .send_event_builder(event_builder)
        .await
        .map_err(|e| e.to_string())?;

    Ok(event_id.to_hex())
}

pub async fn decrypt_pns_event_core(
    pns_nsec: &str,
    content: &str,
) -> Result<serde_json::Value, String> {
    let client = crate::utils::get_client_with_signer(pns_nsec, &[]).await?;
    let my_public_key = client.public_key().await.map_err(|e| e.to_string())?;

    let signer = client.signer().await.map_err(|e| e.to_string())?;
    let decrypted = signer
        .nip44_decrypt(&my_public_key, content)
        .await
        .map_err(|e| e.to_string())?;

    let val: serde_json::Value = serde_json::from_str(&decrypted).map_err(|e| e.to_string())?;
    Ok(val)
}

pub async fn fetch_pns_events_core(
    pns_pubkey: &str,
    event_id: Option<String>,
    relays: &[String],
) -> Result<Vec<EventResponse>, String> {
    let pk = PublicKey::from_hex(pns_pubkey).map_err(|e| e.to_string())?;
    let ephemeral_keys = Keys::generate();
    let client = get_client(&ephemeral_keys, relays).await?;

    let mut filter = Filter::new().author(pk).kind(Kind::from(KIND_PNS_U16));

    if let Some(event_id) = event_id {
        filter = filter.event(EventId::from_hex(&event_id).map_err(|e| e.to_string())?);
    }

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

pub async fn delete_chat_messages_core(
    pns_nsec: &str,
    relays: &[String],
    event_ids: Vec<String>,
) -> Result<String, String> {
    let client = crate::utils::get_client_with_signer(pns_nsec, relays).await?;

    let ids: Result<Vec<EventId>, String> = event_ids
        .into_iter()
        .map(|id| EventId::from_hex(&id).map_err(|e| e.to_string()))
        .collect();

    let event_builder = EventBuilder::delete(EventDeletionRequest::new().ids(ids?));

    let event_id = client
        .send_event_builder(event_builder)
        .await
        .map_err(|e| e.to_string())?;

    Ok(event_id.to_hex())
}

pub async fn fetch_discovery_events_core(
    pubkey: &str,
    relays: &[String],
) -> Result<Vec<EventResponse>, String> {
    let pk = PublicKey::from_hex(pubkey).map_err(|e| e.to_string())?;
    let ephemeral_keys = Keys::generate();
    let client = get_client(&ephemeral_keys, relays).await?;

    let filter = Filter::new()
        .author(pk)
        .kind(Kind::from(KIND_DISCOVERY_U16));

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
