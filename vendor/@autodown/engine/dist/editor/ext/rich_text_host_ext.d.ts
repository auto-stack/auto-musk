import { BlockHostController } from '../engine/host-controller';
export declare function hostTag(kind: string, level?: number): string;
/** Full class chain incl. the base autodown-block-host (BlockHost rendered
 *  ['autodown-block-host', face.cls] — same computed DOM as one string). */
export declare function hostCls(kind: string, level?: number): string;
/** Mount: when the host mounts it IS the newly focused block — inject the
 *  rich snapshot (spansToHtml of the model inlines, evaluated once by the
 *  assembler — the engine is not Vue-reactive, so it never invalidates
 *  under the user's caret), take DOM focus, and register the unmount
 *  deregistration of the focused-rich-host slot (BlockHost's
 *  onBeforeUnmount).
 *
 *  Caret: a pending click-caret handoff (the preview face's click) is
 *  re-resolved against THIS face's glyphs — the faces are layout-identical,
 *  so the browser lands the caret exactly where the user pointed. Every
 *  other mount path (keyboard focus, history remount, programmatic select,
 *  append-at-end flows, Ctrl+End parity) keeps the end-of-text default. */
export declare function mountHost(initialHtml: string, blockId?: string): void;
/** Place the collapsed caret at a text-code-unit offset within the host
 *  (plan-062 T-03) — the inverse of caretOffset. Falls back to end-of-text
 *  when the offset overshoots (clamped model edits). */
export declare function caretToOffset(node: HTMLElement, offset: number): void;
export declare function hostText(el: HTMLElement): string;
/** Caret offset in text-code-unit terms (Range math over the host subtree). */
export declare function caretOffset(el: HTMLElement): number;
/** Is the collapsed caret on the host's FIRST visual line? The caret's
 *  drawn line sits between the last preceding box and the first following
 *  box (affinity); ↑ takes the LATER bound as the caret's line — the
 *  conservative reading that keeps native in-block movement alive. No
 *  preceding box at all means first line (offset 0 / empty host). */
export declare function caretOnFirstLine(el: HTMLElement): boolean;
/** Is the collapsed caret on the host's LAST visual line? Mirror of
 *  caretOnFirstLine: ↓ takes the EARLIER bound (the last preceding box's
 *  line) as the caret's line. No following box means last line. */
export declare function caretOnLastLine(el: HTMLElement): boolean;
export declare function hostInput(el: HTMLElement, controller: BlockHostController): void;
export declare function hostKeydown(e: KeyboardEvent, controller: BlockHostController): void;
export declare function hostPaste(ev: ClipboardEvent, controller: BlockHostController): void;
export declare function hostCompositionBegin(el: HTMLElement, controller: BlockHostController): void;
export declare function hostCompositionUpdate(e: CompositionEvent, controller: BlockHostController): void;
export declare function hostCompositionCommit(el: HTMLElement, controller: BlockHostController): void;
/** Register as the focused rich host so the adapter's mark chains can wrap
 *  this DOM in place (plan 024 P3T1). */
export declare function hostFocus(el: HTMLElement, _controller: BlockHostController): void;
/** Focus leave: flush any pending plain-text diff first (the normal input
 *  path already committed each keystroke), then walk the rich structure back
 *  into the model as one undo step (plan 024 P2T2).
 *
 * Remount guard: replacing the focused host (Enter-split kind flip,
 * input-rule flip, undo/redo epoch hop) fires blur on the OLD element while
 * it still carries the pre-transition DOM — flushing then would re-insert
 * stale text into the just-restored model and drag the selection back. The
 * retired BlockHost guarded via its template ref (unmount nulls el.value);
 * the liveHosts WeakSet is that lifetime here. */
export declare function hostBlur(el: HTMLElement, controller: BlockHostController): void;
