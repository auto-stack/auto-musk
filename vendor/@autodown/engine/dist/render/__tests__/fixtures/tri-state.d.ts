export interface TriStateDoc {
    /** streaming prefix: construct not yet recognizable as its final kind */
    unclosed: string | null;
    /** streaming prefix: construct open but incomplete (fence 族 / table) */
    open: string | null;
    /** complete construct */
    closed: string | null;
    /** WNode type parseDocument(closed, true) must yield at top level */
    closedKind: string | null;
    /** container member — no standalone stream state (states ride the container) */
    ridesContainer?: boolean;
}
/** The 17 BlockType kinds, in BlockType enum order (block-model.ts). */
export declare const TRI_STATE_KINDS: readonly string[];
export declare const TRI_STATE: Record<string, TriStateDoc>;
