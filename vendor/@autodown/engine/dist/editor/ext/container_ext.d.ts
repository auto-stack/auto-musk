import { BlockNode } from '../../parser/block-model';
export { BlockChildren } from '../components/BlockChildren';
export { default as AttrHost } from '../components/AttrHost.vue';
export { ctxReadonly, ctxBlockId, htmlText } from './code_block_widget_ext';
/** The ctx's engine — the controller-prop idiom (engine passed wide-typed,
 *  the 033 CodeBlockWidget fenceEditSlot ruling: controller = engine). */
export declare function ctxEngine(ctx: unknown): unknown;
/** One string attr off the model node ('' when absent). */
export declare function nodeAttrStr(node: BlockNode | undefined, key: string): string;
/** The builtin renderCalloutPanel known-type check (shared list). */
export declare function calloutTypeKnown(type: string): boolean;
/** One bool attr off the model node (default false). */
export declare function nodeAttrBool(node: BlockNode | undefined, key: string): boolean;
/** One int attr off the model node (default 1 — the list start). */
export declare function nodeAttrInt(node: BlockNode | undefined, key: string): number;
/** The edit face's ordered-list start attr: present only in edit mode on an
 *  ordered list (the builtin renderListPanel view carries no start attr —
 *  returning undefined omits it, the Vue attr rule). */
export declare function editOrderedStart(mode: string, ordered: boolean, start: number): number | undefined;
/** The task checkbox verb (edit face): flip `checked` through setBlockAttrs
 *  as ONE undo step — expandedElement's toggleTaskChecked, stopPropagation
 *  riding the DSL modifier. */
export declare function toggleTaskChecked(controller: unknown, itemId: string): void;
/** The block id a container verb addresses: the edit ctx's blockId, falling
 *  back to the model node's own id (the panel path passes no ctx). */
export declare function blockRef(node: BlockNode | undefined, ctx: unknown): string;
/** The details marker verb (both faces): flip `open` through setBlockAttrs
 *  as ONE undo step — the expandedElement inline onClick semantics, with
 *  stopPropagation riding the DSL modifier. */
export declare function toggleDetailsOpen(controller: unknown, blockId: string, open: boolean): void;
