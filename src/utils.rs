use nostr_sdk::prelude::*;

pub fn parse_public_key(pk: &str) -> Result<PublicKey, String> {
    match PublicKey::from_hex(pk) {
        Ok(p) => Ok(p),
        Err(_) => PublicKey::from_bech32(pk).map_err(|e| e.to_string()),
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
