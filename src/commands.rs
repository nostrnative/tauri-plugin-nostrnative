#[cfg(feature = "keys")]
use crate::keys;

#[cfg(feature = "calendar")]
use crate::calendar;

#[cfg(feature = "profile")]
use crate::profile;

#[cfg(feature = "messages")]
use crate::messages;

#[cfg(feature = "bookmarks")]
use crate::bookmarks;

#[cfg(feature = "notes")]
use crate::notes;

#[cfg(feature = "blossom")]
use crate::blossom;

#[cfg(feature = "chat")]
use crate::chat;

#[cfg(feature = "nfp")]
use crate::nfp;

#[cfg(feature = "virtual-fs")]
use crate::virtual_fs;

#[cfg(feature = "bookmarks")]
use crate::models::{BookmarkSource, EventResponse};

#[cfg(feature = "calendar")]
use crate::models::{CalendarEventRequest, CalendarRequest};

#[cfg(feature = "chat")]
use crate::models::{ChatMessageRequest, PnsDerivedKeys};

#[cfg(feature = "profile")]
use crate::models::{Contact, UserProfile};

#[cfg(feature = "nfp")]
use crate::models::{NFPCycleRequest, NFPObservationRequest, NFPSettingsRequest};

#[cfg(feature = "virtual-fs")]
use crate::models::FilesystemRequest;

#[tauri::command]
#[cfg(feature = "keys")]
pub async fn generate_new_nsec() -> String {
    keys::generate_new_nsec()
}

#[tauri::command]
#[cfg(feature = "keys")]
pub async fn parse_pubkey(pubkey: String) -> Result<String, String> {
    keys::parse_pubkey(&pubkey)
}

#[tauri::command]
#[cfg(feature = "keys")]
pub async fn verify_nsec(nsec: String) -> Result<String, String> {
    keys::verify_nsec(&nsec).await
}

#[tauri::command]
#[cfg(feature = "calendar")]
pub async fn fetch_calendar_events(
    pubkey: String,
    nsec: Option<String>,
    relays: Vec<String>,
    range_start: Option<i64>,
    range_end: Option<i64>,
    authors: Option<Vec<String>>,
    client_nsec: Option<String>,
) -> Result<Vec<EventResponse>, String> {
    calendar::fetch_calendar_events_core(
        &pubkey,
        nsec.as_deref(),
        &relays,
        range_start,
        range_end,
        authors,
        client_nsec.as_deref(),
    )
    .await
}

#[tauri::command]
#[cfg(feature = "calendar")]
pub async fn publish_batch_calendar_events(
    nsec: String,
    relays: Vec<String>,
    events: Vec<CalendarEventRequest>,
    client_nsec: Option<String>,
) -> Result<Vec<String>, String> {
    calendar::publish_batch_calendar_events_core(&nsec, &relays, events, client_nsec.as_deref())
        .await
}

#[tauri::command]
#[cfg(feature = "calendar")]
pub async fn publish_calendar_event(
    nsec: String,
    relays: Vec<String>,
    event_data: CalendarEventRequest,
    client_nsec: Option<String>,
) -> Result<String, String> {
    calendar::publish_batch_calendar_events_core(
        &nsec,
        &relays,
        vec![event_data],
        client_nsec.as_deref(),
    )
    .await
    .map(|v| v.first().cloned().unwrap_or_default())
}

#[tauri::command]
#[cfg(feature = "calendar")]
pub async fn delete_calendar_event(
    nsec: String,
    relays: Vec<String>,
    event_ids: Vec<String>,
    client_nsec: Option<String>,
) -> Result<String, String> {
    calendar::delete_calendar_event_core(&nsec, &relays, event_ids, client_nsec.as_deref()).await
}

#[tauri::command]
#[cfg(feature = "calendar")]
pub async fn fetch_calendars(
    pubkey: String,
    relays: Vec<String>,
) -> Result<Vec<EventResponse>, String> {
    calendar::fetch_calendars_core(&pubkey, &relays).await
}

#[tauri::command]
#[cfg(feature = "calendar")]
pub async fn publish_calendar(
    nsec: String,
    relays: Vec<String>,
    calendar: CalendarRequest,
    client_nsec: Option<String>,
) -> Result<String, String> {
    calendar::publish_calendar_core(&nsec, &relays, calendar, client_nsec.as_deref()).await
}

#[tauri::command]
#[cfg(feature = "calendar")]
pub async fn delete_calendar(
    nsec: String,
    relays: Vec<String>,
    identifier: String,
    client_nsec: Option<String>,
) -> Result<String, String> {
    calendar::delete_calendar_core(&nsec, &relays, identifier, client_nsec.as_deref()).await
}

