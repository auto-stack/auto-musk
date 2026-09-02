import { EditorEngine } from './editor-engine';
/** Enter inside a list item: split the tail into a following ListItem; an
 *  empty item exits the list instead (paragraph lands after it). */
export declare function enterInItem(engine: EditorEngine, paragraphId: string, offset: number): void;
/** Backspace at offset 0 of an item paragraph: merge into the previous item's
 *  last paragraph (caret at the junction); a first item lifts out in place. */
export declare function backspaceAtItemStart(engine: EditorEngine, paragraphId: string): void;
/** Tab: move the item into the previous item's nested list (created on
 *  demand, copying ordered/start). The first item cannot indent (no-op). */
export declare function indentItem(engine: EditorEngine, paragraphId: string): void;
/** Shift+Tab: lift the item into the grandparent list right after the item
 *  that held its nested list. A top-level item cannot outdent (no-op). */
export declare function outdentItem(engine: EditorEngine, paragraphId: string): void;
/** Enter inside a blockquote paragraph: split into a continuation paragraph;
 *  an empty paragraph exits the quote instead. */
export declare function enterInQuote(engine: EditorEngine, paragraphId: string, offset: number): void;
/** Lift the paragraph out of the quote (after it); an emptied quote dissolves. */
export declare function exitQuote(engine: EditorEngine, paragraphId: string): void;
