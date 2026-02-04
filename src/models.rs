use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Clone)]
pub struct CalendarEventRequest {
    pub title: String,
    pub description: Option<String>,
    pub start: i64,
    pub end: Option<i64>,
    pub location: Option<String>,
    pub is_all_day: bool,
    pub identifier: String, // d-tag
    pub reminder_minutes: Option<i64>,
    pub old_event_id: Option<String>,
    pub calendar_id: Option<String>,
    pub color: Option<String>,
    pub p_tags: Option<Vec<String>>,
    pub is_private: Option<bool>,
    pub parent: Option<String>,
    pub freq: Option<String>,
    pub until: Option<i64>,
    pub use_different_timestamp: Option<bool>,
    pub start_tzid: Option<String>,
    pub end_tzid: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct CalendarRequest {
    pub name: String,
    pub description: Option<String>,
    pub identifier: String, // d-tag
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Contact {
    pub pubkey: String,
    pub npub: Option<String>,
    pub alias: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserProfile {
    pub pubkey: String,
    pub npub: Option<String>,
    pub name: Option<String>,
    pub display_name: Option<String>,
    pub about: Option<String>,
    pub picture: Option<String>,
    pub banner: Option<String>,
    pub website: Option<String>,
    pub nip05: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum BookmarkSource {
    Public,
    Private,
    Website,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EventResponse {
    pub id: String,
    pub pubkey: String,
    pub created_at: i64,
    pub kind: u16,
    pub tags: Vec<Vec<String>>,
    pub content: String,
    pub is_private: bool,
    pub bech32: Option<String>,
    pub private_tags: Option<Vec<Vec<String>>>,
    pub source: Option<BookmarkSource>,
}

impl From<nostr_sdk::Event> for EventResponse {
    fn from(event: nostr_sdk::Event) -> Self {
        Self {
            id: event.id.to_hex(),
            pubkey: event.pubkey.to_hex(),
            created_at: event.created_at.as_secs() as i64,
            kind: event.kind.as_u16(),
            tags: event.tags.into_iter().map(|t| t.to_vec()).collect(),
            content: event.content.to_string(),
            is_private: false, // Default to false
            bech32: None,
            private_tags: None,
            source: None,
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct ChatMessageRequest {
    pub content: String,
    pub conversation_id: String, // d-tag
    pub role: String,            // user/assistant
    pub client: String,
    pub reply_to: Option<String>, // e-tag
    pub event_id: Option<String>,
    pub context_ids: Option<Vec<String>>,
    pub blob_hashes: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PnsDerivedKeys {
    pub public_key: String,
    pub secret_key: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NFPObservationRequest {
    pub date: String,       // ISO date string (YYYY-MM-DD)
    pub identifier: String, // d-tag
    pub temperature: Option<NFPTemperature>,
    pub cervical_fluid: Option<NFPCervicalFluid>,
    pub cervix_position: Option<NFPCervixPosition>,
    pub bleeding: Option<NFPBleeding>,
    pub notes: Option<String>,
    pub disturbances: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NFPTemperature {
    pub value: f64,     // in Celsius
    pub time: String,   // time taken (HH:MM)
    pub method: String, // oral, vaginal, rectal
    pub disturbance: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NFPCervicalFluid {
    pub sensation: String,          // dry, moist, wet, slippery
    pub appearance: Option<String>, // none, sticky, creamy, egg-white, unusual
    pub amount: Option<String>,     // none, little, moderate, much
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NFPCervixPosition {
    pub height: String,   // low, medium, high
    pub firmness: String, // firm, medium, soft
    pub opening: String,  // closed, slightly-open, open
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NFPBleeding {
    pub flow: String,          // spotting, light, medium, heavy
    pub color: Option<String>, // brown, red, pink
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NFPCycleRequest {
    pub identifier: String,       // d-tag - cycle ID
    pub start_date: String,       // first day of menstruation
    pub end_date: Option<String>, // last day before next cycle starts
    pub length: Option<i64>,      // in days
    pub fertility_window_start: Option<String>,
    pub fertility_window_end: Option<String>,
    pub peak_day: Option<String>,
    pub ovulation_day: Option<String>,
    pub luteal_phase_length: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NFPSettingsRequest {
    pub identifier: String,            // d-tag - always "user-settings"
    pub temperature_unit: String,      // celsius, fahrenheit
    pub temperature_method: String,    // oral, vaginal, rectal
    pub reminder_time: Option<String>, // HH:MM
    pub average_cycle_length: i64,
    pub luteal_phase_length: Option<i64>,
    pub require_pin_on_open: bool,
    pub auto_lock_after_minutes: Option<i64>,
    pub sync_enabled: bool,
    pub encrypted_backup: bool,
    pub theme: String,       // light, dark, system
    pub chart_style: String, // classic, modern
    pub show_predictions: bool,
    pub temperature_reminder: bool,
    pub ovulation_prediction: bool,
    pub cycle_end_prediction: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VirtualFilesystem {
    pub root: VirtualNode,
    pub metadata: FilesystemMetadata,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FilesystemMetadata {
    pub name: String,
    pub version: String,
    pub max_depth: u8,
    pub total_nodes: usize,
    pub total_size: u64,
    pub created_at: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum VirtualNode {
    File {
        name: String,
        depth: u8,
        metadata: FileNodeMetadata,
        content_ref: Option<ContentReference>,
    },
    Directory {
        name: String,
        depth: u8,
        metadata: DirectoryNodeMetadata,
        children: Vec<VirtualNode>,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileNodeMetadata {
    pub created: i64,
    pub modified: i64,
    pub permissions: String,
    pub size: u64,
    pub mime_type: String,
    pub content_hash: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DirectoryNodeMetadata {
    pub created: i64,
    pub modified: i64,
    pub permissions: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ContentReference {
    pub source: String, // e.g., "nostr_event", "blossom"
    pub kind: Option<u16>,
    pub id: String, // event id or blossom hash
    pub url: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct FilesystemRequest {
    pub identifier: String, // d-tag
    pub name: String,
    pub filesystem: VirtualFilesystem,
}
