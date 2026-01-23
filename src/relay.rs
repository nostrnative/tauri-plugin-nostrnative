use nostr_relay_builder::{RelayBuilder, LocalRelay};
use nostr_lmdb::NostrLMDB;
use nostr_sdk::prelude::*;
use std::sync::Arc;
use tokio::sync::Mutex;
use once_cell::sync::Lazy;
use std::collections::HashMap;

static RELAY: Lazy<Arc<Mutex<Option<LocalRelay>>>> = Lazy::new(|| Arc::new(Mutex::new(None)));
static DATABASE: Lazy<Arc<Mutex<Option<Arc<NostrLMDB>>>>> = Lazy::new(|| Arc::new(Mutex::new(None)));

pub async fn start_relay_core(port: u16, db_path: &str) -> Result<(), String> {
    let mut relay_lock = RELAY.lock().await;
    if relay_lock.is_some() {
        return Err("Relay is already running".to_string());
    }

    let database = Arc::new(NostrLMDB::open(db_path).map_err(|e| e.to_string())?);
    let db_clone = database.clone();
    let builder = RelayBuilder::default()
        .port(port)
        .database(database);

    let relay = LocalRelay::new(builder);
    let relay_clone = relay.clone();
    
    tokio::spawn(async move {
        if let Err(e) = relay_clone.run().await {
            eprintln!("Relay error: {}", e);
        }
    });

    *relay_lock = Some(relay);
    let mut db_lock = DATABASE.lock().await;
    *db_lock = Some(db_clone);
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
    Ok(relay_lock.is_some())
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
