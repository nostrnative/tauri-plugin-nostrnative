# tauri-plugin-nostrnative

A modular, high-performance Nostr library and Tauri plugin for Rust and JavaScript. Designed to be clean, easy to integrate, and highly customizable through feature flags.

## Features

- **Modular Design:** Only include the components you need (Calendar, Bookmarks, Blossom, etc.).
- **Tauri Integration:** First-class support for Tauri v2 with a seamless plugin system and typed frontend bindings.
- **Async-First:** Built on `tokio` and `nostr-sdk` for efficient network operations.
- **Comprehensive NIP Support:** Includes implementations for NIP-01, NIP-44 (Encryption), NIP-46 (Nostr Connect), NIP-51 (Bookmarks), NIP-52 (Calendar), and more.

## Installation

### Rust (Backend)

Add `tauri-plugin-nostrnative` to your `src-tauri/Cargo.toml`:

```toml
[dependencies]
# Use specific features to keep your binary small
tauri-plugin-nostrnative = { path = "../../nostrnative", features = ["calendar", "bookmarks", "tauri-plugin"] }

# Or enable everything
# tauri-plugin-nostrnative = { path = "../../nostrnative", features = ["full", "tauri-plugin"] }
```

### JavaScript (Frontend)

Install the plugin bindings in your Tauri app's frontend directory:

```bash
npm install tauri-plugin-nostrnative
```

## Setup in Tauri

### 1. Register the Plugin

In your `src-tauri/src/lib.rs` (or `main.rs`), initialize the plugin:

```rust
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_nostrnative::init()) // Initialize the plugin
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### 2. Use the JavaScript API

Import the typed bindings in your frontend code:

```typescript
import * as nostr from 'tauri-plugin-nostrnative';

// Examples
const nsec = await nostr.generateNewNsec();
const pubkey = await nostr.parsePubkey(somePubkey);

const events = await nostr.fetchCalendarEvents(pubkey, ["wss://relay.damus.io"], {
  rangeStart: Math.floor(Date.now() / 1000)
});

const providers = await nostr.discoverRoutstrProviders(["wss://relay.routstr.com"]);
const models = await nostr.fetchRoutstrModels(providers[0].urls[0]);
```

## Remote signers (NIP-46 / bunker)

Users do not have to paste an `nsec`. Any command that takes a key accepts a
`bunker://` URI instead, and the plugin keeps one authenticated NIP-46 session
per (signer, client key) pair for the lifetime of the process, so the
`connect` handshake (and the one-time secret) is only sent once.

Two pairing directions are supported:

```ts
import {
  bunkerConnect,
  nostrConnectStart,
  nostrConnectPoll,
  nostrConnectCancel,
  bunkerDisconnect,
  bunkerTakeAuthUrls,
} from "tauri-plugin-nostrnative";

// 1. Signer-initiated: the user pastes a bunker:// URI from Amber / nsec.app.
const conn = await bunkerConnect(uri); // performs the handshake (60s timeout)
// Persist conn.bunker_uri (secret stripped) and conn.client_nsec. Pass them as
// `key` / `clientNsec` to every other command; they reuse the live session.

// 2. Client-initiated: show a QR code, the signer app scans it.
const pairing = await nostrConnectStart(["wss://relay.nsec.app"], "My App");
renderQr(pairing.uri); // nostrconnect://...
const status = await nostrConnectPoll(pairing.token); // pending | connected | failed
// On `connected` you get the same { user_pubkey, bunker_uri, client_nsec }.

// Some signers answer with an auth_url challenge the user must open:
for (const url of await bunkerTakeAuthUrls()) openInBrowser(url);

// On logout:
await bunkerDisconnect(conn.bunker_uri, conn.client_nsec);
```

Timeouts: handshake 60s, each signing/encryption request 90s, QR pairing 5min.
Errors are returned as readable strings (e.g. "Timed out waiting for the remote
signer…"). From Rust, the same API lives in `tauri_plugin_nostrnative::bunker`.

Run `cargo test` to execute the end-to-end NIP-46 suite (`tests/nip46_flow.rs`),
which spins up an in-process relay and remote signer.

## Available Features

| Feature        | Description                                                 |
| -------------- | ----------------------------------------------------------- |
| `keys`         | Basic key management (generate, parse, verify).             |
| `calendar`     | Calendar events (NIP-52) and RSVPs.                         |
| `profile`      | User profiles (NIP-01) and contact lists (NIP-02).          |
| `messages`     | Direct messages (NIP-44).                                   |
| `bookmarks`    | Public and private bookmarks (NIP-51).                      |
| `notes`        | Secure notes and notebooks management.                      |
| `blossom`      | Blob Storage Server Operations (mirror, upload, get).       |
| `chat`         | Advanced chat functionality with PNS key derivation.        |
| `routstr`      | Routstr provider discovery and price comparison.            |
| `tauri-plugin` | Exports all enabled features as Tauri commands.             |
| `full`         | Enables all functional features (excluding `tauri-plugin`). |

## Standalone Library Usage

You can also use the core logic directly in any Rust project:

```rust
use tauri_plugin_nostrnative::calendar::fetch_calendar_events_core;

#[tokio::main]
async fn main() {
    let relays = vec!["wss://relay.damus.io".to_string()];
    let events = fetch_calendar_events_core(
        "your_pubkey",
        None, // nsec
        &relays,
        None, // start
        None, // end
        None, // authors
        None  // client_nsec
    ).await;

    println!("Fetched events: {:?}", events);
}
```

## Permissions

This plugin includes a set of default permissions. Check the `permissions/` directory for details on how to configure access to specific Nostr commands in your Tauri application.
