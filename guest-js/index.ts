import { invoke } from "@tauri-apps/api/core";

export interface CalendarEventRequest {
  title: string;
  description?: string;
  start: number;
  end?: number;
  location?: string;
  is_all_day: boolean;
  identifier: string; // d-tag
  reminder_minutes?: number;
  old_event_id?: string;
  calendar_id?: string;
  color?: string;
  p_tags?: string[];
  is_private?: boolean;
  parent?: string;
  freq?: string;
  until?: number;
  use_different_timestamp?: boolean;
}

export interface CalendarRequest {
  name: string;
  description?: string;
  identifier: string; // d-tag
}

export interface CalendarContactListRequest {
  identifier: string;
  contacts: string[];
  is_private: boolean;
}

export interface Contact {
  pubkey: string;
  npub?: string;
  alias?: string | null;
}

export interface UserProfile {
  pubkey: string;
  npub?: string;
  name?: string | null;
  display_name?: string | null;
  about?: string | null;
  picture?: string | null;
  banner?: string | null;
  website?: string | null;
  nip05?: string | null;
}

export type BookmarkSource = "public" | "private" | "website";

export interface NoteRequest {
  content: string;
  note_type: string;
  identifier?: string;
  group_id?: string;
}

export interface NotebookRequest {
  title: string;
  description?: string;
}

export interface EventResponse {
  id: string;
  pubkey: string;
  created_at: number;
  kind: number;
  tags: string[][];
  content: string;
  is_private: boolean;
  bech32?: string;
  private_tags?: string[][];
  source?: BookmarkSource;
}

export interface ChatMessageRequest {
  content: string;
  conversation_id: string; // d-tag
  role: string; // user/assistant
  client: string;
  reply_to?: string; // e-tag
  event_id?: string;
  context_ids?: string[];
  blob_hashes?: string[];
}

export interface PnsDerivedKeys {
  public_key: string;
  secret_key: string;
}

export interface RoutstrProvider {
  pubkey: string;
  name: string;
  about: string;
  urls: string[];
  mints: string[];
  version?: string | null;
  is_official: boolean;
  price_prompt?: number | null;
  price_completion?: number | null;
}

// Keys
export async function hexToNpub(hex: string): Promise<string> {
  return await invoke("plugin:nostrnative|hex_to_npub", { hex });
}

export async function npubToHex(npub: string): Promise<string> {
  return await invoke("plugin:nostrnative|npub_to_hex", { npub });
}

export async function generateNewNsec(): Promise<string> {
  return await invoke("plugin:nostrnative|generate_new_nsec");
}

export async function parsePubkey(pubkey: string): Promise<string> {
  return await invoke("plugin:nostrnative|parse_pubkey", { pubkey });
}

export async function verifyNsec(nsec: string): Promise<string> {
  return await invoke("plugin:nostrnative|verify_nsec", { nsec });
}

// Calendar
export async function fetchCalendarEvents(
  pubkey: string,
  relays: string[],
  options?: {
    nsec?: string;
    rangeStart?: number;
    rangeEnd?: number;
    authors?: string[];
    clientNsec?: string;
    filters?: unknown[];
  },
): Promise<EventResponse[]> {
  return await invoke("plugin:nostrnative|fetch_calendar_events", {
    pubkey,
    nsec: options?.nsec,
    relays,
    rangeStart: options?.rangeStart,
    rangeEnd: options?.rangeEnd,
    authors: options?.authors,
    clientNsec: options?.clientNsec,
    filters: options?.filters,
  });
}

export async function publishBatchCalendarEvents(
  nsec: string,
  relays: string[],
  events: CalendarEventRequest[],
  clientNsec?: string,
): Promise<string[]> {
  return await invoke("plugin:nostrnative|publish_batch_calendar_events", {
    nsec,
    relays,
    events,
    clientNsec,
  });
}

// Relay
export async function startNostrRelay(
  port: number,
  dbPath: string,
): Promise<void> {
  return await invoke("plugin:nostrnative|start_nostr_relay", { port, dbPath });
}

