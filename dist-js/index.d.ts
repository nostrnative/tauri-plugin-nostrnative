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
