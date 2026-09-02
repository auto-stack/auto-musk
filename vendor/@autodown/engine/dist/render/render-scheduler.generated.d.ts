/**
 * @autodown/vue — render scheduler decisions (batch / live-window /
 * typewriter stepping).
 *
 * GENERATED FILE — do not edit by hand.
 * Source: auto/render_scheduler.at (Auto language). Regenerate with: pnpm gen
 * (see auto/README.md for the pipeline and the applied post-fixes)
 */
export declare function nextBatchCount(visible: number, total: number, batchSize: number): number;
export declare function liveWindowStart(visibleEnd: number, maxLive: number): number;
export declare function typewriterNextChars(visible: number, total: number, chunk: number): number;