export async function stopNostrRelay(): Promise<void> {
  return await invoke("plugin:nostrnative|stop_nostr_relay");
}

export async function getNostrRelayStatus(): Promise<boolean> {
  return await invoke("plugin:nostrnative|get_nostr_relay_status");
}

export async function getNostrEventCounts(): Promise<Record<number, number>> {
  return await invoke("plugin:nostrnative|get_nostr_event_counts");
}

export async function getNostrRelayEvents(): Promise<EventResponse[]> {
  return await invoke("plugin:nostrnative|get_nostr_relay_events");
}

export async function publishCalendarEvent(
  nsec: string,
  relays: string[],
  eventData: CalendarEventRequest,
  clientNsec?: string,
): Promise<string> {
  return await invoke("plugin:nostrnative|publish_calendar_event", {
    nsec,
    relays,
    eventData,
    clientNsec,
  });
}

export async function deleteCalendarEvent(
  nsec: string,
  relays: string[],
  eventIds: string[],
  clientNsec?: string,
): Promise<string> {
  return await invoke("plugin:nostrnative|delete_calendar_event", {
    nsec,
    relays,
    eventIds,
    clientNsec,
  });
}

export async function fetchCalendars(
  pubkey: string,
  relays: string[],
): Promise<EventResponse[]> {
  return await invoke("plugin:nostrnative|fetch_calendars", {
    pubkey,
    relays,
  });
}

export async function publishCalendar(
  nsec: string,
  relays: string[],
  calendar: CalendarRequest,
  clientNsec?: string,
): Promise<string> {
  return await invoke("plugin:nostrnative|publish_calendar", {
    nsec,
    relays,
    calendar,
    clientNsec,
  });
}

export async function deleteCalendar(
  nsec: string,
  relays: string[],
  identifier: string,
  clientNsec?: string,
): Promise<string> {
  return await invoke("plugin:nostrnative|delete_calendar", {
    nsec,
    relays,
    identifier,
    clientNsec,
  });
}

export async function fetchCalendarContactLists(
  pubkey: string,
  relays: string[],
  options?: { nsec?: string; clientNsec?: string },
): Promise<EventResponse[]> {
  return await invoke("plugin:nostrnative|fetch_calendar_contact_lists", {
    pubkey,
    relays,
    nsec: options?.nsec,
    clientNsec: options?.clientNsec,
  });
}

export async function publishCalendarContactList(
  nsec: string,
  relays: string[],
  request: CalendarContactListRequest,
  clientNsec?: string,
): Promise<string> {
  return await invoke("plugin:nostrnative|publish_calendar_contact_list", {
    nsec,
    relays,
    request,
    clientNsec,
  });
}

// RSVP
export async function fetchRsvps(
  eventCoordinate: string,
  relays: string[],
): Promise<EventResponse[]> {
  return await invoke("plugin:nostrnative|fetch_rsvps", {
    eventCoordinate,
    relays,
  });
}

export async function fetchUserRsvps(
  pubkey: string,
  relays: string[],
): Promise<EventResponse[]> {
  return await invoke("plugin:nostrnative|fetch_user_rsvps", {
    pubkey,
    relays,
  });
}

export async function fetchReceivedRsvps(
  pubkey: string,
  relays: string[],
): Promise<EventResponse[]> {
  return await invoke("plugin:nostrnative|fetch_received_rsvps", {
    pubkey,
    relays,
  });
}

export async function publishRsvp(
  nsec: string,
  relays: string[],
  eventCoordinate: string,
  status: string,
  eventAuthor?: string,
  clientNsec?: string,
): Promise<string> {
  return await invoke("plugin:nostrnative|publish_rsvp", {
    nsec,
    relays,
    eventCoordinate,
    status,
    eventAuthor,
    clientNsec,
  });
}

// Profile
export async function fetchProfiles(
  pubkeys: string[],
  relays: string[],
): Promise<UserProfile[]> {
  return await invoke("plugin:nostrnative|fetch_profiles", { pubkeys, relays });
}

