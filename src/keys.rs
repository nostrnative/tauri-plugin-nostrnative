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

pub fn verify_nsec(nsec: &str) -> Result<String, String> {
    let keys = Keys::parse(nsec).map_err(|e| e.to_string())?;
    Ok(keys.public_key().to_hex())
}
