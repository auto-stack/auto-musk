import { BlockHostController } from '../engine/host-controller';
export declare function hostTag(kind: string, level?: number): string;
/** Full class chain incl. the base autodown-block-host (BlockHost rendered
 *  ['autodown-block-host', face.cls] — same computed DOM as one string). */
export declare function hostCls(kind: string, level?: number): string;
/** Mount: when the host mounts it IS the newly focused block — inject the
 *  rich snapshot (spansToHtml of the model inlines, evaluated once by the
 *  assembler — the engine is not Vue-reactive, so it never invalidates
 *  under the user's caret), take DOM focus with the caret at the end
 *  (append-at-end flows, Ctrl+End parity), and register the unmount
 *  deregistration of the focused-rich-host slot (BlockHost's
 *  onBeforeUnmount). */
export declare function mountHost(initialHtml: string): void;
/** Chromium renders a trailing space in contenteditable as U+00A0 —
 *  normalize at the DOM boundary or the "- "/"# " input-rule markers never
 *  match and the model collects nbsp pollution. */
export declare function hostText(el: HTMLElement): string;
/** Caret offset in text-code-unit terms (Range math over the host subtree). */
export declare function caretOffset(el: HTMLElement): number;
export declare function previousSiblingId(el: HTMLElement): string | null;
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
