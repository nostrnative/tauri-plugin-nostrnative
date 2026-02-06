use nostr_lmdb::NostrLMDB;
use nostr_relay_builder::prelude::*;
use once_cell::sync::Lazy;
use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};

static RELAY: Lazy<Arc<Mutex<Option<LocalRelay>>>> = Lazy::new(|| Arc::new(Mutex::new(None)));
static ERROR_SENDER: Lazy<Arc<Mutex<Option<mpsc::Sender<String>>>>> =
    Lazy::new(|| Arc::new(Mutex::new(None)));
static DATABASE: Lazy<Arc<Mutex<Option<Arc<NostrLMDB>>>>> =
    Lazy::new(|| Arc::new(Mutex::new(None)));

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct RelaySettings {
    pub relay_allowed_kinds: Option<Vec<u16>>,
    pub relay_allowed_pubkeys: Option<Vec<String>>,
    pub relay_allowed_tagged_pubkeys: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
struct RelayPolicies {
    kinds: Option<HashSet<Kind>>,
    pubkeys: Option<HashSet<PublicKey>>,
    tagged_pubkeys: Option<HashSet<PublicKey>>,
}

impl WritePolicy for RelayPolicies {
    fn admit_event<'a>(
        &'a self,
        event: &'a Event,
        _addr: &'a SocketAddr,
    ) -> BoxedFuture<'a, PolicyResult> {
        Box::pin(async move {
            // Check kinds
            if self
                .kinds
                .as_ref()
                .is_some_and(|k| !k.is_empty() && !k.contains(&event.kind))
            {
                return PolicyResult::Reject(format!(
                    "{}: kind not accepted",
                    MachineReadablePrefix::Blocked
                ));
            }

            // Check authors (pubkeys)
            if self
                .pubkeys
                .as_ref()
                .is_some_and(|p| !p.is_empty() && !p.contains(&event.pubkey))
            {
                return PolicyResult::Reject(format!(
                    "{}: pubkey not allowed",
                    MachineReadablePrefix::Blocked
                ));
            }

            // Check tagged pubkeys ("p" tags)
            if let Some(tagged_pubkeys) = &self.tagged_pubkeys
                && !tagged_pubkeys.is_empty()
            {
                let has_allowed_tag = event.tags.iter().any(|tag| {
                    if tag.kind() == TagKind::p()
                        && let Some(pk_hex) = tag.content()
                        && let Ok(pk) = PublicKey::from_hex(pk_hex)
                    {
                        return tagged_pubkeys.contains(&pk);
                    }
                    false
                });

                if !has_allowed_tag {
                    return PolicyResult::Reject(format!(
                        "{}: no allowed tagged pubkey found",
                        MachineReadablePrefix::Blocked
                    ));
                }
            }

            PolicyResult::Accept
        })
    }
}

pub async fn start_relay_core(
    port: u16,
    db_path: &str,
    public_key: Option<&str>,
    settings: Option<RelaySettings>,
) -> Result<(), String> {
    let database = Arc::new(NostrLMDB::open(db_path).map_err(|e| e.to_string())?);
    let db_clone = database.clone();

    let mut builder = RelayBuilder::default().port(port).database(database);

    if let Some(pubkey) = public_key {
        builder = builder.mode(RelayBuilderMode::PublicKey(
            PublicKey::from_nostr_uri(pubkey).unwrap(),
        ));
    }

    if let Some(s) = settings {
        let policies = RelayPolicies {
            kinds: s
                .relay_allowed_kinds
                .map(|k| k.into_iter().map(Kind::from).collect()),
            pubkeys: s.relay_allowed_pubkeys.map(|p| {
                p.into_iter()
                    .filter_map(|pk| {
                        if pk.starts_with("npub") {
                            PublicKey::from_bech32(&pk).ok()
                        } else {
                            PublicKey::from_hex(&pk).ok()
                        }
                    })
                    .collect()
            }),
            tagged_pubkeys: s.relay_allowed_tagged_pubkeys.map(|p| {
                p.into_iter()
                    .filter_map(|pk| {
                        if pk.starts_with("npub") {
                            PublicKey::from_bech32(&pk).ok()
                        } else {
                            PublicKey::from_hex(&pk).ok()
                        }
                    })
                    .collect()
            }),
        };
        builder = builder.write_policy(policies);
    }

    let relay = LocalRelay::new(builder);
    let relay_clone = relay.clone();

    let (tx, mut rx) = mpsc::channel(1);
    let tx_clone = tx.clone();

    tokio::spawn(async move {
        if let Err(e) = relay_clone.run().await {
            let _ = tx_clone.send(e.to_string()).await;
        }
    });

    let mut relay_lock = RELAY.lock().await;
    if let Some(existing) = relay_lock.take() {
        existing.shutdown();
    }

    *relay_lock = Some(relay);

    let mut error_sender_lock = ERROR_SENDER.lock().await;
    *error_sender_lock = Some(tx);

    let mut db_lock = DATABASE.lock().await;
    *db_lock = Some(db_clone);

    // Optional: Check if the relay failed immediately
    tokio::select! {
        err = rx.recv() => {
            if let Some(e) = err {
                return Err(e);
            }
        }
        _ = tokio::time::sleep(std::time::Duration::from_millis(100)) => {}
    }

    Ok(())
}

pub async fn stop_relay_core() -> Result<(), String> {
    let mut relay_lock = RELAY.lock().await;
    let mut db_lock = DATABASE.lock().await;
    *db_lock = None;
    if let Some(relay) = relay_lock.take() {
        relay.shutdown();
        Ok(())
    } else {
        Err("Relay is not running".to_string())
    }
}

pub async fn get_relay_status_core() -> Result<bool, String> {
    let relay_lock = RELAY.lock().await;
    if relay_lock.is_none() {
        return Ok(false);
    }

    // Check if there's a pending error
    let error_sender_lock = ERROR_SENDER.lock().await;
    if let Some(_tx) = error_sender_lock.as_ref() {
        // If we can't send a ping, the receiver might have been dropped or relay failed
        // But here we use a channel to capture the error from the background task.
        // This is a bit complex without a persistent receiver.
    }

    Ok(true)
}

pub async fn get_event_counts_core() -> Result<HashMap<u64, usize>, String> {
    let db_lock = DATABASE.lock().await;
    if let Some(db) = &*db_lock {
        let filter = Filter::new();
        let events = db.query(filter).await.map_err(|e| e.to_string())?;

        let mut counts = HashMap::new();
        for event in events {
            let kind: u64 = event.kind.as_u16() as u64;
            *counts.entry(kind).or_insert(0) += 1;
        }
        Ok(counts)
    } else {
        Err("Relay database not initialized".to_string())
    }
}

pub async fn get_relay_events_core() -> Result<Vec<Event>, String> {
    let db_lock = DATABASE.lock().await;
    if let Some(db) = &*db_lock {
        let filter = Filter::new();
        let events = db.query(filter).await.map_err(|e| e.to_string())?;
        Ok(events.to_vec())
    } else {
        Err("Relay database not initialized".to_string())
    }
}
