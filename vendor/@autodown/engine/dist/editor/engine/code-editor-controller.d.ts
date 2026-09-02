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
    private readModel;
}