#[tauri::command]
#[cfg(feature = "profile")]
pub async fn fetch_profiles(
    pubkeys: Vec<String>,
    relays: Vec<String>,
) -> Result<Vec<UserProfile>, String> {
    profile::fetch_profiles_core(pubkeys, &relays).await
}

#[tauri::command]
#[cfg(feature = "profile")]
pub async fn fetch_contact_list(
    pubkey: String,
    relays: Vec<String>,
) -> Result<Vec<Contact>, String> {
    profile::fetch_contact_list_core(&pubkey, &relays).await
}

#[tauri::command]
#[cfg(feature = "profile")]
pub async fn update_contact_list(
    nsec: String,
    relays: Vec<String>,
    contacts: Vec<Contact>,
    client_nsec: Option<String>,
) -> Result<String, String> {
    profile::update_contact_list_core(&nsec, &relays, contacts, client_nsec.as_deref()).await
}

#[tauri::command]
#[cfg(feature = "calendar")]
pub async fn fetch_rsvps(
    event_coordinate: String,
    relays: Vec<String>,
) -> Result<Vec<EventResponse>, String> {
    calendar::rsvp::fetch_rsvps_core(&event_coordinate, &relays).await
}

#[tauri::command]
#[cfg(feature = "calendar")]
pub async fn fetch_user_rsvps(
    pubkey: String,
    relays: Vec<String>,
) -> Result<Vec<EventResponse>, String> {
    calendar::rsvp::fetch_user_rsvps_core(&pubkey, &relays).await
}

#[tauri::command]
#[cfg(feature = "calendar")]
pub async fn fetch_received_rsvps(
    pubkey: String,
    relays: Vec<String>,
) -> Result<Vec<EventResponse>, String> {
    calendar::rsvp::fetch_received_rsvps_core(&pubkey, &relays).await
}

#[tauri::command]
#[cfg(feature = "calendar")]
pub async fn publish_rsvp(
    nsec: String,
    relays: Vec<String>,
    event_coordinate: String,
    status: String,
    event_author: Option<String>,
    client_nsec: Option<String>,
) -> Result<String, String> {
    calendar::rsvp::publish_rsvp_core(
        &nsec,
        &relays,
        event_coordinate,
        status,
        event_author,
        client_nsec.as_deref(),
    )
    .await
}

#[tauri::command]
#[cfg(feature = "messages")]
pub async fn send_direct_message(
    nsec: String,
    receiver_pubkey: String,
    message: String,
    relays: Vec<String>,
    client_nsec: Option<String>,
) -> Result<String, String> {
    messages::send_direct_message_core(
        &nsec,
        &receiver_pubkey,
        &message,
        &relays,
        client_nsec.as_deref(),
    )
    .await
}

#[tauri::command]
#[cfg(feature = "bookmarks")]
pub async fn fetch_bookmarks(
    pubkey: String,
    nsec: Option<String>,
    relays: Vec<String>,
    client_nsec: Option<String>,
) -> Result<Vec<EventResponse>, String> {
    bookmarks::fetch_bookmarks_core(&pubkey, nsec.as_deref(), &relays, client_nsec.as_deref()).await
}

#[tauri::command]
#[cfg(feature = "bookmarks")]
pub async fn fetch_event_details(
    identifiers: Vec<String>,
    relays: Vec<String>,
) -> Result<Vec<EventResponse>, String> {
    bookmarks::fetch_event_details_core(identifiers, &relays).await
}

#[tauri::command]
#[cfg(feature = "bookmarks")]
pub async fn save_bookmark(
    nsec: String,
    relays: Vec<String>,
    event_id: String,
    source: BookmarkSource,
    client_nsec: Option<String>,
) -> Result<String, String> {
    bookmarks::save_bookmark_core(&nsec, &relays, event_id, source, client_nsec.as_deref()).await
}

#[tauri::command]
#[cfg(feature = "bookmarks")]
pub async fn remove_bookmark(
    nsec: String,
    relays: Vec<String>,
    event_id: String,
    source: BookmarkSource,
    client_nsec: Option<String>,
) -> Result<String, String> {
    bookmarks::remove_bookmark_core(&nsec, &relays, event_id, source, client_nsec.as_deref()).await
}

#[tauri::command]
#[cfg(feature = "notes")]
pub async fn create_note(
    nsec: String,
    relays: Vec<String>,
    note: notes::NoteRequest,
    client_nsec: Option<String>,
) -> Result<String, String> {
    notes::create_note_core(&nsec, &relays, note, client_nsec.as_deref()).await
}

