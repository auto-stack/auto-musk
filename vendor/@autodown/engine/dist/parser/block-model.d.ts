/**
 * AutoDown Core — unified block model (block tree, selection, op sequence).
 *
 * GENERATED FILE — do not edit by hand.
 * Source: auto/block_model.at (Auto language). Regenerate with: pnpm gen
 * (see auto/README.md for the pipeline and the applied post-fixes)
 */
export declare class SourceRange {
    start: number;
    end: number;
    constructor(start: number, end: number);
}
export declare function rng(s: number, e: number): SourceRange;
export type Value = {
    _tag: "Null";
} | {
    _tag: "Str";
    value: string;
} | {
    _tag: "Int";
    value: number;
} | {
    _tag: "Bool";
    value: boolean;
} | {
    _tag: "ListV";
    value: Value[];
} | {
    _tag: "AttrsV";
    value: Attr[];
};
export declare const Value: {
    Null: () => {
        _tag: "Null";
    };
    Str: (value: string) => {
        _tag: "Str";
        value: string;
    };
    Int: (value: number) => {
        _tag: "Int";
        value: number;
    };
    Bool: (value: boolean) => {
        _tag: "Bool";
        value: boolean;
    };
    ListV: (value: Value[]) => {
        _tag: "ListV";
        value: Value[];
    };
    AttrsV: (value: Attr[]) => {
        _tag: "AttrsV";
        value: Attr[];
    };
};
export declare class Attr {
    key: string;
    value: Value;
    constructor(key: string, value: Value);
}
export declare function attrGet(attrs: Attr[], key: string): Value | null;
export declare function attrGetStr(attrs: Attr[], key: string, dflt: string): string;
export declare function attrGetInt(attrs: Attr[], key: string, dflt: number): number;
export declare function attrGetBool(attrs: Attr[], key: string, dflt: boolean): boolean;
export declare function attrSet(attrs: Attr[], key: string, value: Value): Attr[];
export declare function attrDel(attrs: Attr[], key: string): Attr[];
export declare function dupAttrs(attrs: Attr[]): Attr[];
export declare enum Mark {
    Strong = 0,
    Em = 1,
    Code = 2,
    Link = 3,
    Image = 4,
    Del = 5,
    Underline = 6
}
export declare function hasMark(marks: Mark[], m: Mark): boolean;
export declare function addMark(marks: Mark[], m: Mark): Mark[];
export declare function delMark(marks: Mark[], m: Mark): Mark[];
export declare class InlineSpan {
    text: string;
    marks: Mark[];
    attrs: Attr[];
    constructor(text: string, marks: Mark[], attrs: Attr[]);
}
export declare function span(text: string): InlineSpan;
export declare function markedSpan(text: string, marks: Mark[]): InlineSpan;
export declare function spanWith(text: string, marks: Mark[], attrs: Attr[]): InlineSpan;
export declare function spansText(spans: InlineSpan[]): string;
export declare function dupSpans(spans: InlineSpan[]): InlineSpan[];
export declare function spansInsert(spans: InlineSpan[], offset: number, text: string): InlineSpan[];
export declare function spansDelete(spans: InlineSpan[], lo: number, hi: number): InlineSpan[];
export declare class SpanSplit {
    before: InlineSpan[];
    after: InlineSpan[];
    constructor(before: InlineSpan[], after: InlineSpan[]);
}
export declare function spansSplitAt(spans: InlineSpan[], offset: number): SpanSplit;
export declare enum BlockType {
    Heading = 0,
    Paragraph = 1,
    Fence = 2,
    Blockquote = 3,
    ListBlock = 4,
    ListItem = 5,
    Table = 6,
    TableRow = 7,
    TableCell = 8,
    ThematicBreak = 9,
    Callout = 10,
    Details = 11,
    WikilinkBlock = 12,
    QueryBlock = 13,
    BlockEmbed = 14,
    Mermaid = 15,
    MathBlock = 16
}
export declare class BlockNode {
    id: string;
    kind: BlockType;
    attrs: Attr[];
    children: BlockNode[];
    inlines: InlineSpan[];
    source: SourceRange;
    constructor(id: string, kind: BlockType, attrs: Attr[], children: BlockNode[], inlines: InlineSpan[], source: SourceRange);
}
export declare function block(id: string, kind: BlockType): BlockNode;
export declare function blockFull(id: string, kind: BlockType, attrs: Attr[], children: BlockNode[], inlines: InlineSpan[], source: SourceRange): BlockNode;
export declare function attrOf(key: string, value: Value): Attr;
export declare function leafBlock(id: string, kind: BlockType, text: string): BlockNode;
export declare function blockText(node: BlockNode): string;
export declare function withInlines(node: BlockNode, spans: InlineSpan[]): BlockNode;
export declare function withKind(node: BlockNode, kind: BlockType): BlockNode;
export declare function withChildren(node: BlockNode, kids: BlockNode[]): BlockNode;
export declare function dupNodes(nodes: BlockNode[]): BlockNode[];
export declare function anchorOf(node: BlockNode): string;
export declare function withBlockAnchor(node: BlockNode, newId: string): BlockNode;
export declare function withIdAndAnchor(node: BlockNode, newId: string): BlockNode;
export declare function hasIdDeep(tree: BlockNode, id: string): boolean;
export declare function retargetAnchor(tree: BlockNode, id: string, newId: string): BlockNode;
export declare function findBlock(node: BlockNode, id: string): BlockNode | null;
export declare function parentOf(node: BlockNode, id: string): BlockNode | null;
export declare function pathOf(node: BlockNode, id: string): string[];
export declare function childIndex(node: BlockNode, id: string): number;
export declare function spliceChildren(node: BlockNode, id: string, repl: BlockNode[]): BlockNode;
export declare function replaceNode(node: BlockNode, id: string, repl: BlockNode[]): BlockNode;
export declare function spliceRange(node: BlockNode, parentId: string, lo: number, hi: number, repl: BlockNode[]): BlockNode;
export declare class BlockPos {
    blockId: string;
    offset: number;
    constructor(blockId: string, offset: number);
}
export declare class Selection {
    anchor: BlockPos;
    head: BlockPos;
    constructor(anchor: BlockPos, head: BlockPos);
}
export declare function collapsedSel(blockId: string, offset: number): Selection;
export declare function pos(blockId: string, offset: number): BlockPos;
export declare class InsertTextOp {
    pos: BlockPos;
    text: string;
    constructor(pos: BlockPos, text: string);
}
export declare class SplitBlockOp {
    pos: BlockPos;
    newId: string;
    constructor(pos: BlockPos, newId: string);
}
export declare class MergeBlocksOp {
    aId: string;
    bId: string;
    constructor(aId: string, bId: string);
}
export declare class SetBlockTypeOp {
    id: string;
    kind: BlockType;
    constructor(id: string, kind: BlockType);
}
export declare class LiftBlockOp {
    id: string;
    constructor(id: string);
}
export declare class WrapBlockOp {
    id: string;
    kind: BlockType;
    newId: string;
    constructor(id: string, kind: BlockType, newId: string);
}
export declare class ReplaceRangeOp {
    sel: Selection;
    text: string;
    constructor(sel: Selection, text: string);
}
export type Op = {
    _tag: "InsertText";
    value: InsertTextOp;
} | {
    _tag: "SplitBlock";
    value: SplitBlockOp;
} | {
    _tag: "MergeBlocks";
    value: MergeBlocksOp;
} | {
    _tag: "SetBlockType";
    value: SetBlockTypeOp;
} | {
    _tag: "LiftBlock";
    value: LiftBlockOp;
} | {
    _tag: "WrapBlock";
    value: WrapBlockOp;
} | {
    _tag: "ReplaceRange";
    value: ReplaceRangeOp;
};
export declare const Op: {
    InsertText: (value: InsertTextOp) => {
        _tag: "InsertText";
        value: InsertTextOp;
    };
    SplitBlock: (value: SplitBlockOp) => {
        _tag: "SplitBlock";
        value: SplitBlockOp;
    };
    MergeBlocks: (value: MergeBlocksOp) => {
        _tag: "MergeBlocks";
        value: MergeBlocksOp;
    };
    SetBlockType: (value: SetBlockTypeOp) => {
        _tag: "SetBlockType";
        value: SetBlockTypeOp;
    };
    LiftBlock: (value: LiftBlockOp) => {
        _tag: "LiftBlock";
        value: LiftBlockOp;
    };
    WrapBlock: (value: WrapBlockOp) => {
        _tag: "WrapBlock";
        value: WrapBlockOp;
    };
    ReplaceRange: (value: ReplaceRangeOp) => {
        _tag: "ReplaceRange";
        value: ReplaceRangeOp;
    };
};
export declare class EditResult {
    tree: BlockNode;
    selection: Selection;
    constructor(tree: BlockNode, selection: Selection);
}
export declare function missingBlock(): BlockNode;
export declare function applyOp(tree: BlockNode, selection: Selection, op: Op): EditResult;
export declare function textInRange(tree: BlockNode, sel: Selection): string;
export declare function invertOp(tree: BlockNode, op: Op): Op;
