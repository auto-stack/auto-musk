import { Attr, BlockNode, Mark, Selection } from '../../parser/block-model';
import { EditorEngine } from './editor-engine';
/** Insert template blocks replacing the anchor block (slash templates). */
export declare function insertTemplate(engine: EditorEngine, anchorId: string, blocks: BlockNode[]): void;
/** Replace the selection's block with the given blocks. */
export declare function replaceSelection(engine: EditorEngine, blocks: BlockNode[]): void;
/** Focus a block at an offset; the host picks the move up. No history. */
export declare function focusBlock(engine: EditorEngine, id: string, offset?: number): void;
/** Table: add a row after `afterRowId` (null = first). One undo step. */
export declare function tableAddRow(engine: EditorEngine, tableId: string, afterRowId: string | null): void;
export declare function tableAddRowTree(tree: BlockNode, tableId: string, afterRowId: string | null): BlockNode;
/** Table: delete a row. One undo step. */
export declare function tableDeleteRow(engine: EditorEngine, rowId: string): void;
/** Table: append a column (empty cell on every row). One undo step. */
export declare function tableAddColumn(engine: EditorEngine, tableId: string): void;
export declare function tableAddColumnTree(tree: BlockNode, tableId: string): BlockNode;
/** Table: insert an empty column at `index` on every row (menu
 *  addColumnBefore/addColumnAfter — plan 026 P0T3). */
export declare function tableAddColumnAtTree(tree: BlockNode, tableId: string, index: number): BlockNode;
/** Table: delete column `index` from every row; refuses to empty the table
 *  (menu deleteColumn — plan 026 P0T3). */
export declare function tableDeleteColumnAtTree(tree: BlockNode, tableId: string, index: number): BlockNode;
/** Table: delete the last column of every row. One undo step. */
export declare function tableDeleteColumn(engine: EditorEngine, tableId: string): void;
/** Move a block up/down one position within its parent (drag parity). */
export declare function moveBlock(engine: EditorEngine, id: string, dir: -1 | 1): void;
/** Set attrs on a block (heading level etc. — completes the input rule). */
export declare function setBlockAttrs(engine: EditorEngine, id: string, attrs: Attr[]): void;
/** Toggle a mark over [lo, hi) of the block's inline text. One undo step. */
export declare function toggleMark(engine: EditorEngine, blockId: string, lo: number, hi: number, mark: Mark): void;
/** Link [lo, hi) to href (replacing any previous href). One undo step. */
export declare function setLink(engine: EditorEngine, blockId: string, lo: number, hi: number, href: string): void;
/** Marks active over the engine selection — the adapter isActive source.
 *  Cross-block selections collapse to the anchor position (v1: single-block). */
export declare function marksInRange(engine: EditorEngine, sel: Selection): Mark[];
/** Short persistent anchor id (7 base62 chars) — Obsidian-style, unlike the
 *  engine-internal `block-N` / `b-xxxxxx` fallbacks which never serialize. */
export declare function generateAnchorId(used: Set<string>): string;
/** Return the block's persistent anchor, assigning one on demand (copy-block
 *  link on a not-yet-anchored block). One undo step; emits change so the
 *  autosave persists the new anchor. Returns null when the id is unknown. */
export declare function ensureBlockAnchor(engine: EditorEngine, id: string): string | null;
