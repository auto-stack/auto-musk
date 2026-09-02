import { BlockNode } from '../../parser/block-model';
import { LoadBlockFn } from '../engine/data-loaders';
export interface EmbedSrcParsed {
    /** the page-reference part ('' for the pure-anchor form) */
    title: string;
    /** the bare block anchor id, null for a page-level reference */
    blockId: string | null;
}
export declare function parseEmbedSrc(src: string): EmbedSrcParsed;
/** The family node prop's src attr ('' when absent). */
export declare function embedSrcOf(node: BlockNode | undefined): string;
/** The parsed title part ('' for the pure-anchor form). */
export declare function embedTitle(node: BlockNode | undefined): string;
/** The parsed bare block id (null for a page-level reference). */
export declare function embedBlockId(node: BlockNode | undefined): string | null;
/** The registered block loader (null = the placeholder state — the
 *  EngineEditor loadBlock prop never arrived). */
export declare function blockLoader(): LoadBlockFn | null;
export declare function errorMessage(e: unknown): string;
