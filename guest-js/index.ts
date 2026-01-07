import { invoke } from '@tauri-apps/api/core'

export interface CalendarEventRequest {
  title: string
  description?: string
  start: number
  end?: number
  location?: string
  is_all_day: boolean
  identifier: string // d-tag
  reminder_minutes?: number
  old_event_id?: string
  calendar_id?: string
  color?: string
  p_tags?: string[]
  is_private?: boolean
  parent?: string
  freq?: string
  until?: number
  use_different_timestamp?: boolean
}

export interface CalendarRequest {
  name: string
  description?: string
  identifier: string // d-tag
}

export interface Contact {
  pubkey: string
  alias?: string | null
}

export interface UserProfile {
  pubkey: string
  name?: string | null
  display_name?: string | null
  about?: string | null
  picture?: string | null
  banner?: string | null
  website?: string | null
  nip05?: string | null
}

export type BookmarkSource = 'public' | 'private' | 'website'

export interface EventResponse {
  id: string
  pubkey: string
  created_at: number
  kind: number
  tags: string[][]
  content: string
  is_private: boolean
  bech32?: string
  private_tags?: string[][]
  source?: BookmarkSource
}

export interface ChatMessageRequest {
  content: string
  conversation_id: string // d-tag
  role: string // user/assistant
  client: string
  reply_to?: string // e-tag
  event_id?: string
  context_ids?: string[]
  blob_hashes?: string[]
}

export interface PnsDerivedKeys {
  public_key: string
  secret_key: string
}

// Keys
export async function generateNewNsec(): Promise<string> {
  return await invoke('plugin:nostrnative|generate_new_nsec')
}

export async function parsePubkey(pubkey: string): Promise<string> {
  return await invoke('plugin:nostrnative|parse_pubkey', { pubkey })
}

export async function verifyNsec(nsec: string): Promise<string> {
  return await invoke('plugin:nostrnative|verify_nsec', { nsec })
}

// Calendar
export async function fetchCalendarEvents(
  pubkey: string,
  relays: string[],
  options?: { nsec?: string; rangeStart?: number; rangeEnd?: number; authors?: string[]; clientNsec?: string }
): Promise<EventResponse[]> {
  return await invoke('plugin:nostrnative|fetch_calendar_events', {
    pubkey,
    nsec: options?.nsec,
    relays,
    rangeStart: options?.rangeStart,
    rangeEnd: options?.rangeEnd,
    authors: options?.authors,
    clientNsec: options?.clientNsec
  })
}

export async function publishBatchCalendarEvents(
  nsec: string,
  relays: string[],
  events: CalendarEventRequest[],
  clientNsec?: string
): Promise<string[]> {
  return await invoke('plugin:nostrnative|publish_batch_calendar_events', {
    nsec,
    relays,
    events,
    clientNsec
  })
}

export async function publishCalendarEvent(
  nsec: string,
  relays: string[],
  eventData: CalendarEventRequest,
  clientNsec?: string
): Promise<string> {
  return await invoke('plugin:nostrnative|publish_calendar_event', {
    nsec,
    relays,
    eventData,
    clientNsec
  })
}

export async function deleteCalendarEvent(
  nsec: string,
  relays: string[],
  eventIds: string[],
  clientNsec?: string
): Promise<string> {
  return await invoke('plugin:nostrnative|delete_calendar_event', {
    nsec,
    relays,
    eventIds,
    clientNsec
  })
}

export async function fetchCalendars(
  pubkey: string,
  relays: string[]
): Promise<EventResponse[]> {
  return await invoke('plugin:nostrnative|fetch_calendars', {
    pubkey,
    relays
  })
}

export async function publishCalendar(
  nsec: string,
  relays: string[],
  calendar: CalendarRequest,
  clientNsec?: string
): Promise<string> {
  return await invoke('plugin:nostrnative|publish_calendar', {
    nsec,
    relays,
    calendar,
    clientNsec
  })
}

export async function deleteCalendar(
  nsec: string,
  relays: string[],
  identifier: string,
  clientNsec?: string
): Promise<string> {
  return await invoke('plugin:nostrnative|delete_calendar', {
    nsec,
    relays,
    identifier,
    clientNsec
  })
}

// RSVP
export async function fetchRsvps(
  eventCoordinate: string,
  relays: string[]
): Promise<EventResponse[]> {
  return await invoke('plugin:nostrnative|fetch_rsvps', {
    eventCoordinate,
    relays
  })
}

export async function fetchUserRsvps(
  pubkey: string,
  relays: string[]
): Promise<EventResponse[]> {
  return await invoke('plugin:nostrnative|fetch_user_rsvps', {
    pubkey,
    relays
  })
}

export async function fetchReceivedRsvps(
  pubkey: string,
  relays: string[]
): Promise<EventResponse[]> {
  return await invoke('plugin:nostrnative|fetch_received_rsvps', {
    pubkey,
    relays
  })
}

export async function publishRsvp(
  nsec: string,
  relays: string[],
  eventCoordinate: string,
  status: string,
  eventAuthor?: string,
  clientNsec?: string
): Promise<string> {
  return await invoke('plugin:nostrnative|publish_rsvp', {
    nsec,
    relays,
    eventCoordinate,
    status,
    eventAuthor,
    clientNsec
  })
}

// Profile
export async function fetchProfiles(
  pubkeys: string[],
  relays: string[]
): Promise<UserProfile[]> {
  return await invoke('plugin:nostrnative|fetch_profiles', { pubkeys, relays })
}