export async function fetchContactList(
  pubkey: string,
  relays: string[],
): Promise<Contact[]> {
  return await invoke("plugin:nostrnative|fetch_contact_list", {
    pubkey,
    relays,
  });
}

export async function updateContactList(
  nsec: string,
  relays: string[],
  contacts: Contact[],
  clientNsec?: string,
): Promise<string> {
  return await invoke("plugin:nostrnative|update_contact_list", {
    nsec,
    relays,
    contacts,
    clientNsec,
  });
}

// Messages
export async function sendDirectMessage(
  nsec: string,
  receiverPubkey: string,
  message: string,
  relays: string[],
  clientNsec?: string,
): Promise<string> {
  return await invoke("plugin:nostrnative|send_direct_message", {
    nsec,
    receiverPubkey,
    message,
    relays,
    clientNsec,
  });
}

// Bookmarks
export async function fetchBookmarks(
  pubkey: string,
  relays: string[],
  nsec?: string,
  clientNsec?: string,
): Promise<EventResponse[]> {
  return await invoke("plugin:nostrnative|fetch_bookmarks", {
    pubkey,
    nsec,
    relays,
    clientNsec,
  });
}

export async function fetchEventDetails(
  identifiers: string[],
  relays: string[],
): Promise<EventResponse[]> {
  return await invoke("plugin:nostrnative|fetch_event_details", {
    identifiers,
    relays,
  });
}

export async function saveBookmark(
  nsec: string,
  relays: string[],
  eventId: string,
  source: BookmarkSource,
  clientNsec?: string,
): Promise<string> {
  return await invoke("plugin:nostrnative|save_bookmark", {
    nsec,
    relays,
    eventId,
    source,
    clientNsec,
  });
}

export async function removeBookmark(
  nsec: string,
  relays: string[],
  eventId: string,
  source: BookmarkSource,
  clientNsec?: string,
): Promise<string> {
  return await invoke("plugin:nostrnative|remove_bookmark", {
    nsec,
    relays,
    eventId,
    source,
    clientNsec,
  });
}

// Notes
export async function createNote(
  nsec: string,
  relays: string[],
  note: NoteRequest,
  clientNsec?: string,
): Promise<string> {
  return await invoke("plugin:nostrnative|create_note", {
    nsec,
    relays,
    note,
    clientNsec,
  });
}

export async function updateNote(
  nsec: string,
  relays: string[],
  identifier: string,
  note: NoteRequest,
  clientNsec?: string,
): Promise<string> {
  return await invoke("plugin:nostrnative|update_note", {
    nsec,
    relays,
    identifier,
    note,
    clientNsec,
  });
}

export async function deleteNote(
  nsec: string,
  relays: string[],
  noteId: string,
  clientNsec?: string,
): Promise<string> {
  return await invoke("plugin:nostrnative|delete_note", {
    nsec,
    relays,
    noteId,
    clientNsec,
  });
}

export async function fetchNotes(
  pubkey: string,
  relays: string[],
  options?: { nsec?: string; noteType?: string; clientNsec?: string },
): Promise<EventResponse[]> {
  return await invoke("plugin:nostrnative|fetch_notes", {
    pubkey,
    relays,
    nsec: options?.nsec,
    note_type: options?.noteType,
    clientNsec: options?.clientNsec,
  });
}

export async function createNotebook(
  nsec: string,
  relays: string[],
  notebook: NotebookRequest,
  clientNsec?: string,
): Promise<string> {
  return await invoke("plugin:nostrnative|create_notebook", {
    nsec,
    relays,
    notebook,
    clientNsec,
  });
}

export async function fetchNotebooks(
  pubkey: string,
  relays: string[],
  options?: { nsec?: string; clientNsec?: string },
): Promise<EventResponse[]> {
  return await invoke("plugin:nostrnative|fetch_notebooks", {
    pubkey,
    relays,
    nsec: options?.nsec,
    clientNsec: options?.clientNsec,
  });
}

