use crate::utils::{get_client, parse_public_key};
use nostr_sdk::prelude::*;
use std::borrow::Cow;

pub async fn send_direct_message_core(
    nsec: &str,
    receiver_pubkey: &str,
    message: &str,
    relays: &[String],
) -> Result<String, String> {
    let keys = Keys::parse(nsec).map_err(|e| e.to_string())?;
    let r_pubkey = parse_public_key(receiver_pubkey)?;
    let client = get_client(&keys, relays).await?;

    let secret = keys.secret_key();
    let encrypted_content =
        nip04::encrypt(secret, &r_pubkey, message).map_err(|e| e.to_string())?;

    let kind = Kind::from(4);
    let tags = vec![Tag::custom(
        TagKind::Custom(Cow::Borrowed("p")),
        vec![r_pubkey.to_hex()],
    )];

    let event_builder = EventBuilder::new(kind, encrypted_content).tags(tags);

    let event_id = client
        .send_event_builder(event_builder)
        .await
        .map_err(|e| e.to_string())?;

    Ok(event_id.to_hex())
}
