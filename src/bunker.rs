//! NIP-46 (Nostr Connect / "bunker") session management.
//!
//! Design goals:
//! - One handshake per (remote signer, client key) pair. The `connect` RPC is
//!   sent once; the session is cached and reused by every later command, so a
//!   one-time bunker secret is never replayed.
//! - Bounded waits: the handshake and every signing request have explicit
//!   timeouts, with user-readable errors.
//! - Both pairing directions: `bunker://` (signer-initiated, paste a URI) and
//!   `nostrconnect://` (client-initiated, scan a QR code in the signer app).
//! - `auth_url` challenges are captured so the UI can open them.

use nostr::util::BoxedFuture;
use nostr_connect::prelude::*;
use nostr_sdk::prelude::*;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::task::AbortHandle;

/// Max time to wait for the remote signer to answer the initial `connect`.
pub const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(60);
/// Max time to wait for any single signing / encryption request.
pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(90);
/// How long a `nostrconnect://` pairing stays open waiting for a signer.
pub const PAIRING_TIMEOUT: Duration = Duration::from_secs(300);

const BUNKER_SCHEME: &str = "bunker://";
const NOSTRCONNECT_SCHEME: &str = "nostrconnect://";

/// A live, authenticated NIP-46 session.
#[derive(Clone, Debug)]
pub struct BunkerSession {
    /// Client whose signer is the remote bunker. Relays are the app relays.
    pub client: Client,
    pub signer: NostrConnect,
    pub user_pubkey: PublicKey,
    /// Canonical `bunker://` URI *without* the one-time secret. Safe to persist
    /// and reuse on the next launch together with `client_nsec`.
    pub bunker_uri: String,
    pub client_nsec: String,
}

/// Serializable description of an established session, returned to the UI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BunkerConnection {
    pub user_pubkey: String,
    pub bunker_uri: String,
    pub client_nsec: String,
}

/// A client-initiated pairing waiting for a signer to connect.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NostrConnectPairing {
    pub token: String,
    /// `nostrconnect://...` URI to show as a QR code / deep link.
    pub uri: String,
    pub client_nsec: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum PairingStatus {
    Pending,
    Connected(BunkerConnection),
    Failed { error: String },
    Unknown,
}

type SessionSlot = Arc<Mutex<Option<BunkerSession>>>;
type PairingEntry = (PairingStatus, Option<AbortHandle>);

static SESSIONS: Lazy<Mutex<HashMap<String, SessionSlot>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
static PAIRINGS: Lazy<Mutex<HashMap<String, PairingEntry>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
/// `auth_url` challenges keyed by the client public key (hex) of the session
/// or pairing that received them.
static AUTH_URLS: Lazy<std::sync::Mutex<HashMap<String, Vec<String>>>> =
    Lazy::new(|| std::sync::Mutex::new(HashMap::new()));

pub fn is_bunker_uri(s: &str) -> bool {
    s.trim().starts_with(BUNKER_SCHEME)
}

pub fn is_nostrconnect_uri(s: &str) -> bool {
    s.trim().starts_with(NOSTRCONNECT_SCHEME)
}

/// True when the credential is any NIP-46 URI rather than a raw private key.
pub fn is_remote_signer(s: &str) -> bool {
    is_bunker_uri(s) || is_nostrconnect_uri(s)
}

#[derive(Debug, Clone)]
struct AuthUrlCollector {
    /// Client public key (hex) identifying the session / pairing.
    key: String,
}

impl AuthUrlHandler for AuthUrlCollector {
    fn on_auth_url(&self, auth_url: Url) -> BoxedFuture<'_, nostr::Result<()>> {
        Box::pin(async move {
            if let Ok(mut map) = AUTH_URLS.lock() {
                map.entry(self.key.clone())
                    .or_default()
                    .push(auth_url.to_string());
            }
            Ok(())
        })
    }
}

/// Drain `auth_url` challenges for one session / pairing.
///
/// `token` is the client public key (hex): the pairing token returned by
/// [`start_pairing`], or derived from the client nsec with
/// [`auth_url_token_for_client`]. Only `https://` URLs are returned.
pub fn take_auth_urls(token: &str) -> Vec<String> {
    AUTH_URLS
        .lock()
        .map(|mut map| map.remove(token).unwrap_or_default())
        .unwrap_or_default()
        .into_iter()
        .filter(|u| u.starts_with("https://"))
        .collect()
}

/// Token under which `auth_url` challenges for this client key are collected.
pub fn auth_url_token_for_client(client_nsec: &str) -> Result<String, String> {
    Ok(parse_client_keys(Some(client_nsec))?.public_key().to_hex())
}

fn forget_auth_urls(token: &str) {
    if let Ok(mut map) = AUTH_URLS.lock() {
        map.remove(token);
    }
}

