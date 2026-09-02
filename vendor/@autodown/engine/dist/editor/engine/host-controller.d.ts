import { BlockNode, InlineSpan, Op } from '../../parser/block-model';
import { EditorEngine } from './editor-engine';
import { CompositionSession } from './composition';
export declare class BlockHostController {
    private engine;
    private blockId;
    private knownText;
    readonly composition: CompositionSession;
    constructor(engine: EditorEngine, blockId: string);
    get id(): string;
    get text(): string;
    /** The block's inline spans (rich host mount render — plan 024 P2T1). */
    get inlines(): InlineSpan[];
    /** The host was (re)rendered from the engine — re-sync the known text
     *  (history changes repaint the host). */
    syncFromModel(): string;
    /** `input` DOM event outside composition: old→new text becomes one op. */
    onInput(newText: string): Op | null;
    /** Enter key at caret offset → split the block. Nested paragraphs dispatch
     *  on the parent kind first (plan 025 P1T3): a ListItem parent splits the
     *  ITEM, a Blockquote parent continues the quote; only top-level leaves
     *  take the bare SplitBlock path. */
    onEnter(offset: number, newId: string): void;
    /** Backspace at offset 0 → merge with the previous sibling (if any). In a
     *  list item the structural command owns the semantics (merge into the
     *  previous ITEM / lift the first item out); elsewhere the merge target
     *  must be an editable leaf of the same container — a container sibling
     *  (nested list subtree) never merges. */
    onBackspaceAtStart(previousSiblingId: string | null): boolean;
    /** Tab / Shift+Tab inside a list item → indent / outdent (plan 025 P1T3).
     *  Returns false (browser default) when the block is not in a list. */
    onTab(shift: boolean): boolean;
    compositionBegin(baseline: string, offset: number): void;
    compositionUpdate(preedit: string): void;
    compositionCommit(finalText: string): Op | null;
    compositionCancel(): void;
    /** Markdown / multiline paste: parse to blocks and insert after this one
     *  (plan 018 目标 5 — paste is v1-mandatory; HTML paste degrades to
     *  text/plain per 待澄清 5). */
    onPasteMarkdown(md: string): void;
    /** Focus-leave writeback of the rich host: DOM walk → spans → whole-block
     *  withInlines through applyTree — ONE undo step, CodeEditorBlock protocol.
     *  Returns true when a rewrite landed. */
    onRichBlur(domRoot: HTMLElement): boolean;
    /** Headless core of onRichBlur (the walk itself is e2e-pinned). Blocks
     *  carrying Image marks are skipped: their marks are not rendered in the
     *  rich host, so a rewrite would silently drop them (v1 no-data-loss). */
    commitRichSpans(spans: InlineSpan[]): boolean;
}
/** Is this block a leaf the host can edit (has inline text, no children)?
 *  Container kinds (Details/Callout — children-based) and attr-only blocks
 *  (Query/Embed — source lives in attrs) never host: typing into them
 *  serializes nowhere; their focus state is the preview-side node-view
 *  (plan 026 P2T3). Math/Mermaid stay hostable — their source IS inlines. */
export declare function isEditableLeaf(node: BlockNode): boolean;
