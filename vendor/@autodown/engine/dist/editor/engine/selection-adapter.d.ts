import { Mark } from '../../parser/block-model';
/** Flat-text selection inside one block (model spans coordinates). */
export interface TextRange {
    blockId: string;
    start: number;
    end: number;
}
/** The inline selection/mark verb face (D1 frozen interface). All verbs
 *  report success — false means no usable selection (no focused host,
 *  collapsed, or outside the host), the historical dom-marks no-op. */
export interface SelectionAdapter {
    /** The live selection as model coordinates, or null. */
    getSelection(): TextRange | null;
    /** True iff the whole selection sits inside one run of `mark`. */
    isActive(mark: Mark): boolean;
    /** Apply `mark` to the selection (Link takes the href; an existing link
     *  re-hrefs in place). Returns false when there is no usable selection. */
    applyMark(mark: Mark, href?: string): boolean;
    /** Remove `mark` from the selection (unwrap the enclosing run). */
    removeMark(mark: Mark): boolean;
}
export declare function setFocusedRichHost(el: HTMLElement | null): void;
export declare function getFocusedRichHost(): HTMLElement | null;
/** The DOM implementation: the retired dom-marks.ts bodies, byte-aligned.
 *  The focused host is registered by the ext bridge on focus; everything
 *  here is e2e-pinned (headless envs no-op through the null host slot). */
export declare const domSelectionAdapter: SelectionAdapter;
/** The old domToggleMark decision (isActive ? remove : apply) — a module
 *  convenience for the call sites, deliberately OUTSIDE the frozen
 *  four-method interface (D1). */
export declare function toggleMark(adapter: SelectionAdapter, mark: Mark): boolean;