fn parse_client_keys(client_nsec: Option<&str>) -> Result<Keys, String> {
    match client_nsec.map(str::trim) {
        Some(nsec) if !nsec.is_empty() => {
            Keys::parse(nsec).map_err(|e| format!("Invalid client key: {e}"))
        }
        _ => Ok(Keys::generate()),
    }
}

fn session_key(remote_signer: &PublicKey, client_keys: &Keys) -> String {
    format!(
        "{}|{}",
        remote_signer.to_hex(),
        client_keys.public_key().to_hex()
    )
}

async fn slot_for(key: &str) -> SessionSlot {
    let mut map = SESSIONS.lock().await;
    map.entry(key.to_string())
        .or_insert_with(|| Arc::new(Mutex::new(None)))
        .clone()
}

fn describe_signer_error(e: impl std::fmt::Display) -> String {
    let msg = e.to_string();
    if msg.to_lowercase().contains("timeout") {
        "The remote signer did not respond in time. Make sure your signer app is open and online, then try again.".to_string()
    } else {
        format!("Remote signer error: {msg}")
    }
}

async fn ensure_relays(client: &Client, relays: &[String]) {
    for relay in relays {
        let _ = client.add_relay(relay).await;
    }
    if !relays.is_empty() {
        client.connect().await;
    }
}

async fn build_session(
    signer: NostrConnect,
    client_keys: &Keys,
    user_pubkey: PublicKey,
    relays: &[String],
) -> Result<BunkerSession, String> {
    let bunker_uri = signer
        .bunker_uri()
        .await
        .map_err(describe_signer_error)?
        .to_string();
    let client = Client::builder().signer(signer.clone()).build();
    ensure_relays(&client, relays).await;
    let client_nsec = client_keys
        .secret_key()
        .to_bech32()
        .map_err(|e| e.to_string())?;
    Ok(BunkerSession {
        client,
        signer,
        user_pubkey,
        bunker_uri,
        client_nsec,
    })
}

pub fn connection_of(session: &BunkerSession) -> BunkerConnection {
    BunkerConnection {
        user_pubkey: session.user_pubkey.to_hex(),
        bunker_uri: session.bunker_uri.clone(),
        client_nsec: session.client_nsec.clone(),
    }
}

fn parse_bunker_uri(uri_str: &str) -> Result<(NostrConnectURI, PublicKey), String> {
    let uri =
        NostrConnectURI::parse(uri_str.trim()).map_err(|e| format!("Invalid bunker URI: {e}"))?;
    let remote = match &uri {
        NostrConnectURI::Bunker {
            remote_signer_public_key,
            ..
        } => *remote_signer_public_key,
        NostrConnectURI::Client { .. } => {
            return Err(
                "Expected a bunker:// URI. nostrconnect:// URIs are generated by the app for QR pairing."
                    .to_string(),
            );
        }
    };
    if uri.relays().is_empty() {
        return Err(
            "Bunker URI has no relay. It must contain at least one ?relay=wss://… parameter."
                .to_string(),
        );
    }
    Ok((uri, remote))
}

/// Connect to a `bunker://` signer, or return the cached session.
///
/// `relays` are the *application* relays the returned client should publish
/// to; the signer talks to the relays embedded in the URI.
pub async fn connect(
    uri_str: &str,
    client_nsec: Option<&str>,
    relays: &[String],
) -> Result<BunkerSession, String> {
    connect_with_timeout(uri_str, client_nsec, relays, HANDSHAKE_TIMEOUT).await
}

/// Same as [`connect`] with an explicit handshake timeout.
pub async fn connect_with_timeout(
    uri_str: &str,
    client_nsec: Option<&str>,
    relays: &[String],
    handshake_timeout: Duration,
) -> Result<BunkerSession, String> {
    let (uri, remote) = parse_bunker_uri(uri_str)?;
    let client_keys = parse_client_keys(client_nsec)?;
    let key = session_key(&remote, &client_keys);
    let slot = slot_for(&key).await;
    let mut guard = slot.lock().await;

    if let Some(session) = guard.as_ref() {
        ensure_relays(&session.client, relays).await;
        return Ok(session.clone());
    }

    let mut signer = NostrConnect::new(uri, client_keys.clone(), REQUEST_TIMEOUT, None)
        .map_err(describe_signer_error)?;
    signer.auth_url_handler(AuthUrlCollector {
        key: client_keys.public_key().to_hex(),
    });

    let user_pubkey = tokio::time::timeout(handshake_timeout, signer.get_public_key())
        .await
        .map_err(|_| {
            "Timed out waiting for the remote signer to approve the connection. Open your signer app and approve this client, then try again.".to_string()
        })?
        .map_err(describe_signer_error)?;

    let session = build_session(signer, &client_keys, user_pubkey, relays).await?;
    *guard = Some(session.clone());
    Ok(session)
}

