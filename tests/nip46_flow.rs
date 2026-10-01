//! End-to-end NIP-46 tests against an in-process relay and remote signer.
//!
//! Covers both pairing directions (`bunker://` and `nostrconnect://`), proves
//! the `connect` handshake happens exactly once per session, and exercises a
//! real private calendar event publish → fetch → decrypt round trip through
//! the remote signer.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use nostr::nips::nip46::ResponseResult;
use nostr_connect::prelude::*;
use nostr_relay_builder::prelude::*;
use nostr_sdk::Client;
use tauri_plugin_nostrnative::bunker::{self, PairingStatus};
use tauri_plugin_nostrnative::calendar;
use tauri_plugin_nostrnative::keys;
use tauri_plugin_nostrnative::models::{CalendarEventRequest, DecryptEventRequest};

#[derive(Debug, Clone)]
struct AutoApprove {
    connects: Arc<AtomicUsize>,
    requests: Arc<AtomicUsize>,
}

impl AutoApprove {
    fn new() -> Self {
        Self {
            connects: Arc::new(AtomicUsize::new(0)),
            requests: Arc::new(AtomicUsize::new(0)),
        }
    }
}

impl NostrConnectSignerActions for AutoApprove {
    fn approve(&self, _public_key: &PublicKey, req: &NostrConnectRequest) -> bool {
        if matches!(req, NostrConnectRequest::Connect { .. }) {
            self.connects.fetch_add(1, Ordering::SeqCst);
        }
        self.requests.fetch_add(1, Ordering::SeqCst);
        true
    }
}

async fn start_relay() -> (LocalRelay, String) {
    let relay = LocalRelay::new(RelayBuilder::default());
    relay.run().await.expect("relay starts");
    let url = relay.url().await.to_string();
    (relay, url)
}

fn signer_keys() -> NostrConnectKeys {
    NostrConnectKeys {
        signer: Keys::generate(),
        user: Keys::generate(),
    }
}

fn private_event(identifier: &str) -> CalendarEventRequest {
    CalendarEventRequest {
        title: "Secret meeting".to_string(),
        description: Some("Bring the plans".to_string()),
        start: 1_900_000_000,
        end: Some(1_900_003_600),
        location: Some("HQ".to_string()),
        is_all_day: false,
        identifier: identifier.to_string(),
        reminder_minutes: None,
        old_event_id: None,
        calendar_id: None,
        color: None,
        p_tags: None,
        is_private: Some(true),
        parent: None,
        freq: None,
        until: None,
        use_different_timestamp: None,
        start_tzid: None,
        end_tzid: None,
        start_date: None,
        end_date: None,
    }
}

