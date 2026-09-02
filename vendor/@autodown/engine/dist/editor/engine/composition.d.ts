import { Op } from '../../parser/block-model';
export declare class CompositionSession {
    private active;
    private baseline;
    private blockId;
    private baselineOffset;
    get composing(): boolean;
    /** compositionstart — record the pre-edit state of the focused block. */
    begin(blockId: string, baseline: string, offset: number): void;
    /** compositionupdate — staged preedit; produces NO op by contract. */
    update(_preedit: string): Op | null;
    /** compositionend — diff baseline → final text into one op. */
    commit(finalText: string): Op | null;
    /** composition cancelled — nothing happened, by contract. */
    cancel(): Op | null;
}
