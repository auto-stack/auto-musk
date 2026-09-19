import { BlockNode } from '../../parser/block-model';
import { EditorEngine } from './editor-engine';
export declare class CodeEditorController {
    private engine;
    private blockId;
    private knownCode;
    constructor(engine: EditorEngine, blockId: string);
    get id(): string;
    get code(): string;
    /** The live block (attrs included) — the SFC reads the language from it. */
    node(): BlockNode | null;
    /** The engine repaints after history changes / external edits — re-sync. */
    syncFromModel(): string;
    /** Write the edited code text back; false = no change or block gone. */
    commit(newCode: string): boolean;
    /** The code widget's commit: the edit face drafts the COLLAPSED text
     *  (draftCodeOf strips the closed-fence representation newline), so the
     *  model representation is restored here before the compare+write — an
     *  untouched blur stays a no-op, a real edit always wins, and a trailing
     *  newline the user actually typed becomes model content on top of the
     *  representation. Math/mermaid draft the model text verbatim and keep
     *  using commit. */
    commitDraft(newCode: string): boolean;
    private readModel;
}