export async function fetchContactList(
  pubkey: string,
  relays: string[]
): Promise<Contact[]> {
  return await invoke('plugin:nostrnative|fetch_contact_list', {
    pubkey,
    relays
  })
}

export async function updateContactList(
  nsec: string,
  relays: string[],
  contacts: Contact[],
  clientNsec?: string
): Promise<string> {
  return await invoke('plugin:nostrnative|update_contact_list', {
    nsec,
    relays,
    contacts,
    clientNsec
  })
}

// Messages
export async function sendDirectMessage(
  nsec: string,
  receiverPubkey: string,
  message: string,
  relays: string[],
  clientNsec?: string
): Promise<string> {
  return await invoke('plugin:nostrnative|send_direct_message', {
    nsec,
    receiverPubkey,
    message,
    relays,
    clientNsec
  })
}

// Bookmarks
export async function fetchBookmarks(
  pubkey: string,
  relays: string[],
  nsec?: string,
  clientNsec?: string
): Promise<EventResponse[]> {
  return await invoke('plugin:nostrnative|fetch_bookmarks', {
    pubkey,
    nsec,
    relays,
    clientNsec
  })
}

export async function fetchEventDetails(
  identifiers: string[],
  relays: string[]
): Promise<EventResponse[]> {
  return await invoke('plugin:nostrnative|fetch_event_details', {
    identifiers,
    relays
  })
}

export async function saveBookmark(
  nsec: string,
  relays: string[],
  eventId: string,
  source: BookmarkSource,
  clientNsec?: string
): Promise<string> {
  return await invoke('plugin:nostrnative|save_bookmark', {
    nsec,
    relays,
    eventId,
    source,
    clientNsec
  })
}

export async function removeBookmark(
  nsec: string,
  relays: string[],
  eventId: string,
  source: BookmarkSource,
  clientNsec?: string
): Promise<string> {
  return await invoke('plugin:nostrnative|remove_bookmark', {
    nsec,
    relays,
    eventId,
    source,
    clientNsec
  })
}

// Blossom
export async function blossomMirror(
  serverUrl: string,
  nsec: string,
  url: string,
  clientNsec?: string
): Promise<string> {
  return await invoke('plugin:nostrnative|blossom_mirror', {
    serverUrl,
    nsec,
    url,
    clientNsec
  })
}

export async function blossomUpload(
  serverUrl: string,
  nsec: string,
  filePath: string,
  clientNsec?: string
): Promise<string> {
  return await invoke('plugin:nostrnative|blossom_upload', {
    serverUrl,
    nsec,
    filePath,
    clientNsec
  })
}

export async function blossomUploadContent(
  serverUrl: string,
  nsec: string,
  content: string,
  filename?: string,
  mediaType?: string,
  clientNsec?: string
): Promise<string> {
  return await invoke('plugin:nostrnative|blossom_upload_content', {
    serverUrl,
    nsec,
    content,
    filename,
    mediaType,
    clientNsec
  })
}

export async function blossomGetBlob(
  serverUrl: string,
  hash: string
): Promise<number[]> {
  return await invoke('plugin:nostrnative|blossom_get_blob', {
    serverUrl,
    hash
  })
}

// Chat
export async function derivePnsKeys(
  deviceKeyHex: string
): Promise<PnsDerivedKeys> {
  return await invoke('plugin:nostrnative|derive_pns_keys', {
    deviceKeyHex
  })
}

export async function createDiscoveryEvent(
  nsec: string,
  deviceKeyToStore: string,
  relays: string[],
  clientNsec?: string
): Promise<string> {
  return await invoke('plugin:nostrnative|create_discovery_event', {
    nsec,
    deviceKeyToStore,
    relays,
    clientNsec
  })
}

export async function decryptDiscoveryEvent(
  nsec: string,
  content: string,
  clientNsec?: string
): Promise<string> {
  return await invoke('plugin:nostrnative|decrypt_discovery_event', {
    nsec,
    content,
    clientNsec
  })
}

export async function publishChatMessage(
  pnsNsec: string,
  mainPubkey: string,
  msg: ChatMessageRequest,
  relays: string[],
  clientNsec?: string
): Promise<string> {
  return await invoke('plugin:nostrnative|publish_chat_message', {
    pnsNsec,
    mainPubkey,
    msg,
    relays,
    clientNsec
  })
}

export async function decryptPnsEvent(
  pnsNsec: string,
  content: string,
  clientNsec?: string
): Promise<any> {
  return await invoke('plugin:nostrnative|decrypt_pns_event', {
    pnsNsec,
    content,
    clientNsec
  })
}

export async function fetchPnsEvents(
  pnsPubkey: string,
  relays: string[],
  eventId?: string
): Promise<EventResponse[]> {
  return await invoke('plugin:nostrnative|fetch_pns_events', {
    pnsPubkey,
    relays,
    eventId
  })
}

export async function deleteChatMessages(
  pnsNsec: string,
  relays: string[],
  eventIds: string[],
  clientNsec?: string
): Promise<string> {
  return await invoke('plugin:nostrnative|delete_chat_messages', {
    pnsNsec,
    relays,
    eventIds,
    clientNsec
  })
}

export async function fetchDiscoveryEvents(
  pubkey: string,
  relays: string[]
): Promise<EventResponse[]> {
  return await invoke('plugin:nostrnative|fetch_discovery_events', {
    pubkey,
    relays
  })
}
