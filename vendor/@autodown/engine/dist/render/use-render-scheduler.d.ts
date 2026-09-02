import { Ref } from 'vue';
export interface SchedulerTimer {
    setTimeout(fn: () => void, ms: number): unknown;
    clearTimeout(handle: unknown): void;
}
export interface RenderSchedulerOptions {
    /** progressive batching enabled (false renders everything immediately) */
    enabled: boolean;
    /** nodes rendered per tick */
    batchSize: number;
    /** ms between batch ticks */
    batchDelay: number;
    /** max simultaneously mounted nodes (<= 0 disables windowing) */
    maxLiveNodes: number;
    /** typewriter effect on the last text-bearing node */
    typewriter: boolean;
    /** characters revealed per typewriter tick */
    typewriterChunk: number;
    timer?: SchedulerTimer;
}
/**
 * Drives progressive rendering of a parsed node array. Exposes:
 * - `visibleNodes`: the windowed slice that should be mounted
 * - `typewriterChars`: characters of the last node's flattened text that
 *   are revealed (Infinity when the typewriter is off / finished)
 */
export declare function useRenderScheduler(nodes: Ref<any[]>, opts: RenderSchedulerOptions): {
    visibleNodes: import('vue').ComputedRef<any[]>;
    visibleCount: Ref<number, number>;
    typewriterChars: Ref<number, number>;
    windowStart: import('vue').ComputedRef<number>;
};