#[tauri::command]
#[cfg(feature = "notes")]
pub async fn update_note(
    nsec: String,
    relays: Vec<String>,
    identifier: String,
    note: notes::NoteRequest,
    client_nsec: Option<String>,
) -> Result<String, String> {
    notes::update_note_core(&nsec, &relays, identifier, note, client_nsec.as_deref()).await
}

#[tauri::command]
#[cfg(feature = "notes")]
pub async fn delete_note(
    nsec: String,
    relays: Vec<String>,
    note_id: String,
    client_nsec: Option<String>,
) -> Result<String, String> {
    notes::delete_note_core(&nsec, &relays, note_id, client_nsec.as_deref()).await
}

#[tauri::command]
#[cfg(feature = "notes")]
pub async fn fetch_notes(
    pubkey: String,
    relays: Vec<String>,
    nsec: Option<String>,
    note_type: Option<String>,
    client_nsec: Option<String>,
) -> Result<Vec<EventResponse>, String> {
    notes::fetch_notes_core(
        &pubkey,
        nsec.as_deref(),
        &relays,
        note_type,
        client_nsec.as_deref(),
    )
    .await
}

#[tauri::command]
#[cfg(feature = "notes")]
pub async fn create_notebook(
    nsec: String,
    relays: Vec<String>,
    notebook: notes::NotebookRequest,
    client_nsec: Option<String>,
) -> Result<String, String> {
    notes::create_notebook_core(&nsec, &relays, notebook, client_nsec.as_deref()).await
}

#[tauri::command]
#[cfg(feature = "notes")]
pub async fn fetch_notebooks(
    pubkey: String,
    relays: Vec<String>,
    nsec: Option<String>,
    client_nsec: Option<String>,
) -> Result<Vec<EventResponse>, String> {
    notes::fetch_notebooks_core(&pubkey, &relays, nsec.as_deref(), client_nsec.as_deref()).await
}

#[tauri::command]
#[cfg(feature = "notes")]
pub async fn add_note_to_notebook(
    nsec: String,
    relays: Vec<String>,
    notebook_id: String,
    note_id: String,
    client_nsec: Option<String>,
) -> Result<String, String> {
    notes::add_note_to_notebook_core(&nsec, &relays, notebook_id, note_id, client_nsec.as_deref())
        .await
}

#[tauri::command]
#[cfg(feature = "notes")]
pub async fn remove_note_from_notebook(
    nsec: String,
    relays: Vec<String>,
    notebook_id: String,
    note_id: String,
    client_nsec: Option<String>,
) -> Result<String, String> {
    notes::remove_note_from_notebook_core(
        &nsec,
        &relays,
        notebook_id,
        note_id,
        client_nsec.as_deref(),
    )
    .await
}

#[tauri::command]
#[cfg(feature = "notes")]
pub async fn delete_notebook(
    nsec: String,
    relays: Vec<String>,
    notebook_id: String,
    client_nsec: Option<String>,
) -> Result<String, String> {
    notes::delete_notebook_core(&nsec, &relays, notebook_id, client_nsec.as_deref()).await
}

#[tauri::command]
#[cfg(feature = "blossom")]
pub async fn blossom_mirror(
    server_url: String,
    nsec: String,
    url: String,
    client_nsec: Option<String>,
) -> Result<String, String> {
    blossom::blossom_mirror_core(&server_url, &nsec, &url, client_nsec.as_deref()).await
}

#[tauri::command]
#[cfg(feature = "blossom")]
pub async fn blossom_upload(
    server_url: String,
    nsec: String,
    file_path: String,
    client_nsec: Option<String>,
) -> Result<String, String> {
    blossom::blossom_upload_core(&server_url, &nsec, &file_path, client_nsec.as_deref()).await
}

#[tauri::command]
#[cfg(feature = "blossom")]
pub async fn blossom_upload_content(
    server_url: String,
    nsec: String,
    content: String,
    filename: Option<String>,
    media_type: Option<String>,
    client_nsec: Option<String>,
) -> Result<String, String> {
    blossom::blossom_upload_content_core(
        &server_url,
        &nsec,
        content,
        filename,
        media_type,
        client_nsec.as_deref(),
    )
    .await
}

#[tauri::command]
#[cfg(feature = "blossom")]
pub async fn blossom_get_blob(server_url: String, hash: String) -> Result<Vec<u8>, String> {
    blossom::blossom_get_blob_core(&server_url, &hash).await
}

