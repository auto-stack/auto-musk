import { Mark } from '../../parser/block-model';
import { EditorEngine } from './editor-engine';
export interface InlineRule {
    marker: string;
    mark: Mark;
}
/** Longest markers first: `**` must be tried before `*` so `**a**` fires as
 *  Strong, and `~~` before anything of its prefix family. Backtick has no
 *  prefix family (pairs are unambiguous). */
export declare const INLINE_INPUT_RULES: InlineRule[];
export interface InlineRuleMatch {
    /** [start, end) covers marker + inner + marker — the region to rewrite. */
    start: number;
    end: number;
    /** where the inner text lands after the markers are stripped */
    innerStart: number;
    innerLen: number;
    mark: Mark;
    marker: string;
}
/** Match the inline rule whose CLOSING marker just completed: the marker's
 *  last char sits at caret-1. Scan left for the last unescaped opener with
 *  non-empty inner and no newline in the region. Returns null otherwise. */
export declare function matchInlineRule(text: string, caret: number): InlineRuleMatch | null;
/** Fire the inline rule that just closed at the engine caret: strip both
 *  markers, mark the inner text, park the caret after the mark — all as ONE
 *  undo step that reverts to the typed-marker text (AC-03). Returns false
 *  when nothing matched or the Code guard rejects the region. */
export declare function fireInlineRuleOn(engine: EditorEngine, blockId: string): boolean;
