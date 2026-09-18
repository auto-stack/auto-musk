import { BlockNode } from '../../parser/block-model';
/** Ancestor ids of `focusedId` up to (excluding) the document root — the
 *  containers that render EXPANDED while the focus sits inside them. Every
 *  subtree hanging off this chain stays preview. */
export declare function focusPathOf(tree: BlockNode, focusedId: string): Set<string>;
/** Composite container kinds (plan 042): their family edit face is a CARD
 *  whose children stay individually addressable — focus keeps descending to
 *  the deepest editable leaf, so one click lands the caret in the text (the
 *  atomic-face rule below applies to kinds like Fence/Table whose edit face
 *  is the whole block). Without this exemption the registered container edit
 *  slots would capture focus on the card itself and deep editing would need
 *  a second click. Exported for click-caret's wrapper pairing (same ruling,
 *  one source). */
export declare const COMPOSITE_CONTAINER_KINDS: Set<number>;
/** The block that actually takes focus when `node` is selected: containers
 *  resolve to their first focusable descendant; leaves / edit faces stay.
 *  Null when the subtree has nothing focusable (e.g. a lone ThematicBreak). */
export declare function focusTargetOf(node: BlockNode): BlockNode | null;
/** Last focusable block of the subtree (Ctrl+End lands here): post-order,
 *  last child first. */
export declare function lastFocusTargetOf(node: BlockNode): BlockNode | null;
