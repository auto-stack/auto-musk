import { BlockNode, BlockType, Op } from '../../parser/block-model';
export interface InputRule {
    /** exact marker that triggers (typed at block start, whole-block) */
    marker: string;
    kind: BlockType;
    /** attr patch applied with the kind change (e.g. heading level) */
    level?: number;
    /** container semantics: wrap instead of converting the kind — ListBlock
     *  wraps as ListBlock>ListItem>block, Blockquote as Blockquote>block */
    wrap?: BlockType.ListBlock | BlockType.Blockquote;
}
export declare const INPUT_RULES: InputRule[];
/** Match a rule against the current block text (whole-block exact marker). */
export declare function matchInputRule(text: string): InputRule | null;
export interface InputRuleResult {
    /** op sequence for the engine: delete the marker, then convert the kind
     *  (container rules stop at the marker delete — the wrap is the after fn) */
    ops: Op[];
    /** attr patch applied after the ops (heading level etc.) */
    rule: InputRule;
}
/** Build the op sequence for a fired rule on the given block. */
export declare function inputRuleOps(tree: BlockNode, blockId: string, rule: InputRule): InputRuleResult | null;
/** Apply the attr patch after the ops ran (heading level etc.). */
export declare function applyRuleAttrs(tree: BlockNode, blockId: string, rule: InputRule): BlockNode;
/** Container wrap for a fired rule: paragraph → ListBlock>ListItem>paragraph
 *  (list markers) or Blockquote>paragraph (quote marker). The block id and
 *  its inline text are preserved — the host keeps editing the same block. */
export declare function applyRuleWrap(tree: BlockNode, blockId: string, rule: InputRule, ids: string[]): BlockNode;
/** Convenience orchestrator: fire the matching rule on the engine block as
 *  ONE undo step (marker ops + wrap/heading-level attr patch via tree fn). */
export declare function fireRuleOn(engine: {
    applyGroup(ops: Op[], after?: (tree: import('../../parser/block-model').BlockNode) => import('../../parser/block-model').BlockNode): void;
    doc: import('../../parser/block-model').BlockNode;
}, blockId: string): boolean;
