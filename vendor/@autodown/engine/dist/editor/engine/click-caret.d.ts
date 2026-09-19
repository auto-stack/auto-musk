import { BlockNode } from '../../parser/block-model';
export interface ClickCaret {
    /** block that will take focus (the consuming face's block id) */
    blockId: string;
    /** capture time — stale handoffs expire (see STALE_MS) */
    at: number;
    /** point payload: click position relative to the block's rendered element */
    dx?: number;
    dy?: number;
    /** offset payload: plaintext offset within the block's text */
    offset?: number;
    /** table faces: the cell block to focus inside the mounted face */
    cellId?: string;
}
/** Record a handoff. Always replaces the previous entry: the click that is
 *  about to remount a face is the only one that can be pending. */
export declare function setClickCaret(entry: ClickCaret): void;
export declare function clearClickCaret(): void;
/** Consume (and clear) the pending handoff for `blockId`. Returns null when
 *  nothing matches — the caller keeps its own default. */
export declare function takeClickCaret(blockId: string): ClickCaret | null;
/** Capture the pointed-at position of a slot click: the block's own rendered
 *  element is the anchor, so the point survives the face swap even when the
 *  page shifts (the focused face mounts inside the SAME slot chrome as its
 *  preview twin — plan 039 T4b — so the anchor geometry is stable). `cellId`
 *  rides the table face's handoff (the cell is its caret unit). */
export declare function captureClickCaret(ev: MouseEvent, blockId: string, anchor: Element | null, cellId?: string): void;
/** The block's own rendered element inside a slot's chrome
 *  (slot > .node-content > element). */
export declare function slotAnchorOf(slotEl: Element | null): HTMLElement | null;
/** Place the DOM caret at a captured point inside the freshly mounted edit
 *  face, letting the browser re-resolve the glyph from the point (the faces
 *  are layout-identical by design, so the point lands on the same glyph the
 *  user clicked). False when the point resolves outside the host — stale
 *  geometry, or a container slot whose anchor is not this face's element —
 *  and the caller falls back to its default. */
export declare function placeCaretAtPoint(host: HTMLElement, entry: ClickCaret): boolean;
/** What a slot click addresses: the block whose edit face will mount (and
 *  consume the handoff), the element the point is recorded against (null =
 *  no point handoff), and the table cell to focus. */
export interface ClickHit {
    targetId: string;
    anchor: HTMLElement | null;
    cellId?: string;
}
/** The block a click inside a slot addresses. Leaf slots keep the node
 *  itself; a table slot addresses the clicked cell; a collapsed container
 *  slot addresses the atomic descendant whose wrapper the point landed on
 *  (targetId '' = unresolved — the caller keeps deep selection). */
export declare function resolveClickHit(node: BlockNode, slotEl: Element | null, ev: MouseEvent): ClickHit;
