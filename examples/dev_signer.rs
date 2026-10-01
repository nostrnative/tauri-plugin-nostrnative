//! dev_signer: a prototype NIP-46 remote signer for manual and E2E testing.
//!
//! Behaves like Amber / nsec.app from the client's point of view:
//! - prints a `bunker://` URI with a one-time secret,
//! - remembers authorized client keys on disk, so a client can reconnect
//!   after restart *without* the secret (and is refused if it was revoked),
//! - can pair with a client-initiated `nostrconnect://` URI (QR flow),
//! - can inject an `auth_url` challenge, artificial latency, or denials,
//! - logs every request so you can see exactly what the app asks for.
//!
//! Usage (from the nostrnative repo):
//!
//! ```text
//! cargo run --example dev_signer -- --local-relay 7777 \
//!     --advertise ws://10.0.2.2:7777          # what an Android emulator sees
//! cargo run --example dev_signer -- --relay wss://relay.nsec.app \
//!     --nostrconnect 'nostrconnect://…'        # answer a QR code
//! ```
//!
//! Flags:
//!   --state DIR           persistent state dir (default: ./dev-signer-state)
//!   --relay URL           relay the signer listens on (repeatable)
//!   --local-relay PORT    also host an in-memory relay on 0.0.0.0:PORT
//!   --advertise URL       relay written into the bunker URI (repeatable)
//!   --nostrconnect URI    pair with a client-initiated nostrconnect:// URI
//!   --delay-ms N          wait N ms before answering every request
//!   --deny RULE           refuse a method (sign_event, nip44_decrypt, …) or kind:N
//!   --auth-url URL        challenge new clients with this auth_url first
//!   --revoke HEX          remove an authorized client key and exit
//!   --reset               forget all authorized clients on start
//!   --relay-only          only host --local-relay (no signer); keeps events
//!                         across signer restarts in tests

use std::collections::BTreeSet;
use std::net::{IpAddr, Ipv4Addr};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use nostr::nips::nip46::ResponseResult;
use nostr_relay_builder::prelude::*;
use nostr_sdk::prelude::*;
use tokio::sync::Mutex;

#[derive(Debug, Default, Clone)]
struct Args {
    state: PathBuf,
    relays: Vec<String>,
    local_relay: Option<u16>,
    advertise: Vec<String>,
    nostrconnect: Option<String>,
    delay: Duration,
    deny: Vec<String>,
    auth_url: Option<String>,
    revoke: Option<String>,
    reset: bool,
    relay_only: bool,
}

fn parse_args() -> Result<Args, String> {
    let mut args = Args {
        state: PathBuf::from("dev-signer-state"),
        ..Default::default()
    };
    let mut it = std::env::args().skip(1);
    while let Some(flag) = it.next() {
        let mut value = || it.next().ok_or(format!("{flag} needs a value"));
        match flag.as_str() {
            "--state" => args.state = PathBuf::from(value()?),
            "--relay" => args.relays.push(value()?),
            "--local-relay" => {
                args.local_relay = Some(value()?.parse().map_err(|e| format!("port: {e}"))?)
            }
            "--advertise" => args.advertise.push(value()?),
            "--nostrconnect" => args.nostrconnect = Some(value()?),
            "--delay-ms" => {
                args.delay =
                    Duration::from_millis(value()?.parse().map_err(|e| format!("delay: {e}"))?)
            }
            "--deny" => args.deny.push(value()?),
            "--auth-url" => args.auth_url = Some(value()?),
            "--revoke" => args.revoke = Some(value()?),
            "--reset" => args.reset = true,
            "--relay-only" => args.relay_only = true,
            "-h" | "--help" => {
                println!("see the doc comment at the top of examples/dev_signer.rs");
                std::process::exit(0);
            }
            other => return Err(format!("unknown flag {other}")),
        }
    }
    Ok(args)
}

/// Persistent signer state.
struct State {
    dir: PathBuf,
    user: Keys,
    signer: Keys,
    authorized: BTreeSet<PublicKey>,
    /// One-time bunker secret; consumed by the first successful connect.
    secret: Option<String>,
    /// Clients that already went through the auth_url challenge.
    challenged: BTreeSet<PublicKey>,
}

fn load_or_create_keys(path: &Path) -> Keys {
    if let Ok(s) = std::fs::read_to_string(path)
        && let Ok(k) = Keys::parse(s.trim())
    {
        return k;
    }
    let k = Keys::generate();
    std::fs::write(path, k.secret_key().to_bech32().unwrap()).expect("write key");
    k
}

