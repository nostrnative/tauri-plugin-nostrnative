use crate::utils::parse_public_key;
use nostr_sdk::prelude::*;
use std::borrow::Cow;

pub async fn send_direct_message_core(
    nsec: &str,
    receiver_pubkey: &str,
    message: &str,
    relays: &[String],
) -> Result<String, String> {
    let client = crate::utils::get_client_with_signer(nsec, relays).await?;
    let r_pubkey = parse_public_key(receiver_pubkey)?;

    let signer = client.signer().await.map_err(|e| e.to_string())?;
    let encrypted_content = signer
        .nip04_encrypt(&r_pubkey, message)
        .await
        .map_err(|e| e.to_string())?;

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
