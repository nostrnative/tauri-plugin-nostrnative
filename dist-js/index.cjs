'use strict';

var core = require('@tauri-apps/api/core');

// Keys
async function hexToNpub(hex) {
    return await core.invoke("plugin:nostrnative|hex_to_npub", { hex });
}
async function npubToHex(npub) {
    return await core.invoke("plugin:nostrnative|npub_to_hex", { npub });
}
async function nsecToHex(nsec) {
    return await core.invoke("plugin:nostrnative|nsec_to_hex", { nsec });
}
async function generateNewNsec() {
    return await core.invoke("plugin:nostrnative|generate_new_nsec");
}
async function parsePubkey(pubkey) {
    return await core.invoke("plugin:nostrnative|parse_pubkey", { pubkey });
}
async function verifyNsec(nsec, options) {
    return await core.invoke("plugin:nostrnative|verify_nsec", {
        nsec,
        clientNsec: options?.clientNsec,
    });
}
function isBunkerUri(value) {
    return value.trim().startsWith("bunker://");
}
function isNostrConnectUri(value) {
    return value.trim().startsWith("nostrconnect://");
}
/** Connect to a bunker:// signer (or reuse the cached session). */
async function bunkerConnect(uri, clientNsec) {
    return await core.invoke("plugin:nostrnative|bunker_connect", { uri, clientNsec });
}
/** Tear down the cached session for this bunker (e.g. on logout). */
async function bunkerDisconnect(uri, clientNsec) {
    return await core.invoke("plugin:nostrnative|bunker_disconnect", { uri, clientNsec });
}
/**
 * Drain https auth_url challenges the signer asked the user to open, for a
 * pairing (`token`) or a bunker login (`clientNsec`).
 */
