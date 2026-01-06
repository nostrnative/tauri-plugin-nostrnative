import { invoke } from '@tauri-apps/api/core';

async function generateNewNsec() {
    return await invoke('plugin:nostrnative|generate_new_nsec');
}
async function parsePubkey(pubkey) {
    return await invoke('plugin:nostrnative|parse_pubkey', { pubkey });
}
async function verifyNsec(nsec) {
    return await invoke('plugin:nostrnative|verify_nsec', { nsec });
}
// Calendar
async function fetchCalendarEvents(pubkey, relays, options) {
    return await invoke('plugin:nostrnative|fetch_calendar_events', {
        pubkey,
        nsec: options?.nsec,
        relays,
        rangeStart: options?.rangeStart,
        rangeEnd: options?.rangeEnd,
        authors: options?.authors
    });
}
async function publishCalendarEvent(nsec, relays, eventData) {
    return await invoke('plugin:nostrnative|publish_calendar_event', {
        nsec,
        relays,
        eventData
    });
}
// Profile
async function fetchProfiles(pubkeys, relays) {
    return await invoke('plugin:nostrnative|fetch_profiles', { pubkeys, relays });
}
// Blossom
async function blossomUpload(serverUrl, nsec, filePath) {
    return await invoke('plugin:nostrnative|blossom_upload', {
        serverUrl,
        nsec,
        filePath
    });
}
// Chat
async function publishChatMessage(pnsNsec, mainPubkey, msg, relays) {
    return await invoke('plugin:nostrnative|publish_chat_message', {
        pnsNsec,
        mainPubkey,
        msg,
        relays
    });
}

export { blossomUpload, fetchCalendarEvents, fetchProfiles, generateNewNsec, parsePubkey, publishCalendarEvent, publishChatMessage, verifyNsec };