/// Shut down and forget the session for this URI / client key pair.
pub async fn disconnect(uri_str: &str, client_nsec: Option<&str>) -> Result<bool, String> {
    let (_, remote) = parse_bunker_uri(uri_str)?;
    let client_keys = parse_client_keys(client_nsec)?;
    let key = session_key(&remote, &client_keys);
    let slot = SESSIONS.lock().await.remove(&key);
    let Some(slot) = slot else {
        return Ok(false);
    };
    let session = slot.lock().await.take();
    if let Some(session) = session {
        let _ = session.client.shutdown().await;
        session.signer.shutdown().await;
        return Ok(true);
    }
    Ok(false)
}

/// Shut down every cached session (e.g. on app exit).
pub async fn disconnect_all() {
    let slots: Vec<SessionSlot> = SESSIONS.lock().await.drain().map(|(_, s)| s).collect();
    for slot in slots {
        if let Some(session) = slot.lock().await.take() {
            let _ = session.client.shutdown().await;
            session.signer.shutdown().await;
        }
    }
}

/// Whether a live session exists for this URI / client key pair.
pub async fn has_session(uri_str: &str, client_nsec: Option<&str>) -> bool {
    let Ok((_, remote)) = parse_bunker_uri(uri_str) else {
        return false;
    };
    let Ok(client_keys) = parse_client_keys(client_nsec) else {
        return false;
    };
    let key = session_key(&remote, &client_keys);
    let slot = SESSIONS.lock().await.get(&key).cloned();
    match slot {
        Some(slot) => slot.lock().await.is_some(),
        None => false,
    }
}

/// Number of live sessions (for diagnostics / tests).
pub async fn session_count() -> usize {
    let map = SESSIONS.lock().await;
    let mut n = 0;
    for slot in map.values() {
        if slot.lock().await.is_some() {
            n += 1;
        }
    }
    n
}

/// Begin a client-initiated pairing. Show `uri` as a QR code; poll with
/// [`poll_pairing`] until the status is `Connected` or `Failed`.
pub async fn start_pairing(
    relays: &[String],
    app_name: &str,
) -> Result<NostrConnectPairing, String> {
    if relays.is_empty() {
        return Err("At least one relay is required for nostrconnect pairing.".to_string());
    }
    let relay_urls: Vec<RelayUrl> = relays
        .iter()
        .map(|r| RelayUrl::parse(r).map_err(|e| format!("Invalid relay {r}: {e}")))
        .collect::<Result<_, _>>()?;

    let client_keys = Keys::generate();
    let uri = NostrConnectURI::client(client_keys.public_key(), relay_urls, app_name);
    let token = client_keys.public_key().to_hex();
    let client_nsec = client_keys
        .secret_key()
        .to_bech32()
        .map_err(|e| e.to_string())?;

    let mut signer = NostrConnect::new(uri.clone(), client_keys.clone(), REQUEST_TIMEOUT, None)
        .map_err(describe_signer_error)?;
    signer.auth_url_handler(AuthUrlCollector { key: token.clone() });

    let relays_owned = relays.to_vec();
    let task_token = token.clone();
    let handle = tokio::spawn(async move {
        let outcome = match tokio::time::timeout(PAIRING_TIMEOUT, signer.get_public_key()).await {
            Err(_) => PairingStatus::Failed {
                error:
                    "No signer connected within 5 minutes. Generate a new QR code and try again."
                        .to_string(),
            },
            Ok(Err(e)) => PairingStatus::Failed {
                error: describe_signer_error(e),
            },
            Ok(Ok(user_pubkey)) => {
                match build_session(signer, &client_keys, user_pubkey, &relays_owned).await {
                    Ok(session) => match parse_bunker_uri(&session.bunker_uri) {
                        Ok((_, remote)) => {
                            // Register the session only if the pairing is still
                            // wanted; a cancel that raced with us must not leak
                            // a live signer connection into SESSIONS.
                            let mut pairings = PAIRINGS.lock().await;
                            if pairings.contains_key(&task_token) {
                                let key = session_key(&remote, &client_keys);
                                let slot = slot_for(&key).await;
                                *slot.lock().await = Some(session.clone());
                                if let Some(entry) = pairings.get_mut(&task_token) {
                                    entry.0 = PairingStatus::Connected(connection_of(&session));
                                    entry.1 = None;
                                }
                            } else {
                                drop(pairings);
                                let _ = session.client.shutdown().await;
                                session.signer.shutdown().await;
                            }
                            return;
                        }
                        Err(e) => PairingStatus::Failed { error: e },
                    },
                    Err(error) => PairingStatus::Failed { error },
                }
            }
        };
        let mut map = PAIRINGS.lock().await;
        if let Some(entry) = map.get_mut(&task_token) {
            entry.0 = outcome;
            entry.1 = None;
        }
    });

    PAIRINGS.lock().await.insert(
        token.clone(),
        (PairingStatus::Pending, Some(handle.abort_handle())),
    );

    Ok(NostrConnectPairing {
        token,
        uri: uri.to_string(),
        client_nsec,
    })
}