async function bunkerTakeAuthUrls(scope) {
    return await core.invoke("plugin:nostrnative|bunker_take_auth_urls", scope);
}
/** Start a client-initiated (QR code) pairing. */
async function nostrConnectStart(relays, appName) {
    return await core.invoke("plugin:nostrnative|nostrconnect_start", { relays, appName });
}
async function nostrConnectPoll(token) {
    return await core.invoke("plugin:nostrnative|nostrconnect_poll", { token });
}
async function nostrConnectCancel(token) {
    return await core.invoke("plugin:nostrnative|nostrconnect_cancel", { token });
}
// Calendar
async function deriveCalendarKeys(deviceKeyHex) {
    return await core.invoke("plugin:nostrnative|derive_calendar_keys", {
        deviceKeyHex,
    });
}
async function fetchCalendarEvents(pubkey, relays, options) {
    return await core.invoke("plugin:nostrnative|fetch_calendar_events", {
        pubkey,
        key: options?.key,
        relays,
        rangeStart: options?.rangeStart,
        rangeEnd: options?.rangeEnd,
        authors: options?.authors,
        clientNsec: options?.clientNsec,
        filters: options?.filters,
    });
}
async function publishBatchCalendarEvents(key, relays, events, clientNsec) {
    return await core.invoke("plugin:nostrnative|publish_batch_calendar_events", {
        key,
        relays,
        events,
        clientNsec,
    });
}
// Relay
async function startNostrRelay(port, dbPath) {
    return await core.invoke("plugin:nostrnative|start_nostr_relay", { port, dbPath });
}
async function stopNostrRelay() {
    return await core.invoke("plugin:nostrnative|stop_nostr_relay");
}
async function getNostrRelayStatus() {
    return await core.invoke("plugin:nostrnative|get_nostr_relay_status");
}
async function getNostrEventCounts() {
    return await core.invoke("plugin:nostrnative|get_nostr_event_counts");
}
async function getNostrRelayEvents() {
    return await core.invoke("plugin:nostrnative|get_nostr_relay_events");
}
async function publishCalendarEvent(key, relays, eventData, clientNsec) {
    return await core.invoke("plugin:nostrnative|publish_calendar_event", {
        key,
        relays,
        eventData,
        clientNsec,
    });
}
async function deleteCalendarEvent(key, relays, eventIds, clientNsec) {
    return await core.invoke("plugin:nostrnative|delete_calendar_event", {
        key,
        relays,
        eventIds,
        clientNsec,
    });
}
async function fetchCalendars(pubkey, relays) {
    return await core.invoke("plugin:nostrnative|fetch_calendars", {
        pubkey,
        relays,
    });
}
async function publishCalendar(key, relays, calendar, clientNsec) {
    return await core.invoke("plugin:nostrnative|publish_calendar", {
        key,
        relays,
        calendar,
        clientNsec,
    });
}
async function deleteCalendar(key, relays, identifier, clientNsec) {
    return await core.invoke("plugin:nostrnative|delete_calendar", {
        key,
        relays,
        identifier,
        clientNsec,
    });
}
async function fetchCalendarContactLists(pubkey, relays, options) {
    return await core.invoke("plugin:nostrnative|fetch_calendar_contact_lists", {
        pubkey,
        relays,
        key: options?.key,
        clientNsec: options?.clientNsec,
    });
}
async function publishCalendarContactList(key, relays, request, clientNsec) {
    return await core.invoke("plugin:nostrnative|publish_calendar_contact_list", {
        key,
        relays,
        request,
        clientNsec,
    });
}
// Decrypt private events
async function decryptPrivateEvents(key, relays, events, clientNsec) {
    return await core.invoke("plugin:nostrnative|decrypt_private_events", {
        key,
        relays,
        events,
        clientNsec,
    });
}
// RSVP
async function fetchRsvps(eventCoordinate, relays) {
    return await core.invoke("plugin:nostrnative|fetch_rsvps", {
        eventCoordinate,
        relays,
    });
}
async function fetchUserRsvps(pubkey, relays) {
    return await core.invoke("plugin:nostrnative|fetch_user_rsvps", {
        pubkey,
        relays,
    });
}
async function fetchReceivedRsvps(pubkey, relays) {
    return await core.invoke("plugin:nostrnative|fetch_received_rsvps", {
        pubkey,
        relays,
    });
}
async function publishRsvp(key, relays, eventCoordinate, status, eventAuthor, clientNsec, fb, content) {
    return await core.invoke("plugin:nostrnative|publish_rsvp", {
        key,
        relays,
        eventCoordinate,
        status,
        eventAuthor,
        clientNsec,
        fb,
        content,
    });
}
// Profile
async function fetchProfiles(pubkeys, relays) {
    return await core.invoke("plugin:nostrnative|fetch_profiles", { pubkeys, relays });
}
async function fetchContactList(pubkey, relays) {
    return await core.invoke("plugin:nostrnative|fetch_contact_list", {
        pubkey,
        relays,
    });
}
async function updateContactList(nsec, relays, contacts, clientNsec) {
    return await core.invoke("plugin:nostrnative|update_contact_list", {
        nsec,
        relays,
        contacts,
        clientNsec,
    });
}
// Messages
async function sendDirectMessage(nsec, receiverPubkey, message, relays, clientNsec) {
    return await core.invoke("plugin:nostrnative|send_direct_message", {
        nsec,
        receiverPubkey,
        message,
        relays,
        clientNsec,
    });
}
// Bookmarks
async function fetchBookmarks(pubkey, relays, nsec, clientNsec) {
    return await core.invoke("plugin:nostrnative|fetch_bookmarks", {
        pubkey,
        nsec,
        relays,
        clientNsec,
    });
}
async function fetchEventDetails(identifiers, relays) {
    return await core.invoke("plugin:nostrnative|fetch_event_details", {
        identifiers,
        relays,
    });
}
async function saveBookmark(nsec, relays, eventId, source, clientNsec) {
    return await core.invoke("plugin:nostrnative|save_bookmark", {
        nsec,
        relays,
        eventId,
        source,
        clientNsec,
    });
}
async function removeBookmark(nsec, relays, eventId, source, clientNsec) {
    return await core.invoke("plugin:nostrnative|remove_bookmark", {
        nsec,
        relays,
        eventId,
        source,
        clientNsec,
    });
}
// Notes
async function createNote(nsec, relays, note, clientNsec) {
    return await core.invoke("plugin:nostrnative|create_note", {
        nsec,
        relays,
        note,
        clientNsec,
    });
}
async function updateNote(nsec, relays, identifier, note, clientNsec) {
    return await core.invoke("plugin:nostrnative|update_note", {
        nsec,
        relays,
        identifier,
        note,
        clientNsec,
    });
}
async function deleteNote(nsec, relays, noteId, clientNsec) {
    return await core.invoke("plugin:nostrnative|delete_note", {
        nsec,
        relays,
        noteId,
        clientNsec,
    });
}
async function fetchNotes(pubkey, relays, options) {
    return await core.invoke("plugin:nostrnative|fetch_notes", {
        pubkey,
        relays,
        nsec: options?.nsec,
        note_type: options?.noteType,
        clientNsec: options?.clientNsec,
    });
}
async function createNotebook(nsec, relays, notebook, clientNsec) {
    return await core.invoke("plugin:nostrnative|create_notebook", {
        nsec,
        relays,
        notebook,
        clientNsec,
    });
}
async function fetchNotebooks(pubkey, relays, options) {
    return await core.invoke("plugin:nostrnative|fetch_notebooks", {
        pubkey,
        relays,
        nsec: options?.nsec,
        clientNsec: options?.clientNsec,
    });
}
async function addNoteToNotebook(nsec, relays, notebookId, noteId, clientNsec) {
    return await core.invoke("plugin:nostrnative|add_note_to_notebook", {
        nsec,
        relays,
        notebook_id: notebookId,
        note_id: noteId,
        clientNsec,
    });
}
async function removeNoteFromNotebook(nsec, relays, notebookId, noteId, clientNsec) {
    return await core.invoke("plugin:nostrnative|remove_note_from_notebook", {
        nsec,
        relays,
        notebook_id: notebookId,
        note_id: noteId,
        clientNsec,
    });
}
async function deleteNotebook(nsec, relays, notebookId, clientNsec) {
    return await core.invoke("plugin:nostrnative|delete_notebook", {
        nsec,
        relays,
        notebook_id: notebookId,
        clientNsec,
    });
}
// Blossom
async function blossomMirror(serverUrl, authHeader, url) {
    return await core.invoke("plugin:nostrnative|blossom_mirror", {
        serverUrl,
        authHeader,
        url,
    });
}
async function blossomUpload(serverUrl, authHeader, filePath) {
    return await core.invoke("plugin:nostrnative|blossom_upload", {
        serverUrl,
        authHeader,
        filePath,
    });
}
async function blossomUploadContent(serverUrl, authHeader, content, filename, mediaType) {
    return await core.invoke("plugin:nostrnative|blossom_upload_content", {
        serverUrl,
        authHeader,
        content,
        filename,
        mediaType,
    });
}
async function blossomGetBlob(serverUrl, hash) {
    return await core.invoke("plugin:nostrnative|blossom_get_blob", {
        serverUrl,
        hash,
    });
}
async function blossomDelete(serverUrl, authHeader, hash) {
    return await core.invoke("plugin:nostrnative|blossom_delete", {
        serverUrl,
        authHeader,
        hash,
    });
}
async function blossomList(serverUrl, pubkey, authHeader) {
    return await core.invoke("plugin:nostrnative|blossom_list", {
        serverUrl,
        pubkey,
        authHeader,
    });
}
// Chat
async function derivePnsKeys(deviceKeyHex) {
    return await core.invoke("plugin:nostrnative|derive_pns_keys", {
        deviceKeyHex,
    });
}
async function createDiscoveryEvent(nsec, deviceKeyToStore, relays, clientNsec) {
    return await core.invoke("plugin:nostrnative|create_discovery_event", {
        nsec,
        deviceKeyToStore,
        relays,
        clientNsec,
    });
}
async function decryptDiscoveryEvent(nsec, content, clientNsec) {
    return await core.invoke("plugin:nostrnative|decrypt_discovery_event", {
        nsec,
        content,
        clientNsec,
    });
}
async function publishChatMessage(pnsNsec, mainPubkey, msg, relays, clientNsec) {
    return await core.invoke("plugin:nostrnative|publish_chat_message", {
        pnsNsec,
        mainPubkey,
        msg,
        relays,
        clientNsec,
    });
}
async function decryptPnsEvent(pnsNsec, content, clientNsec) {
    return await core.invoke("plugin:nostrnative|decrypt_pns_event", {
        pnsNsec,
        content,
        clientNsec,
    });
}
async function fetchPnsEvents(pnsPubkey, relays, eventId) {
    return await core.invoke("plugin:nostrnative|fetch_pns_events", {
        pnsPubkey,
        relays,
        eventId,
    });
}
async function deleteChatMessages(pnsNsec, relays, eventIds, clientNsec) {
    return await core.invoke("plugin:nostrnative|delete_chat_messages", {
        pnsNsec,
        relays,
        eventIds,
        clientNsec,
    });
}
async function fetchDiscoveryEvents(pubkey, relays) {
    return await core.invoke("plugin:nostrnative|fetch_discovery_events", {
        pubkey,
        relays,
    });
}
// Routstr
async function discoverRoutstrProviders(relays) {
    return await core.invoke("plugin:nostrnative|discover_routstr_providers", {
        relays,
    });
}
async function compareRoutstrPrices(providerUrls) {
    return await core.invoke("plugin:nostrnative|compare_routstr_prices", {
        providerUrls,
    });
}
// NFP Functions
async function fetchNfpObservations(pubkey, nsec, relays, rangeStart, rangeEnd, clientNsec) {
    return await core.invoke("plugin:nostrnative|fetch_nfp_observations", {
        pubkey,
        nsec,
        relays,
        rangeStart,
        rangeEnd,
        clientNsec,
    });
}
async function publishNfpObservation(nsec, relays, observation, clientNsec) {
    return await core.invoke("plugin:nostrnative|publish_nfp_observation", {
        nsec,
        relays,
        observation,
        clientNsec,
    });
}
async function fetchNfpCycles(pubkey, nsec, relays, clientNsec) {
    return await core.invoke("plugin:nostrnative|fetch_nfp_cycles", {
        pubkey,
        nsec,
        relays,
        clientNsec,
    });
}
async function publishNfpCycle(nsec, relays, cycle, clientNsec) {
    return await core.invoke("plugin:nostrnative|publish_nfp_cycle", {
        nsec,
        relays,
        cycle,
        clientNsec,
    });
}
async function fetchNfpSettings(pubkey, nsec, relays, clientNsec) {
    return await core.invoke("plugin:nostrnative|fetch_nfp_settings", {
        pubkey,
        nsec,
        relays,
        clientNsec,
    });
}
async function publishNfpSettings(nsec, relays, settings, clientNsec) {
    return await core.invoke("plugin:nostrnative|publish_nfp_settings", {
        nsec,
        relays,
        settings,
        clientNsec,
    });
}
async function deleteNfpData(nsec, relays, eventIds, clientNsec) {
    return await core.invoke("plugin:nostrnative|delete_nfp_data", {
        nsec,
        relays,
        eventIds,
        clientNsec,
    });
}
// Virtual FS
async function publishVirtualFs(nsec, relays, req, clientNsec) {
    return await core.invoke("plugin:nostrnative|publish_virtual_fs", {
        nsec,
        relays,
        req,
        clientNsec,
    });
}
async function fetchVirtualFs(pubkey, relays) {
    return await core.invoke("plugin:nostrnative|fetch_virtual_fs", {
        pubkey,
        relays,
    });
}
async function deleteVirtualFs(nsec, relays, identifier, clientNsec) {
    return await core.invoke("plugin:nostrnative|delete_virtual_fs", {
        nsec,
        relays,
        identifier,
        clientNsec,
    });
}
async function fetchRoutstrModels(providerUrl) {
    return await core.invoke("plugin:nostrnative|fetch_routstr_models", {
        providerUrl,
    });
}

