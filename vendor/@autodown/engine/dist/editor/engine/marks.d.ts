import { InlineSpan, Mark } from '../../parser/block-model';
/** Merge adjacent spans with equal marks+attrs; drop empty fragments. */
export declare function normalizeSpans(spans: InlineSpan[]): InlineSpan[];
/** Marks carried by every char of [lo, hi) — the isActive semantics source.
 *  A collapsed range reads the span enclosing the offset. */
export declare function marksAtRange(spans: InlineSpan[], lo: number, hi: number): Mark[];
/** Toggle a mark over [lo, hi): remove it when every char in range already
 *  has it, add it otherwise. Returns the input reference for a no-op range. */
export declare function toggleMarkOnSpans(spans: InlineSpan[], lo: number, hi: number, mark: Mark): InlineSpan[];
/** Link [lo, hi): set the Link mark and (re)point href, keeping other marks. */
export declare function setLinkOnSpans(spans: InlineSpan[], lo: number, hi: number, href: string): InlineSpan[];
