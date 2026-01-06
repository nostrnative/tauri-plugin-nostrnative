# nostrnative

A modular, high-performance Nostr library and Tauri plugin for Rust and JavaScript. Designed to be clean, easy to integrate, and highly customizable through feature flags.

## Features

- **Modular Design:** Only include the components you need (Calendar, Bookmarks, Blossom, etc.).
- **Tauri Integration:** First-class support for Tauri v2 with a seamless plugin system and typed frontend bindings.
- **Async-First:** Built on `tokio` and `nostr-sdk` for efficient network operations.
- **Comprehensive NIP Support:** Includes implementations for NIP-01, NIP-04, NIP-51 (Bookmarks), NIP-52 (Calendar), and more.

## Installation

### Rust (Backend)

Add `nostrnative` to your `src-tauri/Cargo.toml`:

```toml
[dependencies]
# Use specific features to keep your binary small
nostrnative = { path = "../../nostrnative", features = ["calendar", "bookmarks", "tauri-plugin"] }

# Or enable everything
# nostrnative = { path = "../../nostrnative", features = ["full", "tauri-plugin"] }
```

### JavaScript (Frontend)

Install the plugin bindings in your Tauri app's frontend directory:

```bash
npm install tauri-plugin-nostr-native
```

## Setup in Tauri

### 1. Register the Plugin

In your `src-tauri/src/lib.rs` (or `main.rs`), initialize the plugin:

```rust
pub fn run() {
    tauri::Builder::default()
        .plugin(nostrnative::init()) // Initialize the plugin
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### 2. Use the JavaScript API

Import the typed bindings in your frontend code:

```typescript
import * as nostr from 'tauri-plugin-nostr-native';

// Examples
const nsec = await nostr.generateNewNsec();
const pubkey = await nostr.parsePubkey(somePubkey);

const events = await nostr.fetchCalendarEvents(pubkey, ["wss://relay.damus.io"], {
  rangeStart: Math.floor(Date.now() / 1000)
});
```

## Available Features

| Feature        | Description                                                 |
| -------------- | ----------------------------------------------------------- |
| `keys`         | Basic key management (generate, parse, verify).             |
| `calendar`     | Calendar events (NIP-52) and RSVPs.                         |
| `profile`      | User profiles (NIP-01) and contact lists (NIP-02).          |
| `messages`     | Direct messages (NIP-04).                                   |
| `bookmarks`    | Public and private bookmarks (NIP-51).                      |
| `blossom`      | Blob Storage Server Operations (mirror, upload, get).       |
| `chat`         | Advanced chat functionality with PNS key derivation.        |
| `tauri-plugin` | Exports all enabled features as Tauri commands.             |
| `full`         | Enables all functional features (excluding `tauri-plugin`). |

## Standalone Library Usage

You can also use the core logic directly in any Rust project:

```rust
use nostrnative::calendar::fetch_calendar_events_core;

#[tokio::main]
async fn main() {
    let relays = vec!["wss://relay.damus.io".to_string()];
    let events = fetch_calendar_events_core(
        "your_pubkey",
        None, // nsec
        &relays,
        None, // start
        None, // end
        None  // authors
    ).await;

    println!("Fetched events: {:?}", events);
}
```

## Permissions

This plugin includes a set of default permissions. Check the `permissions/` directory for details on how to configure access to specific Nostr commands in your Tauri application.
