use nostr_connect::client::NostrConnect;
use nostr_sdk::prelude::*;
use std::time::Duration;

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

pub fn get_public_key_from_signer(signer_str: &str) -> Result<PublicKey, String> {
    if signer_str.starts_with("bunker://") {
        let uri = NostrConnectURI::parse(signer_str).map_err(|e| e.to_string())?;
        match uri {
            NostrConnectURI::Bunker {
                remote_signer_public_key,
                ..
            } => Ok(remote_signer_public_key),
            _ => Err("Invalid bunker URI".to_string()),
        }
    } else {
        let keys = Keys::parse(signer_str).map_err(|e| e.to_string())?;
        Ok(keys.public_key())
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
    let client = if signer_str.starts_with("bunker://") {
        let uri = NostrConnectURI::parse(signer_str).map_err(|e| e.to_string())?;
        let app_keys = if let Some(nsec) = client_nsec {
            Keys::parse(nsec).map_err(|e| e.to_string())?
        } else {
            Keys::generate()
        };
        let signer = NostrConnect::new(uri, app_keys, Duration::from_secs(60), None)
            .map_err(|e| e.to_string())?;
        Client::new(signer)
    } else {
        let keys = Keys::parse(signer_str).map_err(|e| e.to_string())?;
        Client::new(keys)
    };

    for relay in relays {
        client.add_relay(relay).await.map_err(|e| e.to_string())?;
    }
    client.connect().await;
    Ok(client)
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
