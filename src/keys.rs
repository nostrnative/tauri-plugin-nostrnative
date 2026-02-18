use nostr_connect::prelude::*;
use nostr_sdk::prelude::{Keys, NostrConnectURI, PublicKey};

pub fn generate_new_nsec() -> String {
    let keys = Keys::generate();
    keys.secret_key().to_bech32().unwrap()
}

pub fn parse_pubkey(pubkey: &str) -> Result<String, String> {
    let pk = crate::utils::parse_public_key(pubkey)?;
    Ok(pk.to_hex())
}

pub fn hex_to_npub(hex: &str) -> Result<String, String> {
    let pk = PublicKey::from_hex(hex).map_err(|e| e.to_string())?;
    pk.to_bech32().map_err(|e| e.to_string())
}

pub fn npub_to_hex(npub: &str) -> Result<String, String> {
    let pk = PublicKey::from_bech32(npub).map_err(|e| e.to_string())?;
    Ok(pk.to_hex())
}

pub async fn verify_nsec(nsec: &str, client_nsec: Option<&str>) -> Result<String, String> {
    if nsec.starts_with("bunker://") {
        let uri = NostrConnectURI::parse(nsec).map_err(|e| e.to_string())?;
        let app_keys = if let Some(c_nsec) = client_nsec {
            Keys::parse(c_nsec).map_err(|e| e.to_string())?
        } else {
            Keys::generate()
        };
        let signer = NostrConnect::new(uri, app_keys, std::time::Duration::from_secs(120), None)
            .map_err(|e| e.to_string())?;
        let user_pubkey = signer.get_public_key().await.map_err(|e| e.to_string())?;
        Ok(user_pubkey.to_hex())
    } else {
        let keys = Keys::parse(nsec).map_err(|e| e.to_string())?;
        Ok(keys.public_key().to_hex())
    }
}
