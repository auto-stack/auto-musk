import { VNode } from 'vue';
import { BlockNode } from '../parser/block-model';
import { EditorEngine } from '../editor/engine/editor-engine';
export interface BlockEditCtx {
    /** command entry (applyOp / commands.ts chain) */
    engine: EditorEngine;
    blockId: string;
    /** v1 ruling (plan 023): true while streaming — editing face renders read-only */
    readonly: boolean;
}
export interface BlockComponent {
    view(node: BlockNode, final: boolean): VNode;
    stream?(node: BlockNode, final: boolean): VNode;
    edit?(node: BlockNode, ctx: BlockEditCtx): VNode;
}
/** Registry keys are canonical: 'Fence' / 'Table' / 'CodeBlock' / 'Details'... */
export declare function canonicalKind(kind: string): string;
/** Register (or extend) the component for a block kind. Slots omitted here
 *  fall through to the builtin view; call again to add more slots. */
export declare function registerBlockComponent(kind: string, comp: Partial<BlockComponent>): void;
/** Drop one kind's registration entirely (builtin fallback resumes). */
export declare function unregisterBlockComponent(kind: string): void;
/** Test/teardown helper: drop every registration at once. */
export declare function clearBlockComponents(): void;
/** Resolve a kind's component. Returns a fully view-capable component in
 *  every case (registered slots win; missing slots get the builtin), so
 *  callers never null-check view. edit/stream stay undefined when neither a
 *  registration nor a builtin provides them — the caller's BlockHost
 *  fallback covers edit; streaming keeps the markdown segment path. */
export declare function resolveBlockComponent(kind: string): BlockComponent;
/** Convenience: a registered edit slot as a VNode factory, or undefined when
 *  the kind has no typed editing face (use BlockHost). */
export declare function editSlotFor(kind: string): ((node: BlockNode, ctx: BlockEditCtx) => VNode) | undefined;
/** Vue-side wrapper helper: mount an SFC component as an edit slot. */
export declare function sfcEditSlot(comp: unknown): (node: BlockNode, ctx: BlockEditCtx) => VNode;
