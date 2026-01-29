'use strict';

var core = require('@tauri-apps/api/core');

// Keys
async function generateNewNsec() {
    return await core.invoke("plugin:nostrnative|generate_new_nsec");
}
async function parsePubkey(pubkey) {
    return await core.invoke("plugin:nostrnative|parse_pubkey", { pubkey });
}
async function verifyNsec(nsec) {
    return await core.invoke("plugin:nostrnative|verify_nsec", { nsec });
}
// Calendar
async function fetchCalendarEvents(pubkey, relays, options) {
    return await core.invoke("plugin:nostrnative|fetch_calendar_events", {
        pubkey,
        nsec: options?.nsec,
        relays,
        rangeStart: options?.rangeStart,
        rangeEnd: options?.rangeEnd,
        authors: options?.authors,
        clientNsec: options?.clientNsec,
    });
}
async function publishBatchCalendarEvents(nsec, relays, events, clientNsec) {
    return await core.invoke("plugin:nostrnative|publish_batch_calendar_events", {
        nsec,
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
async function publishCalendarEvent(nsec, relays, eventData, clientNsec) {
    return await core.invoke("plugin:nostrnative|publish_calendar_event", {
        nsec,
        relays,
        eventData,
        clientNsec,
    });
}
async function deleteCalendarEvent(nsec, relays, eventIds, clientNsec) {
    return await core.invoke("plugin:nostrnative|delete_calendar_event", {
        nsec,
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
async function publishCalendar(nsec, relays, calendar, clientNsec) {
    return await core.invoke("plugin:nostrnative|publish_calendar", {
        nsec,
        relays,
        calendar,
        clientNsec,
    });
}
async function deleteCalendar(nsec, relays, identifier, clientNsec) {
    return await core.invoke("plugin:nostrnative|delete_calendar", {
        nsec,
        relays,
        identifier,
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
async function publishRsvp(nsec, relays, eventCoordinate, status, eventAuthor, clientNsec) {
    return await core.invoke("plugin:nostrnative|publish_rsvp", {
        nsec,
        relays,
        eventCoordinate,
        status,
        eventAuthor,
        clientNsec,
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
exports.compareRoutstrPrices = compareRoutstrPrices;
exports.createDiscoveryEvent = createDiscoveryEvent;
exports.createNote = createNote;
exports.createNotebook = createNotebook;
exports.decryptDiscoveryEvent = decryptDiscoveryEvent;
exports.decryptPnsEvent = decryptPnsEvent;
exports.deleteCalendar = deleteCalendar;
exports.deleteCalendarEvent = deleteCalendarEvent;
exports.deleteChatMessages = deleteChatMessages;
exports.deleteNfpData = deleteNfpData;
exports.deleteNote = deleteNote;
exports.deleteNotebook = deleteNotebook;
exports.deleteVirtualFs = deleteVirtualFs;
exports.derivePnsKeys = derivePnsKeys;
exports.discoverRoutstrProviders = discoverRoutstrProviders;
exports.fetchBookmarks = fetchBookmarks;
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
exports.parsePubkey = parsePubkey;
exports.publishBatchCalendarEvents = publishBatchCalendarEvents;
exports.publishCalendar = publishCalendar;
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