export async function addNoteToNotebook(
  nsec: string,
  relays: string[],
  notebookId: string,
  noteId: string,
  clientNsec?: string,
): Promise<string> {
  return await invoke("plugin:nostrnative|add_note_to_notebook", {
    nsec,
    relays,
    notebook_id: notebookId,
    note_id: noteId,
    clientNsec,
  });
}

export async function removeNoteFromNotebook(
  nsec: string,
  relays: string[],
  notebookId: string,
  noteId: string,
  clientNsec?: string,
): Promise<string> {
  return await invoke("plugin:nostrnative|remove_note_from_notebook", {
    nsec,
    relays,
    notebook_id: notebookId,
    note_id: noteId,
    clientNsec,
  });
}

export async function deleteNotebook(
  nsec: string,
  relays: string[],
  notebookId: string,
  clientNsec?: string,
): Promise<string> {
  return await invoke("plugin:nostrnative|delete_notebook", {
    nsec,
    relays,
    notebook_id: notebookId,
    clientNsec,
  });
}

// Blossom
export async function blossomMirror(
  serverUrl: string,
  authHeader: string,
  url: string,
): Promise<string> {
  return await invoke("plugin:nostrnative|blossom_mirror", {
    serverUrl,
    authHeader,
    url,
  });
}

export async function blossomUpload(
  serverUrl: string,
  authHeader: string,
  filePath: string,
): Promise<string> {
  return await invoke("plugin:nostrnative|blossom_upload", {
    serverUrl,
    authHeader,
    filePath,
  });
}

export async function blossomUploadContent(
  serverUrl: string,
  authHeader: string,
  content: string,
  filename?: string,
  mediaType?: string,
): Promise<string> {
  return await invoke("plugin:nostrnative|blossom_upload_content", {
    serverUrl,
    authHeader,
    content,
    filename,
    mediaType,
  });
}

export async function blossomGetBlob(
  serverUrl: string,
  hash: string,
): Promise<number[]> {
  return await invoke("plugin:nostrnative|blossom_get_blob", {
    serverUrl,
    hash,
  });
}

export async function blossomDelete(
  serverUrl: string,
  authHeader: string,
  hash: string,
): Promise<void> {
  return await invoke("plugin:nostrnative|blossom_delete", {
    serverUrl,
    authHeader,
    hash,
  });
}

export async function blossomList(
  serverUrl: string,
  pubkey: string,
  authHeader?: string,
): Promise<any[]> {
  return await invoke("plugin:nostrnative|blossom_list", {
    serverUrl,
    pubkey,
    authHeader,
  });
}

// Chat
export async function derivePnsKeys(
  deviceKeyHex: string,
): Promise<PnsDerivedKeys> {
  return await invoke("plugin:nostrnative|derive_pns_keys", {
    deviceKeyHex,
  });
}

export async function createDiscoveryEvent(
  nsec: string,
  deviceKeyToStore: string,
  relays: string[],
  clientNsec?: string,
): Promise<string> {
  return await invoke("plugin:nostrnative|create_discovery_event", {
    nsec,
    deviceKeyToStore,
    relays,
    clientNsec,
  });
}

export async function decryptDiscoveryEvent(
  nsec: string,
  content: string,
  clientNsec?: string,
): Promise<string> {
  return await invoke("plugin:nostrnative|decrypt_discovery_event", {
    nsec,
    content,
    clientNsec,
  });
}

export async function publishChatMessage(
  pnsNsec: string,
  mainPubkey: string,
  msg: ChatMessageRequest,
  relays: string[],
  clientNsec?: string,
): Promise<string> {
  return await invoke("plugin:nostrnative|publish_chat_message", {
    pnsNsec,
    mainPubkey,
    msg,
    relays,
    clientNsec,
  });
}

export async function decryptPnsEvent(
  pnsNsec: string,
  content: string,
  clientNsec?: string,
): Promise<any> {
  return await invoke("plugin:nostrnative|decrypt_pns_event", {
    pnsNsec,
    content,
    clientNsec,
  });
}