pub async fn poll_pairing(token: &str) -> PairingStatus {
    PAIRINGS
        .lock()
        .await
        .get(token)
        .map(|(status, _)| status.clone())
        .unwrap_or(PairingStatus::Unknown)
}

/// Abandon a pairing. If the signer already connected, that session is kept
/// and is reachable through the `Connected` status returned earlier; otherwise
/// the waiting task is aborted and nothing is left behind.
pub async fn cancel_pairing(token: &str) {
    let removed = PAIRINGS.lock().await.remove(token);
    forget_auth_urls(token);
    if let Some((_, Some(handle))) = removed {
        handle.abort();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_uri_kinds() {
        assert!(is_bunker_uri("bunker://abc?relay=wss://r"));
        assert!(is_bunker_uri("  bunker://abc"));
        assert!(!is_bunker_uri("nsec1abc"));
        assert!(is_nostrconnect_uri("nostrconnect://abc"));
        assert!(is_remote_signer("bunker://x"));
        assert!(is_remote_signer("nostrconnect://x"));
        assert!(!is_remote_signer("nsec1x"));
    }

    #[test]
    fn rejects_bunker_uri_without_relay() {
        let pk = Keys::generate().public_key().to_hex();
        let err = parse_bunker_uri(&format!("bunker://{pk}")).unwrap_err();
        assert!(err.contains("relay"), "{err}");
    }

    #[test]
    fn rejects_nostrconnect_uri_as_bunker() {
        let uri = NostrConnectURI::client(
            Keys::generate().public_key(),
            vec![RelayUrl::parse("wss://r.example").unwrap()],
            "app",
        )
        .to_string();
        assert!(is_nostrconnect_uri(&uri), "{uri}");
        let err = parse_bunker_uri(&uri).unwrap_err();
        assert!(err.contains("bunker://"), "{err}");
    }

    #[test]
    fn rejects_garbage_uri() {
        assert!(parse_bunker_uri("hello").is_err());
        assert!(parse_client_keys(Some("not-a-key")).is_err());
        assert!(parse_client_keys(None).is_ok());
        assert!(parse_client_keys(Some("   ")).is_ok());
    }

    #[test]
    fn session_key_is_stable_per_pair() {
        let remote = Keys::generate().public_key();
        let client = Keys::generate();
        assert_eq!(session_key(&remote, &client), session_key(&remote, &client));
        assert_ne!(
            session_key(&remote, &client),
            session_key(&remote, &Keys::generate())
        );
    }

    #[tokio::test]
    async fn pairing_requires_relays() {
        assert!(start_pairing(&[], "app").await.is_err());
        assert!(start_pairing(&["not a url".into()], "app").await.is_err());
    }

    #[tokio::test]
    async fn pairing_uri_is_nostrconnect_and_cancellable() {
        let pairing = start_pairing(&["wss://relay.example.com".into()], "Yawmiya")
            .await
            .unwrap();
        assert!(
            pairing.uri.starts_with(NOSTRCONNECT_SCHEME),
            "{}",
            pairing.uri
        );
        assert!(pairing.client_nsec.starts_with("nsec1"));
        assert_eq!(poll_pairing(&pairing.token).await, PairingStatus::Pending);
        cancel_pairing(&pairing.token).await;
        assert_eq!(poll_pairing(&pairing.token).await, PairingStatus::Unknown);
    }

    #[test]
    fn auth_urls_are_scoped_and_https_only() {
        {
            let mut map = AUTH_URLS.lock().unwrap();
            map.entry("one".into())
                .or_default()
                .extend(["https://a".to_string(), "file:///etc/passwd".to_string()]);
            map.entry("two".into())
                .or_default()
                .push("https://b".into());
        }
        assert_eq!(take_auth_urls("one"), vec!["https://a".to_string()]);
        assert!(take_auth_urls("one").is_empty());
        assert_eq!(take_auth_urls("two"), vec!["https://b".to_string()]);
        assert!(take_auth_urls("missing").is_empty());
    }

    #[test]
    fn auth_url_token_is_client_pubkey() {
        let keys = Keys::generate();
        let nsec = keys.secret_key().to_bech32().unwrap();
        assert_eq!(
            auth_url_token_for_client(&nsec).unwrap(),
            keys.public_key().to_hex()
        );
        assert!(auth_url_token_for_client("junk").is_err());
    }

    #[test]
    fn timeout_errors_are_user_readable() {
        let msg = describe_signer_error("request timeout");
        assert!(msg.contains("signer app"));
    }
}
