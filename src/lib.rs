//! tauri-plugin-nostrnative - A modular Nostr library.
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

#[cfg(feature = "bookmarks")]
pub mod bookmarks;

#[cfg(feature = "notes")]
pub mod notes;

#[cfg(feature = "blossom")]
pub mod blossom;

#[cfg(feature = "chat")]
pub mod chat;

#[cfg(feature = "nfp")]
pub mod nfp;

#[cfg(feature = "virtual-fs")]
pub mod virtual_fs;

#[cfg(feature = "relay")]
pub mod relay;

pub mod routstr;

#[cfg(feature = "tauri-plugin")]
mod commands;

pub mod utils;
pub use utils::get_client_with_signer;

#[cfg(feature = "tauri-plugin")]
use tauri::{
    Runtime,
    plugin::{Builder, TauriPlugin},
};

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

#[cfg(feature = "tauri-plugin")]
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("nostrnative")
        .invoke_handler(tauri::generate_handler![
            commands::generate_new_nsec,
            commands::parse_pubkey,
            commands::verify_nsec,
            #[cfg(feature = "calendar")]
            commands::fetch_calendar_events,
            #[cfg(feature = "calendar")]
            commands::publish_batch_calendar_events,
            #[cfg(feature = "calendar")]
            commands::publish_calendar_event,
            #[cfg(feature = "calendar")]
            commands::delete_calendar_event,
            #[cfg(feature = "calendar")]
            commands::fetch_calendars,
            #[cfg(feature = "calendar")]
            commands::publish_calendar,
            #[cfg(feature = "calendar")]
            commands::delete_calendar,
            #[cfg(feature = "calendar")]
            commands::fetch_rsvps,
            #[cfg(feature = "calendar")]
            commands::fetch_user_rsvps,
            #[cfg(feature = "calendar")]
            commands::fetch_received_rsvps,
            #[cfg(feature = "calendar")]
            commands::publish_rsvp,
            #[cfg(feature = "profile")]
            commands::fetch_profiles,
            #[cfg(feature = "profile")]
            commands::fetch_contact_list,
            #[cfg(feature = "profile")]
            commands::update_contact_list,
            #[cfg(feature = "messages")]
            commands::send_direct_message,
            #[cfg(feature = "bookmarks")]
            commands::fetch_bookmarks,
            #[cfg(feature = "bookmarks")]
            commands::fetch_event_details,
            #[cfg(feature = "bookmarks")]
            commands::save_bookmark,
            #[cfg(feature = "bookmarks")]
            commands::remove_bookmark,
            #[cfg(feature = "notes")]
            commands::create_note,
            #[cfg(feature = "notes")]
            commands::update_note,
            #[cfg(feature = "notes")]
            commands::delete_note,
            #[cfg(feature = "notes")]
            commands::fetch_notes,
            #[cfg(feature = "notes")]
            commands::create_notebook,
            #[cfg(feature = "notes")]
            commands::fetch_notebooks,
            #[cfg(feature = "notes")]
            commands::add_note_to_notebook,
            #[cfg(feature = "notes")]
            commands::remove_note_from_notebook,
            #[cfg(feature = "notes")]
            commands::delete_notebook,
            #[cfg(feature = "blossom")]
            commands::blossom_mirror,
            #[cfg(feature = "blossom")]
            commands::blossom_upload,
            #[cfg(feature = "blossom")]
            commands::blossom_upload_content,
            #[cfg(feature = "blossom")]
            commands::blossom_get_blob,
            #[cfg(feature = "blossom")]
            commands::blossom_delete,
            #[cfg(feature = "blossom")]
            commands::blossom_list,
            #[cfg(feature = "chat")]
            commands::derive_pns_keys,
            #[cfg(feature = "chat")]
            commands::create_discovery_event,
            #[cfg(feature = "chat")]
            commands::decrypt_discovery_event,
            #[cfg(feature = "chat")]
            commands::publish_chat_message,
            #[cfg(feature = "chat")]
            commands::decrypt_pns_event,
            #[cfg(feature = "chat")]
            commands::fetch_pns_events,
            #[cfg(feature = "chat")]
            commands::delete_chat_messages,
            #[cfg(feature = "chat")]
            commands::fetch_discovery_events,
            #[cfg(feature = "nfp")]
            commands::fetch_nfp_observations,
            #[cfg(feature = "nfp")]
            commands::publish_nfp_observation,
            #[cfg(feature = "nfp")]
            commands::update_nfp_observation,
            #[cfg(feature = "nfp")]
            commands::fetch_nfp_cycles,
            #[cfg(feature = "nfp")]
            commands::publish_nfp_cycle,
            #[cfg(feature = "nfp")]
            commands::update_nfp_cycle,
            #[cfg(feature = "nfp")]
            commands::fetch_nfp_settings,
            #[cfg(feature = "nfp")]
            commands::publish_nfp_settings,
            #[cfg(feature = "nfp")]
            commands::update_nfp_settings,
            #[cfg(feature = "nfp")]
            commands::delete_nfp_data,
            #[cfg(feature = "virtual-fs")]
            commands::publish_virtual_fs,
            #[cfg(feature = "virtual-fs")]
            commands::fetch_virtual_fs,
            #[cfg(feature = "virtual-fs")]
            commands::delete_virtual_fs,
            #[cfg(feature = "relay")]
            commands::start_nostr_relay,
            #[cfg(feature = "relay")]
            commands::stop_nostr_relay,
            #[cfg(feature = "relay")]
            commands::get_nostr_relay_status,
            #[cfg(feature = "relay")]
            commands::get_nostr_event_counts,
            #[cfg(feature = "relay")]
            commands::get_nostr_relay_events,
            commands::discover_routstr_providers,
            commands::compare_routstr_prices,
            commands::fetch_routstr_models,
        ])
        .build()
}