export async function fetchPnsEvents(
  pnsPubkey: string,
  relays: string[],
  eventId?: string,
): Promise<EventResponse[]> {
  return await invoke("plugin:nostrnative|fetch_pns_events", {
    pnsPubkey,
    relays,
    eventId,
  });
}

export async function deleteChatMessages(
  pnsNsec: string,
  relays: string[],
  eventIds: string[],
  clientNsec?: string,
): Promise<string> {
  return await invoke("plugin:nostrnative|delete_chat_messages", {
    pnsNsec,
    relays,
    eventIds,
    clientNsec,
  });
}

export async function fetchDiscoveryEvents(
  pubkey: string,
  relays: string[],
): Promise<EventResponse[]> {
  return await invoke("plugin:nostrnative|fetch_discovery_events", {
    pubkey,
    relays,
  });
}

// Routstr
export async function discoverRoutstrProviders(
  relays: string[],
): Promise<RoutstrProvider[]> {
  return await invoke("plugin:nostrnative|discover_routstr_providers", {
    relays,
  });
}

export async function compareRoutstrPrices(
  providerUrls: string[],
): Promise<any> {
  return await invoke("plugin:nostrnative|compare_routstr_prices", {
    providerUrls,
  });
}

export interface NFPTemperature {
  value: number; // in Celsius
  time: string; // time taken (HH:MM)
  method: string; // oral, vaginal, rectal
  disturbance?: string;
}

export interface NFPCervicalFluid {
  sensation: string; // dry, moist, wet, slippery
  appearance?: string; // none, sticky, creamy, egg-white, unusual
  amount?: string; // none, little, moderate, much
}

export interface NFPCervixPosition {
  height: string; // low, medium, high
  firmness: string; // firm, medium, soft
  opening: string; // closed, slightly-open, open
}

export interface NFPBleeding {
  flow: string; // spotting, light, medium, heavy
  color?: string; // brown, red, pink
}

export interface NFPObservationRequest {
  date: string; // ISO date string (YYYY-MM-DD)
  identifier: string; // d-tag
  temperature?: NFPTemperature;
  cervical_fluid?: NFPCervicalFluid;
  cervix_position?: NFPCervixPosition;
  bleeding?: NFPBleeding;
  notes?: string;
  disturbances?: string[];
}

export interface NFPCycleRequest {
  identifier: string; // d-tag - cycle ID
  start_date: string; // first day of menstruation
  end_date?: string; // last day before next cycle starts
  length?: number; // in days
  fertility_window_start?: string;
  fertility_window_end?: string;
  peak_day?: string;
  ovulation_day?: string;
  luteal_phase_length?: number;
}

export interface NFPSettingsRequest {
  identifier: string; // d-tag - always "user-settings"
  temperature_unit: string; // celsius, fahrenheit
  temperature_method: string; // oral, vaginal, rectal
  reminder_time?: string; // HH:MM
  average_cycle_length: number;
  luteal_phase_length?: number;
  require_pin_on_open: boolean;
  auto_lock_after_minutes?: number;
  sync_enabled: boolean;
  encrypted_backup: boolean;
  theme: string; // light, dark, system
  chart_style: string; // classic, modern
  show_predictions: boolean;
  temperature_reminder: boolean;
  ovulation_prediction: boolean;
  cycle_end_prediction: boolean;
}

export interface FilesystemMetadata {
  name: string;
  version: string;
  max_depth: number;
  total_nodes: number;
  total_size: number;
  created_at: number;
}

export interface FileNodeMetadata {
  created: number;
  modified: number;
  permissions: string;
  size: number;
  mime_type: string;
  content_hash: string;
}

export interface DirectoryNodeMetadata {
  created: number;
  modified: number;
  permissions: string;
}

export interface ContentReference {
  source: string; // e.g., "nostr_event", "blossom"
  kind?: number;
  id: string; // event id or blossom hash
  url?: string;
}

