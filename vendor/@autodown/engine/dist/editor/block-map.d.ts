export declare const BLOCK_ID_PREFIX = "block-";
export interface BlockInfo {
    id: string;
    index: number;
    pos: number;
    el: HTMLElement;
    top: number;
    height: number;
}
/** Anchor block info from a rendered editor root element. */
export declare function getBlockMap(root?: HTMLElement | null): BlockInfo[];
