//! nostrnative - A modular Nostr library.
//!
//! This crate provides a feature-based system for interacting with the Nostr protocol.
//! Features can be enabled in Cargo.toml to include only what is needed.

pub mod models;

#[cfg(feature = "keys")]
pub mod keys;

#[cfg(feature = "calendar")]
pub mod calendar;

#[cfg(feature = "profile")]
pub mod profile;

#[cfg(feature = "messages")]
pub mod messages;

#[cfg(feature = "tauri-plugin")]
pub mod tauri_plugin;

pub mod utils;

/// A simple configuration for initializing the library.
#[derive(Debug, Default, Clone)]
pub struct NostrConfig {
    pub name: String,
}

/// The main entry point for initializing the library components.
pub struct NostrNative {
    pub config: NostrConfig,
}

impl NostrNative {
    /// Creates a new builder for NostrNative.
    pub fn builder() -> NostrNativeBuilder {
        NostrNativeBuilder::default()
    }
}

#[derive(Default)]
pub struct NostrNativeBuilder {
    config: NostrConfig,
}

impl NostrNativeBuilder {
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.config.name = name.into();
        self
    }

    pub fn build(self) -> NostrNative {
        NostrNative {
            config: self.config,
        }
    }
}
