import { BlockNode } from '../../parser/block-model';
import { CodeEditorController } from '../engine/code-editor-controller';
/** Highlight HTML for the view face — the builtin panel's resolution order
 *  verbatim (capability gate → bound impl → lowlight), '' when the pipeline
 *  yields nothing so the plain-text branch applies. */
export declare function renderViewHighlight(code: string, language: string): string;
/** Escaped text for a v-html binding — the DSL's `text` emits a
 *  <span>{{}}</span> wrapper, but the builtin panel's byte contract pins
 *  bare text children (render.test's pre>code regex among them). */
export declare function htmlText(s: string): string;
/** The view pre's complete <code> child as one markup string — the two
 *  builtin branches byte-for-byte (highlighted: innerHTML + data-highlighted
 *  in the builtin's attr order; plain: escaped text child). <code> is not
 *  in the DSL element table and html: on a dyn element does not compile to
 *  v-html, so the string is the only byte-exact route. */
export declare function viewCodeInner(code: string, language: string): string;
/** The shared root's data-language: present on the edit wrapper (the
 *  CodeBlockMenu host contract), omitted in view modes (the builtin panel
 *  root carries none — undefined drops the attr). */
export declare function rootDataLanguage(mode: string, language: string): string | undefined;
/** Highlight HTML for the overlay pre: the render pipeline's highlight
 *  bridge with the Vue-layer lowlight fallback, degrading to escaped plain
 *  text so the transparent-text textarea always has visible text under it. */
export declare function renderCodeHighlight(code: string, language: string): string;
/** The edit overlay pre's complete inner markup (plan 039 T9): the
 *  highlighted spans wrapped in a <code> element, mirroring viewCodeInner.
 *  The token color rules chain through `pre code .hljs-*` — bare spans
 *  under the pre (the old renderCodeHighlight contract) never matched and
 *  the edit face read as unhighlighted. Wrapped like the view face, the
 *  same selectors color both. */
export declare function editCodeInner(code: string, language: string): string;
export declare function focusCodeArea(el: HTMLElement | null, readonly: boolean): void;
export declare function resizeCodeArea(el: HTMLElement | null): void;
/** Keep the overlay pre glued to the textarea: mirror its height and any
 *  scroll offsets (the textarea auto-resizes, but wrapping/zoom edges can
 *  still scroll transiently). */
export declare function syncCodeHighlight(areaEl: HTMLElement | null, preEl: HTMLElement | null): void;
export declare function nodeLanguage(node: BlockNode | undefined): string;
export declare function nodeText(node: BlockNode | undefined): string;
export declare function nodeLoading(node: BlockNode | undefined): boolean;
export declare function ctxReadonly(ctx: unknown): boolean;
export declare function ctxBlockId(ctx: unknown): string;
/** The edit face's headless commit controller (whole-text blur commit, one
 *  undo step). Null when no ctx arrived (view/stream modes). */
export declare function codeController(ctx: unknown): CodeEditorController | null;
/** An attribute only the edit face carries (undefined drops the attr —
 *  view/stream roots must not grow stray empty markers). */
export declare function editOnlyAttr(mode: string, v: string): string | undefined;
/** A bare marker attribute only the view/stream faces carry (the node-view
 *  contract's data-*-block="" shape). */
export declare function viewMarker(mode: string): string | undefined;
