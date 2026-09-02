import { InlineSpan } from '../../parser/block-model';
export declare function escapeHtml(s: string): string;
/** The focused host's initial inner HTML. The v1 mark set is exactly the
 *  five inline elements; anything else renders as escaped plain text.
 *  Trailing spaces become &nbsp; — a collapsible trailing space in a
 *  contenteditable gets normalized away by the browser on the next edit
 *  (Chromium drops it mid-typing); nbsp keeps it until the blur walk
 *  normalizes back (plan 025 P2T1). */
export declare function spansToHtml(spans: InlineSpan[]): string;
/** Injected node description of a rich host subtree (headless-testable). */
export interface RichNode {
    tag?: string;
    text?: string;
    children?: RichNode[];
    attrs?: Record<string, string>;
}
/** Walk a rich-node tree collecting (text, marks, attrs) runs; adjacent
 *  same-format runs merge (normalizeSpans). Structure-only elements (br)
 *  contribute nothing. */
export declare function richTreeToSpans(root: RichNode): InlineSpan[];
/** Real-DOM adapter for richTreeToSpans (e2e-pinned). */
export declare function domRootToSpans(root: HTMLElement): InlineSpan[];
