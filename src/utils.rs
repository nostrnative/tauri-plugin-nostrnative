use nostr_sdk::prelude::*;

pub fn parse_public_key(pk: &str) -> Result<PublicKey, String> {
    let mut clean_pk = pk.trim();
    if let Some(rest) = clean_pk.strip_prefix("nostr:") {
        clean_pk = rest;
    }
    if let Some(rest) = clean_pk.strip_prefix('@') {
        clean_pk = rest;
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

/// Build a client that can sign as the user.
///
/// `signer_str` is either a private key (`nsec1…` / hex) or a `bunker://` URI.
/// For bunkers the NIP-46 session is cached (see [`crate::bunker`]), so the
/// remote signer handshake happens once per app run, not once per command.
pub async fn get_client_with_signer(
    signer_str: &str,
    relays: &[String],
    client_nsec: Option<&str>,
) -> Result<Client, String> {
    if crate::bunker::is_remote_signer(signer_str) {
        let session = crate::bunker::connect(signer_str, client_nsec, relays).await?;
        return Ok(session.client);
    }
    let keys = Keys::parse(signer_str.trim()).map_err(|e| format!("Invalid private key: {e}"))?;
    get_client(&keys, relays).await
}

/// Prefix marking errors that came from the signer (local key or NIP-46),
/// as opposed to relays. The UI only treats an error as a user rejection when
/// it carries this prefix, so relay messages such as "blocked: not allowed"
/// can never cause a queued event to be dropped.
pub const SIGNER_ERROR_PREFIX: &str = "Signer refused: ";

/// Human-readable error for a failed `send_event_builder` / `sign_event_builder`.
pub fn describe_send_error(e: &nostr_sdk::client::Error) -> String {
    if is_signer_error(e) {
        format!("{SIGNER_ERROR_PREFIX}{e}")
    } else {
        e.to_string()
    }
}

/// Signer failures surface either directly or wrapped by the event builder
/// (`EventBuilder::sign` -> `UnsignedEvent::sign` -> `event::Error::Signer`).
fn is_signer_error(e: &nostr_sdk::client::Error) -> bool {
    use nostr::event::Error as EventError;
    use nostr::event::builder::Error as BuilderError;
    use nostr_sdk::client::Error as ClientError;
    matches!(
        e,
        ClientError::Signer(_)
            | ClientError::EventBuilder(BuilderError::Event(EventError::Signer(_)))
    )
}

/// Error for a failed signer encryption / decryption call.
pub fn describe_signer_call_error(what: &str, e: impl std::fmt::Display) -> String {
    format!("{SIGNER_ERROR_PREFIX}failed to {what}: {e}")
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

#[cfg(test)]
mod tests {
    use super::*;

    fn keys() -> Keys {
        Keys::generate()
    }

    #[test]
    fn parses_hex_npub_and_prefixed_forms() {
        let k = keys();
        let hex = k.public_key().to_hex();
        let npub = k.public_key().to_bech32().unwrap();
        assert_eq!(parse_public_key(&hex).unwrap(), k.public_key());
        assert_eq!(parse_public_key(&npub).unwrap(), k.public_key());
        assert_eq!(
            parse_public_key(&format!("nostr:{npub}")).unwrap(),
            k.public_key()
        );
        assert_eq!(
            parse_public_key(&format!("@{npub}")).unwrap(),
            k.public_key()
        );
        assert_eq!(
            parse_public_key(&format!("  {hex}  ")).unwrap(),
            k.public_key()
        );
    }

    #[test]
    fn parses_nprofile() {
        let k = keys();
        let profile = Nip19Profile::new(
            k.public_key(),
            vec![RelayUrl::parse("wss://r.example").unwrap()],
        );
        let nprofile = profile.to_bech32().unwrap();
        assert_eq!(parse_public_key(&nprofile).unwrap(), k.public_key());
    }

    #[test]
    fn rejects_invalid_public_keys() {
        assert!(parse_public_key("").is_err());
        assert!(parse_public_key("npub1invalid").is_err());
        assert!(parse_public_key("deadbeef").is_err());
        let nsec = keys().secret_key().to_bech32().unwrap();
        assert!(
            parse_public_key(&nsec).is_err(),
            "nsec must not parse as a pubkey"
        );
    }

    #[test]
    fn bech32_uses_naddr_for_addressable_events() {
        let k = keys();
        let event = EventBuilder::new(Kind::from(31923), "")
            .tag(Tag::identifier("my-event"))
            .sign_with_keys(&k)
            .unwrap();
        let encoded = get_event_bech32(&event, None);
        assert!(encoded.starts_with("naddr1"), "{encoded}");
        let decoded = Nip19::from_bech32(&encoded).unwrap();
        match decoded {
            Nip19::Coordinate(c) => {
                assert_eq!(c.coordinate.identifier, "my-event");
                assert_eq!(c.coordinate.public_key, k.public_key());
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn bech32_uses_nevent_for_regular_events() {
        let k = keys();
        let event = EventBuilder::text_note("hi").sign_with_keys(&k).unwrap();
        let encoded = get_event_bech32(
            &event,
            Some(vec!["wss://r.example".into(), "bad url".into()]),
        );
        assert!(encoded.starts_with("nevent1"), "{encoded}");
        match Nip19::from_bech32(&encoded).unwrap() {
            Nip19::Event(e) => {
                assert_eq!(e.event_id, event.id);
                assert_eq!(e.author, Some(k.public_key()));
                assert_eq!(e.relays.len(), 1);
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn signer_errors_are_tagged_relay_errors_are_not() {
        let signer =
            nostr_sdk::client::Error::Signer(SignerError::from("user rejected the request"));
        assert_eq!(
            describe_send_error(&signer),
            "Signer refused: user rejected the request"
        );
        // What NostrConnect actually returns from `send_event_builder`.
        let wrapped = nostr_sdk::client::Error::EventBuilder(nostr::event::builder::Error::Event(
            nostr::event::Error::Signer("user rejected the request".into()),
        ));
        assert_eq!(
            describe_send_error(&wrapped),
            "Signer refused: user rejected the request"
        );
        let relay = nostr_sdk::client::Error::PrivateMsgRelaysNotFound;
        assert!(!describe_send_error(&relay).starts_with(SIGNER_ERROR_PREFIX));
        assert!(
            describe_signer_call_error("encrypt title", "timeout").starts_with(SIGNER_ERROR_PREFIX)
        );
    }

    #[tokio::test]
    async fn signer_rejects_bad_private_key() {
        let err = get_client_with_signer("nsec1garbage", &[], None)
            .await
            .unwrap_err();
        assert!(err.contains("Invalid private key"), "{err}");
    }

    #[tokio::test]
    async fn signer_accepts_nsec_without_relays() {
        let nsec = keys().secret_key().to_bech32().unwrap();
        let client = get_client_with_signer(&nsec, &[], None).await.unwrap();
        assert!(client.signer().await.is_ok());
    }
}