exports.addNoteToNotebook = addNoteToNotebook;
exports.blossomDelete = blossomDelete;
exports.blossomGetBlob = blossomGetBlob;
exports.blossomList = blossomList;
exports.blossomMirror = blossomMirror;
exports.blossomUpload = blossomUpload;
exports.blossomUploadContent = blossomUploadContent;
exports.bunkerConnect = bunkerConnect;
exports.bunkerDisconnect = bunkerDisconnect;
exports.bunkerTakeAuthUrls = bunkerTakeAuthUrls;
exports.compareRoutstrPrices = compareRoutstrPrices;
exports.createDiscoveryEvent = createDiscoveryEvent;
exports.createNote = createNote;
exports.createNotebook = createNotebook;
exports.decryptDiscoveryEvent = decryptDiscoveryEvent;
exports.decryptPnsEvent = decryptPnsEvent;
exports.decryptPrivateEvents = decryptPrivateEvents;
exports.deleteCalendar = deleteCalendar;
exports.deleteCalendarEvent = deleteCalendarEvent;
exports.deleteChatMessages = deleteChatMessages;
exports.deleteNfpData = deleteNfpData;
exports.deleteNote = deleteNote;
exports.deleteNotebook = deleteNotebook;
exports.deleteVirtualFs = deleteVirtualFs;
exports.deriveCalendarKeys = deriveCalendarKeys;
exports.derivePnsKeys = derivePnsKeys;
exports.discoverRoutstrProviders = discoverRoutstrProviders;
exports.fetchBookmarks = fetchBookmarks;
exports.fetchCalendarContactLists = fetchCalendarContactLists;
exports.fetchCalendarEvents = fetchCalendarEvents;
exports.fetchCalendars = fetchCalendars;
exports.fetchContactList = fetchContactList;
exports.fetchDiscoveryEvents = fetchDiscoveryEvents;
exports.fetchEventDetails = fetchEventDetails;
exports.fetchNfpCycles = fetchNfpCycles;
exports.fetchNfpObservations = fetchNfpObservations;
exports.fetchNfpSettings = fetchNfpSettings;
exports.fetchNotebooks = fetchNotebooks;
exports.fetchNotes = fetchNotes;
exports.fetchPnsEvents = fetchPnsEvents;
exports.fetchProfiles = fetchProfiles;
exports.fetchReceivedRsvps = fetchReceivedRsvps;
exports.fetchRoutstrModels = fetchRoutstrModels;
exports.fetchRsvps = fetchRsvps;
exports.fetchUserRsvps = fetchUserRsvps;
exports.fetchVirtualFs = fetchVirtualFs;
exports.generateNewNsec = generateNewNsec;
exports.getNostrEventCounts = getNostrEventCounts;
exports.getNostrRelayEvents = getNostrRelayEvents;
exports.getNostrRelayStatus = getNostrRelayStatus;
exports.hexToNpub = hexToNpub;
exports.isBunkerUri = isBunkerUri;
exports.isNostrConnectUri = isNostrConnectUri;
exports.nostrConnectCancel = nostrConnectCancel;
exports.nostrConnectPoll = nostrConnectPoll;
exports.nostrConnectStart = nostrConnectStart;
exports.npubToHex = npubToHex;
exports.nsecToHex = nsecToHex;
exports.parsePubkey = parsePubkey;
exports.publishBatchCalendarEvents = publishBatchCalendarEvents;
exports.publishCalendar = publishCalendar;
exports.publishCalendarContactList = publishCalendarContactList;
exports.publishCalendarEvent = publishCalendarEvent;
exports.publishChatMessage = publishChatMessage;
exports.publishNfpCycle = publishNfpCycle;
exports.publishNfpObservation = publishNfpObservation;
exports.publishNfpSettings = publishNfpSettings;
exports.publishRsvp = publishRsvp;
exports.publishVirtualFs = publishVirtualFs;
exports.removeBookmark = removeBookmark;
exports.removeNoteFromNotebook = removeNoteFromNotebook;
exports.saveBookmark = saveBookmark;
exports.sendDirectMessage = sendDirectMessage;
exports.startNostrRelay = startNostrRelay;
exports.stopNostrRelay = stopNostrRelay;
exports.updateContactList = updateContactList;
exports.updateNote = updateNote;
exports.verifyNsec = verifyNsec;
