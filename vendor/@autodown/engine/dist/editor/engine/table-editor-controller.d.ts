import { BlockNode } from '../../parser/block-model';
import { EditorEngine } from './editor-engine';
export declare class TableEditorController {
    private engine;
    private tableId;
    constructor(engine: EditorEngine, tableId: string);
    table(): BlockNode | null;
    get rows(): BlockNode[];
    cellText(cellId: string): string;
    /** Append an empty row after the last row (or after the header when only
     *  the header exists). One undo step. */
    addRow(): void;
    /** Insert an empty row ABOVE the header (TableMenu absorption, plan 026
     *  adjudication #1 — single table entry). One undo step. */
    addRowAbove(): void;
    /** Insert an empty column at index 0. One undo step. */
    addColumnBefore(): void;
    /** Remove the whole table; the dangling selection collapses to the first
     *  block (the menu chain's deleteTable repair, same semantics). */
    deleteTable(): void;
    /** Remove the last row. Refused (no-op) when only the header remains. */
    deleteRow(): boolean;
    /** Append an empty column. One undo step. */
    addColumn(): void;
    /** Remove the last column. Refused (no-op) below one column. */
    deleteColumn(): boolean;
    /** Cell blur-commit: old→new text as one diff op (BlockHost protocol).
     *  The selection stays anchored on the TABLE — the op's position points at
     *  the cell and applyOp would otherwise drag the anchor into it, dropping
     *  the top-level focus that assembles this editing face (found live in the
     *  demo: committing a cell unmounted the table editor).
     *  Returns false when the text is unchanged or the cell is gone. */
    commitCell(cellId: string, newText: string): boolean;
}
