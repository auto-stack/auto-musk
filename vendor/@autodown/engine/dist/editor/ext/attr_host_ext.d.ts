/** The block's current attr text — the mounted/sync target. */
export declare function attrModelValue(controller: any, blockId: string, attrKey: string): string;
/** Mount: inject the model value as the host's text (the assembler passes
 *  it as the flat `value` prop — the engine is not Vue-reactive, so the
 *  snapshot never goes stale under the user's caret). */
export declare function mountAttrHost(el: unknown, value: string): void;
/** The version-watch sync (the retired AttrHost.vue watch verbatim): when
 *  the parent's repaint version moves and the host is NOT focused, re-sync
 *  the text from the model — never while focused, so the user's caret is
 *  never clobbered mid-edit. */
export declare function isFocused(el: unknown): boolean;
export declare function syncAttrFromModel(el: unknown, controller: any, blockId: string, attrKey: string): void;
/** Blur commit (one undo step): nbsp from contentediting normalizes back to
 *  a plain space, an unchanged text skips the command, readonly (the
 *  stream→edit v1 gate) skips entirely. */
export declare function commitAttr(el: unknown, controller: any, blockId: string, attrKey: string, readonly: boolean): void;
/** Enter/Escape simply blur (commit) — preventDefault rides the DSL key
 *  modifiers (onkeydown.enter.prevent / .escape.prevent). */
export declare function blurAttrHost(el: unknown): void;
