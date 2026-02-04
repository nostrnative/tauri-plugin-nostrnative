use crate::utils::parse_public_key;
use nostr_sdk::prelude::*;

pub fn generate_new_nsec() -> String {
    let keys = Keys::generate();
    keys.secret_key().to_bech32().unwrap()
}

pub fn parse_pubkey(pubkey: &str) -> Result<String, String> {
    let pk = parse_public_key(pubkey)?;
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

pub async fn verify_nsec(nsec: &str) -> Result<String, String> {
    if nsec.starts_with("bunker://") {
        let uri = NostrConnectURI::parse(nsec).map_err(|e| e.to_string())?;
        match uri {
            NostrConnectURI::Bunker {
                remote_signer_public_key,
                ..
            } => Ok(remote_signer_public_key.to_hex()),
            _ => Err("Invalid bunker URI".to_string()),
        }
    } else {
        let keys = Keys::parse(nsec).map_err(|e| e.to_string())?;
        Ok(keys.public_key().to_hex())
    }
}
