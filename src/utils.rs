use nostr_sdk::prelude::*;

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

pub fn get_event_bech32(event: &Event, relays: Option<Vec<String>>) -> String {
    let kind_u16 = event.kind.as_u16();
    let is_replaceable = (10000..20000).contains(&kind_u16);
    let is_parameterized_replaceable = (30000..40000).contains(&kind_u16);

    let relay_urls: Vec<String> = relays.unwrap_or_default();

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

    if let Ok(nevent) = Nip19Event::new(event.id, relay_urls)
        .author(event.pubkey)
        .to_bech32()
    {
        return nevent;
    }

    event.id.to_bech32().unwrap_or_else(|_| event.id.to_hex())
}
