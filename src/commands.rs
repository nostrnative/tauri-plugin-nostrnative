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

#[cfg(feature = "blossom")]
use crate::blossom;

#[cfg(feature = "chat")]
use crate::chat;

use crate::models::{
    BookmarkSource, CalendarEventRequest, CalendarRequest, ChatMessageRequest, Contact,
    EventResponse, PnsDerivedKeys, UserProfile,
};

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
