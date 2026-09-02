/** One row of a QueryResultEnvelope (jade's /api/query QueryResponse item —
 *  marker/priority/content plus the source-label pair; only content is
 *  required). */
export interface QueryResultItem {
    marker?: string;
    priority?: number;
    content: string;
    title?: string;
    page_path?: string;
}
/** The query envelope: `{ results: QueryResultItem[] }` (jade's
 *  QueryResponse shape — normalizeQueryResults reads res.results). */
export interface QueryResultEnvelope {
    results: QueryResultItem[];
}
/** A loaded embedded block (jade's getBlock().block shape). */
export interface EmbeddedBlock {
    title?: string;
    content: string;
}
export type RunQueryFn = (q: string) => Promise<QueryResultEnvelope>;
export type LoadBlockFn = (id: string) => Promise<EmbeddedBlock | null>;
export interface DataLoaders {
    runQuery?: RunQueryFn;
    loadBlock?: LoadBlockFn;
}
/** Register (or clear with an empty object) the data loaders. Replaces the
 *  whole slot — the EngineEditor props watch passes both keys every time,
 *  a partial object intentionally leaves the other loader unset. */
export declare function setDataLoaders(next: DataLoaders): void;
/** The currently registered loaders (never null — read `.runQuery` /
 *  `.loadBlock` and undefined-check, the widget placeholder semantics). */
export declare function getDataLoaders(): DataLoaders;
/** Run fn with `next` as the active loaders, restoring the previous
 *  registration on exit (test seam; same window shape as
 *  pushNodeViewHost/popNodeViewHost). */
export declare function withDataLoaders<T>(next: DataLoaders, fn: () => T): T;
