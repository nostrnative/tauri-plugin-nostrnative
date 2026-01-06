export declare function generateNewNsec(): Promise<string>;
export declare function parsePubkey(pubkey: string): Promise<string>;
export declare function verifyNsec(nsec: string): Promise<string>;
export declare function fetchCalendarEvents(pubkey: string, relays: string[], options?: {
    nsec?: string;
    rangeStart?: number;
    rangeEnd?: number;
    authors?: string[];
}): Promise<any[]>;
export declare function publishCalendarEvent(nsec: string, relays: string[], eventData: any): Promise<string>;
export declare function fetchProfiles(pubkeys: string[], relays: string[]): Promise<any[]>;
export declare function blossomUpload(serverUrl: string, nsec: string, filePath: string): Promise<string>;
export declare function publishChatMessage(pnsNsec: string, mainPubkey: string, msg: any, relays: string[]): Promise<string>;
