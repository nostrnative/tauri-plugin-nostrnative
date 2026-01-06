'use strict';

var core = require('@tauri-apps/api/core');

async function generateNewNsec() {
    return await core.invoke('plugin:nostrnative|generate_new_nsec');
}
async function parsePubkey(pubkey) {
    return await core.invoke('plugin:nostrnative|parse_pubkey', { pubkey });
}
async function verifyNsec(nsec) {
    return await core.invoke('plugin:nostrnative|verify_nsec', { nsec });
}
// Calendar
async function fetchCalendarEvents(pubkey, relays, options) {
    return await core.invoke('plugin:nostrnative|fetch_calendar_events', {
        pubkey,
        nsec: options?.nsec,
        relays,
        rangeStart: options?.rangeStart,
        rangeEnd: options?.rangeEnd,
        authors: options?.authors
    });
}
async function publishCalendarEvent(nsec, relays, eventData) {
    return await core.invoke('plugin:nostrnative|publish_calendar_event', {
        nsec,
        relays,
        eventData
    });
}
// Profile
async function fetchProfiles(pubkeys, relays) {
    return await core.invoke('plugin:nostrnative|fetch_profiles', { pubkeys, relays });
}
// Blossom
async function blossomUpload(serverUrl, nsec, filePath) {
    return await core.invoke('plugin:nostrnative|blossom_upload', {
        serverUrl,
        nsec,
        filePath
    });
}
// Chat
async function publishChatMessage(pnsNsec, mainPubkey, msg, relays) {
    return await core.invoke('plugin:nostrnative|publish_chat_message', {
        pnsNsec,
        mainPubkey,
        msg,
        relays
    });
}

exports.blossomUpload = blossomUpload;
exports.fetchCalendarEvents = fetchCalendarEvents;
exports.fetchProfiles = fetchProfiles;
exports.generateNewNsec = generateNewNsec;
exports.parsePubkey = parsePubkey;
exports.publishCalendarEvent = publishCalendarEvent;
exports.publishChatMessage = publishChatMessage;
exports.verifyNsec = verifyNsec;