#[tauri::command]
#[cfg(feature = "blossom")]
pub async fn blossom_delete(
    server_url: String,
    nsec: String,
    hash: String,
    client_nsec: Option<String>,
) -> Result<(), String> {
    blossom::blossom_delete_core(&server_url, &nsec, &hash, client_nsec.as_deref()).await
}

#[tauri::command]
#[cfg(feature = "blossom")]
pub async fn blossom_list(
    server_url: String,
    pubkey: String,
    nsec: Option<String>,
    client_nsec: Option<String>,
) -> Result<serde_json::Value, String> {
    blossom::blossom_list_core(
        &server_url,
        &pubkey,
        nsec.as_deref(),
        client_nsec.as_deref(),
    )
    .await
}

#[tauri::command]
#[cfg(feature = "chat")]
pub async fn derive_pns_keys(device_key_hex: String) -> Result<PnsDerivedKeys, String> {
    chat::derive_pns_keys_core(&device_key_hex)
}

#[tauri::command]
#[cfg(feature = "chat")]
pub async fn create_discovery_event(
    nsec: String,
    device_key_to_store: String,
    relays: Vec<String>,
    client_nsec: Option<String>,
) -> Result<String, String> {
    chat::create_discovery_event_core(&nsec, device_key_to_store, &relays, client_nsec.as_deref())
        .await
}

#[tauri::command]
#[cfg(feature = "chat")]
pub async fn decrypt_discovery_event(
    nsec: String,
    content: String,
    client_nsec: Option<String>,
) -> Result<String, String> {
    chat::decrypt_discovery_event_core(&nsec, &content, client_nsec.as_deref()).await
}

#[tauri::command]
#[cfg(feature = "chat")]
pub async fn publish_chat_message(
    pns_nsec: String,
    main_pubkey: String,
    msg: ChatMessageRequest,
    relays: Vec<String>,
    client_nsec: Option<String>,
) -> Result<String, String> {
    chat::publish_chat_message_core(
        &pns_nsec,
        &main_pubkey,
        msg,
        &relays,
        client_nsec.as_deref(),
    )
    .await
}

#[tauri::command]
#[cfg(feature = "chat")]
pub async fn decrypt_pns_event(
    pns_nsec: String,
    content: String,
    client_nsec: Option<String>,
) -> Result<serde_json::Value, String> {
    chat::decrypt_pns_event_core(&pns_nsec, &content, client_nsec.as_deref()).await
}

#[tauri::command]
#[cfg(feature = "chat")]
pub async fn fetch_pns_events(
    pns_pubkey: String,
    event_id: Option<String>,
    relays: Vec<String>,
) -> Result<Vec<EventResponse>, String> {
    chat::fetch_pns_events_core(&pns_pubkey, event_id, &relays).await
}

#[tauri::command]
#[cfg(feature = "chat")]
pub async fn delete_chat_messages(
    pns_nsec: String,
    relays: Vec<String>,
    event_ids: Vec<String>,
    client_nsec: Option<String>,
) -> Result<String, String> {
    chat::delete_chat_messages_core(&pns_nsec, &relays, event_ids, client_nsec.as_deref()).await
}

#[tauri::command]
#[cfg(feature = "chat")]
pub async fn fetch_discovery_events(
    pubkey: String,
    relays: Vec<String>,
) -> Result<Vec<EventResponse>, String> {
    chat::fetch_discovery_events_core(&pubkey, &relays).await
}

#[tauri::command]
#[cfg(feature = "nfp")]
pub async fn fetch_nfp_observations(
    pubkey: String,
    nsec: Option<String>,
    relays: Vec<String>,
    range_start: Option<i64>,
    range_end: Option<i64>,
    client_nsec: Option<String>,
) -> Result<Vec<EventResponse>, String> {
    nfp::fetch_nfp_observations_core(
        &pubkey,
        nsec.as_deref(),
        &relays,
        range_start,
        range_end,
        client_nsec.as_deref(),
    )
    .await
}

#[tauri::command]
#[cfg(feature = "nfp")]
pub async fn publish_nfp_observation(
    nsec: String,
    relays: Vec<String>,
    observation: NFPObservationRequest,
    client_nsec: Option<String>,
) -> Result<String, String> {
    nfp::publish_nfp_observation_core(&nsec, &relays, observation, client_nsec.as_deref()).await
}

#[tauri::command]
#[cfg(feature = "nfp")]
pub async fn update_nfp_observation(
    nsec: String,
    relays: Vec<String>,
    event_id: String,
    observation: NFPObservationRequest,
    client_nsec: Option<String>,
) -> Result<String, String> {
    nfp::update_nfp_observation_core(
        &nsec,
        &relays,
        &event_id,
        observation,
        client_nsec.as_deref(),
    )
    .await
}