impl State {
    fn load(dir: &Path, reset: bool) -> Self {
        std::fs::create_dir_all(dir).expect("state dir");
        let user = load_or_create_keys(&dir.join("user.nsec"));
        let signer = load_or_create_keys(&dir.join("signer.nsec"));
        let authorized = if reset {
            BTreeSet::new()
        } else {
            std::fs::read_to_string(dir.join("authorized.txt"))
                .unwrap_or_default()
                .lines()
                .filter_map(|l| PublicKey::from_hex(l.trim()).ok())
                .collect()
        };
        let secret = Some(Keys::generate().public_key().to_hex()[..16].to_string());
        let state = Self {
            dir: dir.to_path_buf(),
            user,
            signer,
            authorized,
            secret,
            challenged: BTreeSet::new(),
        };
        state.save();
        state
    }

    fn save(&self) {
        let body: Vec<String> = self.authorized.iter().map(|p| p.to_hex()).collect();
        std::fs::write(self.dir.join("authorized.txt"), body.join("\n")).expect("save");
    }
}

struct Signer {
    args: Args,
    state: Mutex<State>,
    client: Client,
    started: Instant,
    seq: AtomicU64,
}

fn method_name(req: &NostrConnectRequest) -> &'static str {
    match req {
        NostrConnectRequest::Connect { .. } => "connect",
        NostrConnectRequest::GetPublicKey => "get_public_key",
        NostrConnectRequest::SignEvent(_) => "sign_event",
        NostrConnectRequest::Nip04Encrypt { .. } => "nip04_encrypt",
        NostrConnectRequest::Nip04Decrypt { .. } => "nip04_decrypt",
        NostrConnectRequest::Nip44Encrypt { .. } => "nip44_encrypt",
        NostrConnectRequest::Nip44Decrypt { .. } => "nip44_decrypt",
        NostrConnectRequest::Ping => "ping",
    }
}

impl Signer {
    fn log(&self, msg: impl std::fmt::Display) {
        println!("[{:>7.2}s] {msg}", self.started.elapsed().as_secs_f64());
    }

    fn denied(&self, req: &NostrConnectRequest) -> bool {
        let method = method_name(req);
        self.args.deny.iter().any(|rule| {
            rule == method
                || matches!(req, NostrConnectRequest::SignEvent(u)
                    if rule.strip_prefix("kind:").and_then(|k| k.parse::<u16>().ok()) == Some(u.kind.as_u16()))
        })
    }

    async fn send(&self, to: PublicKey, msg: NostrConnectMessage) -> Result<()> {
        let keys = self.state.lock().await.signer.clone();
        let event = EventBuilder::nostr_connect(&keys, to, msg)?.sign_with_keys(&keys)?;
        self.client.send_event(&event).await?;
        Ok(())
    }

    async fn respond(&self, to: PublicKey, id: &str, res: NostrConnectResponse) {
        if let Err(e) = self.send(to, NostrConnectMessage::response(id, res)).await {
            self.log(format!("!! failed to send response: {e}"));
        }
    }

