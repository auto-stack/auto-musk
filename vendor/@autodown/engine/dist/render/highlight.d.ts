export type HighlightFn = (code: string, language: string) => string | undefined;
/** Bind the platform implementation. Passing null reverts to "no impl"
 *  (the Vue default is then resolved by the caller — see resolveHighlighter). */
export declare function setHighlightImpl(impl: HighlightFn | null): void;
/** Currently bound platform implementation, if any. */
export declare function getHighlightImpl(): HighlightFn | null;
