import { BlockNode, EditResult, Op, Selection } from '../../parser/block-model';
export interface EngineChange {
    tree: BlockNode;
    selection: Selection;
    /** true when this change came from undo/redo (hosts usually skip echoing) */
    history: boolean;
}
export type EngineListener = (change: EngineChange) => void;
export declare class EditorEngine {
    private tree;
    private sel;
    private undoStack;
    private redoStack;
    private listeners;
    constructor(tree: BlockNode, sel?: Selection);
    get doc(): BlockNode;
    get selection(): Selection;
    get canUndo(): boolean;
    get canRedo(): boolean;
    onChange(fn: EngineListener): void;
    /** Apply one op through the 016 kernel, recording an undo entry.
     *  Adjacent InsertText typing coalesces into the previous entry. */
    apply(op: Op, opts?: {
        coalesce?: boolean;
    }): EditResult;
    /** Apply a composed op group as ONE undo step (input rules etc.). */
    applyGroup(ops: Op[], after?: (tree: BlockNode) => BlockNode): void;
    /** Apply a pure tree transform as ONE undo step (command layer:
     *  insertTemplate / table ops / moveBlock — Phase 3). */
    applyTree(fn: (tree: BlockNode) => BlockNode): void;
    /** Set the selection without a document change (focus moves). */
    select(sel: Selection): void;
    private thread;
    undo(): boolean;
    redo(): boolean;
    /** Streaming append (plan 018 待澄清 1 — 追加分流裁定): AI/stream blocks
     *  land at the document tail without touching the focused block or the
     *  selection; not an undoable user edit. */
    appendBlocks(blocks: BlockNode[]): void;
    /** External document replacement (file load, full paste). Not undoable —
     *  callers that need undo wrap it in their own op. */
    replaceDoc(tree: BlockNode, sel?: Selection): void;
    private emit;
}