fn tag_value<'a>(tags: &'a [Vec<String>], name: &str) -> Option<&'a str> {
    tags.iter()
        .find(|t| t.first().map(String::as_str) == Some(name))
        .and_then(|t| t.get(1))
        .map(String::as_str)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn bunker_login_signs_and_encrypts_through_remote_signer() {
    let (_relay, url) = start_relay().await;
    let nc_keys = signer_keys();
    let user_pubkey = nc_keys.user.public_key();

    let remote = NostrConnectRemoteSigner::new(
        nc_keys.clone(),
        [url.as_str()],
        Some("one-time-secret".to_string()),
        None,
    )
    .expect("remote signer");
    let bunker_uri = remote.bunker_uri().to_string();
    assert!(bunker_uri.contains("secret=one-time-secret"));

    let actions = AutoApprove::new();
    let serve_actions = actions.clone();
    tokio::spawn(async move {
        let _ = remote.serve(serve_actions).await;
    });

    let client_nsec = keys::generate_new_nsec();

    // 1. Login (verify) performs the handshake.
    let pk = keys::verify_nsec(&bunker_uri, Some(&client_nsec))
        .await
        .expect("bunker login");
    assert_eq!(pk, user_pubkey.to_hex());

    // 2. A second login call reuses the cached session: no second `connect`.
    let conn = keys::connect_bunker(&bunker_uri, Some(&client_nsec))
        .await
        .expect("connect_bunker");
    assert_eq!(conn.user_pubkey, user_pubkey.to_hex());
    assert_eq!(conn.client_nsec, client_nsec);
    assert!(conn.bunker_uri.starts_with("bunker://"));
    assert!(
        !conn.bunker_uri.contains("secret="),
        "persisted URI must not carry the one-time secret: {}",
        conn.bunker_uri
    );
    assert_eq!(actions.connects.load(Ordering::SeqCst), 1);
    assert!(bunker::has_session(&bunker_uri, Some(&client_nsec)).await);
    assert!(bunker::has_session(&conn.bunker_uri, Some(&client_nsec)).await);

    // 3. Publish a private event using the canonical (secret-less) URI.
    let ids = calendar::publish_batch_calendar_events_core(
        &conn.bunker_uri,
        std::slice::from_ref(&url),
        vec![private_event("nip46-test")],
        Some(&client_nsec),
    )
    .await
    .expect("publish through bunker");
    assert_eq!(ids.len(), 1);

    // 4. Fetch it back: it is encrypted on the relay.
    let events = calendar::fetch_calendar_events_core(
        &user_pubkey.to_hex(),
        Some(&conn.bunker_uri),
        std::slice::from_ref(&url),
        None,
        None,
        None,
        Some(&client_nsec),
        None,
    )
    .await
    .expect("fetch");
    let ev = events
        .iter()
        .find(|e| tag_value(&e.tags, "d") == Some("nip46-test"))
        .expect("published event is on the relay");
    assert!(ev.is_private);
    assert_eq!(ev.pubkey, user_pubkey.to_hex());
    let encrypted_title = tag_value(&ev.tags, "title").expect("title tag").to_string();
    assert_ne!(encrypted_title, "Secret meeting");

    // 5. Decrypt via the remote signer (NIP-44 decrypt RPC).
    let decrypted = calendar::decrypt_private_events_core(
        &conn.bunker_uri,
        std::slice::from_ref(&url),
        vec![DecryptEventRequest {
            id: ev.id.clone(),
            pubkey: ev.pubkey.clone(),
            encrypted_title: Some(encrypted_title),
            encrypted_content: Some(ev.content.clone()),
            encrypted_location: tag_value(&ev.tags, "location").map(str::to_string),
        }],
        Some(&client_nsec),
    )
    .await
    .expect("decrypt");
    assert_eq!(decrypted.len(), 1);
    assert_eq!(decrypted[0].title.as_deref(), Some("Secret meeting"));
    assert_eq!(decrypted[0].description.as_deref(), Some("Bring the plans"));
    assert_eq!(decrypted[0].location.as_deref(), Some("HQ"));

    // Still exactly one handshake for the whole flow.
    assert_eq!(actions.connects.load(Ordering::SeqCst), 1);
    assert!(
        actions.requests.load(Ordering::SeqCst) > 3,
        "sign + encrypt + decrypt went remote"
    );

    // 6. Logout tears the session down.
    assert!(
        bunker::disconnect(&conn.bunker_uri, Some(&client_nsec))
            .await
            .unwrap()
    );
    assert!(!bunker::has_session(&conn.bunker_uri, Some(&client_nsec)).await);
    assert!(
        !bunker::disconnect(&conn.bunker_uri, Some(&client_nsec))
            .await
            .unwrap()
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn nostrconnect_pairing_connects_when_signer_scans() {
    let (_relay, url) = start_relay().await;
    let nc_keys = signer_keys();
    let user_pubkey = nc_keys.user.public_key();

    let pairing = bunker::start_pairing(std::slice::from_ref(&url), "Yawmiya Test")
        .await
        .expect("pairing");
    assert!(pairing.uri.starts_with("nostrconnect://"));
    assert_eq!(
        bunker::poll_pairing(&pairing.token).await,
        PairingStatus::Pending
    );

    // The signer app "scans" the QR code.
    let uri = NostrConnectURI::parse(&pairing.uri).expect("uri parses");
    let client_pubkey = match &uri {
        NostrConnectURI::Client { public_key, .. } => *public_key,
        _ => panic!("expected client uri"),
    };
    let remote = NostrConnectRemoteSigner::from_uri(uri, nc_keys.clone(), None, None)
        .expect("remote signer from client uri");
    let actions = AutoApprove::new();
    let serve_actions = actions.clone();
    tokio::spawn(async move {
        let _ = remote.serve(serve_actions).await;
    });

    // Per NIP-46, a signer that scans a nostrconnect:// URI answers with a
    // `connect` *response* (ack). nostr-connect's bundled remote signer sends
    // a request instead, so emulate what Amber / nsec.app do here. The ack is
    // re-sent until the client picks it up (its subscription may not be live
    // yet), which keeps the test free of fixed sleeps.
    let signer_app = Client::new(nc_keys.signer.clone());
    signer_app.add_relay(url.as_str()).await.unwrap();
    signer_app.connect().await;
    let send_ack = || async {
        let ack = EventBuilder::nostr_connect(
            &nc_keys.signer,
            client_pubkey,
            NostrConnectMessage::response(
                "pairing",
                NostrConnectResponse::with_result(ResponseResult::Ack),
            ),
        )
        .expect("ack builder")
        .sign_with_keys(&nc_keys.signer)
        .expect("ack signed");
        signer_app.send_event(&ack).await.expect("ack sent");
    };

    let deadline = tokio::time::Instant::now() + Duration::from_secs(20);
    let conn = loop {
        match bunker::poll_pairing(&pairing.token).await {
            PairingStatus::Connected(c) => break c,
            PairingStatus::Failed { error } => panic!("pairing failed: {error}"),
            _ => {
                assert!(tokio::time::Instant::now() < deadline, "pairing timed out");
                send_ack().await;
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
        }
    };
    assert_eq!(conn.user_pubkey, user_pubkey.to_hex());
    assert_eq!(conn.client_nsec, pairing.client_nsec);
    assert!(conn.bunker_uri.starts_with("bunker://"));

    // The resulting credentials work like a normal bunker login, reusing the session.
    let client = tauri_plugin_nostrnative::get_client_with_signer(
        &conn.bunker_uri,
        std::slice::from_ref(&url),
        Some(&conn.client_nsec),
    )
    .await
    .expect("client from paired session");
    assert_eq!(client.public_key().await.unwrap(), user_pubkey);
    let event = client
        .sign_event_builder(EventBuilder::text_note("signed remotely"))
        .await
        .expect("remote sign");
    assert_eq!(event.pubkey, user_pubkey);
    assert!(event.verify().is_ok());
    assert_eq!(
        actions.connects.load(Ordering::SeqCst),
        0,
        "client-initiated flow sends no connect"
    );

    bunker::cancel_pairing(&pairing.token).await;
    assert!(
        bunker::disconnect(&conn.bunker_uri, Some(&conn.client_nsec))
            .await
            .unwrap()
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn bunker_handshake_times_out_without_signer() {
    let (_relay, url) = start_relay().await;
    let ghost = Keys::generate().public_key().to_hex();
    let uri = format!("bunker://{ghost}?relay={url}");
    let err = bunker::connect_with_timeout(&uri, None, &[], Duration::from_secs(2))
        .await
        .unwrap_err();
    assert!(err.contains("Timed out"), "{err}");
    assert!(!bunker::has_session(&uri, None).await);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cancelled_pairing_leaves_no_session_or_auth_urls() {
    let (_relay, url) = start_relay().await;
    let pairing = bunker::start_pairing(std::slice::from_ref(&url), "Yawmiya Test")
        .await
        .expect("pairing");
    assert!(bunker::take_auth_urls(&pairing.token).is_empty());
    bunker::cancel_pairing(&pairing.token).await;
    assert_eq!(
        bunker::poll_pairing(&pairing.token).await,
        PairingStatus::Unknown
    );
    // The client key was never paired with any signer.
    let ghost = format!(
        "bunker://{}?relay={url}",
        Keys::generate().public_key().to_hex()
    );
    assert!(!bunker::has_session(&ghost, Some(&pairing.client_nsec)).await);
}

#[derive(Debug, Clone)]
struct DenySigning;

impl NostrConnectSignerActions for DenySigning {
    fn approve(&self, _public_key: &PublicKey, req: &NostrConnectRequest) -> bool {
        !matches!(req, NostrConnectRequest::SignEvent(_))
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn signer_denial_is_reported_as_signer_refusal() {
    let (_relay, url) = start_relay().await;
    let remote = NostrConnectRemoteSigner::new(signer_keys(), [url.as_str()], None, None)
        .expect("remote signer");
    let bunker_uri = remote.bunker_uri().to_string();
    tokio::spawn(async move {
        let _ = remote.serve(DenySigning).await;
    });
    let client_nsec = keys::generate_new_nsec();
    let mut public = private_event("denied");
    public.is_private = Some(false);
    let err = calendar::publish_batch_calendar_events_core(
        &bunker_uri,
        std::slice::from_ref(&url),
        vec![public],
        Some(&client_nsec),
    )
    .await
    .expect_err("signer denied the event");
    // The UI drops the queued event only for signer-tagged rejections.
    assert!(
        err.contains("Signer refused: ") && err.to_lowercase().contains("reject"),
        "{err}"
    );
}
