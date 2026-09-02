export type WikilinkOpener = (title: string, blockId?: string) => void;
/** Register (or clear with null) the app-facing wikilink click handler. */
export declare function registerWikilinkOpener(open: WikilinkOpener | null): void;
/** The currently registered handler (identity checks on unmount). */
export declare function currentWikilinkOpener(): WikilinkOpener | null;
export declare function openWikilink(title: string, blockId?: string): void;