    async fn handle(self: Arc<Self>, from: PublicKey, id: String, req: NostrConnectRequest) {
        let n = self.seq.fetch_add(1, Ordering::SeqCst);
        let method = method_name(&req);
        let short = &from.to_hex()[..8];
        let detail = match &req {
            NostrConnectRequest::SignEvent(u) => format!(" kind={}", u.kind.as_u16()),
            NostrConnectRequest::Connect { secret, .. } => {
                format!(" secret={}", if secret.is_some() { "yes" } else { "no" })
            }
            _ => String::new(),
        };
        self.log(format!("<- #{n} {method}{detail} from client {short}"));
        if !self.args.delay.is_zero() {
            tokio::time::sleep(self.args.delay).await;
        }

        // Authorization.
        if let NostrConnectRequest::Connect {
            remote_signer_public_key,
            secret,
        } = &req
        {
            let mut st = self.state.lock().await;
            if *remote_signer_public_key != st.signer.public_key() {
                drop(st);
                self.log(format!("-> #{n} error: wrong remote signer key"));
                self.respond(
                    from,
                    &id,
                    NostrConnectResponse::with_error("remote signer public key mismatch"),
                )
                .await;
                return;
            }
            let fresh = matches!((secret, &st.secret), (Some(s), Some(expected)) if s == expected);
            if !fresh && !st.authorized.contains(&from) {
                drop(st);
                self.log(format!(
                    "-> #{n} error: unauthorized (bad/missing secret, client not known)"
                ));
                self.respond(
                    from,
                    &id,
                    NostrConnectResponse::with_error(
                        "unauthorized: invalid or already used secret",
                    ),
                )
                .await;
                return;
            }
            if let Some(url) = &self.args.auth_url
                && !st.challenged.contains(&from)
            {
                st.challenged.insert(from);
                drop(st);
                self.log(format!("-> #{n} auth_url challenge {url}"));
                self.respond(
                    from,
                    &id,
                    NostrConnectResponse::new(Some(ResponseResult::AuthUrl), Some(url.clone())),
                )
                .await;
                tokio::time::sleep(Duration::from_secs(3)).await;
                st = self.state.lock().await;
            }
            if fresh {
                st.secret = None; // one-time
                self.log("   secret consumed; new clients now need a restart for a fresh secret");
            }
            st.authorized.insert(from);
            st.save();
            drop(st);
            self.log(format!("-> #{n} ack (client {short} authorized)"));
            self.respond(
                from,
                &id,
                NostrConnectResponse::with_result(ResponseResult::Ack),
            )
            .await;
            return;
        }

        if !self.state.lock().await.authorized.contains(&from) {
            self.log(format!("-> #{n} error: client {short} not authorized"));
            self.respond(
                from,
                &id,
                NostrConnectResponse::with_error("unauthorized client"),
            )
            .await;
            return;
        }
        if self.denied(&req) {
            self.log(format!("-> #{n} DENIED by policy"));
            self.respond(
                from,
                &id,
                NostrConnectResponse::with_error("user rejected the request"),
            )
            .await;
            return;
        }

        let user = self.state.lock().await.user.clone();
        let result: std::result::Result<ResponseResult, String> = match req {
            NostrConnectRequest::GetPublicKey => {
                Ok(ResponseResult::GetPublicKey(user.public_key()))
            }
            NostrConnectRequest::Ping => Ok(ResponseResult::Pong),
            NostrConnectRequest::SignEvent(unsigned) => {
                if unsigned.pubkey != user.public_key() {
                    Err("event pubkey does not match signer user".into())
                } else {
                    unsigned
                        .sign_with_keys(&user)
                        .map(|e| ResponseResult::SignEvent(Box::new(e)))
                        .map_err(|e| e.to_string())
                }
            }
            NostrConnectRequest::Nip44Encrypt { public_key, text } => nip44::encrypt(
                user.secret_key(),
                &public_key,
                text,
                nip44::Version::default(),
            )
            .map(|ciphertext| ResponseResult::Nip44Encrypt { ciphertext })
            .map_err(|e| e.to_string()),
            NostrConnectRequest::Nip44Decrypt {
                public_key,
                ciphertext,
            } => nip44::decrypt(user.secret_key(), &public_key, ciphertext)
                .map(|plaintext| ResponseResult::Nip44Decrypt { plaintext })
                .map_err(|e| e.to_string()),
            NostrConnectRequest::Nip04Encrypt { public_key, text } => {
                nip04::encrypt(user.secret_key(), &public_key, text)
                    .map(|ciphertext| ResponseResult::Nip04Encrypt { ciphertext })
                    .map_err(|e| e.to_string())
            }
            NostrConnectRequest::Nip04Decrypt {
                public_key,
                ciphertext,
            } => nip04::decrypt(user.secret_key(), &public_key, ciphertext)
                .map(|plaintext| ResponseResult::Nip04Decrypt { plaintext })
                .map_err(|e| e.to_string()),
            NostrConnectRequest::Connect { .. } => unreachable!(),
        };
        match result {
            Ok(r) => {
                self.log(format!("-> #{n} ok"));
                self.respond(from, &id, NostrConnectResponse::with_result(r))
                    .await;
            }
            Err(e) => {
                self.log(format!("-> #{n} error: {e}"));
                self.respond(from, &id, NostrConnectResponse::with_error(e))
                    .await;
            }
        }
    }