export type VirtualNode =
  | {
      type: "file";
      name: string;
      depth: number;
      metadata: FileNodeMetadata;
      content_ref?: ContentReference;
    }
  | {
      type: "directory";
      name: string;
      depth: number;
      metadata: DirectoryNodeMetadata;
      children: VirtualNode[];
    };

export interface VirtualFilesystem {
  root: VirtualNode;
  metadata: FilesystemMetadata;
}

export interface FilesystemRequest {
  identifier: string; // d-tag
  name: string;
  filesystem: VirtualFilesystem;
}

// NFP Functions
export async function fetchNfpObservations(
  pubkey: string,
  nsec: string | null,
  relays: string[],
  rangeStart: number | null,
  rangeEnd: number | null,
  clientNsec: string | null,
): Promise<EventResponse[]> {
  return await invoke("plugin:nostrnative|fetch_nfp_observations", {
    pubkey,
    nsec,
    relays,
    rangeStart,
    rangeEnd,
    clientNsec,
  });
}

export async function publishNfpObservation(
  nsec: string,
  relays: string[],
  observation: NFPObservationRequest,
  clientNsec: string | null,
): Promise<string> {
  return await invoke("plugin:nostrnative|publish_nfp_observation", {
    nsec,
    relays,
    observation,
    clientNsec,
  });
}

export async function fetchNfpCycles(
  pubkey: string,
  nsec: string | null,
  relays: string[],
  clientNsec: string | null,
): Promise<EventResponse[]> {
  return await invoke("plugin:nostrnative|fetch_nfp_cycles", {
    pubkey,
    nsec,
    relays,
    clientNsec,
  });
}

export async function publishNfpCycle(
  nsec: string,
  relays: string[],
  cycle: NFPCycleRequest,
  clientNsec: string | null,
): Promise<string> {
  return await invoke("plugin:nostrnative|publish_nfp_cycle", {
    nsec,
    relays,
    cycle,
    clientNsec,
  });
}

export async function fetchNfpSettings(
  pubkey: string,
  nsec: string | null,
  relays: string[],
  clientNsec: string | null,
): Promise<EventResponse | null> {
  return await invoke("plugin:nostrnative|fetch_nfp_settings", {
    pubkey,
    nsec,
    relays,
    clientNsec,
  });
}

export async function publishNfpSettings(
  nsec: string,
  relays: string[],
  settings: NFPSettingsRequest,
  clientNsec: string | null,
): Promise<string> {
  return await invoke("plugin:nostrnative|publish_nfp_settings", {
    nsec,
    relays,
    settings,
    clientNsec,
  });
}

export async function deleteNfpData(
  nsec: string,
  relays: string[],
  eventIds: string[],
  clientNsec: string | null,
): Promise<string> {
  return await invoke("plugin:nostrnative|delete_nfp_data", {
    nsec,
    relays,
    eventIds,
    clientNsec,
  });
}

// Virtual FS
export async function publishVirtualFs(
  nsec: string,
  relays: string[],
  req: FilesystemRequest,
  clientNsec?: string,
): Promise<string> {
  return await invoke("plugin:nostrnative|publish_virtual_fs", {
    nsec,
    relays,
    req,
    clientNsec,
  });
}

export async function fetchVirtualFs(
  pubkey: string,
  relays: string[],
): Promise<EventResponse[]> {
  return await invoke("plugin:nostrnative|fetch_virtual_fs", {
    pubkey,
    relays,
  });
}

export async function deleteVirtualFs(
  nsec: string,
  relays: string[],
  identifier: string,
  clientNsec?: string,
): Promise<string> {
  return await invoke("plugin:nostrnative|delete_virtual_fs", {
    nsec,
    relays,
    identifier,
    clientNsec,
  });
}

export async function fetchRoutstrModels(providerUrl: string): Promise<any[]> {
  return await invoke("plugin:nostrnative|fetch_routstr_models", {
    providerUrl,
  });
}
