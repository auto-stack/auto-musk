export interface WalkNode {
    readonly raw: unknown;
    readonly isText: boolean;
    readonly text: string;
    readonly children: WalkNode[];
}
/** Minimal structural description of a rich host (test / injectable input). */
export interface MiniNode {
    text?: string;
    children?: MiniNode[];
}
export declare function walkFromMini(m: MiniNode): WalkNode;
/** Model offset of a DOM point (container identity + DOM offset). Text nodes
 *  map through their accumulated prefix; element offsets resolve the child
 *  boundary to the nearest text position. -1 = not under this host / no text. */
export declare function pointOffset(root: WalkNode, containerRaw: unknown, domOffset: number): number;
/** DOM anchor for a model offset: {raw text node, inner offset}. Boundaries
 *  anchor at the end of the earlier leaf. Null only for a textless host. */
export declare function offsetPoint(root: WalkNode, offset: number): {
    raw: unknown;
    inner: number;
} | null;
export interface BlockRange {
    blockId: string;
    lo: number;
    hi: number;
}
/** Current window selection as a single-host block range; null when the
 *  selection is empty, outside the host, or crosses hosts (v1: single-block). */
export declare function domRangeToBlockRange(hostEl: HTMLElement, blockId: string): BlockRange | null;
/** Reverse mapping: build a DOM Range for [lo, hi) inside the host. */
export declare function blockRangeToDomRange(hostEl: HTMLElement, lo: number, hi: number): Range;
