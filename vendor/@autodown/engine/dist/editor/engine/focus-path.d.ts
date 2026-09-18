import { BlockNode } from '../../parser/block-model';
/** Ancestor ids of `focusedId` up to (excluding) the document root — the
 *  containers that render EXPANDED while the focus sits inside them. Every
 *  subtree hanging off this chain stays preview. */
export declare function focusPathOf(tree: BlockNode, focusedId: string): Set<string>;
/** The block that actually takes focus when `node` is selected: containers
 *  resolve to their first focusable descendant; leaves / edit faces stay.
 *  Null when the subtree has nothing focusable (e.g. a lone ThematicBreak). */
export declare function focusTargetOf(node: BlockNode): BlockNode | null;
/** Last focusable block of the subtree (Ctrl+End lands here): post-order,
 *  last child first. */
export declare function lastFocusTargetOf(node: BlockNode): BlockNode | null;
