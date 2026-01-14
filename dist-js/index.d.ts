export interface CalendarEventRequest {
    title: string;
    description?: string;
    start: number;
    end?: number;
    location?: string;
    is_all_day: boolean;
    identifier: string;
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
    identifier: string;
}
export interface Contact {
    pubkey: string;
    alias?: string | null;
}
export interface UserProfile {
    pubkey: string;
    name?: string | null;
    display_name?: string | null;
    about?: string | null;
    picture?: string | null;
    banner?: string | null;
    website?: string | null;
    nip05?: string | null;
}
export type BookmarkSource = 'public' | 'private' | 'website';
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
    conversation_id: string;
    role: string;
    client: string;
    reply_to?: string;
    event_id?: string;
    context_ids?: string[];
    blob_hashes?: string[];
}
export interface PnsDerivedKeys {
    public_key: string;
    secret_key: string;
}
export declare function generateNewNsec(): Promise<string>;
export declare function parsePubkey(pubkey: string): Promise<string>;
export declare function verifyNsec(nsec: string): Promise<string>;
export declare function fetchCalendarEvents(pubkey: string, relays: string[], options?: {
    nsec?: string;
    rangeStart?: number;
    rangeEnd?: number;
    authors?: string[];
    clientNsec?: string;
}): Promise<EventResponse[]>;
export declare function publishBatchCalendarEvents(nsec: string, relays: string[], events: CalendarEventRequest[], clientNsec?: string): Promise<string[]>;
export declare function publishCalendarEvent(nsec: string, relays: string[], eventData: CalendarEventRequest, clientNsec?: string): Promise<string>;
export declare function deleteCalendarEvent(nsec: string, relays: string[], eventIds: string[], clientNsec?: string): Promise<string>;
export declare function fetchCalendars(pubkey: string, relays: string[]): Promise<EventResponse[]>;
export declare function publishCalendar(nsec: string, relays: string[], calendar: CalendarRequest, clientNsec?: string): Promise<string>;
export declare function deleteCalendar(nsec: string, relays: string[], identifier: string, clientNsec?: string): Promise<string>;
export declare function fetchRsvps(eventCoordinate: string, relays: string[]): Promise<EventResponse[]>;
export declare function fetchUserRsvps(pubkey: string, relays: string[]): Promise<EventResponse[]>;
export declare function fetchReceivedRsvps(pubkey: string, relays: string[]): Promise<EventResponse[]>;
export declare function publishRsvp(nsec: string, relays: string[], eventCoordinate: string, status: string, eventAuthor?: string, clientNsec?: string): Promise<string>;
export declare function fetchProfiles(pubkeys: string[], relays: string[]): Promise<UserProfile[]>;
export declare function fetchContactList(pubkey: string, relays: string[]): Promise<Contact[]>;
export declare function updateContactList(nsec: string, relays: string[], contacts: Contact[], clientNsec?: string): Promise<string>;
export declare function sendDirectMessage(nsec: string, receiverPubkey: string, message: string, relays: string[], clientNsec?: string): Promise<string>;
export declare function fetchBookmarks(pubkey: string, relays: string[], nsec?: string, clientNsec?: string): Promise<EventResponse[]>;
export declare function fetchEventDetails(identifiers: string[], relays: string[]): Promise<EventResponse[]>;
export declare function saveBookmark(nsec: string, relays: string[], eventId: string, source: BookmarkSource, clientNsec?: string): Promise<string>;
export declare function removeBookmark(nsec: string, relays: string[], eventId: string, source: BookmarkSource, clientNsec?: string): Promise<string>;
export declare function createNote(nsec: string, relays: string[], note: NoteRequest, clientNsec?: string): Promise<string>;
export declare function updateNote(nsec: string, relays: string[], identifier: string, note: NoteRequest, clientNsec?: string): Promise<string>;
export declare function deleteNote(nsec: string, relays: string[], noteId: string, clientNsec?: string): Promise<string>;
export declare function fetchNotes(pubkey: string, relays: string[], options?: {
    nsec?: string;
    noteType?: string;
    clientNsec?: string;
}): Promise<EventResponse[]>;
export declare function createNotebook(nsec: string, relays: string[], notebook: NotebookRequest, clientNsec?: string): Promise<string>;
export declare function fetchNotebooks(pubkey: string, relays: string[], options?: {
    nsec?: string;
    clientNsec?: string;
}): Promise<EventResponse[]>;
export declare function addNoteToNotebook(nsec: string, relays: string[], notebookId: string, noteId: string, clientNsec?: string): Promise<string>;
export declare function removeNoteFromNotebook(nsec: string, relays: string[], notebookId: string, noteId: string, clientNsec?: string): Promise<string>;
export declare function deleteNotebook(nsec: string, relays: string[], notebookId: string, clientNsec?: string): Promise<string>;
export declare function blossomMirror(serverUrl: string, nsec: string, url: string, clientNsec?: string): Promise<string>;
export declare function blossomUpload(serverUrl: string, nsec: string, filePath: string, clientNsec?: string): Promise<string>;
export declare function blossomUploadContent(serverUrl: string, nsec: string, content: string, filename?: string, mediaType?: string, clientNsec?: string): Promise<string>;
export declare function blossomGetBlob(serverUrl: string, hash: string): Promise<number[]>;
export declare function derivePnsKeys(deviceKeyHex: string): Promise<PnsDerivedKeys>;
export declare function createDiscoveryEvent(nsec: string, deviceKeyToStore: string, relays: string[], clientNsec?: string): Promise<string>;
export declare function decryptDiscoveryEvent(nsec: string, content: string, clientNsec?: string): Promise<string>;
export declare function publishChatMessage(pnsNsec: string, mainPubkey: string, msg: ChatMessageRequest, relays: string[], clientNsec?: string): Promise<string>;
export declare function decryptPnsEvent(pnsNsec: string, content: string, clientNsec?: string): Promise<any>;
export declare function fetchPnsEvents(pnsPubkey: string, relays: string[], eventId?: string): Promise<EventResponse[]>;
export declare function deleteChatMessages(pnsNsec: string, relays: string[], eventIds: string[], clientNsec?: string): Promise<string>;
export declare function fetchDiscoveryEvents(pubkey: string, relays: string[]): Promise<EventResponse[]>;
export interface NFPTemperature {
    value: number;
    time: string;
    method: string;
    disturbance?: string;
}
export interface NFPCervicalFluid {
    sensation: string;
    appearance?: string;
    amount?: string;
}
export interface NFPCervixPosition {
    height: string;
    firmness: string;
    opening: string;
}
export interface NFPBleeding {
    flow: string;
    color?: string;
}
export interface NFPObservationRequest {
    date: string;
    identifier: string;
    temperature?: NFPTemperature;
    cervical_fluid?: NFPCervicalFluid;
    cervix_position?: NFPCervixPosition;
    bleeding?: NFPBleeding;
    notes?: string;
    disturbances?: string[];
}
export interface NFPCycleRequest {
    identifier: string;
    start_date: string;
    end_date?: string;
    length?: number;
    fertility_window_start?: string;
    fertility_window_end?: string;
    peak_day?: string;
    ovulation_day?: string;
    luteal_phase_length?: number;
}
export interface NFPSettingsRequest {
    identifier: string;
    temperature_unit: string;
    temperature_method: string;
    reminder_time?: string;
    average_cycle_length: number;
    luteal_phase_length?: number;
    require_pin_on_open: boolean;
    auto_lock_after_minutes?: number;
    sync_enabled: boolean;
    encrypted_backup: boolean;
    theme: string;
    chart_style: string;
    show_predictions: boolean;
    temperature_reminder: boolean;
    ovulation_prediction: boolean;
    cycle_end_prediction: boolean;
}
export declare function fetchNfpObservations(pubkey: string, nsec: string | null, relays: string[], rangeStart: number | null, rangeEnd: number | null, clientNsec: string | null): Promise<EventResponse[]>;
export declare function publishNfpObservation(nsec: string, relays: string[], observation: NFPObservationRequest, clientNsec: string | null): Promise<string>;
export declare function fetchNfpCycles(pubkey: string, nsec: string | null, relays: string[], clientNsec: string | null): Promise<EventResponse[]>;
export declare function publishNfpCycle(nsec: string, relays: string[], cycle: NFPCycleRequest, clientNsec: string | null): Promise<string>;
export declare function fetchNfpSettings(pubkey: string, nsec: string | null, relays: string[], clientNsec: string | null): Promise<EventResponse | null>;
export declare function publishNfpSettings(nsec: string, relays: string[], settings: NFPSettingsRequest, clientNsec: string | null): Promise<string>;
export declare function deleteNfpData(nsec: string, relays: string[], eventIds: string[], clientNsec: string | null): Promise<string>;
