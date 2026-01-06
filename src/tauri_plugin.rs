use tauri::{
    plugin::{Builder, TauriPlugin},
    Runtime,
};

#[cfg(feature = "keys")]
use crate::keys;

#[cfg(feature = "calendar")]
use crate::calendar;

#[cfg(feature = "profile")]
use crate::profile;

#[cfg(feature = "messages")]
use crate::messages;

use crate::models::{CalendarEventRequest, CalendarRequest, Contact, EventResponse, UserProfile};

#[tauri::command]
#[cfg(feature = "keys")]
async fn generate_new_nsec() -> String {
    keys::generate_new_nsec()
}

#[tauri::command]
#[cfg(feature = "keys")]
async fn parse_pubkey(pubkey: String) -> Result<String, String> {
    keys::parse_pubkey(&pubkey)
}

#[tauri::command]
#[cfg(feature = "keys")]
async fn verify_nsec(nsec: String) -> Result<String, String> {
    keys::verify_nsec(&nsec)
}

#[tauri::command]
#[cfg(feature = "calendar")]
async fn fetch_calendar_events(
    pubkey: String,
    nsec: Option<String>,
    relays: Vec<String>,
    range_start: Option<i64>,
    range_end: Option<i64>,
    authors: Option<Vec<String>>,
) -> Result<Vec<EventResponse>, String> {
    calendar::fetch_calendar_events_core(
        &pubkey,
        nsec.as_deref(),
        &relays,
        range_start,
        range_end,
        authors,
    )
    .await
}

#[tauri::command]
#[cfg(feature = "calendar")]
async fn publish_batch_calendar_events(
    nsec: String,
    relays: Vec<String>,
    events: Vec<CalendarEventRequest>,
) -> Result<Vec<String>, String> {
    calendar::publish_batch_calendar_events_core(&nsec, &relays, events).await
}

#[tauri::command]
#[cfg(feature = "calendar")]
async fn publish_calendar_event(
    nsec: String,
    relays: Vec<String>,
    event_data: CalendarEventRequest,
) -> Result<String, String> {
    calendar::publish_batch_calendar_events_core(&nsec, &relays, vec![event_data])
        .await
        .map(|v| v.first().cloned().unwrap_or_default())
}

#[tauri::command]
#[cfg(feature = "calendar")]
async fn delete_calendar_event(
    nsec: String,
    relays: Vec<String>,
    event_ids: Vec<String>,
) -> Result<String, String> {
    calendar::delete_calendar_event_core(&nsec, &relays, event_ids).await
}

#[tauri::command]
#[cfg(feature = "calendar")]
async fn fetch_calendars(pubkey: String, relays: Vec<String>) -> Result<Vec<EventResponse>, String> {
    calendar::fetch_calendars_core(&pubkey, &relays).await
}

#[tauri::command]
#[cfg(feature = "calendar")]
async fn publish_calendar(
    nsec: String,
    relays: Vec<String>,
    calendar: CalendarRequest,
) -> Result<String, String> {
    calendar::publish_calendar_core(&nsec, &relays, calendar).await
}

#[tauri::command]
#[cfg(feature = "calendar")]
async fn delete_calendar(
    nsec: String,
    relays: Vec<String>,
    identifier: String,
) -> Result<String, String> {
    calendar::delete_calendar_core(&nsec, &relays, identifier).await
}

#[tauri::command]
#[cfg(feature = "profile")]
async fn fetch_profiles(pubkeys: Vec<String>, relays: Vec<String>) -> Result<Vec<UserProfile>, String> {
    profile::fetch_profiles_core(pubkeys, &relays).await
}

#[tauri::command]
#[cfg(feature = "profile")]
async fn fetch_contact_list(pubkey: String, relays: Vec<String>) -> Result<Vec<Contact>, String> {
    profile::fetch_contact_list_core(&pubkey, &relays).await
}

#[tauri::command]
#[cfg(feature = "profile")]
async fn update_contact_list(
    nsec: String,
    relays: Vec<String>,
    contacts: Vec<Contact>,
) -> Result<String, String> {
    profile::update_contact_list_core(&nsec, &relays, contacts).await
}

#[tauri::command]
#[cfg(feature = "calendar")]
async fn fetch_rsvps(event_coordinate: String, relays: Vec<String>) -> Result<Vec<EventResponse>, String> {
    calendar::rsvp::fetch_rsvps_core(&event_coordinate, &relays).await
}

#[tauri::command]
#[cfg(feature = "calendar")]
async fn fetch_user_rsvps(pubkey: String, relays: Vec<String>) -> Result<Vec<EventResponse>, String> {
    calendar::rsvp::fetch_user_rsvps_core(&pubkey, &relays).await
}

#[tauri::command]
#[cfg(feature = "calendar")]
async fn fetch_received_rsvps(pubkey: String, relays: Vec<String>) -> Result<Vec<EventResponse>, String> {
    calendar::rsvp::fetch_received_rsvps_core(&pubkey, &relays).await
}

#[tauri::command]
#[cfg(feature = "calendar")]
async fn publish_rsvp(
    nsec: String,
    relays: Vec<String>,
    event_coordinate: String,
    status: String,
    event_author: Option<String>,
) -> Result<String, String> {
    calendar::rsvp::publish_rsvp_core(&nsec, &relays, event_coordinate, status, event_author).await
}

#[tauri::command]
#[cfg(feature = "messages")]
async fn send_direct_message(
    nsec: String,
    receiver_pubkey: String,
    message: String,
    relays: Vec<String>,
) -> Result<String, String> {
    messages::send_direct_message_core(&nsec, &receiver_pubkey, &message, &relays).await
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("nostrnative")
        .invoke_handler(tauri::generate_handler![
            #[cfg(feature = "keys")]
            generate_new_nsec,
            #[cfg(feature = "keys")]
            parse_pubkey,
            #[cfg(feature = "keys")]
            verify_nsec,
            #[cfg(feature = "calendar")]
            fetch_calendar_events,
            #[cfg(feature = "calendar")]
            publish_batch_calendar_events,
            #[cfg(feature = "calendar")]
            publish_calendar_event,
            #[cfg(feature = "calendar")]
            delete_calendar_event,
            #[cfg(feature = "calendar")]
            fetch_calendars,
            #[cfg(feature = "calendar")]
            publish_calendar,
            #[cfg(feature = "calendar")]
            delete_calendar,
            #[cfg(feature = "profile")]
            fetch_profiles,
            #[cfg(feature = "profile")]
            fetch_contact_list,
            #[cfg(feature = "profile")]
            update_contact_list,
            #[cfg(feature = "calendar")]
            fetch_rsvps,
            #[cfg(feature = "calendar")]
            fetch_user_rsvps,
            #[cfg(feature = "calendar")]
            fetch_received_rsvps,
            #[cfg(feature = "calendar")]
            publish_rsvp,
            #[cfg(feature = "messages")]
            send_direct_message,
        ])
        .build()
}