#[tauri::command]
#[cfg(feature = "nfp")]
pub async fn fetch_nfp_cycles(
    pubkey: String,
    nsec: Option<String>,
    relays: Vec<String>,
    client_nsec: Option<String>,
) -> Result<Vec<EventResponse>, String> {
    nfp::fetch_nfp_cycles_core(&pubkey, nsec.as_deref(), &relays, client_nsec.as_deref()).await
}

#[tauri::command]
#[cfg(feature = "nfp")]
pub async fn publish_nfp_cycle(
    nsec: String,
    relays: Vec<String>,
    cycle: NFPCycleRequest,
    client_nsec: Option<String>,
) -> Result<String, String> {
    nfp::publish_nfp_cycle_core(&nsec, &relays, cycle, client_nsec.as_deref()).await
}

#[tauri::command]
#[cfg(feature = "nfp")]
pub async fn fetch_nfp_settings(
    pubkey: String,
    nsec: Option<String>,
    relays: Vec<String>,
    client_nsec: Option<String>,
) -> Result<Option<EventResponse>, String> {
    nfp::fetch_nfp_settings_core(&pubkey, nsec.as_deref(), &relays, client_nsec.as_deref()).await
}

#[tauri::command]
#[cfg(feature = "nfp")]
pub async fn publish_nfp_settings(
    nsec: String,
    relays: Vec<String>,
    settings: NFPSettingsRequest,
    client_nsec: Option<String>,
) -> Result<String, String> {
    nfp::publish_nfp_settings_core(&nsec, &relays, settings, client_nsec.as_deref()).await
}

#[tauri::command]
#[cfg(feature = "nfp")]
pub async fn delete_nfp_data(
    nsec: String,
    relays: Vec<String>,
    event_ids: Vec<String>,
    client_nsec: Option<String>,
) -> Result<String, String> {
    nfp::delete_nfp_data_core(&nsec, &relays, event_ids, client_nsec.as_deref()).await
}

#[tauri::command]
#[cfg(feature = "nfp")]
pub async fn update_nfp_cycle(
    nsec: String,
    relays: Vec<String>,
    event_id: String,
    cycle: NFPCycleRequest,
    client_nsec: Option<String>,
) -> Result<String, String> {
    nfp::update_nfp_cycle_core(&nsec, &relays, &event_id, cycle, client_nsec.as_deref()).await
}

#[tauri::command]
#[cfg(feature = "nfp")]
pub async fn update_nfp_settings(
    nsec: String,
    relays: Vec<String>,
    event_id: String,
    settings: NFPSettingsRequest,
    client_nsec: Option<String>,
) -> Result<String, String> {
    nfp::update_nfp_settings_core(&nsec, &relays, &event_id, settings, client_nsec.as_deref()).await
}

#[tauri::command]
#[cfg(feature = "virtual-fs")]
pub async fn publish_virtual_fs(
    nsec: String,
    relays: Vec<String>,
    req: FilesystemRequest,
    client_nsec: Option<String>,
) -> Result<String, String> {
    virtual_fs::publish_filesystem(&nsec, &relays, req, client_nsec.as_deref()).await
}

#[tauri::command]
#[cfg(feature = "virtual-fs")]
pub async fn fetch_virtual_fs(
    pubkey: String,
    relays: Vec<String>,
) -> Result<Vec<EventResponse>, String> {
    virtual_fs::fetch_filesystems(&pubkey, &relays).await
}

#[tauri::command]
#[cfg(feature = "virtual-fs")]
pub async fn delete_virtual_fs(
    nsec: String,
    relays: Vec<String>,
    identifier: String,
    client_nsec: Option<String>,
) -> Result<String, String> {
    virtual_fs::delete_virtual_fs(&nsec, &relays, identifier, client_nsec.as_deref()).await
}

#[cfg(feature = "relay")]
use crate::relay;

#[tauri::command]
#[cfg(feature = "relay")]
pub async fn start_nostr_relay(port: u16, db_path: String) -> Result<(), String> {
    relay::start_relay_core(port, &db_path).await
}

#[tauri::command]
#[cfg(feature = "relay")]
pub async fn stop_nostr_relay() -> Result<(), String> {
    relay::stop_relay_core().await
}

#[tauri::command]
#[cfg(feature = "relay")]
pub async fn get_nostr_relay_status() -> Result<bool, String> {
    relay::get_relay_status_core().await
}

#[tauri::command]
#[cfg(feature = "relay")]
pub async fn get_nostr_event_counts() -> Result<std::collections::HashMap<u64, usize>, String> {
    relay::get_event_counts_core().await
}
