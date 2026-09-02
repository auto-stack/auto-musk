/** The artifact cache key: `kind:<utf16 len of source>:<8-hex FNV-1a>`.
 *  Same (kind, source) -> same key on TS and rust (VM/iced disk caches
 *  read what the web side wrote). */
export declare function artifactHash(kind: string, source: string): string;
