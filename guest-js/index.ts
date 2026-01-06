import { invoke } from '@tauri-apps/api/core'

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
  options?: { nsec?: string; rangeStart?: number; rangeEnd?: number; authors?: string[] }
): Promise<any[]> {
  return await invoke('plugin:nostrnative|fetch_calendar_events', {
    pubkey,
    nsec: options?.nsec,
    relays,
    rangeStart: options?.rangeStart,
    rangeEnd: options?.rangeEnd,
    authors: options?.authors
  })
}

export async function publishCalendarEvent(
  nsec: string,
  relays: string[],
  eventData: any
): Promise<string> {
  return await invoke('plugin:nostrnative|publish_calendar_event', {
    nsec,
    relays,
    eventData
  })
}

// Profile
export async function fetchProfiles(
  pubkeys: string[],
  relays: string[]
): Promise<any[]> {
  return await invoke('plugin:nostrnative|fetch_profiles', { pubkeys, relays })
}

// Blossom
export async function blossomUpload(
  serverUrl: string,
  nsec: string,
  filePath: string
): Promise<string> {
  return await invoke('plugin:nostrnative|blossom_upload', {
    serverUrl,
    nsec,
    filePath
  })
}

// Chat
export async function publishChatMessage(
  pnsNsec: string,
  mainPubkey: string,
  msg: any,
  relays: string[]
): Promise<string> {
  return await invoke('plugin:nostrnative|publish_chat_message', {
    pnsNsec,
    mainPubkey,
    msg,
    relays
  })
}
