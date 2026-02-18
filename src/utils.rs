use nostr_connect::prelude::*;
use nostr_sdk::prelude::*;
use once_cell::sync::Lazy;
use std::time::Duration;
use tokio::sync::Mutex;

struct CachedBunkerClient {
    client: Client,
    signer_uri: String,
    client_nsec: Option<String>,
}

static BUNKER_CLIENT_CACHE: Lazy<Mutex<Option<CachedBunkerClient>>> =
    Lazy::new(|| Mutex::new(None));

pub fn parse_public_key(pk: &str) -> Result<PublicKey, String> {
    let mut clean_pk = pk.trim();
    if clean_pk.starts_with("nostr:") {
        clean_pk = &clean_pk[6..];
    }
    if clean_pk.starts_with('@') {
        clean_pk = &clean_pk[1..];
    }

    match PublicKey::from_hex(clean_pk) {
        Ok(p) => Ok(p),
        Err(_) => match PublicKey::from_bech32(clean_pk) {
            Ok(p) => Ok(p),
            Err(_) => match Nip19::from_bech32(clean_pk) {
                Ok(Nip19::Pubkey(pk)) => Ok(pk),
                Ok(Nip19::Profile(p)) => Ok(p.public_key),
                _ => Err("Invalid public key format".to_string()),
            },
        },
    }
}

pub async fn get_client(keys: &Keys, relays: &[String]) -> Result<Client, String> {
    let client = Client::new(keys.clone());
    for relay in relays {
        client.add_relay(relay).await.map_err(|e| e.to_string())?;
    }
    client.connect().await;
    Ok(client)
}

pub async fn get_client_with_signer(
    signer_str: &str,
    relays: &[String],
    client_nsec: Option<&str>,
) -> Result<Client, String> {
    if signer_str.starts_with("bunker://") {
        eprintln!("[bunker] get_client_with_signer called, relays={relays:?}");
        let mut cache = BUNKER_CLIENT_CACHE.lock().await;

        // Reuse cached client if signer_uri and client_nsec match
        if let Some(ref cached) = *cache
            && cached.signer_uri == signer_str
            && cached.client_nsec.as_deref() == client_nsec
        {
            eprintln!("[bunker] reusing cached client");
            // Add any new relays and reconnect
            for relay in relays {
                let _ = cached.client.add_relay(relay).await;
            }
            cached.client.connect().await;
            return Ok(cached.client.clone());
        }

        // No cache match — create new client with NIP-46 signer
        eprintln!("[bunker] creating new NIP-46 signer...");
        let uri = NostrConnectURI::parse(signer_str).map_err(|e| {
            eprintln!("[bunker] URI parse error: {e}");
            e.to_string()
        })?;
        let app_keys = if let Some(nsec) = client_nsec {
            Keys::parse(nsec).map_err(|e| {
                eprintln!("[bunker] client_nsec parse error: {e}");
                e.to_string()
            })?
        } else {
            Keys::generate()
        };
        let signer =
            NostrConnect::new(uri, app_keys, Duration::from_secs(12000), None).map_err(|e| {
                eprintln!("[bunker] NostrConnect::new error: {e}");
                e.to_string()
            })?;

        let client = Client::builder().signer(signer.clone()).build();
        for relay in relays {
            client.add_relay(relay).await.map_err(|e| e.to_string())?;
        }
        client.connect().await;
        eprintln!("[bunker] relays connected, performing NIP-46 handshake...");

        // Perform NIP-46 handshake
        match signer.get_public_key().await {
            Ok(pk) => eprintln!("[bunker] handshake OK, pubkey={}", pk.to_hex()),
            Err(ref e) => {
                eprintln!("[bunker] handshake FAILED: {e}");
                return Err(e.to_string());
            }
        }

        // Cache the connected client
        *cache = Some(CachedBunkerClient {
            client: client.clone(),
            signer_uri: signer_str.to_string(),
            client_nsec: client_nsec.map(|s| s.to_string()),
        });

        Ok(client)
    } else {
        let keys = Keys::parse(signer_str).map_err(|e| e.to_string())?;
        let client = Client::new(keys);
        for relay in relays {
            client.add_relay(relay).await.map_err(|e| e.to_string())?;
        }
        client.connect().await;
        Ok(client)
    }
}

pub fn get_event_bech32(event: &Event, relays: Option<Vec<String>>) -> String {
    let kind_u16 = event.kind.as_u16();
    let is_replaceable = (10000..20000).contains(&kind_u16);
    let is_parameterized_replaceable = (30000..40000).contains(&kind_u16);

    let relay_urls: Vec<RelayUrl> = relays
        .unwrap_or_default()
        .into_iter()
        .filter_map(|url| RelayUrl::parse(&url).ok())
        .collect();

    if is_replaceable || is_parameterized_replaceable {
        let identifier = event
            .tags
            .iter()
            .find(|t| t.as_slice().first().map(|v| v.as_str()) == Some("d"))
            .and_then(|t| t.as_slice().get(1).map(|v| v.to_string()))
            .unwrap_or_default();

        let coordinate = Coordinate::new(event.kind, event.pubkey).identifier(identifier);
        if let Ok(naddr) = coordinate.to_bech32() {
            return naddr;
        }
    }

    if let Ok(nevent) = Nip19Event::new(event.id)
        .relays(relay_urls)
        .author(event.pubkey)
        .to_bech32()
    {
        return nevent;
    }

    event.id.to_bech32().unwrap_or_else(|_| event.id.to_hex())
}