    async fn pair(&self, uri: &str) -> Result<()> {
        let uri = NostrConnectURI::parse(uri)?;
        let NostrConnectURI::Client {
            public_key, relays, ..
        } = &uri
        else {
            return Err("expected a nostrconnect:// URI".into());
        };
        for r in relays {
            self.client.add_relay(r.as_str()).await?;
        }
        self.client.connect().await;
        tokio::time::sleep(Duration::from_secs(1)).await;
        {
            let mut st = self.state.lock().await;
            st.authorized.insert(*public_key);
            st.save();
        }
        // Spec: the signer answers a client-initiated pairing with a
        // `connect` response ("ack") addressed to the client key.
        let msg = NostrConnectMessage::response(
            Keys::generate().public_key().to_hex()[..12].to_string(),
            NostrConnectResponse::with_result(ResponseResult::Ack),
        );
        self.send(*public_key, msg).await?;
        self.log(format!(
            "-> pairing ack sent to client {} via {:?}",
            &public_key.to_hex()[..8],
            relays.iter().map(|r| r.to_string()).collect::<Vec<_>>()
        ));
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = parse_args().map_err(|e| {
        eprintln!("error: {e}");
        std::process::exit(2);
    })?;

    if let Some(hex) = &args.revoke {
        let mut st = State::load(&args.state, false);
        let pk = PublicKey::from_hex(hex)?;
        let removed = st.authorized.remove(&pk);
        st.save();
        println!("revoked {hex}: {removed}");
        return Ok(());
    }

    let _relay = if let Some(port) = args.local_relay {
        let relay = LocalRelay::new(
            RelayBuilder::default()
                .addr(IpAddr::V4(Ipv4Addr::UNSPECIFIED))
                .port(port),
        );
        relay.run().await?;
        args.relays.push(format!("ws://127.0.0.1:{port}"));
        println!("local relay listening on 0.0.0.0:{port}");
        if args.relay_only {
            println!("READY");
            tokio::signal::ctrl_c().await?;
            return Ok(());
        }
        Some(relay)
    } else {
        None
    };
    if args.relays.is_empty() && args.nostrconnect.is_none() {
        eprintln!("error: give --relay, --local-relay or --nostrconnect");
        std::process::exit(2);
    }
    if args.advertise.is_empty() {
        args.advertise = args.relays.clone();
    }

    let state = State::load(&args.state, args.reset);
    let client = Client::new(state.signer.clone());
    for r in &args.relays {
        client.add_relay(r.as_str()).await?;
    }
    client.connect().await;

    println!("user   npub : {}", state.user.public_key().to_bech32()?);
    println!("signer key  : {}", state.signer.public_key().to_hex());
    println!("authorized  : {} client(s)", state.authorized.len());
    let advertised: Vec<RelayUrl> = args
        .advertise
        .iter()
        .filter_map(|r| RelayUrl::parse(r).ok())
        .collect();
    let bunker = NostrConnectURI::Bunker {
        remote_signer_public_key: state.signer.public_key(),
        relays: advertised,
        secret: state.secret.clone(),
    };
    println!("BUNKER_URI={bunker}");

    let filter = Filter::new()
        .kind(Kind::NostrConnect)
        .pubkey(state.signer.public_key())
        .since(Timestamp::now() - Duration::from_secs(10));

    let signer = Arc::new(Signer {
        args: args.clone(),
        state: Mutex::new(state),
        client: client.clone(),
        started: Instant::now(),
        seq: AtomicU64::new(0),
    });

    if let Some(uri) = &args.nostrconnect {
        signer.pair(uri).await?;
    }
    client.subscribe(filter, None).await?;
    println!("READY");

    let signer_keys = signer.state.lock().await.signer.clone();
    let seen = Arc::new(Mutex::new(BTreeSet::<EventId>::new()));
    client
        .handle_notifications(|n| {
            let signer = signer.clone();
            let signer_keys = signer_keys.clone();
            let seen = seen.clone();
            async move {
                let RelayPoolNotification::Event { event, .. } = n else {
                    return Ok(false);
                };
                if event.kind != Kind::NostrConnect || !seen.lock().await.insert(event.id) {
                    return Ok(false);
                }
                let plaintext =
                    nip44::decrypt(signer_keys.secret_key(), &event.pubkey, &event.content)
                        .or_else(|_| {
                            nip04::decrypt(signer_keys.secret_key(), &event.pubkey, &event.content)
                        });
                let Ok(plaintext) = plaintext else {
                    signer.log(format!(
                        "!! undecryptable message from {}",
                        &event.pubkey.to_hex()[..8]
                    ));
                    return Ok(false);
                };
                let Ok(msg) = NostrConnectMessage::from_json(&plaintext) else {
                    signer.log(format!("!! bad message: {plaintext}"));
                    return Ok(false);
                };
                if msg.is_request() {
                    let id = msg.id().to_string();
                    match msg.to_request() {
                        Ok(req) => {
                            tokio::spawn(signer.clone().handle(event.pubkey, id, req));
                        }
                        Err(e) => signer.log(format!("!! unparseable request: {e}")),
                    }
                }
                Ok(false)
            }
        })
        .await?;
    Ok(())
}
