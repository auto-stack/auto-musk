import { EditorEngine } from './editor-engine';
export interface ChainLike {
    focus(): ChainLike;
    setHeading(opts: {
        level: number;
    }): ChainLike;
    insertContent(content: string): ChainLike;
    deleteRange(range: {
        from: number;
        to: number;
    }): ChainLike;
    insertTable(opts: unknown): ChainLike;
    setImage(opts: Record<string, unknown>): ChainLike;
    /** Inline mark toggles (plan 024 P3T1): wrap the focused host's live DOM;
     *  the model catches up on the blur writeback. No-op without a host. */
    toggleBold(): ChainLike;
    toggleItalic(): ChainLike;
    toggleStrike(): ChainLike;
    toggleCode(): ChainLike;
    toggleUnderline(): ChainLike;
    setLink(opts: {
        href: string;
    }): ChainLike;
    unsetLink(): ChainLike;
    /** Table verbs (plan 026 P0T3) — resolved against the focused cell. */
    addRowBefore(): ChainLike;
    addRowAfter(): ChainLike;
    deleteRow(): ChainLike;
    addColumnBefore(): ChainLike;
    addColumnAfter(): ChainLike;
    deleteColumn(): ChainLike;
    deleteTable(): ChainLike;
    /** Code language channel (plan 026 P0T3): setBlockAttrs(language) IAL. */
    setCodeBlockLanguage(lang: string): ChainLike;
    setCodeBlock(opts?: {
        language?: string;
    }): ChainLike;
    /** Slash Details template (plan 026 P2T3): kind + summary attr. */
    setDetails(opts?: {
        summary?: string;
    }): ChainLike;
    /** Slash Callout template (plan 030 T7): kind + type/title attrs. */
    setCallout(opts?: {
        type?: string;
        title?: string;
    }): ChainLike;
    /** Task list verb (plan 030 T7): focused ListItem toggles the checked
     *  attr (task ⇄ plain bullet); non-list converts like toggleBulletList. */
    toggleTaskList(): ChainLike;
    run(): boolean;
}
/** tiptap-shaped event callback (the mounted chrome subscribes with
 *  `editor.on('selectionUpdate', cb)` — payload unused by the widgets). */
export type AdapterListener = () => void;
/** The view shim (plan 026 P0T2): the mounted chrome anchors its floating
 *  menus against `view.dom` (the editor content element) and, on the
 *  no-trigger fallback path, asks `nodeDOM(from)` for the focused block's
 *  element. Lazy + DOM-optional so headless/SSR consumers never touch
 *  `document`. */
export interface AdapterView {
    readonly dom: HTMLElement | null;
    readonly state: {
        selection: {
            from: number;
            to: number;
        };
    };
    nodeDOM(from: number): HTMLElement | null;
    /** Caret viewport coords for the focused rich host's char offset
     *  (plan 028 P3T1) — the floating menus' positioning source. Optional on
     *  the frozen interface — createEditorAdapter always sets it. */
    coordsAtPos?(from: number): {
        top: number;
        left: number;
        right: number;
        bottom: number;
    } | null;
}
export interface EditorAdapter {
    storage: Record<string, any>;
    chain(): ChainLike;
    isActive(_name: string, _attrs?: any): boolean;
    /** Focused-block attrs as a plain object (plan 026 P0T2); {} when the
     *  name does not match the focused block's family. Optional on the frozen
     *  interface — createEditorAdapter always sets it. */
    getAttributes?(_name: string): Record<string, unknown>;
    /** Floating-menu anchor (plan 026 P0T2). Same optional-member rule. */
    view?: AdapterView;
    isEditable: boolean;
    /** Event surface (plan 026 P0T1): 'selectionUpdate' subscribers are
     *  notified when an engine change moves the selection. Optional on the
     *  frozen interface — createEditorAdapter always sets it. */
    on?(event: string, cb: AdapterListener): void;
    off?(event: string, cb: AdapterListener): void;
    /** The wrapped session — engine-native readers (slash-manifest's
     *  getCurrentBlockAnchor / ensureBlockAnchor) reach the model through it.
     *  Optional: createEditorAdapter always sets it, but the interface is on
     *  the 1.0.0 frozen surface (plan 020 Phase 4) — a required field would
     *  break external implementors. */
    __engine?: EditorEngine;
}
export declare function createEditorAdapter(engine: EditorEngine): EditorAdapter;
/** Derive the slash query from a host's text + caret: the '/' must sit at
 *  block start or after whitespace, with the query between it and the caret.
 *  Mirrors the Suggestion(char: '/') behavior closely enough for v1. */
export declare function slashQueryAt(text: string, offset: number): string | null;
/** Dispatch the slash CustomEvents for the current host state (the engine
 *  replacement for Tiptap Suggestion's onStart/onUpdate). */
export declare function dispatchSlashState(query: string | null, blockId: string, offset: number): void;
